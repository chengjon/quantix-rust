//! 新闻搜索模块
//!
//! 提供多源新闻搜索和聚合能力

pub mod aggregator;
pub mod cache;
pub mod provider;
pub mod providers;
pub mod types;

pub use aggregator::NewsAggregator;
pub use cache::NewsCache;
pub use provider::NewsProvider;
pub use types::{NewsArticle, NewsProviderConfig, NewsSearchRequest, NewsSearchResult};
