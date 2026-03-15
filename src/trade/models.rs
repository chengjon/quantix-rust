use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::core::{QuantixError, Result};

pub const DEFAULT_ACCOUNT_ID: &str = "default";
pub const PAPER_TRADE_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaperTradeState {
    pub version: u32,
    pub account: Option<PaperTradeAccount>,
    pub trade_records: Vec<TradeRecord>,
}

impl Default for PaperTradeState {
    fn default() -> Self {
        Self {
            version: PAPER_TRADE_STATE_VERSION,
            account: None,
            trade_records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaperTradeAccount {
    pub account_id: String,
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub fee_config: FeeConfig,
    pub positions: BTreeMap<String, TradePosition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradePosition {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_price: Decimal,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeRecord {
    pub id: String,
    pub code: String,
    pub side: TradeSide,
    pub price: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeeConfig {
    pub commission_rate: Decimal,
    pub commission_min: Decimal,
    pub stamp_duty_rate: Decimal,
    pub transfer_fee_rate: Decimal,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            commission_rate: dec!(0.00025),
            commission_min: dec!(5),
            stamp_duty_rate: dec!(0.001),
            transfer_fee_rate: dec!(0.00001),
        }
    }
}

impl FeeConfig {
    pub fn from_inputs(
        commission_rate: Option<f64>,
        commission_min: Option<f64>,
        stamp_duty_rate: Option<f64>,
        transfer_fee_rate: Option<f64>,
    ) -> Result<Self> {
        let defaults = Self::default();

        Ok(Self {
            commission_rate: parse_optional_non_negative_decimal(
                "trade init --commission-rate",
                commission_rate,
            )?
            .unwrap_or(defaults.commission_rate),
            commission_min: parse_optional_non_negative_decimal(
                "trade init --commission-min",
                commission_min,
            )?
            .unwrap_or(defaults.commission_min),
            stamp_duty_rate: parse_optional_non_negative_decimal(
                "trade init --stamp-duty-rate",
                stamp_duty_rate,
            )?
            .unwrap_or(defaults.stamp_duty_rate),
            transfer_fee_rate: parse_optional_non_negative_decimal(
                "trade init --transfer-fee-rate",
                transfer_fee_rate,
            )?
            .unwrap_or(defaults.transfer_fee_rate),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeeBreakdown {
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CashSnapshot {
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub estimated_position_value: Decimal,
    pub estimated_total_assets: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitAccountRequest {
    pub capital: Decimal,
    pub fee_config: FeeConfig,
}

impl InitAccountRequest {
    pub fn new(
        capital: Option<f64>,
        commission_rate: Option<f64>,
        commission_min: Option<f64>,
        stamp_duty_rate: Option<f64>,
        transfer_fee_rate: Option<f64>,
    ) -> Result<Self> {
        Ok(Self {
            capital: parse_optional_positive_decimal("trade init --capital", capital)?
                .unwrap_or(dec!(1000000)),
            fee_config: FeeConfig::from_inputs(
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeOrderRequest {
    pub code: String,
    pub price: Decimal,
    pub volume: i64,
}

impl TradeOrderRequest {
    pub fn new(code: impl Into<String>, price: f64, volume: i64) -> Result<Self> {
        let code = code.into().trim().to_string();
        if code.is_empty() {
            return Err(QuantixError::Other("trade order code 不能为空".to_string()));
        }
        validate_trade_code(&code)?;

        Ok(Self {
            code,
            price: parse_required_positive_decimal("trade order --price", price)?,
            volume: validate_positive_volume(volume)?,
        })
    }
}

fn parse_optional_positive_decimal(flag: &str, value: Option<f64>) -> Result<Option<Decimal>> {
    value
        .map(|value| parse_required_positive_decimal(flag, value))
        .transpose()
}

fn parse_required_positive_decimal(flag: &str, value: f64) -> Result<Decimal> {
    if !value.is_finite() || value <= 0.0 {
        return Err(QuantixError::Other(format!("{flag} 必须是有限正数")));
    }

    Decimal::from_str_exact(&value.to_string())
        .map_err(|_| QuantixError::Other(format!("{flag} 无法转换为 Decimal")))
}

fn parse_optional_non_negative_decimal(flag: &str, value: Option<f64>) -> Result<Option<Decimal>> {
    value
        .map(|value| {
            if !value.is_finite() || value < 0.0 {
                return Err(QuantixError::Other(format!("{flag} 必须是有限非负数")));
            }

            Decimal::from_str_exact(&value.to_string())
                .map_err(|_| QuantixError::Other(format!("{flag} 无法转换为 Decimal")))
        })
        .transpose()
}

fn validate_positive_volume(volume: i64) -> Result<i64> {
    if volume <= 0 {
        return Err(QuantixError::Other(
            "trade order --volume 必须是正整数".to_string(),
        ));
    }

    Ok(volume)
}

fn validate_trade_code(code: &str) -> Result<()> {
    let is_valid = code.len() == 6 && code.chars().all(|ch| ch.is_ascii_digit());
    if is_valid {
        Ok(())
    } else {
        Err(QuantixError::Other(format!(
            "trade order 股票代码格式不合法: {code}"
        )))
    }
}
