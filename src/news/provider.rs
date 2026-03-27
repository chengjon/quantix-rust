//! 新闻提供商 Trait 定义
//!
//! 定义统一的新闻搜索接口

use async_trait::async_trait;
use crate::core::Result;
use super::types::{NewsSearchRequest, NewsSearchResult, NewsProviderConfig};

/// 新闻提供商 Trait
///
/// 所有新闻源都需要实现此接口
#[async_trait]
pub trait NewsProvider: Send + Sync {
    /// 提供商名称
    fn name(&self) -> &'static str;

    /// 搜索新闻
    async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult>;

    /// 根据股票代码搜索新闻
    async fn search_by_code(&self, code: &str, days: u32, max_results: usize) -> Result<NewsSearchResult> {
        let request = NewsSearchRequest::new(code)
            .with_code(code)
            .with_days(days)
            .with_max_results(max_results);
        self.search(&request).await
    }

    /// 检查提供商是否可用
    fn is_available(&self) -> bool {
        true
    }

    /// 获取提供商配置
    fn config(&self) -> &NewsProviderConfig;

    /// 获取剩余配额（如果有限制）
    fn remaining_quota(&self) -> Option<u32> {
        None
    }
}

/// 新闻提供商构建器 Trait
pub trait NewsProviderBuilder: Send + Sync {
    /// 提供商名称
    fn name(&self) -> &'static str;

    /// 从配置创建提供商
    fn build(&self, config: NewsProviderConfig) -> Result<Box<dyn NewsProvider>>;

    /// 检查配置是否有效
    fn validate_config(&self, config: &NewsProviderConfig) -> bool {
        config.enabled && config.api_key.is_some()
    }
}
