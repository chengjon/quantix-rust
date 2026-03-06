/// 均线交叉策略
///
/// MA 金叉死叉策略示例

use async_trait::async_trait;

use crate::data::models::Kline;
use crate::strategy::trait_def::{Signal, Strategy};

/// MA 金叉死叉策略
pub struct MACrossStrategy {
    short_period: usize,
    long_period: usize,
    name: String,
}

impl MACrossStrategy {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
            name: format!("MA_{}_{}", short_period, long_period),
        }
    }
}

#[async_trait]
impl Strategy for MACrossStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        // TODO: 实现 MA 金叉死叉逻辑
        Ok(Signal::Hold)
    }
}
