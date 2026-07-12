#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
