//! 新闻缓存
//!
//! 内存缓存实现

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use super::types::NewsSearchResult;

/// 缓存条目
struct CacheEntry {
    result: NewsSearchResult,
    expires_at: Instant,
}

/// 新闻缓存
pub struct NewsCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    max_size: usize,
    default_ttl: Duration,
}

impl NewsCache {
    /// 创建新的缓存
    pub fn new(max_size: usize, default_ttl_seconds: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            default_ttl: Duration::from_secs(default_ttl_seconds),
        }
    }

    /// 获取缓存
    pub async fn get(&self, query: &str) -> Option<NewsSearchResult> {
        let cache = self.cache.read().await;
        cache.get(query).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.result.clone())
            } else {
                None
            }
        })
    }

    /// 设置缓存
    pub async fn set(&self, query: &str, result: &NewsSearchResult, ttl_seconds: u64) {
        let mut cache = self.cache.write().await;

        // 简单的 LRU：如果超过最大大小，删除过期条目
        if cache.len() >= self.max_size {
            let now = Instant::now();
            cache.retain(|_, entry| entry.expires_at > now);

            // 如果还是太大，删除最早的一半
            if cache.len() >= self.max_size {
                let to_remove = cache.len() / 2;
                let mut keys: Vec<_> = cache.keys().cloned().collect();
                keys.truncate(to_remove);
                for key in keys {
                    cache.remove(&key);
                }
            }
        }

        cache.insert(
            query.to_lowercase(),
            CacheEntry {
                result: result.clone(),
                expires_at: Instant::now() + Duration::from_secs(ttl_seconds),
            },
        );
    }

    /// 清除缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 清除过期缓存
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// 获取缓存统计
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Instant::now();
        let valid = cache.values().filter(|e| e.expires_at > now).count();
        let expired = cache.len() - valid;

        CacheStats {
            total_entries: cache.len(),
            valid_entries: valid,
            expired_entries: expired,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
}

impl Default for NewsCache {
    fn default() -> Self {
        Self::new(1000, 3600) // 默认 1000 条，1 小时过期
    }
}
