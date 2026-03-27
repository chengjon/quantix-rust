//! 舆情数据提供商 Trait

use async_trait::async_trait;
use crate::core::Result;
use super::types::{SentimentData, SentimentScore, SocialMention, SentimentHistoryPoint};

/// 舆情数据提供商 Trait
#[async_trait]
pub trait SentimentProvider: Send + Sync {
    /// 提供商名称
    fn name(&self) -> &'static str;

    /// 获取情绪数据
    async fn get_sentiment(&self, code: &str) -> Result<SentimentData>;

    /// 获取情绪得分
    async fn get_score(&self, code: &str) -> Result<SentimentScore>;

    /// 获取最近提及
    async fn get_mentions(&self, code: &str, limit: usize) -> Result<Vec<SocialMention>>;

    /// 获取历史情绪
    async fn get_history(&self, code: &str, days: u32) -> Result<Vec<SentimentHistoryPoint>>;

    /// 检查是否可用
    fn is_available(&self) -> bool {
        true
    }
}
