use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    /// 返回稳定字符串标识（"day"/"week"/"month"），用于入库与 API 参数。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

impl std::str::FromStr for BarPeriod {
    type Err = crate::core::QuantixError;

    /// Accepts only `day` | `week` | `month` (any case). Rejects
    /// `daily`/`weekly`/`monthly` aliases and any `minute*` value
    /// (P0.13b scope) — see design D6.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            other => Err(crate::core::QuantixError::Config(format!(
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
    /// 返回稳定字符串标识（"1m"/"5m"/"15m"/"30m"/"60m"），匹配 OpenStock `_PERIOD_MAP` token，用于 API 调用。
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
    type Err = crate::core::QuantixError;

    /// 仅接受 `1m|5m|15m|30m|60m`（任意大小写）。拒绝所有别名
    /// （`1min|minute|5min|1h|hour` 等）和任何非 5 个主 token 的值。
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1m" => Ok(Self::Minute1),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "30m" => Ok(Self::Minute30),
            "60m" => Ok(Self::Minute60),
            other => Err(crate::core::QuantixError::Config(format!(
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

    /// Stable string form for CLI display (`none` | `qfq` | `hfq`).
    /// Mirrors the values accepted by `FromStr`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::QFQ => "qfq",
            Self::HFQ => "hfq",
        }
    }
}

impl std::str::FromStr for AdjustType {
    type Err = crate::core::QuantixError;

    /// Accepts `none` | `qfq` | `hfq` (any case).
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "qfq" => Ok(Self::QFQ),
            "hfq" => Ok(Self::HFQ),
            other => Err(crate::core::QuantixError::Config(format!(
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

/// CLI 互斥输入：单日（`--date`）或封闭范围（`--start`/`--end`）。
///
/// 由 `from_cli` 唯一构造——编译时强制半开区间和 `(None, None, None)` 不可达。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOrRange {
    /// 单日查询（向后兼容 P0.13b-1/2 `--date` 路径）
    Date(chrono::NaiveDate),
    /// 多日范围（inclusive on both ends）
    Range {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },
}

impl DateOrRange {
    /// 从 CLI 三 `Option<&str>` 输入构造 `DateOrRange`。
    ///
    /// 校验规则（spec §3.1 + D5）：
    ///   - `(Some(d), None, None)` → `Date(d)`
    ///   - `(None, Some(s), Some(e))` → `Range { start: s, end: e }`（s ≤ e）
    ///   - 其它所有形态 → `Err`
    pub fn from_cli(
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Self, crate::core::error::QuantixError> {
        use crate::core::error::QuantixError;

        let has_date = date.is_some();
        let has_range = start.is_some() || end.is_some();

        if has_date && has_range {
            return Err(QuantixError::Config(
                "--date cannot be combined with --start/--end; use either --date for single day or --start/--end for range".to_string(),
            ));
        }
        if has_date {
            let date = date.ok_or_else(|| {
                QuantixError::Config("--date was set but the argument value is missing".to_string())
            })?;
            let parsed = parse_date_arg(date, "--date")?;
            return Ok(DateOrRange::Date(parsed));
        }
        if has_range {
            let (Some(s_str), Some(e_str)) = (start, end) else {
                return Err(QuantixError::Config(
                    "--start and --end must be provided together (semi-open ranges are not supported)".to_string(),
                ));
            };
            let s = parse_date_arg(s_str, "--start")?;
            let e = parse_date_arg(e_str, "--end")?;
            if s > e {
                return Err(QuantixError::Config(format!(
                    "--start ({}) must be on or before --end ({})",
                    s_str, e_str
                )));
            }
            return Ok(DateOrRange::Range { start: s, end: e });
        }
        // 全 None
        Err(QuantixError::Config(
            "at least one of --date or (--start, --end) is required".to_string(),
        ))
    }
}

/// 解析单个 `YYYY-MM-DD` CLI 日期参数，失败时返回包含 flag 名的错误。
fn parse_date_arg(
    s: &str,
    flag_name: &str,
) -> Result<chrono::NaiveDate, crate::core::error::QuantixError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        crate::core::error::QuantixError::Config(format!(
            "{}: invalid date '{}': {}",
            flag_name, s, e
        ))
    })
}

/// 生成 `start..=end` 的日历日迭代器（含非交易日，调用方负责处理空响应）。
pub fn iter_dates_inclusive(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> impl Iterator<Item = chrono::NaiveDate> {
    (0..=((end - start).num_days() as u64)).map(move |n| start + chrono::Duration::days(n as i64))
}

/// 把 `[start..=end]` 切成连续的 7 天段（P0.13d D2）。
///
/// 返回 `Vec<(NaiveDate, NaiveDate)>`，覆盖 `[start..=end]`：
///   - 第一段从 `start` 开始
///   - 每段长度 ≤ 7 天（含端点）
///   - 段与段之间无 gap、无 overlap（`chunks[i].1 + 1 day == chunks[i+1].0`）
///   - `start == end` 时返回单元素 `vec![(start, end)]`
///   - `start > end` 时返回空 `Vec`（防御性，调用方应保证 `start <= end`）
///
/// 不依赖 `chrono::Weekday`；纯算术切片，便于测试。
///
/// 注：spec 描述为 "private"，此处使用 `pub(crate)` —— Rust 中 "crate 内部可见" 的惯用法，
/// 以便 `src/sources/openstock_client.rs`（Task 3 的 stream 方法所在）等模块可直接复用。
pub(crate) fn chunk_range_weekly(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Vec<(chrono::NaiveDate, chrono::NaiveDate)> {
    // Defensive: caller (DateOrRange::from_cli) already guarantees start <= end,
    // but the function is pure and should not panic on edge cases.
    if start > end {
        return vec![];
    }
    let mut out = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        // (end - cursor).num_days() is exclusive of end; +1 gives inclusive count.
        // Take a 7-day chunk only if MORE than 7 inclusive days remain.
        // i.e. for an exact 7-day remaining span, num_days() == 6, so we take it
        // whole (seg_end = end); for an 8-day span, num_days() == 7 >= 7, so we
        // cut at cursor+6 and leave the extra day for the next chunk.
        let seg_end = if (end - cursor).num_days() >= 7 {
            cursor + chrono::Duration::days(6)
        } else {
            end
        };
        out.push((cursor, seg_end));
        // next segment starts the day after seg_end; if seg_end == end loop exits
        cursor = seg_end + chrono::Duration::days(1);
    }
    out
}
