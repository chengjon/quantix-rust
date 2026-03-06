/// 数据库模块
///
/// 与 Python quantix 项目共享数据库连接
/// - PostgreSQL: 日线、股票信息等结构化数据
/// - TDengine: 高频时序数据 (分钟线/tick)

pub mod postgresql;
pub mod tdengine;

pub use postgresql::{PostgresClient, KlineDaily, StockInfo};
pub use tdengine::{TDengineClient, MinuteKline};
