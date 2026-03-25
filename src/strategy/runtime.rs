use async_trait::async_trait;

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::execution::models::SignalEnvelope;
use crate::strategy::ma_cross::MACrossStrategy;
use crate::strategy::{Strategy, trait_def::Signal};

#[async_trait]
pub trait StrategyBarLoader: Send + Sync {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>>;
}

#[derive(Debug, Clone)]
pub struct StrategyRuntime<L> {
    loader: L,
}

impl<L> StrategyRuntime<L> {
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
            latest_signal = strategy
                .on_bar(bar)
                .await
                .map_err(|err| QuantixError::Other(format!("strategy paper 策略执行失败: {err}")))?;
        }
        strategy
            .finish()
            .await
            .map_err(|err| QuantixError::Other(format!("strategy paper 策略收尾失败: {err}")))?;

        Ok(SignalEnvelope::new(latest_signal))
    }
}
