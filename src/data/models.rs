/// 数据模型
///
/// 定义 K线、行情、交易等核心数据结构
/// 与 Python quantix 项目保持一致
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::QuantixError;

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

/// `/data/bars` 周期参数 (OpenStock API).
///
/// Named `BarPeriod` (not `KlinePeriod`) to avoid collision with the
/// aggregator-side `KlinePeriod` in `src/sources/kline_aggregator.rs`,
/// which represents 1m/5m/1d aggregation windows — a different semantic
/// domain from the OpenStock `/data/bars` `period` request parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarPeriod {
    Day,
    Week,
    Month,
}

impl BarPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

impl std::str::FromStr for BarPeriod {
    type Err = QuantixError;

    /// Accepts only `day` | `week` | `month` (any case). Rejects
    /// `daily`/`weekly`/`monthly` aliases and any `minute*` value
    /// (P0.13b scope) — see design D6.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            other => Err(QuantixError::Config(format!(
                "unsupported BarPeriod `{}`: expected one of day|week|month",
                other
            ))),
        }
    }
}

impl AdjustType {
    /// Returns the OpenStock `/data/bars` `adjust` parameter value, or
    /// `None` when the field should be omitted entirely (matches the
    /// existing `fetch_daily_klines` behavior — it omits the `adjust`
    /// field rather than sending `"adjust": ""`).
    pub fn as_openstock_param(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::QFQ => Some("qfq"),
            Self::HFQ => Some("hfq"),
        }
    }
}

impl std::str::FromStr for AdjustType {
    type Err = QuantixError;

    /// Accepts `none` | `qfq` | `hfq` (any case).
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "qfq" => Ok(Self::QFQ),
            "hfq" => Ok(Self::HFQ),
            other => Err(QuantixError::Config(format!(
                "unsupported AdjustType `{}`: expected one of none|qfq|hfq",
                other
            ))),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn bar_period_as_str_round_trip() {
        assert_eq!(BarPeriod::Day.as_str(), "day");
        assert_eq!(BarPeriod::Week.as_str(), "week");
        assert_eq!(BarPeriod::Month.as_str(), "month");
    }

    #[test]
    fn bar_period_from_str_accepts_canonical_case_insensitive() {
        assert!(matches!(BarPeriod::from_str("day"), Ok(BarPeriod::Day)));
        assert!(matches!(BarPeriod::from_str("WEEK"), Ok(BarPeriod::Week)));
        assert!(matches!(BarPeriod::from_str("Month"), Ok(BarPeriod::Month)));
    }

    #[test]
    fn bar_period_from_str_rejects_aliases() {
        // D6: strict — reject daily/weekly/monthly/minute* aliases
        assert!(BarPeriod::from_str("daily").is_err());
        assert!(BarPeriod::from_str("weekly").is_err());
        assert!(BarPeriod::from_str("monthly").is_err());
        assert!(BarPeriod::from_str("1m").is_err());
        assert!(BarPeriod::from_str("minute").is_err());
        assert!(BarPeriod::from_str("").is_err());
    }

    #[test]
    fn adjust_type_as_openstock_param() {
        assert_eq!(AdjustType::None.as_openstock_param(), None);
        assert_eq!(AdjustType::QFQ.as_openstock_param(), Some("qfq"));
        assert_eq!(AdjustType::HFQ.as_openstock_param(), Some("hfq"));
    }

    #[test]
    fn adjust_type_from_str_case_insensitive() {
        assert!(matches!(AdjustType::from_str("none"), Ok(AdjustType::None)));
        assert!(matches!(AdjustType::from_str("QFQ"), Ok(AdjustType::QFQ)));
        assert!(matches!(AdjustType::from_str("Hfq"), Ok(AdjustType::HFQ)));
        assert!(AdjustType::from_str("front").is_err());
        assert!(AdjustType::from_str("").is_err());
    }
}
