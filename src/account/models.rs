//! Account Models
//!
//! 账户配置和状态模型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 账户类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 模拟交易账户
    Paper,
    /// 实盘交易账户
    Live,
    /// 模拟实盘账户
    MockLive,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Paper => write!(f, "paper"),
            AccountType::Live => write!(f, "qmt_live"),
            AccountType::MockLive => write!(f, "mock_live"),
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "paper" => Ok(AccountType::Paper),
            "qmt_live" | "live" => Ok(AccountType::Live),
            "mock_live" | "mocklive" => Ok(AccountType::MockLive),
            _ => Err(format!("Unknown account type: {}", s)),
        }
    }
}

/// 账户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// 账户 ID
    pub account_id: String,
    /// 账户名称
    pub account_name: String,
    /// 账户类型
    pub account_type: AccountType,
    /// 适配器名称 (paper, qmt_live, mock_live)
    pub adapter_name: String,
    /// 初始资金
    pub initial_capital: Decimal,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 扩展元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AccountConfig {
    /// 创建新的账户配置
    pub fn new(account_id: String, account_type: AccountType, initial_capital: Decimal) -> Self {
        let now = Utc::now();
        let adapter_name = match &account_type {
            AccountType::Paper => "paper".to_string(),
            AccountType::Live => "qmt_live".to_string(),
            AccountType::MockLive => "mock_live".to_string(),
        };

        Self {
            account_id,
            account_name: String::new(),
            account_type,
            adapter_name,
            initial_capital,
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// 设置账户名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.account_name = name.into();
        self
    }

    /// 设置适配器
    pub fn with_adapter(mut self, adapter: impl Into<String>) -> Self {
        self.adapter_name = adapter.into();
        self
    }

    /// 禁用账户
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// 更新时间戳
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// 账户组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGroup {
    /// 组 ID
    pub group_id: String,
    /// 组名称
    pub group_name: String,
    /// 组内账户 ID 列表
    pub account_ids: Vec<String>,
    /// 资金分配策略
    pub allocation_strategy: AllocationStrategy,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl AccountGroup {
    /// 创建新的账户组
    pub fn new(group_id: String, group_name: String) -> Self {
        let now = Utc::now();
        Self {
            group_id,
            group_name,
            account_ids: Vec::new(),
            allocation_strategy: AllocationStrategy::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加账户
    pub fn add_account(&mut self, account_id: String) {
        if !self.account_ids.contains(&account_id) {
            self.account_ids.push(account_id);
            self.updated_at = Utc::now();
        }
    }

    /// 移除账户
    pub fn remove_account(&mut self, account_id: &str) {
        self.account_ids.retain(|id| id != account_id);
        self.updated_at = Utc::now();
    }
}

/// 资金分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AllocationStrategy {
    /// 平均分配
    #[default]
    Equal,
    /// 按资金比例分配
    Proportional,
    /// 自定义权重
    Weighted(HashMap<String, Decimal>),
    /// 主账户优先
    PrimaryFirst { primary_account_id: String },
}

/// 账户状态汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    /// 账户 ID
    pub account_id: String,
    /// 账户名称
    pub account_name: String,
    /// 账户类型
    pub account_type: AccountType,
    /// 总资产
    pub total_asset: Decimal,
    /// 可用现金
    pub available_cash: Decimal,
    /// 持仓市值
    pub position_value: Decimal,
    /// 持仓数量
    pub position_count: usize,
    /// 今日盈亏
    pub today_pnl: Decimal,
    /// 是否启用
    pub enabled: bool,
}

/// 账户组汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGroupSummary {
    /// 组 ID
    pub group_id: String,
    /// 组名称
    pub group_name: String,
    /// 账户数量
    pub account_count: usize,
    /// 总资产
    pub total_asset: Decimal,
    /// 总可用现金
    pub total_cash: Decimal,
    /// 总持仓市值
    pub total_position_value: Decimal,
    /// 各账户汇总
    pub accounts: Vec<AccountSummary>,
}

/// 订单拆分请求
#[derive(Debug, Clone)]
pub struct OrderSplitRequest {
    /// 股票代码
    pub symbol: String,
    /// 买卖方向
    pub side: String,
    /// 总数量
    pub total_quantity: i64,
    /// 价格 (None 为市价)
    pub price: Option<Decimal>,
    /// 目标账户或账户组
    pub target: SplitTarget,
}

/// 拆分目标
#[derive(Debug, Clone)]
pub enum SplitTarget {
    /// 单一账户
    Single(String),
    /// 账户组
    Group(String),
}

/// 拆分后的子订单
#[derive(Debug, Clone)]
pub struct SplitOrder {
    /// 账户 ID
    pub account_id: String,
    /// 数量
    pub quantity: i64,
    /// 价格
    pub price: Option<Decimal>,
}

/// 拆分结果
#[derive(Debug, Clone)]
pub struct OrderSplitResult {
    /// 原始请求
    pub request: OrderSplitRequest,
    /// 拆分后的订单
    pub splits: Vec<SplitOrder>,
    /// 拆分策略
    pub strategy: AllocationStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_account_config_new() {
        let config = AccountConfig::new(
            "test-001".to_string(),
            AccountType::Paper,
            Decimal::new(100000, 0),
        );

        assert_eq!(config.account_id, "test-001");
        assert_eq!(config.account_type, AccountType::Paper);
        assert_eq!(config.adapter_name, "paper");
        assert!(config.enabled);
    }

    #[test]
    fn test_account_type_from_str() {
        assert_eq!(AccountType::from_str("paper").unwrap(), AccountType::Paper);
        assert_eq!(AccountType::from_str("LIVE").unwrap(), AccountType::Live);
        assert_eq!(
            AccountType::from_str("qmt_live").unwrap(),
            AccountType::Live
        );
        assert_eq!(
            AccountType::from_str("mock_live").unwrap(),
            AccountType::MockLive
        );
        assert!(AccountType::from_str("invalid").is_err());
    }

    #[test]
    fn test_account_type_display_prefers_qmt_live_for_live_accounts() {
        assert_eq!(AccountType::Paper.to_string(), "paper");
        assert_eq!(AccountType::Live.to_string(), "qmt_live");
        assert_eq!(AccountType::MockLive.to_string(), "mock_live");
    }

    #[test]
    fn test_account_group() {
        let mut group = AccountGroup::new("group-001".to_string(), "Test Group".to_string());
        group.add_account("account-1".to_string());
        group.add_account("account-2".to_string());
        group.add_account("account-1".to_string()); // 重复添加

        assert_eq!(group.account_ids.len(), 2);

        group.remove_account("account-1");
        assert_eq!(group.account_ids.len(), 1);
        assert_eq!(group.account_ids[0], "account-2");
    }
}
