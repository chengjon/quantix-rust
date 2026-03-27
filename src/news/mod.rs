//! 新闻搜索模块
//!
//! 提供多源新闻搜索和聚合能力

pub mod types;
pub mod provider;
pub mod providers;
pub mod aggregator;
pub mod cache;

pub use types::{NewsArticle, NewsSearchRequest, NewsSearchResult, NewsProviderConfig};
pub use provider::NewsProvider;
pub use aggregator::NewsAggregator;
pub use cache::NewsCache;
