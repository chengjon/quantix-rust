//! 新闻搜索数据类型
//!
//! 定义新闻搜索相关的核心数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 新闻文章
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    /// 文章标题
    pub title: String,
    /// 文章链接
    pub url: String,
    /// 来源
    pub source: String,
    /// 发布时间
    pub published_at: Option<DateTime<Utc>>,
    /// 摘要
    pub summary: Option<String>,
    /// 正文内容
    pub content: Option<String>,
    /// 相关股票代码
    pub related_codes: Vec<String>,
    /// 相关标签
    pub tags: Vec<String>,
    /// 图片URL
    pub image_url: Option<String>,
    /// 作者
    pub author: Option<String>,
    /// 语言
    pub language: String,
    /// 情感得分 (-1.0 到 1.0)
    pub sentiment: Option<f64>,
}

impl NewsArticle {
    /// 创建新的新闻文章
    pub fn new(title: String, url: String, source: String) -> Self {
        Self {
            title,
            url,
            source,
            published_at: None,
            summary: None,
            content: None,
            related_codes: Vec::new(),
            tags: Vec::new(),
            image_url: None,
            author: None,
            language: "zh".to_string(),
            sentiment: None,
        }
    }

    /// 添加相关股票代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.related_codes.push(code.into());
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 新闻搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSearchRequest {
    /// 搜索关键词
    pub query: String,
    /// 相关股票代码
    pub codes: Vec<String>,
    /// 时间范围（天数）
    pub days: u32,
    /// 最大结果数
    pub max_results: usize,
    /// 指定提供商
    pub provider: Option<String>,
    /// 语言
    pub language: Option<String>,
    /// 是否包含全文
    pub include_content: bool,
}

impl Default for NewsSearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            codes: Vec::new(),
            days: 3,
            max_results: 20,
            provider: None,
            language: Some("zh".to_string()),
            include_content: false,
        }
    }
}

impl NewsSearchRequest {
    /// 创建新的搜索请求
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// 设置股票代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.codes.push(code.into());
        self
    }

    /// 设置时间范围
    pub fn with_days(mut self, days: u32) -> Self {
        self.days = days;
        self
    }

    /// 设置最大结果数
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
}

/// 新闻搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSearchResult {
    /// 搜索到的文章列表
    pub articles: Vec<NewsArticle>,
    /// 总结果数
    pub total: usize,
    /// 搜索耗时（毫秒）
    pub elapsed_ms: u64,
    /// 使用的提供商
    pub provider: String,
    /// 是否来自缓存
    pub from_cache: bool,
    /// 搜索时间
    pub searched_at: DateTime<Utc>,
}

impl NewsSearchResult {
    /// 创建新的搜索结果
    pub fn new(articles: Vec<NewsArticle>, provider: String) -> Self {
        let total = articles.len();
        Self {
            articles,
            total,
            elapsed_ms: 0,
            provider,
            from_cache: false,
            searched_at: Utc::now(),
        }
    }

    /// 设置耗时
    pub fn with_elapsed(mut self, ms: u64) -> Self {
        self.elapsed_ms = ms;
        self
    }

    /// 标记为缓存结果
    pub fn from_cache(mut self) -> Self {
        self.from_cache = true;
        self
    }
}

/// 新闻提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsProviderConfig {
    /// 是否启用
    pub enabled: bool,
    /// 优先级（数字越小优先级越高）
    pub priority: u8,
    /// API Key 环境变量名
    pub api_key_env: Option<String>,
    /// API Key 值
    pub api_key: Option<String>,
    /// Base URL
    pub base_url: Option<String>,
    /// 请求超时（秒）
    pub timeout_seconds: u64,
    /// 每日请求限制
    pub daily_limit: Option<u32>,
}

impl Default for NewsProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 10,
            api_key_env: None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            daily_limit: None,
        }
    }
}

/// 新闻趋势项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsTrend {
    /// 日期
    pub date: chrono::NaiveDate,
    /// 文章数量
    pub article_count: usize,
    /// 热门关键词
    pub hot_keywords: Vec<KeywordCount>,
    /// 情感分布
    pub sentiment_distribution: SentimentDistribution,
    /// 热门来源
    pub top_sources: Vec<SourceCount>,
}

/// 关键词计数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordCount {
    pub keyword: String,
    pub count: usize,
}

/// 来源计数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCount {
    pub source: String,
    pub count: usize,
}

/// 情感分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentDistribution {
    pub positive: usize,
    pub neutral: usize,
    pub negative: usize,
}

impl Default for SentimentDistribution {
    fn default() -> Self {
        Self {
            positive: 0,
            neutral: 0,
            negative: 0,
        }
    }
}
