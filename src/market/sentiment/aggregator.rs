//! 舆情数据聚合器

use chrono::Utc;
use crate::core::Result;
use super::provider::SentimentProvider;
use super::types::{SentimentData, SentimentHistoryPoint, SentimentLevel, SentimentTrend, SocialMention};

/// 舆情聚合器
pub struct SentimentAggregator {
    providers: Vec<Box<dyn SentimentProvider>>,
}

impl SentimentAggregator {
    /// 创建新的聚合器
    pub fn new(providers: Vec<Box<dyn SentimentProvider>>) -> Self {
        Self { providers }
    }

    /// 获取聚合情绪数据
    pub async fn get_sentiment(&self, code: &str) -> Result<SentimentData> {
        let source_scores =
            super::aggregator_support::collect_source_scores(&self.providers, code).await;
        let overall_score = super::aggregator_support::compute_overall_score(&source_scores);
        let sentiment_level = SentimentLevel::from_score(overall_score);
        let recent_mentions =
            super::aggregator_support::collect_recent_mentions(&self.providers, code, 5, 20).await;
        let sources = super::aggregator_support::available_provider_names(&self.providers);

        Ok(SentimentData {
            code: code.to_string(),
            timestamp: Utc::now(),
            overall_score,
            sentiment_level,
            source_scores,
            recent_mentions,
            trend: SentimentTrend::Stable, // TODO: 计算趋势
            sources,
        })
    }

    /// 获取可用提供商列表
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.iter()
            .filter(|p| p.is_available())
            .map(|p| p.name())
            .collect()
    }

    /// 获取社交媒体提及
    pub async fn get_mentions(&self, code: &str, limit: usize) -> Result<Vec<SocialMention>> {
        let mut mentions = Vec::new();
        for provider in &self.providers {
            if provider.is_available() {
                if let Ok(m) = provider.get_mentions(code, limit).await {
                    mentions.extend(m);
                }
            }
        }
        mentions.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        mentions.truncate(limit);
        Ok(mentions)
    }

    /// 获取历史情绪数据
    pub async fn get_history(&self, code: &str, days: u32) -> Result<Vec<SentimentHistoryPoint>> {
        let mut all_history = Vec::new();
        for provider in &self.providers {
            if provider.is_available() {
                if let Ok(history) = provider.get_history(code, days).await {
                    all_history.extend(history);
                }
            }
        }
        all_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(all_history)
    }
}
