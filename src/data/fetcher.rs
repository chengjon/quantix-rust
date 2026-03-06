/// 数据获取器
///
/// 统一数据获取接口

use async_trait::async_trait;

use crate::core::Result;
use crate::data::models::{Kline, StockInfo};

/// 数据源 trait
#[async_trait]
pub trait Fetcher: Send + Sync {
    /// 获取股票信息
    async fn get_stock_info(&self, code: &str) -> Result<Option<StockInfo>>;

    /// 获取 K线数据
    async fn get_kline(
        &self,
        code: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<Vec<Kline>>;

    /// 检查连接
    async fn check_connection(&self) -> Result<()>;
}
