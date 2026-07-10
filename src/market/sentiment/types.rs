//! 舆情数据类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 情绪数据汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentData {
    /// 股票代码
    pub code: String,
    /// 数据时间
    pub timestamp: DateTime<Utc>,
    /// 综合情绪得分 (-1.0 到 1.0)
    pub overall_score: f64,
    /// 情绪等级
    pub sentiment_level: SentimentLevel,
    /// 各来源得分
    pub source_scores: Vec<SentimentScore>,
    /// 最近提及
    pub recent_mentions: Vec<SocialMention>,
    /// 趋势方向
    pub trend: SentimentTrend,
    /// 数据来源
    pub sources: Vec<String>,
}

/// 情绪等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentLevel {
    /// 非常看空
    VeryBearish,
    /// 看空
    Bearish,
    /// 中性偏空
    SlightlyBearish,
    /// 中性
    Neutral,
    /// 中性偏多
    SlightlyBullish,
    /// 看多
    Bullish,
    /// 非常看多
    VeryBullish,
}

impl SentimentLevel {
    /// 把 [-1, 1] 区间的得分映射到 7 档情绪：≥0.6 VeryBullish、≥0.4 Bullish、≥0.2 SlightlyBullish、≥-0.2 Neutral、≥-0.4 SlightlyBearish、≥-0.6 Bearish、其余 VeryBearish。
    pub fn from_score(score: f64) -> Self {
        if score >= 0.6 {
            Self::VeryBullish
        } else if score >= 0.4 {
            Self::Bullish
        } else if score >= 0.2 {
            Self::SlightlyBullish
        } else if score >= -0.2 {
            Self::Neutral
        } else if score >= -0.4 {
            Self::SlightlyBearish
        } else if score >= -0.6 {
            Self::Bearish
        } else {
            Self::VeryBearish
        }
    }

    /// 返回对应 emoji（🚀/📈/🙂/😐/😟/📉/🔥），用于 CLI 与报告可视化。
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::VeryBullish => "🚀",
            Self::Bullish => "📈",
            Self::SlightlyBullish => "🙂",
            Self::Neutral => "😐",
            Self::SlightlyBearish => "😟",
            Self::Bearish => "📉",
            Self::VeryBearish => "🔥",
        }
    }

    /// 返回中文情绪标签（"非常看多"/"看多"/"中性偏多"/"中性"/"中性偏空"/"看空"/"非常看空"）。
    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryBullish => "非常看多",
            Self::Bullish => "看多",
            Self::SlightlyBullish => "中性偏多",
            Self::Neutral => "中性",
            Self::SlightlyBearish => "中性偏空",
            Self::Bearish => "看空",
            Self::VeryBearish => "非常看空",
        }
    }
}

/// 情绪趋势
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentTrend {
    /// 趋势不可用
    Unavailable,
    /// 快速上升
    RisingFast,
    /// 上升
    Rising,
    /// 平稳
    Stable,
    /// 下降
    Falling,
    /// 快速下降
    FallingFast,
}

/// 单个来源的情绪得分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    /// 来源名称
    pub source: String,
    /// 得分 (-1.0 到 1.0)
    pub score: f64,
    /// 样本数量
    pub sample_count: usize,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 社交媒体提及
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMention {
    /// 平台
    pub platform: String,
    /// 内容摘要
    pub content: String,
    /// 作者
    pub author: Option<String>,
    /// 发布时间
    pub published_at: Option<DateTime<Utc>>,
    /// 点赞/互动数
    pub engagement: Option<u64>,
    /// 情绪得分
    pub sentiment: Option<f64>,
    /// 链接
    pub url: Option<String>,
}

/// 情绪历史数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentHistoryPoint {
    /// 时间
    pub timestamp: DateTime<Utc>,
    /// 情绪得分
    pub score: f64,
    /// 样本数量
    pub sample_count: usize,
}
