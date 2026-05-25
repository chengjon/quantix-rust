//! 舆情数据聚合器
#![allow(clippy::collapsible_if)]

use super::provider::SentimentProvider;
use super::types::{
    SentimentData, SentimentHistoryPoint, SentimentLevel, SentimentTrend, SocialMention,
};
use crate::core::Result;
use chrono::Utc;

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
        recent_mentions.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        recent_mentions.truncate(20);

        let sources: Vec<String> = self
            .providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.name().to_string())
            .collect();

        let mut history = Vec::new();
        for provider in &self.providers {
            if provider.is_available() {
                if let Ok(points) = provider.get_history(code, 7).await {
                    history.extend(points);
                }
            }
        }

        Ok(SentimentData {
            code: code.to_string(),
            timestamp: Utc::now(),
            overall_score,
            sentiment_level,
            source_scores,
            recent_mentions,
            trend: infer_trend_from_history(&history),
            sources,
        })
    }

    /// 获取可用提供商列表
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
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

fn infer_trend_from_history(history: &[SentimentHistoryPoint]) -> SentimentTrend {
    if history.len() < 2 {
        return SentimentTrend::Unavailable;
    }

    let mut points = history.to_vec();
    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let split_at = points.len() / 2;
    let previous = weighted_average_score(&points[..split_at]);
    let recent = weighted_average_score(&points[split_at..]);
    let delta = recent - previous;

    if delta >= 0.4 {
        SentimentTrend::RisingFast
    } else if delta >= 0.1 {
        SentimentTrend::Rising
    } else if delta <= -0.4 {
        SentimentTrend::FallingFast
    } else if delta <= -0.1 {
        SentimentTrend::Falling
    } else {
        SentimentTrend::Stable
    }
}

fn weighted_average_score(points: &[SentimentHistoryPoint]) -> f64 {
    let mut weighted_score = 0.0;
    let mut total_weight = 0.0;

    for point in points {
        let weight = point.sample_count.max(1) as f64;
        weighted_score += point.score * weight;
        total_weight += weight;
    }

    weighted_score / total_weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::sentiment::types::SentimentScore;
    use async_trait::async_trait;
    use chrono::TimeZone;

    struct StaticSentimentProvider {
        history: Vec<SentimentHistoryPoint>,
    }

    #[async_trait]
    impl SentimentProvider for StaticSentimentProvider {
        fn name(&self) -> &'static str {
            "static"
        }

        async fn get_sentiment(&self, code: &str) -> Result<SentimentData> {
            Ok(SentimentData {
                code: code.to_string(),
                timestamp: Utc::now(),
                overall_score: 0.0,
                sentiment_level: SentimentLevel::Neutral,
                source_scores: Vec::new(),
                recent_mentions: Vec::new(),
                trend: SentimentTrend::Unavailable,
                sources: vec![self.name().to_string()],
            })
        }

        async fn get_score(&self, _code: &str) -> Result<SentimentScore> {
            Ok(SentimentScore {
                source: self.name().to_string(),
                score: 0.45,
                sample_count: 10,
                updated_at: Utc::now(),
            })
        }

        async fn get_mentions(&self, _code: &str, _limit: usize) -> Result<Vec<SocialMention>> {
            Ok(Vec::new())
        }

        async fn get_history(&self, _code: &str, _days: u32) -> Result<Vec<SentimentHistoryPoint>> {
            Ok(self.history.clone())
        }
    }

    #[tokio::test]
    async fn get_sentiment_without_providers_marks_trend_unavailable() {
        let aggregator = SentimentAggregator::new(vec![]);

        let data = aggregator.get_sentiment("000001").await.unwrap();

        assert_eq!(data.trend, SentimentTrend::Unavailable);
    }

    #[tokio::test]
    async fn get_sentiment_derives_trend_from_provider_history() {
        let provider = StaticSentimentProvider {
            history: vec![
                SentimentHistoryPoint {
                    timestamp: Utc.with_ymd_and_hms(2026, 5, 10, 9, 30, 0).unwrap(),
                    score: 0.10,
                    sample_count: 8,
                },
                SentimentHistoryPoint {
                    timestamp: Utc.with_ymd_and_hms(2026, 5, 12, 9, 30, 0).unwrap(),
                    score: 0.36,
                    sample_count: 10,
                },
            ],
        };
        let aggregator = SentimentAggregator::new(vec![Box::new(provider)]);

        let data = aggregator.get_sentiment("000001").await.unwrap();

        assert_eq!(data.trend, SentimentTrend::Rising);
    }
}
