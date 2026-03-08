/// 数据存储
///
/// 统一数据存储接口
use crate::core::Result;
use crate::data::models::Kline;

/// 存储 trait
pub trait Storage: Send + Sync {
    /// 保存 K线数据
    fn save_klines(&self, klines: Vec<Kline>) -> Result<()>;

    /// 查询 K线数据
    fn query_klines(
        &self,
        code: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<Vec<Kline>>;
}
