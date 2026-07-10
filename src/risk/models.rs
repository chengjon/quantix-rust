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

/// 风控状态聚合（账户维度持久化根）：版本/账户 ID/规则列表/当日基线/买入锁/事件日志。Default 使用 RISK_STATE_VERSION + DEFAULT_RISK_ACCOUNT_ID。
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

/// 单条风控规则：rule_type 决定 value 的解析格式；enabled/created_at/updated_at 控制生效与审计。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskRule {
    pub rule_type: RiskRuleType,
    pub value: RuleValue,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 风控规则类型：PositionLimit 单票仓位上限、DailyLossLimit 日亏损限制、VolatilityLimit 波动率限制、IndustryLimit 行业集中度限制、AutoReduce 自动减仓、IndustryBlocklist 行业黑名单。
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
    /// 从 CLI 字符串解析（`"position-limit"` / `"daily-loss-limit"` / 等）。
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

    /// 返回 CLI 标识串，与 [`Self::parse`] 互逆。
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

    /// 返回面向用户的中文标签。
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

/// 规则取值：Percentage 百分比、Amount 金额、TextList 文本列表（行业黑名单等）。由 RuleValue::parse 按 rule_type 约束格式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleValue {
    Percentage(Decimal),
    Amount(Decimal),
    TextList(Vec<String>),
}

impl RuleValue {
    /// 按 `rule_type` 期望的格式解析原始字符串。
    ///
    /// - `IndustryBlocklist`: 逗号分隔行业名 → `TextList`，至少一个。
    /// - `PositionLimit`/`VolatilityLimit`/`IndustryLimit`: 仅支持百分比值（如 `20%`）。
    /// - `DailyLossLimit`/`AutoReduce`: 既支持百分比也支持金额。
    ///
    /// 数值必须 > 0。
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

    /// 转回面向用户的展示字符串（` Percentage → "20%"`，`Amount → "5000"`，`TextList → "a,b"`）。
    pub fn display(&self) -> String {
        match self {
            Self::Percentage(value) => format!("{value}%"),
            Self::Amount(value) => value.to_string(),
            Self::TextList(values) => values.join(","),
        }
    }
}

/// 当日风控基线：trading_date 交易日、starting_total_assets 开盘总资产，用于日内盈亏计算与日亏损限制判断。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyRiskBaseline {
    pub trading_date: NaiveDate,
    pub starting_total_assets: Decimal,
}

/// 买入锁状态：locked 是否锁定、reason/triggered_at/trading_date/released_for_date 描述触发与释放；触发后当日禁止买入，次日自动释放。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BuyLockState {
    pub locked: bool,
    pub reason: Option<String>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub trading_date: Option<NaiveDate>,
    pub released_for_date: Option<NaiveDate>,
}

/// 风控事件类型：规则设置/启用/禁用、日亏损锁触发/释放/清除、行业集中度超限、自动减仓触发/执行。入库为 snake_case 字符串。
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
    /// 从 CLI 字符串解析事件类型。
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

    /// 返回 CLI 标识串，与 [`Self::parse`] 互逆。
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

    /// 返回面向用户的中文标签。
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

/// 单条风控事件日志：ts 时间戳、event_type、可选 trading_date、detail 文本描述，追加到 RiskState.events 审计轨迹。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskLogEvent {
    pub ts: DateTime<Utc>,
    pub event_type: RiskLogEventType,
    pub trading_date: Option<NaiveDate>,
    pub detail: String,
}

/// 风控状态对外展示快照：账户/交易日/总资产/盈亏、买入锁详情（含锁来源/原因/触发时间/生效日）、持仓风险行、规则快照、自动减仓建议。
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
    pub auto_reduce_recommendation: Option<AutoReduceRecommendation>,
}

/// 自动减仓建议：current_loss_pct 当前亏损比例、reduce_ratio 建议减仓比例、position_codes 命中持仓代码、triggered_at 触发时间。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoReduceRecommendation {
    pub current_loss_pct: Decimal,
    pub reduce_ratio: Decimal,
    pub position_codes: Vec<String>,
    pub triggered_at: DateTime<Utc>,
}

/// 单只持仓的风险行：code、market_value 市值、ratio_pct 占总资产百分比，用于集中度展示与超限判断。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionRiskRow {
    pub code: String,
    pub market_value: Decimal,
    pub ratio_pct: Decimal,
}

/// 规则快照：rule_type + value + enabled，用于 RiskStatus 对外展示当前生效规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskRuleSnapshot {
    pub rule_type: RiskRuleType,
    pub value: RuleValue,
    pub enabled: bool,
}

/// 风控账户快照：account_id、total_assets 总资产、positions 持仓列表（code + market_value），RiskStatus 计算的输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAccountSnapshot {
    pub account_id: String,
    pub total_assets: Decimal,
    pub positions: Vec<RiskPositionSnapshot>,
}

impl RiskAccountSnapshot {
    /// 用账户 ID、总资产和 `(code, market_value)` 持仓元组列表构造快照。
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

/// 风控持仓单项：code 标的代码、market_value 市值，是 RiskAccountSnapshot.positions 的元素。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskPositionSnapshot {
    pub code: String,
    pub market_value: Decimal,
}

/// 买入预估影响：code、projected_position_value 买入后持仓市值、projected_total_assets 买入后总资产，用于事前风险评估。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedBuyImpact {
    pub code: String,
    pub projected_position_value: Decimal,
    pub projected_total_assets: Decimal,
}

impl ProjectedBuyImpact {
    /// 构造买入后的预估快照（持仓市值 + 总资产）。
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

/// 风控账户来源：Paper 纸面交易、LiveImport 实盘导入。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskAccountSource {
    Paper,
    LiveImport,
}

impl RiskAccountSource {
    /// 从字符串解析（`"paper"` / `"live_import"`），未匹配返回 `None`。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "paper" => Some(Self::Paper),
            "live_import" => Some(Self::LiveImport),
            _ => None,
        }
    }

    /// 返回标识串，与 [`Self::from_str`] 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paper => "paper",
            Self::LiveImport => "live_import",
        }
    }
}

/// 实盘导入记录类型：Trade 交易、Cash 资金存取。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportRecordType {
    Trade,
    Cash,
}

impl LiveImportRecordType {
    /// 从字符串解析（`"trade"` / `"cash"`）。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "trade" => Some(Self::Trade),
            "cash" => Some(Self::Cash),
            _ => None,
        }
    }

    /// 返回标识串，与 [`Self::from_str`] 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trade => "trade",
            Self::Cash => "cash",
        }
    }
}

/// 实盘导入交易方向：Buy 买入、Sell 卖出。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportTradeSide {
    Buy,
    Sell,
}

impl LiveImportTradeSide {
    /// 从字符串解析（`"buy"` / `"sell"`）。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            _ => None,
        }
    }

    /// 返回标识串，与 [`Self::from_str`] 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

/// 实盘导入资金业务类型：Deposit 入金、Withdraw 出金。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveImportCashBusinessType {
    Deposit,
    Withdraw,
}

impl LiveImportCashBusinessType {
    /// 从字符串解析（`"deposit"` / `"withdraw"`）。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "deposit" => Some(Self::Deposit),
            "withdraw" => Some(Self::Withdraw),
            _ => None,
        }
    }

    /// 返回标识串，与 [`Self::from_str`] 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Withdraw => "withdraw",
        }
    }
}

/// 实盘导入单条原始记录：record_type 区分交易/资金；交易字段（code/side/price/volume/fee_total/executed_at）与资金字段（business_type/amount/occurred_at）按类型填入，其余为 None。
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

/// 实盘导入批次摘要：batch_id/account_id/总行数/插入数/跳过重复数/冲突数，由导入流水线汇总。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportBatchSummary {
    pub batch_id: String,
    pub account_id: String,
    pub total_rows: usize,
    pub inserted: usize,
    pub skipped_duplicates: usize,
    pub conflicts: usize,
}

/// 实盘导入冲突记录：相同 external_id 已存在但字段不一致，existing_record_json 与 incoming_record_json 并存供人工裁决，detail 描述差异点。
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

/// 实盘镜像单只持仓：code 标的代码、volume 持仓量、avg_cost 平均成本、last_trade_at 最近交易时间。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveImportMirrorPosition {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_at: DateTime<Utc>,
}

/// 实盘镜像账户聚合：account_id/trading_date、as_of 快照时间、起始/当前总资产、现金、已实现盈亏、累计费用、last_rebuild_at 最近重建时间、positions 持仓列表。
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
