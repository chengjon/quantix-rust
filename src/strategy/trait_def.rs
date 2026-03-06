/// 策略 trait 定义
///
/// 所有策略实现统一的 Strategy 接口

use async_trait::async_trait;

use crate::data::models::Kline;

/// 策略 trait
#[async_trait]
pub trait Strategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 初始化策略
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// 处理 K线数据
    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        Ok(Signal::Hold)
    }

    /// 策略结束
    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// 交易信号
#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}
