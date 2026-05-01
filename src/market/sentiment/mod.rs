//! 市场舆情分析模块
//!
//! 提供美股/港股情绪分析能力

pub mod aggregator;
pub mod provider;
pub mod types;

pub use aggregator::SentimentAggregator;
pub use provider::SentimentProvider;
pub use types::{SentimentData, SentimentScore, SocialMention};
