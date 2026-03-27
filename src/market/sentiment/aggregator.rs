//! 舆情数据聚合器

use std::sync::Arc;
use chrono::Utc;
use crate::core::Result;
use super::provider::SentimentProvider;
use super::types::{SentimentData, SentimentScore, SentimentLevel, SentimentTrend, SocialMention};

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
        let mut source_scores = Vec::new();
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }

            if let Ok(score) = provider.get_score(code).await {
                let weight = score.sample_count as f64;
                total_score += score.score * weight;
                total_weight += weight;
                source_scores.push(score);
            }
        }

        let overall_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };

        let sentiment_level = SentimentLevel::from_score(overall_score);

        // 收集最近提及
        let mut recent_mentions = Vec::new();
        for provider in &self.providers {
            if provider.is_available() {
                if let Ok(mentions) = provider.get_mentions(code, 5).await {
                    recent_mentions.extend(mentions);
                }
            }
        }

        // 按时间排序
        recent_mentions.sort_by(|a, b| {
            b.published_at.cmp(&a.published_at)
        });
        recent_mentions.truncate(20);

        let sources: Vec<String> = self.providers.iter()
            .filter(|p| p.is_available())
            .map(|p| p.name().to_string())
            .collect();

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
}
