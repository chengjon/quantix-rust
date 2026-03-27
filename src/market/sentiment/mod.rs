//! 市场舆情分析模块
//!
//! 提供美股/港股情绪分析能力

pub mod types;
pub mod provider;
pub mod aggregator;

pub use types::{SentimentData, SentimentScore, SocialMention};
pub use provider::SentimentProvider;
pub use aggregator::SentimentAggregator;
