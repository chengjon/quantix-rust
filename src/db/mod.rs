/// 数据库模块
///
/// 主存储使用 ClickHouse（高性能 OLAP）
/// - stock_info: 股票基本信息
/// - stock_realtime_quotes: 实时行情
/// - kline_data: K线数据
/// - limit_up_events: 涨停事件

pub mod clickhouse;
pub mod postgresql;
pub mod tdengine;

pub use clickhouse::{
    ClickHouseClient,
    StockInfoCH, StockQuoteCH, KlineDataCH, LimitUpEventCH,
};
pub use postgresql::{PostgresClient, KlineDaily, StockInfo};
pub use tdengine::{TDengineClient, MinuteKline};
