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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// `/data/bars` 分钟周期参数 (P0.13b-1, OpenStock API).
///
/// 与 `BarPeriod`（day/week/month）语义域不同：分钟蜡烛返回
/// `Vec<MinuteBar>`（含 `NaiveDateTime` 时间戳），日线/周线/月线
/// 返回 `Vec<Kline>`（仅 `NaiveDate`）。类型系统强制调用方区分。
///
/// Wire tokens `1m|5m|15m|30m|60m` 直接对应 OpenStock `_PERIOD_MAP`
/// 主 token。**拒绝所有别名**（`1min|minute|5min|1h|hour` 等），
/// 因为 `_PERIOD_MAP.get(period, "day")` 对未知 token 静默回退到
/// day——严格白名单 + fail-fast 是唯一安全策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinutePeriod {
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Minute60,
}

impl MinutePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minute1 => "1m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Minute30 => "30m",
            Self::Minute60 => "60m",
        }
    }
}

impl std::str::FromStr for MinutePeriod {
    type Err = QuantixError;

    /// 仅接受 `1m|5m|15m|30m|60m`（任意大小写）。拒绝所有别名
    /// （`1min|minute|5min|1h|hour` 等）和任何非 5 个主 token 的值。
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1m" => Ok(Self::Minute1),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "30m" => Ok(Self::Minute30),
            "60m" => Ok(Self::Minute60),
            other => Err(QuantixError::Config(format!(
                "unsupported MinutePeriod `{}`: expected one of 1m|5m|15m|30m|60m",
                other
            ))),
        }
    }
}

/// 分钟级 K 线蜡烛（P0.13b-1 新增）。
///
/// **命名说明**：命名为 `MinuteBar`（不是 `MinuteKline`），因为
/// `src/db/tdengine.rs:37` 已存在公开 re-export 的 `MinuteKline`{
/// ts: DateTime<Utc>, code, open: f64, ... }——TDengine 行映射用 f64。
/// 本类型用 `Decimal` + `AdjustType`，语义不同，必须避免名称碰撞。
/// `MinuteBar` 与 P0.13a `BarPeriod` 形成请求/响应语义对。
///
/// 与 `Kline`（日线）的区别：
/// - `timestamp: NaiveDateTime`（精确到分钟）vs `date: NaiveDate`
/// - 其他字段与 `Kline` 一致
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteBar {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: AdjustType,
}

/// 分时点序列（P0.13b-2 新增）。
///
/// 对应 OpenStock `MINUTE_DATA` category。与 `MinuteBar` 区别：
/// - 无 OHLC（仅单一 `price`）
/// - 含 `avg_price`（均价，业务关键字段）
///
/// **Option 字段说明**：业务字段全部用 `Option` 包裹以支持 INV-2C
/// （单条记录字段缺失时 warn + skip，不中断整批）。serde 反序列化
/// 在 Option 字段缺失时返回 None 而非失败；parser 阶段检查关键字段
/// （price/volume/amount/avg_price），任一为 None 则 warn + skip。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteShare {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub price: Option<Decimal>,
    pub volume: Option<i64>,
    pub amount: Option<Decimal>,
    pub avg_price: Option<Decimal>,
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

    #[test]
    fn minute_period_as_str_round_trip() {
        assert_eq!(MinutePeriod::Minute1.as_str(), "1m");
        assert_eq!(MinutePeriod::Minute5.as_str(), "5m");
        assert_eq!(MinutePeriod::Minute15.as_str(), "15m");
        assert_eq!(MinutePeriod::Minute30.as_str(), "30m");
        assert_eq!(MinutePeriod::Minute60.as_str(), "60m");
    }

    #[test]
    fn minute_period_from_str_accepts_canonical_case_insensitive() {
        assert!(matches!(
            MinutePeriod::from_str("1m"),
            Ok(MinutePeriod::Minute1)
        ));
        assert!(matches!(
            MinutePeriod::from_str("5M"),
            Ok(MinutePeriod::Minute5)
        ));
        assert!(matches!(
            MinutePeriod::from_str("15m"),
            Ok(MinutePeriod::Minute15)
        ));
        assert!(matches!(
            MinutePeriod::from_str("30M"),
            Ok(MinutePeriod::Minute30)
        ));
        assert!(matches!(
            MinutePeriod::from_str("60m"),
            Ok(MinutePeriod::Minute60)
        ));
    }

    #[test]
    fn minute_period_from_str_rejects_aliases() {
        // D4 strict — reject 1min/minute/5min/1h/hour and any day* value
        assert!(MinutePeriod::from_str("1min").is_err());
        assert!(MinutePeriod::from_str("minute").is_err());
        assert!(MinutePeriod::from_str("5min").is_err());
        assert!(MinutePeriod::from_str("1h").is_err());
        assert!(MinutePeriod::from_str("hour").is_err());
        assert!(MinutePeriod::from_str("day").is_err());
        assert!(MinutePeriod::from_str("").is_err());
    }

    #[test]
    fn minute_share_round_trip_serde() {
        use chrono::NaiveDate;
        let share = crate::data::models::MinuteShare {
            code: "sh600000".to_string(),
            timestamp: NaiveDate::from_ymd_opt(2026, 7, 1)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
            price: Some(Decimal::from_str("10.50").unwrap()),
            volume: Some(123_456),
            amount: Some(Decimal::from_str("1296288.00").unwrap()),
            avg_price: Some(Decimal::from_str("10.4975").unwrap()),
        };
        let json = serde_json::to_string(&share).expect("serialize");
        let back: crate::data::models::MinuteShare =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.code, "sh600000");
        assert_eq!(back.volume, Some(123_456));
        assert_eq!(back.avg_price, Some(Decimal::from_str("10.4975").unwrap()));
    }

    #[test]
    fn minute_share_allows_missing_optional_fields() {
        // Missing price/volume/amount/avg_price fields → all None (INV-2C foundation)
        let json = r#"{"code":"sh600000","timestamp":"2026-07-01T09:30:00"}"#;
        let share: crate::data::models::MinuteShare =
            serde_json::from_str(json).expect("deserialize with missing optionals");
        assert_eq!(share.code, "sh600000");
        assert_eq!(share.price, None);
        assert_eq!(share.volume, None);
        assert_eq!(share.amount, None);
        assert_eq!(share.avg_price, None);
    }
}
