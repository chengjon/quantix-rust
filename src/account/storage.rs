//! Account Storage
//!
//! 账户注册表持久化存储

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{QuantixError, Result};
use super::models::{AccountConfig, AccountGroup};
use super::registry::AccountRegistry;

/// 账户注册表存储接口
#[async_trait::async_trait]
pub trait AccountRegistryStore: Send + Sync {
    /// 加载注册表
    async fn load(&self) -> Result<Option<AccountRegistryData>>;

    /// 保存注册表
    async fn save(&self, data: &AccountRegistryData) -> Result<()>;
}

/// 账户注册表数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountRegistryData {
    /// 版本
    pub version: u32,
    /// 账户配置
    pub accounts: std::collections::HashMap<String, AccountConfig>,
    /// 账户组
    pub groups: std::collections::HashMap<String, AccountGroup>,
    /// 默认账户 ID
    pub default_account_id: String,
}

impl Default for AccountRegistryData {
    fn default() -> Self {
        Self {
            version: 1,
            accounts: std::collections::HashMap::new(),
            groups: std::collections::HashMap::new(),
            default_account_id: "default".to_string(),
        }
    }
}

/// JSON 文件存储
#[derive(Debug)]
pub struct JsonAccountRegistryStore {
    path: PathBuf,
    cache: Arc<RwLock<Option<AccountRegistryData>>>,
}

impl JsonAccountRegistryStore {
    /// 创建新的 JSON 存储
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取默认存储路径
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".quantix").join("accounts").join("registry.json")
    }

    /// 创建默认存储
    pub fn default_store() -> Self {
        Self::new(Self::default_path())
    }

    /// 确保目录存在
    fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    QuantixError::Other(format!("无法创建目录 {}: {}", parent.display(), e))
                })?;
            }
        }
        Ok(())
    }

    /// 刷新缓存
    pub async fn refresh_cache(&self) -> Result<()> {
        let data = self.load().await?;
        let mut cache = self.cache.write().await;
        *cache = data;
        Ok(())
    }

    /// 从缓存获取
    pub async fn get_cached(&self) -> Option<AccountRegistryData> {
        let cache = self.cache.read().await;
        cache.clone()
    }
}

#[async_trait::async_trait]
impl AccountRegistryStore for JsonAccountRegistryStore {
    async fn load(&self) -> Result<Option<AccountRegistryData>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| QuantixError::Other(format!("无法读取文件: {}", e)))?;

        let data: AccountRegistryData = serde_json::from_str(&content)
            .map_err(|e| QuantixError::Other(format!("JSON 解析失败: {}", e)))?;

        // 更新缓存
        let mut cache = self.cache.write().await;
        *cache = Some(data.clone());

        Ok(Some(data))
    }

    async fn save(&self, data: &AccountRegistryData) -> Result<()> {
        self.ensure_parent_dir()?;

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| QuantixError::Other(format!("JSON 序列化失败: {}", e)))?;

        tokio::fs::write(&self.path, content)
            .await
            .map_err(|e| QuantixError::Other(format!("无法写入文件: {}", e)))?;

        // 更新缓存
        let mut cache = self.cache.write().await;
        *cache = Some(data.clone());

        Ok(())
    }
}

/// 从存储加载注册表
pub async fn load_registry(store: &dyn AccountRegistryStore) -> Result<AccountRegistry> {
    match store.load().await? {
        Some(data) => Ok(AccountRegistry::with_accounts(
            data.accounts,
            data.default_account_id,
        )),
        None => Ok(AccountRegistry::new()),
    }
}

/// 保存注册表到存储
pub async fn save_registry(
    store: &dyn AccountRegistryStore,
    registry: &AccountRegistry,
) -> Result<()> {
    let data = AccountRegistryData {
        version: 1,
        accounts: registry.export_accounts().await,
        groups: registry.export_groups().await,
        default_account_id: registry.get_default_account_id().await,
    };

    store.save(&data).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_registry.json");
        let store = JsonAccountRegistryStore::new(&path);

        // 保存数据
        let data = AccountRegistryData {
            version: 1,
            accounts: std::collections::HashMap::new(),
            groups: std::collections::HashMap::new(),
            default_account_id: "test-default".to_string(),
        };

        store.save(&data).await.unwrap();

        // 加载数据
        let loaded = store.load().await.unwrap();
        assert!(loaded.is_some());
        let loaded_data = loaded.unwrap();
        assert_eq!(loaded_data.default_account_id, "test-default");
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let store = JsonAccountRegistryStore::new(&path);

        let result = store.load().await.unwrap();
        assert!(result.is_none());
    }
}
