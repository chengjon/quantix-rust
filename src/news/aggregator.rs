//! 新闻聚合器
//!
//! 多源聚合、去重、排序

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use crate::core::{QuantixError, Result};
use super::provider::NewsProvider;
use super::types::{NewsArticle, NewsSearchRequest, NewsSearchResult};
use super::cache::NewsCache;

/// 新闻聚合器
///
/// 管理多个新闻源，提供统一的搜索接口
pub struct NewsAggregator {
    providers: Vec<Box<dyn NewsProvider>>,
    cache: Option<Arc<NewsCache>>,
    config: AggregatorConfig,
}

/// 聚合器配置
#[derive(Debug, Clone)]
pub struct AggregatorConfig {
    /// 启用的提供商列表
    pub enabled_providers: Vec<String>,
    /// 最大并发请求数
    pub max_concurrent: usize,
    /// 请求超时（秒）
    pub timeout_seconds: u64,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存 TTL（秒）
    pub cache_ttl_seconds: u64,
    /// 失败时是否 fallback 到下一个提供商
    pub fallback_on_error: bool,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            enabled_providers: vec!["tavily".to_string(), "serpapi".to_string(), "bocha".to_string()],
            max_concurrent: 3,
            timeout_seconds: 30,
            enable_cache: true,
            cache_ttl_seconds: 3600,
            fallback_on_error: true,
        }
    }
}

impl NewsAggregator {
    /// 创建新的聚合器
    pub fn new(providers: Vec<Box<dyn NewsProvider>>, config: AggregatorConfig) -> Self {
        Self {
            providers,
            cache: None,
            config,
        }
    }

    /// 添加缓存
    pub fn with_cache(mut self, cache: Arc<NewsCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// 搜索新闻
    ///
    /// 按优先级尝试各个提供商，直到成功或全部失败
    pub async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
        let start = Instant::now();

        // 检查缓存
        if let Some(cache) = &self.cache {
            if self.config.enable_cache {
                if let Some(cached) = cache.get(&request.query).await {
                    let mut result = cached;
                    result.from_cache = true;
                    return Ok(result);
                }
            }
        }

        // 获取可用的提供商（按优先级排序）
        let available_providers: Vec<_> = self.providers.iter()
            .filter(|p| p.is_available())
            .filter(|p| {
                if let Some(ref provider) = request.provider {
                    p.name() == provider
                } else {
                    self.config.enabled_providers.contains(&p.name().to_string())
                }
            })
            .collect();

        if available_providers.is_empty() {
            return Err(QuantixError::Other("No news providers available".to_string()));
        }

        // 尝试每个提供商
        let mut last_error = None;
        for provider in available_providers {
            match provider.search(request).await {
                Ok(result) => {
                    // 去重和排序
                    let deduped = self.deduplicate(result.articles);

                    // 缓存结果
                    if let Some(cache) = &self.cache {
                        if self.config.enable_cache {
                            let cache_result = NewsSearchResult::new(
                                deduped.clone(),
                                result.provider.clone(),
                            ).with_elapsed(result.elapsed_ms);
                            cache.set(&request.query, &cache_result, self.config.cache_ttl_seconds).await;
                        }
                    }

                    return Ok(NewsSearchResult::new(deduped, result.provider)
                        .with_elapsed(start.elapsed().as_millis() as u64));
                }
                Err(e) => {
                    tracing::warn!("News provider {} failed: {}", provider.name(), e);
                    last_error = Some(e);

                    if !self.config.fallback_on_error {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| QuantixError::Other("All news providers failed".to_string())))
    }

    /// 并行搜索多个提供商并合并结果
    pub async fn search_parallel(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
        let start = Instant::now();

        // 检查缓存
        if let Some(cache) = &self.cache {
            if self.config.enable_cache {
                if let Some(cached) = cache.get(&request.query).await {
                    let mut result = cached;
                    result.from_cache = true;
                    return Ok(result);
                }
            }
        }

        // 并行请求所有可用的提供商
        let available_providers: Vec<_> = self.providers.iter()
            .filter(|p| p.is_available())
            .filter(|p| {
                if let Some(ref provider) = request.provider {
                    p.name() == provider
                } else {
                    self.config.enabled_providers.contains(&p.name().to_string())
                }
            })
            .collect();

        if available_providers.is_empty() {
            return Err(QuantixError::Other("No news providers available".to_string()));
        }

        // 使用 tokio 并行执行
        let mut tasks = Vec::new();
        for provider in available_providers {
            let request = request.clone();
            let task = async move {
                provider.search(&request).await
            };
            tasks.push(task);
        }

        // 等待所有任务完成
        let results: Vec<_> = futures::future::join_all(tasks).await;

        // 合并结果
        let mut all_articles = Vec::new();
        let mut used_providers = Vec::new();

        for result in results {
            if let Ok(r) = result {
                all_articles.extend(r.articles);
                used_providers.push(r.provider);
            }
        }

        if all_articles.is_empty() {
            return Err(QuantixError::Other("All news providers returned no results".to_string()));
        }

        // 去重和排序
        let deduped = self.deduplicate(all_articles);

        let result = NewsSearchResult::new(deduped, used_providers.join(","))
            .with_elapsed(start.elapsed().as_millis() as u64);

        // 缓存结果
        if let Some(cache) = &self.cache {
            if self.config.enable_cache {
                cache.set(&request.query, &result, self.config.cache_ttl_seconds).await;
            }
        }

        Ok(result)
    }

    /// 去重文章
    fn deduplicate(&self, articles: Vec<NewsArticle>) -> Vec<NewsArticle> {
        let mut seen_urls = HashSet::new();
        let mut seen_titles = HashSet::new();
        let mut result = Vec::new();

        for article in articles {
            // URL 去重
            if seen_urls.contains(&article.url) {
                continue;
            }
            seen_urls.insert(article.url.clone());

            // 标题相似度去重（简单版本：完全匹配）
            let title_lower = article.title.to_lowercase();
            if seen_titles.contains(&title_lower) {
                continue;
            }
            seen_titles.insert(title_lower);

            result.push(article);
        }

        // 按发布时间排序（最新的在前）
        result.sort_by(|a, b| {
            match (&a.published_at, &b.published_at) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        result
    }

    /// 获取可用的提供商列表
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.iter()
            .filter(|p| p.is_available())
            .map(|p| p.name())
            .collect()
    }

    /// 添加提供商
    pub fn add_provider(&mut self, provider: Box<dyn NewsProvider>) {
        self.providers.push(provider);
    }
}
