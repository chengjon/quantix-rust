use async_trait::async_trait;

use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::execution::models::SignalEnvelope;
use crate::strategy::Strategy;
use crate::strategy::ma_cross::MACrossStrategy;

/// K 线加载 trait：从数据源（ClickHouse/TDengine）按 code 与 limit 取最近 N 根日线。Send + Sync 以适配 runtime 的并发模型。
#[async_trait]
pub trait StrategyBarLoader: Send + Sync {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>>;
}

#[async_trait]
impl<T> StrategyBarLoader for &T
where
    T: StrategyBarLoader + Send + Sync,
{
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        (*self).load_daily_bars(code, limit).await
    }
}

/// 策略运行时：注入 StrategyBarLoader，对外提供 `run_ma_cross_once` 等单次评估入口。泛型 L 让数据源（生产 ClickHouse / 测试 mock）可插拔。
#[derive(Debug, Clone)]
pub struct StrategyRuntime<L> {
    loader: L,
}

impl<L> StrategyRuntime<L> {
    /// 创建 runtime：注入 loader 实现（如 ClickHouseBarLoader），后续评估调用会通过它拉取 K 线。
    pub fn new(loader: L) -> Self {
        Self { loader }
    }
}

impl<L> StrategyRuntime<L>
where
    L: StrategyBarLoader,
{
    pub async fn run_ma_cross_once(
        &self,
        code: &str,
        short: usize,
        long: usize,
    ) -> Result<SignalEnvelope> {
        let bars = self.loader.load_daily_bars(code, 10_000).await?;
        if bars.len() < long {
            return Err(QuantixError::Other(format!(
                "strategy paper 数据不足，至少需要 {long} 条，当前 {}",
                bars.len()
            )));
        }

        let mut strategy = MACrossStrategy::new(short, long);
        let mut latest_signal = Signal::Hold;
        for bar in &bars {
            latest_signal = strategy.on_bar(bar).await.map_err(|err| {
                QuantixError::Other(format!("strategy paper 策略执行失败: {err}"))
            })?;
        }
        strategy
            .finish()
            .await
            .map_err(|err| QuantixError::Other(format!("strategy paper 策略收尾失败: {err}")))?;

        Ok(SignalEnvelope::new(latest_signal))
    }
}
