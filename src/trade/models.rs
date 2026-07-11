use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::core::{QuantixError, Result};

/// 默认账户标识：未指定 account_id 时使用此值（"default"）。
pub const DEFAULT_ACCOUNT_ID: &str = "default";
/// PaperTradeState 持久化版本号：当前 schema 版本为 1，升级时递增以做兼容迁移。
pub const PAPER_TRADE_STATE_VERSION: u32 = 1;

/// 纸面交易持久化根：版本号、可选账户、交易记录列表。Default 使用 PAPER_TRADE_STATE_VERSION + 空 account + 空 records。
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

/// 纸面交易账户：account_id 账户 ID、initial_capital 初始资金、available_cash 可用现金、fee_config 费率配置、positions 持仓字典、created_at/updated_at 时间戳。
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

/// 单只标的的纸面持仓：code、volume 持仓量、avg_cost 平均成本、last_trade_price 最近成交价、opened_at 建仓时间、updated_at 最近更新时间。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradePosition {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_price: Decimal,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 单条交易入库记录：id 主键、code/side/price/volume/amount 订单信息、commission/stamp_duty/transfer_fee/total_fee 费用分解、executed_at 成交时间。
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

/// 交易方向：Buy 买入、Sell 卖出。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 手续费率配置：commission_rate 佣金率、commission_min 最低佣金、stamp_duty_rate 印花税率、transfer_fee_rate 过户费率。Default 为 A 股常见费率。
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
    /// 从 CLI 输入构造 [`FeeConfig`]，缺失字段回退到 [`FeeConfig::default`]。
    ///
    /// 负值会被 [`QuantixError`] 拒绝。
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

/// 单笔交易费用分解：commission 佣金、stamp_duty 印花税、transfer_fee 过户费、total_fee 合计。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeeBreakdown {
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
}

/// 资金快照：initial_capital 初始资金、available_cash 可用现金、estimated_position_value 估算持仓市值、estimated_total_assets 估算总资产。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CashSnapshot {
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub estimated_position_value: Decimal,
    pub estimated_total_assets: Decimal,
}

/// 交易历史展示行：executed_at/code/side/price/volume/amount、total_fee 总费用、net_cash_impact 净现金流影响（买入为负，卖出为正）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeHistoryRow {
    pub executed_at: DateTime<Utc>,
    pub code: String,
    pub side: TradeSide,
    pub price: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub total_fee: Decimal,
    pub net_cash_impact: Decimal,
}

/// 单笔交易费用展示行：executed_at/code/side + commission/stamp_duty/transfer_fee/total_fee，用于费用分析报表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeFeeRow {
    pub executed_at: DateTime<Utc>,
    pub code: String,
    pub side: TradeSide,
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
}

/// 账户总览：初始资金/可用现金/账面持仓市值/账面总资产/交易笔数/持仓数/累计买入/累计卖出/累计费用，可选实时估值与行情覆盖率（已取/总数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeOverview {
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub booked_position_value: Decimal,
    pub booked_total_assets: Decimal,
    pub trade_count: usize,
    pub holding_count: usize,
    pub total_buy_amount: Decimal,
    pub total_sell_amount: Decimal,
    pub total_fee: Decimal,
    pub live_position_value: Option<Decimal>,
    pub live_total_assets: Option<Decimal>,
    pub quote_coverage: Option<(usize, usize)>,
}

/// 持仓行情状态：BookOnly 仅有账面价、Live 已取到实时行情、Missing 缺失行情（用于 TradeOverview 的 quote_coverage 统计）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeQuoteStatus {
    BookOnly,
    Live,
    Missing,
}

/// 持仓当前展示行：账面字段（code/volume/avg_cost/last_trade_price）+ 实时字段（current_price/current_market_value/unrealized_pnl/unrealized_pnl_pct，无行情时为 None）+ quote_status。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradePositionCurrentRow {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_price: Decimal,
    pub current_price: Option<Decimal>,
    pub current_market_value: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub unrealized_pnl_pct: Option<Decimal>,
    pub quote_status: TradeQuoteStatus,
}

/// trade init 命令请求：capital 初始资金、fee_config 费率配置；由 InitAccountRequest::new 从 CLI 输入解析。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitAccountRequest {
    pub capital: Decimal,
    pub fee_config: FeeConfig,
}

impl InitAccountRequest {
    /// 从 CLI 输入构造初始化账户请求；`capital` 缺省回退到 `1_000_000`。
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

/// trade order 命令请求：code 标的代码、price 价格、volume 数量；由 TradeOrderRequest::new 校验代码格式与价格/数量为正。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeOrderRequest {
    pub code: String,
    pub price: Decimal,
    pub volume: i64,
}

impl TradeOrderRequest {
    /// 构造交易下单请求，校验代码非空、价格 / 数量为正。
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
