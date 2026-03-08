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
    QFQ = 1, // 前复权
    HFQ = 2, // 后复权
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
    SH, // 上海
    SZ, // 深圳
    BJ, // 北京
}

/// 股本变迁事件 (除权除息)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GbbqEvent {
    /// 股票代码
    pub code: String,
    /// 事件日期
    pub event_date: NaiveDate,
    /// 信息类别
    /// 1=除权除息, 2=送配股上市, 3=非流通股上市, 4=未知变动,
    /// 5=股本变化, 6=增发新股, 7=股份回购, 8=增发上市, 9=转配上市,
    /// 10=可转债上市, 11=扩缩股, 12=缩股, 13=认购权证, 14=认沽权证
    pub category: u8,
    /// 分红（每10股派现金x元）
    pub dividend: f32,
    /// 配股价（每股配股价x元）
    pub bonus_price: f32,
    /// 送转股（每10股送转股比例x股）
    pub bonus_share: f32,
    /// 配股（每10股配股比例x股）
    pub rights_share: f32,
    /// 除权价
    pub ex_price: Option<f64>,
    /// 登记日
    pub record_date: Option<NaiveDate>,
}

/// 股本变更摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalChange {
    /// 股票代码
    pub code: String,
    /// 变更日期
    pub change_date: NaiveDate,
    /// 变更前总股本
    pub before_total: Option<f64>,
    /// 变更后总股本
    pub after_total: Option<f64>,
    /// 变更前流通股
    pub before_float: Option<f64>,
    /// 变更后流通股
    pub after_float: Option<f64>,
    /// 变更类型
    pub change_type: String,
}
