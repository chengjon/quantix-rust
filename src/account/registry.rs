//! Account Registry
//!
//! 账户注册表，管理所有账户配置和账户组

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::{AccountConfig, AccountGroup, AccountType, AllocationStrategy};
use crate::core::{QuantixError, Result};

/// 账户注册表
#[derive(Debug, Clone)]
pub struct AccountRegistry {
    inner: Arc<RwLock<AccountRegistryInner>>,
}

#[derive(Debug, Clone)]
struct AccountRegistryInner {
    /// 账户配置
    accounts: HashMap<String, AccountConfig>,
    /// 账户组
    groups: HashMap<String, AccountGroup>,
    /// 默认账户 ID
    default_account_id: String,
}

impl AccountRegistry {
    /// 创建新的账户注册表
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AccountRegistryInner {
                accounts: HashMap::new(),
                groups: HashMap::new(),
                default_account_id: "default".to_string(),
            })),
        }
    }

    /// 使用预加载的账户创建注册表
    pub fn with_accounts(accounts: HashMap<String, AccountConfig>, default_id: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AccountRegistryInner {
                accounts,
                groups: HashMap::new(),
                default_account_id: default_id,
            })),
        }
    }

    /// 注册账户
    pub async fn register_account(&self, config: AccountConfig) -> Result<()> {
        validate_account_config(&config)?;
        let mut inner = self.inner.write().await;

        if inner.accounts.contains_key(&config.account_id) {
            return Err(QuantixError::Other(format!(
                "账户已存在: {}",
                config.account_id
            )));
        }

        inner.accounts.insert(config.account_id.clone(), config);
        Ok(())
    }

    /// 更新账户
    pub async fn update_account(&self, config: AccountConfig) -> Result<()> {
        validate_account_config(&config)?;
        let mut inner = self.inner.write().await;

        if !inner.accounts.contains_key(&config.account_id) {
            return Err(QuantixError::Other(format!(
                "账户不存在: {}",
                config.account_id
            )));
        }

        inner.accounts.insert(config.account_id.clone(), config);
        Ok(())
    }

    /// 注销账户
    pub async fn unregister_account(&self, account_id: &str) -> Result<AccountConfig> {
        let mut inner = self.inner.write().await;

        inner
            .accounts
            .remove(account_id)
            .ok_or_else(|| QuantixError::Other(format!("账户不存在: {}", account_id)))
    }

    /// 获取账户
    pub async fn get_account(&self, account_id: &str) -> Option<AccountConfig> {
        let inner = self.inner.read().await;
        inner.accounts.get(account_id).cloned()
    }

    /// 获取默认账户
    pub async fn get_default_account(&self) -> Option<AccountConfig> {
        let inner = self.inner.read().await;
        inner.accounts.get(&inner.default_account_id).cloned()
    }

    /// 设置默认账户
    pub async fn set_default_account(&self, account_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;

        if !inner.accounts.contains_key(account_id) {
            return Err(QuantixError::Other(format!("账户不存在: {}", account_id)));
        }

        inner.default_account_id = account_id.to_string();
        Ok(())
    }

    /// 列出所有账户
    pub async fn list_accounts(&self) -> Vec<AccountConfig> {
        let inner = self.inner.read().await;
        inner.accounts.values().cloned().collect()
    }

    /// 列出指定类型的账户
    pub async fn list_accounts_by_type(&self, account_type: AccountType) -> Vec<AccountConfig> {
        let inner = self.inner.read().await;
        inner
            .accounts
            .values()
            .filter(|a| a.account_type == account_type)
            .cloned()
            .collect()
    }

    /// 列出启用的账户
    pub async fn list_enabled_accounts(&self) -> Vec<AccountConfig> {
        let inner = self.inner.read().await;
        inner
            .accounts
            .values()
            .filter(|a| a.enabled)
            .cloned()
            .collect()
    }

    /// 获取账户数量
    pub async fn account_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.accounts.len()
    }

    // ========== 账户组管理 ==========

    /// 创建账户组
    pub async fn create_group(&self, group_id: String, group_name: String) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.groups.contains_key(&group_id) {
            return Err(QuantixError::Other(format!("账户组已存在: {}", group_id)));
        }

        inner
            .groups
            .insert(group_id.clone(), AccountGroup::new(group_id, group_name));
        Ok(())
    }

    /// 获取账户组
    pub async fn get_group(&self, group_id: &str) -> Option<AccountGroup> {
        let inner = self.inner.read().await;
        inner.groups.get(group_id).cloned()
    }

    /// 删除账户组
    pub async fn delete_group(&self, group_id: &str) -> Result<AccountGroup> {
        let mut inner = self.inner.write().await;

        inner
            .groups
            .remove(group_id)
            .ok_or_else(|| QuantixError::Other(format!("账户组不存在: {}", group_id)))
    }

    /// 向账户组添加账户
    pub async fn add_account_to_group(&self, group_id: &str, account_id: String) -> Result<()> {
        let inner = self.inner.read().await;

        // 检查账户是否存在
        if !inner.accounts.contains_key(&account_id) {
            return Err(QuantixError::Other(format!("账户不存在: {}", account_id)));
        }
        drop(inner);

        let mut inner = self.inner.write().await;
        let group = inner
            .groups
            .get_mut(group_id)
            .ok_or_else(|| QuantixError::Other(format!("账户组不存在: {}", group_id)))?;

        group.add_account(account_id);
        Ok(())
    }

    /// 从账户组移除账户
    pub async fn remove_account_from_group(&self, group_id: &str, account_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;
        let group = inner
            .groups
            .get_mut(group_id)
            .ok_or_else(|| QuantixError::Other(format!("账户组不存在: {}", group_id)))?;

        group.remove_account(account_id);
        Ok(())
    }

    /// 设置账户组分配策略
    pub async fn set_group_allocation_strategy(
        &self,
        group_id: &str,
        strategy: AllocationStrategy,
    ) -> Result<()> {
        let mut inner = self.inner.write().await;
        let group = inner
            .groups
            .get_mut(group_id)
            .ok_or_else(|| QuantixError::Other(format!("账户组不存在: {}", group_id)))?;

        group.allocation_strategy = strategy;
        group.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// 列出所有账户组
    pub async fn list_groups(&self) -> Vec<AccountGroup> {
        let inner = self.inner.read().await;
        inner.groups.values().cloned().collect()
    }

    /// 获取账户所属的组
    pub async fn get_account_groups(&self, account_id: &str) -> Vec<AccountGroup> {
        let inner = self.inner.read().await;
        inner
            .groups
            .values()
            .filter(|g| g.account_ids.contains(&account_id.to_string()))
            .cloned()
            .collect()
    }

    // ========== 导入导出 ==========

    /// 导出所有账户配置
    pub async fn export_accounts(&self) -> HashMap<String, AccountConfig> {
        let inner = self.inner.read().await;
        inner.accounts.clone()
    }

    /// 导出所有账户组
    pub async fn export_groups(&self) -> HashMap<String, AccountGroup> {
        let inner = self.inner.read().await;
        inner.groups.clone()
    }

    /// 获取默认账户 ID
    pub async fn get_default_account_id(&self) -> String {
        let inner = self.inner.read().await;
        inner.default_account_id.clone()
    }
}

fn validate_account_config(config: &AccountConfig) -> Result<()> {
    if config.initial_capital <= rust_decimal::Decimal::ZERO {
        return Err(QuantixError::Other(format!(
            "账户初始资金必须大于 0: {} = {}",
            config.account_id, config.initial_capital
        )));
    }

    Ok(())
}

impl Default for AccountRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_register_and_get_account() {
        let registry = AccountRegistry::new();
        let config = AccountConfig::new("test-001".to_string(), AccountType::Paper, dec!(100000));

        registry.register_account(config.clone()).await.unwrap();
        let retrieved = registry.get_account("test-001").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().account_id, "test-001");
    }

    #[tokio::test]
    async fn test_duplicate_account() {
        let registry = AccountRegistry::new();
        let config = AccountConfig::new("test-001".to_string(), AccountType::Paper, dec!(100000));

        registry.register_account(config.clone()).await.unwrap();
        let result = registry.register_account(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_group() {
        let registry = AccountRegistry::new();

        // 创建账户
        let account1 = AccountConfig::new("acc-1".to_string(), AccountType::Paper, dec!(100000));
        let account2 = AccountConfig::new("acc-2".to_string(), AccountType::Paper, dec!(200000));
        registry.register_account(account1).await.unwrap();
        registry.register_account(account2).await.unwrap();

        // 创建账户组
        registry
            .create_group("group-1".to_string(), "Test Group".to_string())
            .await
            .unwrap();

        // 添加账户到组
        registry
            .add_account_to_group("group-1", "acc-1".to_string())
            .await
            .unwrap();
        registry
            .add_account_to_group("group-1", "acc-2".to_string())
            .await
            .unwrap();

        let group = registry.get_group("group-1").await.unwrap();
        assert_eq!(group.account_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_default_account() {
        let registry = AccountRegistry::new();
        let config = AccountConfig::new("default".to_string(), AccountType::Paper, dec!(100000));

        registry.register_account(config).await.unwrap();
        registry.set_default_account("default").await.unwrap();

        let default = registry.get_default_account().await;
        assert!(default.is_some());
        assert_eq!(default.unwrap().account_id, "default");
    }

    #[tokio::test]
    async fn test_register_rejects_non_positive_capital() {
        let registry = AccountRegistry::new();

        let zero_result = registry
            .register_account(AccountConfig::new(
                "zero".to_string(),
                AccountType::Paper,
                dec!(0),
            ))
            .await;
        assert!(zero_result.is_err());
        assert!(zero_result.unwrap_err().to_string().contains("必须大于 0"));

        let negative_result = registry
            .register_account(AccountConfig::new(
                "negative".to_string(),
                AccountType::Paper,
                dec!(-1),
            ))
            .await;
        assert!(negative_result.is_err());
        assert!(
            negative_result
                .unwrap_err()
                .to_string()
                .contains("必须大于 0")
        );
    }

    #[tokio::test]
    async fn test_update_rejects_non_positive_capital() {
        let registry = AccountRegistry::new();
        let mut config =
            AccountConfig::new("test-001".to_string(), AccountType::Paper, dec!(100000));

        registry.register_account(config.clone()).await.unwrap();

        config.initial_capital = dec!(0);
        let result = registry.update_account(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("必须大于 0"));
    }
}
