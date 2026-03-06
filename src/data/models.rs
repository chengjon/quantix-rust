/// 数据模型
///
/// 定义 K线、行情、交易等核心数据结构
/// 与 Python quantix 项目保持一致

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// K线数据（日线）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub code: String,
    pub date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: AdjustType,
}

/// 复权类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AdjustType {
    None = 0,
    QFQ = 1,  // 前复权
    HFQ = 2,  // 后复权
}

/// Tick 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub price: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub direction: TradeDirection,
}

/// 交易方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TradeDirection {
    Buy,
    Sell,
    Neutral,
}

/// 股票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market: Market,
    pub list_date: Option<NaiveDate>,
    pub delist_date: Option<NaiveDate>,
}

/// 市场类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Market {
    SH,  // 上海
    SZ,  // 深圳
    BJ,  // 北京
}
