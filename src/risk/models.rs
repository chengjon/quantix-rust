#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{QuantixError, Result};

mod lock_state;
pub use lock_state::RiskLockStateSource;

pub const DEFAULT_RISK_ACCOUNT_ID: &str = "default";
pub const RISK_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskState {
    pub version: u32,
    pub account_id: String,
    pub rules: Vec<RiskRule>,
    pub daily_baseline: Option<DailyRiskBaseline>,
    pub buy_lock: BuyLockState,
    #[serde(default)]
    pub events: Vec<RiskLogEvent>,
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            version: RISK_STATE_VERSION,
            account_id: DEFAULT_RISK_ACCOUNT_ID.to_string(),
            rules: Vec::new(),
            daily_baseline: None,
            buy_lock: BuyLockState::default(),
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskRule {
    pub rule_type: RiskRuleType,
    pub value: RuleValue,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskRuleType {
    PositionLimit,
    DailyLossLimit,
    VolatilityLimit,
    /// 行业集中度限制 - 单一行业持仓占总资产的最大比例
    IndustryLimit,
    /// 自动减仓 - 当亏损达到阈值时自动卖出
    AutoReduce,
    IndustryBlocklist,
}

impl RiskRuleType {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "position-limit" => Ok(Self::PositionLimit),
            "daily-loss-limit" => Ok(Self::DailyLossLimit),
            "volatility-limit" => Ok(Self::VolatilityLimit),
            "industry-limit" => Ok(Self::IndustryLimit),
            "auto-reduce" => Ok(Self::AutoReduce),
            "industry-blocklist" => Ok(Self::IndustryBlocklist),
            other => Err(QuantixError::Other(format!(
                "risk rule 不支持的类型: {other}"
            ))),
        }
    }

    pub fn as_cli_str(self) -> &'static str {
        match self {
            Self::PositionLimit => "position-limit",
            Self::DailyLossLimit => "daily-loss-limit",
            Self::VolatilityLimit => "volatility-limit",
            Self::IndustryLimit => "industry-limit",
            Self::AutoReduce => "auto-reduce",
            Self::IndustryBlocklist => "industry-blocklist",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::PositionLimit => "单票仓位上限",
            Self::DailyLossLimit => "日亏损限制",
            Self::VolatilityLimit => "波动率限制",
            Self::IndustryLimit => "行业集中度限制",
            Self::AutoReduce => "自动减仓",
            Self::IndustryBlocklist => "行业黑名单",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleValue {
    Percentage(Decimal),
    Amount(Decimal),
    TextList(Vec<String>),
}

impl RuleValue {
    pub fn parse(rule_type: RiskRuleType, raw: &str) -> Result<Self> {
        if rule_type == RiskRuleType::IndustryBlocklist {
            let values = raw
                .split(',')
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            if values.is_empty() {
                return Err(QuantixError::Other(format!(
                    "risk rule industry-blocklist 至少需要一个行业名称: {raw}"
                )));
            }

            return Ok(Self::TextList(values));
        }

        let value = raw.trim();
        let is_percentage = value.ends_with('%');
        let number = if is_percentage {
            &value[..value.len() - 1]
        } else {
            value
        }
        .trim();

        if number.is_empty() {
            return Err(QuantixError::Other(format!(
                "risk rule value 不能为空: {raw}"
            )));
        }

        let decimal = Decimal::from_str_exact(number)
            .map_err(|_| QuantixError::Other(format!("risk rule value 无法解析: {raw}")))?;

        if decimal <= Decimal::ZERO {
            return Err(QuantixError::Other(format!(
                "risk rule value 必须大于 0: {raw}"
            )));
        }

        match (rule_type, is_percentage) {
            (RiskRuleType::PositionLimit, true) => Ok(Self::Percentage(decimal)),
            (RiskRuleType::PositionLimit, false) => Err(QuantixError::Other(
                "risk rule position-limit 仅支持百分比值，例如 20%".to_string(),
            )),
            (RiskRuleType::DailyLossLimit, true) => Ok(Self::Percentage(decimal)),
            (RiskRuleType::DailyLossLimit, false) => Ok(Self::Amount(decimal)),
            (RiskRuleType::VolatilityLimit, true) => Ok(Self::Percentage(decimal)),
            (RiskRuleType::VolatilityLimit, false) => Err(QuantixError::Other(
                "risk rule volatility-limit 仅支持百分比值，例如 4%".to_string(),
            )),
            (RiskRuleType::IndustryLimit, true) => Ok(Self::Percentage(decimal)),
            (RiskRuleType::IndustryLimit, false) => Err(QuantixError::Other(
                "risk rule industry-limit 仅支持百分比值，例如 30%".to_string(),
            )),
            (RiskRuleType::AutoReduce, true) => Ok(Self::Percentage(decimal)),
            (RiskRuleType::AutoReduce, false) => Ok(Self::Amount(decimal)),
            (RiskRuleType::IndustryBlocklist, _) => unreachable!(),
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Percentage(value) => format!("{value}%"),
            Self::Amount(value) => value.to_string(),
            Self::TextList(values) => values.join(","),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyRiskBaseline {
    pub trading_date: NaiveDate,
    pub starting_total_assets: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BuyLockState {
    pub locked: bool,
    pub reason: Option<String>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub trading_date: Option<NaiveDate>,
    pub released_for_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLogEventType {
    RuleSet,
    RuleEnabled,
    RuleDisabled,
    DailyLossLockTriggered,
    BuyLockReleased,
    BuyLockCleared,
    IndustryLimitTriggered,
    AutoReduceTriggered,
    AutoReduceExecuted,
}

impl RiskLogEventType {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "rule-set" => Ok(Self::RuleSet),
            "rule-enabled" => Ok(Self::RuleEnabled),
            "rule-disabled" => Ok(Self::RuleDisabled),
            "daily-loss-lock-triggered" => Ok(Self::DailyLossLockTriggered),
            "buy-lock-released" => Ok(Self::BuyLockReleased),
            "buy-lock-cleared" => Ok(Self::BuyLockCleared),
            "industry-limit-triggered" => Ok(Self::IndustryLimitTriggered),
            "auto-reduce-triggered" => Ok(Self::AutoReduceTriggered),
            "auto-reduce-executed" => Ok(Self::AutoReduceExecuted),
            other => Err(QuantixError::Other(format!(
                "risk log 不支持的类型: {other}"
            ))),
        }
    }

    pub fn as_cli_str(self) -> &'static str {
        match self {
            Self::RuleSet => "rule-set",
            Self::RuleEnabled => "rule-enabled",
            Self::RuleDisabled => "rule-disabled",
            Self::DailyLossLockTriggered => "daily-loss-lock-triggered",
            Self::BuyLockReleased => "buy-lock-released",
            Self::BuyLockCleared => "buy-lock-cleared",
            Self::IndustryLimitTriggered => "industry-limit-triggered",
            Self::AutoReduceTriggered => "auto-reduce-triggered",
            Self::AutoReduceExecuted => "auto-reduce-executed",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::RuleSet => "规则设置",
            Self::RuleEnabled => "规则启用",
            Self::RuleDisabled => "规则禁用",
            Self::DailyLossLockTriggered => "日亏损锁触发",
            Self::BuyLockReleased => "买入锁释放",
            Self::BuyLockCleared => "买入锁清除",
            Self::IndustryLimitTriggered => "行业集中度超限",
            Self::AutoReduceTriggered => "自动减仓触发",
            Self::AutoReduceExecuted => "自动减仓执行",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskLogEvent {
    pub ts: DateTime<Utc>,
    pub event_type: RiskLogEventType,
    pub trading_date: Option<NaiveDate>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskStatus {
    pub account_id: String,
    pub trading_date: NaiveDate,
    pub starting_total_assets: Decimal,
    pub current_total_assets: Decimal,
    pub daily_pnl: Decimal,
    pub daily_pnl_pct: Decimal,
    pub buy_locked: bool,
    pub manual_release_active: bool,
    pub lock_state_source: RiskLockStateSource,
    pub lock_reason: Option<String>,
    pub lock_trigger_reason: Option<String>,
    pub lock_triggered_at: Option<DateTime<Utc>>,
    pub lock_effective_trading_date: Option<NaiveDate>,
    pub position_ratios: Vec<PositionRiskRow>,
    pub rules: Vec<RiskRuleSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionRiskRow {
    pub code: String,
    pub market_value: Decimal,
    pub ratio_pct: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskRuleSnapshot {
    pub rule_type: RiskRuleType,
    pub value: RuleValue,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAccountSnapshot {
    pub account_id: String,
    pub total_assets: Decimal,
    pub positions: Vec<RiskPositionSnapshot>,
}

impl RiskAccountSnapshot {
    pub fn new(
        account_id: impl Into<String>,
        total_assets: Decimal,
        positions: Vec<(String, Decimal)>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            total_assets,
            positions: positions
                .into_iter()
                .map(|(code, market_value)| RiskPositionSnapshot { code, market_value })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskPositionSnapshot {
    pub code: String,
    pub market_value: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedBuyImpact {
    pub code: String,
    pub projected_position_value: Decimal,
    pub projected_total_assets: Decimal,
}

impl ProjectedBuyImpact {
    pub fn new(
        code: impl Into<String>,
        projected_position_value: Decimal,
        projected_total_assets: Decimal,
    ) -> Self {
        Self {
            code: code.into(),
            projected_position_value,
            projected_total_assets,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskAccountSource {
    Paper,
    LiveImport,
}

impl RiskAccountSource {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "paper" => Some(Self::Paper),
            "live_import" => Some(Self::LiveImport),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paper => "paper",
            Self::LiveImport => "live_import",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportRecordType {
    Trade,
    Cash,
}

impl LiveImportRecordType {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "trade" => Some(Self::Trade),
            "cash" => Some(Self::Cash),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trade => "trade",
            Self::Cash => "cash",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportTradeSide {
    Buy,
    Sell,
}

impl LiveImportTradeSide {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportCashBusinessType {
    Deposit,
    Withdraw,
}

impl LiveImportCashBusinessType {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "deposit" => Some(Self::Deposit),
            "withdraw" => Some(Self::Withdraw),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Withdraw => "withdraw",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportRecord {
    pub record_type: LiveImportRecordType,
    pub account_id: String,
    pub external_id: String,
    pub code: Option<String>,
    pub side: Option<LiveImportTradeSide>,
    pub price: Option<Decimal>,
    pub volume: Option<i64>,
    pub fee_total: Option<Decimal>,
    pub business_type: Option<LiveImportCashBusinessType>,
    pub amount: Option<Decimal>,
    pub executed_at: Option<DateTime<Utc>>,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportBatchSummary {
    pub batch_id: String,
    pub account_id: String,
    pub total_rows: usize,
    pub inserted: usize,
    pub skipped_duplicates: usize,
    pub conflicts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportConflict {
    pub id: String,
    pub batch_id: String,
    pub account_id: String,
    pub external_id: String,
    pub existing_record_json: Value,
    pub incoming_record_json: Value,
    pub detail: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportMirrorPosition {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportMirrorAccount {
    pub account_id: String,
    pub trading_date: NaiveDate,
    pub as_of: DateTime<Utc>,
    pub starting_total_assets: Decimal,
    pub current_total_assets: Decimal,
    pub cash_balance: Decimal,
    pub realized_pnl: Decimal,
    pub total_fees: Decimal,
    pub last_rebuild_at: DateTime<Utc>,
    pub positions: Vec<LiveImportMirrorPosition>,
}
