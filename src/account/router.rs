//! Account Router
//!
//! 智能订单路由，根据账户配置将订单分配到正确的执行适配器

use rust_decimal::Decimal;
use std::collections::HashMap;

use super::models::{
    AccountConfig, AllocationStrategy, OrderSplitRequest, OrderSplitResult, SplitOrder, SplitTarget,
};
use super::registry::AccountRegistry;
use crate::core::{QuantixError, Result};

/// 账户路由器
///
/// 负责将订单路由到正确的账户和适配器
#[derive(Debug, Clone)]
pub struct AccountRouter {
    registry: AccountRegistry,
}

impl AccountRouter {
    /// 创建新的路由器
    pub fn new(registry: AccountRegistry) -> Self {
        Self { registry }
    }

    /// 获取账户的适配器名称
    pub async fn get_adapter_name(&self, account_id: &str) -> Result<String> {
        let account = self
            .registry
            .get_account(account_id)
            .await
            .ok_or_else(|| QuantixError::Other(format!("账户不存在: {}", account_id)))?;

        if !account.enabled {
            return Err(QuantixError::Other(format!("账户已禁用: {}", account_id)));
        }

        Ok(account.adapter_name)
    }

    /// 检查账户是否可用
    pub async fn is_account_available(&self, account_id: &str) -> bool {
        if let Some(account) = self.registry.get_account(account_id).await {
            return account.enabled;
        }
        false
    }

    /// 拆分订单到多个账户
    pub async fn split_order(&self, request: OrderSplitRequest) -> Result<OrderSplitResult> {
        if request.total_quantity <= 0 {
            return Err(QuantixError::Other(format!(
                "订单数量必须大于 0: {}",
                request.total_quantity
            )));
        }

        let total_quantity = request.total_quantity;
        let price = request.price;
        let target = request.target.clone();

        match &target {
            SplitTarget::Single(account_id) => {
                // 单账户，不拆分
                self.validate_account(account_id).await?;

                Ok(OrderSplitResult {
                    request,
                    splits: vec![SplitOrder {
                        account_id: account_id.clone(),
                        quantity: total_quantity,
                        price,
                    }],
                    strategy: AllocationStrategy::Equal,
                })
            }
            SplitTarget::Group(group_id) => {
                // 账户组，按策略拆分
                let group =
                    self.registry.get_group(group_id).await.ok_or_else(|| {
                        QuantixError::Other(format!("账户组不存在: {}", group_id))
                    })?;

                self.split_order_to_group(request, &group).await
            }
        }
    }

    /// 按账户组拆分订单
    async fn split_order_to_group(
        &self,
        request: OrderSplitRequest,
        group: &super::models::AccountGroup,
    ) -> Result<OrderSplitResult> {
        let enabled_accounts = self.get_enabled_group_accounts(&group.account_ids).await;

        if enabled_accounts.is_empty() {
            return Err(QuantixError::Other(format!(
                "账户组 {} 中没有可用账户",
                group.group_id
            )));
        }

        let strategy = group.allocation_strategy.clone();
        let total_quantity = request.total_quantity;
        let price = request.price;

        let splits = match &strategy {
            AllocationStrategy::Equal => self.split_equal(&enabled_accounts, total_quantity, price),
            AllocationStrategy::Proportional => {
                self.split_proportional(&enabled_accounts, total_quantity, price)
            }
            AllocationStrategy::Weighted(weights) => {
                self.split_weighted(&enabled_accounts, weights, total_quantity, price)
            }
            AllocationStrategy::PrimaryFirst { primary_account_id } => self.split_primary_first(
                &enabled_accounts,
                primary_account_id,
                total_quantity,
                price,
            ),
        };

        Ok(OrderSplitResult {
            request,
            splits,
            strategy,
        })
    }

    /// 获取组内启用的账户
    async fn get_enabled_group_accounts(&self, account_ids: &[String]) -> Vec<AccountConfig> {
        let mut accounts = Vec::new();
        for account_id in account_ids {
            if let Some(account) = self.registry.get_account(account_id).await
                && account.enabled
                && account.initial_capital > Decimal::ZERO
            {
                accounts.push(account);
            }
        }
        accounts
    }

    /// 平均分配
    fn split_equal(
        &self,
        accounts: &[AccountConfig],
        total_quantity: i64,
        price: Option<Decimal>,
    ) -> Vec<SplitOrder> {
        let n = accounts.len() as i64;
        if n == 0 {
            return Vec::new();
        }

        let base_qty = total_quantity / n;
        let remainder = total_quantity % n;

        accounts
            .iter()
            .enumerate()
            .map(|(i, account)| SplitOrder {
                account_id: account.account_id.clone(),
                quantity: base_qty + if (i as i64) < remainder { 1 } else { 0 },
                price,
            })
            .filter(|s| s.quantity > 0)
            .collect()
    }

    /// 按资金比例分配
    fn split_proportional(
        &self,
        accounts: &[AccountConfig],
        total_quantity: i64,
        price: Option<Decimal>,
    ) -> Vec<SplitOrder> {
        // 计算总资金
        let total_capital: Decimal = accounts.iter().map(|a| a.initial_capital).sum();

        if total_capital <= Decimal::ZERO {
            return self.split_equal(accounts, total_quantity, price);
        }

        let mut splits = Vec::new();
        let mut allocated: i64 = 0;

        for account in accounts {
            let ratio = account.initial_capital / total_capital;
            let qty = (Decimal::from(total_quantity) * ratio)
                .to_string()
                .parse::<i64>()
                .unwrap_or(0);

            if qty > 0 {
                splits.push(SplitOrder {
                    account_id: account.account_id.clone(),
                    quantity: qty,
                    price,
                });
                allocated += qty;
            }
        }

        // 调整余数
        let remainder = total_quantity - allocated;
        if remainder != 0 && !splits.is_empty() {
            splits[0].quantity += remainder;
        }

        splits
    }

    /// 按权重分配
    fn split_weighted(
        &self,
        accounts: &[AccountConfig],
        weights: &HashMap<String, Decimal>,
        total_quantity: i64,
        price: Option<Decimal>,
    ) -> Vec<SplitOrder> {
        // 计算总权重
        let total_weight: Decimal = accounts
            .iter()
            .filter_map(|a| weights.get(&a.account_id))
            .sum();

        if total_weight <= Decimal::ZERO {
            return self.split_equal(accounts, total_quantity, price);
        }

        let mut splits = Vec::new();
        let mut allocated: i64 = 0;

        for account in accounts {
            let weight = weights
                .get(&account.account_id)
                .copied()
                .unwrap_or(Decimal::ZERO);
            if weight <= Decimal::ZERO {
                continue;
            }

            let ratio = weight / total_weight;
            let qty = (Decimal::from(total_quantity) * ratio)
                .to_string()
                .parse::<i64>()
                .unwrap_or(0);

            if qty > 0 {
                splits.push(SplitOrder {
                    account_id: account.account_id.clone(),
                    quantity: qty,
                    price,
                });
                allocated += qty;
            }
        }

        // 调整余数
        let remainder = total_quantity - allocated;
        if remainder != 0 && !splits.is_empty() {
            splits[0].quantity += remainder;
        }

        splits
    }

    /// 主账户优先分配
    fn split_primary_first(
        &self,
        accounts: &[AccountConfig],
        primary_account_id: &str,
        total_quantity: i64,
        price: Option<Decimal>,
    ) -> Vec<SplitOrder> {
        let mut splits = Vec::new();

        // 先分配给主账户
        for account in accounts {
            if account.account_id == primary_account_id {
                splits.push(SplitOrder {
                    account_id: account.account_id.clone(),
                    quantity: total_quantity,
                    price,
                });
                return splits;
            }
        }

        // 如果主账户不存在或不可用，平均分配
        self.split_equal(accounts, total_quantity, price)
    }

    /// 验证账户
    async fn validate_account(&self, account_id: &str) -> Result<()> {
        let account = self
            .registry
            .get_account(account_id)
            .await
            .ok_or_else(|| QuantixError::Other(format!("账户不存在: {}", account_id)))?;

        if !account.enabled {
            return Err(QuantixError::Other(format!("账户已禁用: {}", account_id)));
        }

        if account.initial_capital <= Decimal::ZERO {
            return Err(QuantixError::Other(format!(
                "账户初始资金必须大于 0: {} = {}",
                account_id, account.initial_capital
            )));
        }

        Ok(())
    }

    /// 获取账户配置
    pub async fn get_account_config(&self, account_id: &str) -> Option<AccountConfig> {
        self.registry.get_account(account_id).await
    }

    /// 获取默认账户
    pub async fn get_default_account(&self) -> Option<AccountConfig> {
        self.registry.get_default_account().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    async fn create_test_registry() -> AccountRegistry {
        let registry = AccountRegistry::new();

        // 创建测试账户
        let account1 = AccountConfig::new(
            "acc-1".to_string(),
            super::super::models::AccountType::Paper,
            dec!(100000),
        );
        let account2 = AccountConfig::new(
            "acc-2".to_string(),
            super::super::models::AccountType::Paper,
            dec!(200000),
        );
        let account3 = AccountConfig::new(
            "acc-3".to_string(),
            super::super::models::AccountType::Paper,
            dec!(300000),
        );

        registry.register_account(account1).await.unwrap();
        registry.register_account(account2).await.unwrap();
        registry.register_account(account3).await.unwrap();

        // 创建账户组
        registry
            .create_group("group-1".to_string(), "Test Group".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-1".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-2".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-3".to_string())
            .await
            .unwrap();

        registry
    }

    #[tokio::test]
    async fn test_split_order_single_account() {
        let registry = create_test_registry().await;
        let router = AccountRouter::new(registry);

        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 1000,
            price: Some(dec!(100)),
            target: SplitTarget::Single("acc-1".to_string()),
        };

        let result = router.split_order(request).await.unwrap();
        assert_eq!(result.splits.len(), 1);
        assert_eq!(result.splits[0].quantity, 1000);
        assert_eq!(result.splits[0].account_id, "acc-1");
    }

    #[tokio::test]
    async fn test_split_order_group_equal() {
        let registry = create_test_registry().await;
        let router = AccountRouter::new(registry);

        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 1000,
            price: Some(dec!(100)),
            target: SplitTarget::Group("group-1".to_string()),
        };

        let result = router.split_order(request).await.unwrap();
        assert_eq!(result.splits.len(), 3);

        let total: i64 = result.splits.iter().map(|s| s.quantity).sum();
        assert_eq!(total, 1000);
    }

    #[tokio::test]
    async fn test_split_order_rejects_non_positive_total_quantity() {
        let registry = create_test_registry().await;
        let router = AccountRouter::new(registry);

        for total_quantity in [0, -100] {
            let request = OrderSplitRequest {
                symbol: "600519.SH".to_string(),
                side: "buy".to_string(),
                total_quantity,
                price: Some(dec!(100)),
                target: SplitTarget::Single("acc-1".to_string()),
            };

            let result = router.split_order(request).await;

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("订单数量必须大于 0")
            );
        }
    }

    #[tokio::test]
    async fn test_split_order_invalid_account() {
        let registry = create_test_registry().await;
        let router = AccountRouter::new(registry);

        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 1000,
            price: Some(dec!(100)),
            target: SplitTarget::Single("nonexistent".to_string()),
        };

        let result = router.split_order(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_split_order_rejects_single_account_with_non_positive_capital() {
        let registry = AccountRegistry::new();
        registry
            .register_account(AccountConfig::new(
                "acc-1".to_string(),
                super::super::models::AccountType::Paper,
                dec!(0),
            ))
            .await
            .unwrap_err();

        let registry = AccountRegistry::with_accounts(
            HashMap::from([(
                "acc-1".to_string(),
                AccountConfig::new(
                    "acc-1".to_string(),
                    super::super::models::AccountType::Paper,
                    dec!(0),
                ),
            )]),
            "acc-1".to_string(),
        );
        let router = AccountRouter::new(registry);

        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 1000,
            price: Some(dec!(100)),
            target: SplitTarget::Single("acc-1".to_string()),
        };

        let result = router.split_order(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("必须大于 0"));
    }

    #[tokio::test]
    async fn test_split_order_group_skips_non_positive_capital_accounts() {
        let registry = AccountRegistry::with_accounts(
            HashMap::from([
                (
                    "acc-1".to_string(),
                    AccountConfig::new(
                        "acc-1".to_string(),
                        super::super::models::AccountType::Paper,
                        dec!(0),
                    ),
                ),
                (
                    "acc-2".to_string(),
                    AccountConfig::new(
                        "acc-2".to_string(),
                        super::super::models::AccountType::Paper,
                        dec!(200000),
                    ),
                ),
            ]),
            "acc-2".to_string(),
        );

        registry
            .create_group("group-1".to_string(), "Test Group".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-1".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-2".to_string())
            .await
            .unwrap();

        let router = AccountRouter::new(registry);
        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 1000,
            price: Some(dec!(100)),
            target: SplitTarget::Group("group-1".to_string()),
        };

        let result = router.split_order(request).await.unwrap();
        assert_eq!(result.splits.len(), 1);
        assert_eq!(result.splits[0].account_id, "acc-2");
        assert_eq!(result.splits[0].quantity, 1000);
    }
}
