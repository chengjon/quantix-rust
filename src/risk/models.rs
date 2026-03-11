use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};

pub const DEFAULT_RISK_ACCOUNT_ID: &str = "default";
pub const RISK_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskState {
    pub version: u32,
    pub account_id: String,
    pub rules: Vec<RiskRule>,
    pub daily_baseline: Option<DailyRiskBaseline>,
    pub buy_lock: BuyLockState,
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            version: RISK_STATE_VERSION,
            account_id: DEFAULT_RISK_ACCOUNT_ID.to_string(),
            rules: Vec::new(),
            daily_baseline: None,
            buy_lock: BuyLockState::default(),
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
}

impl RiskRuleType {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "position-limit" => Ok(Self::PositionLimit),
            "daily-loss-limit" => Ok(Self::DailyLossLimit),
            other => Err(QuantixError::Other(format!(
                "risk rule 不支持的类型: {other}"
            ))),
        }
    }

    pub fn as_cli_str(self) -> &'static str {
        match self {
            Self::PositionLimit => "position-limit",
            Self::DailyLossLimit => "daily-loss-limit",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleValue {
    Percentage(Decimal),
    Amount(Decimal),
}

impl RuleValue {
    pub fn parse(rule_type: RiskRuleType, raw: &str) -> Result<Self> {
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
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Percentage(value) => format!("{value}%"),
            Self::Amount(value) => value.to_string(),
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
    pub lock_reason: Option<String>,
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
