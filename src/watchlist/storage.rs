use crate::core::Result;
use crate::watchlist::models::WatchlistStore;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WatchlistStorage {
    path: PathBuf,
}

impl WatchlistStorage {
    /// 构造指向指定 path 的 JSON 存储适配器；不读不写，仅记录路径。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 返回底层 JSON 文件路径（用于日志/diagnostics）。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 读取并反序列化 watchlist；文件不存在返回 Ok(None)，读取或 JSON 解析失败透传。
    pub fn load(&self) -> Result<Option<WatchlistStore>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&self.path)?;
        let store = serde_json::from_str(&raw)?;
        Ok(Some(store))
    }

    /// 读取已有 watchlist；若文件不存在则用默认空 store 立即写盘并返回。读取、序列化或写入失败透传。
    pub fn load_or_create(&self) -> Result<WatchlistStore> {
        if let Some(store) = self.load()? {
            return Ok(store);
        }

        let store = WatchlistStore::default();
        self.save(&store)?;
        Ok(store)
    }

    /// 序列化为 pretty JSON 并写盘；自动创建父目录。序列化、目录创建或写入失败透传。
    pub fn save(&self, store: &WatchlistStore) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_string_pretty(store)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }
}
