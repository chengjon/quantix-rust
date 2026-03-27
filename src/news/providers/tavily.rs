//! Tavily 新闻搜索提供商
//!
//! 高质量 AI 友好的搜索 API

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::core::{QuantixError, Result};
use super::super::provider::NewsProvider;
use super::super::types::{NewsArticle, NewsProviderConfig, NewsSearchRequest, NewsSearchResult};

/// Tavily 提供商
pub struct TavilyProvider {
    config: NewsProviderConfig,
    client: Client,
}

/// Tavily API 请求
#[derive(Serialize)]
struct TavilyRequest {
    query: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include_domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_domains: Vec<String>,
    #[serde(rename = "search_depth")]
    search_depth: String,
    #[serde(rename = "max_results")]
    max_results: usize,
    #[serde(rename = "include_answer")]
    include_answer: bool,
    #[serde(rename = "include_raw_content")]
    include_raw_content: bool,
    #[serde(rename = "include_images")]
    include_images: bool,
    topic: String,
}

/// Tavily API 响应
#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
    answer: Option<String>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    raw_content: Option<String>,
    score: Option<f64>,
    published_date: Option<String>,
}

impl TavilyProvider {
    /// 创建新的 Tavily 提供商
    pub fn new(config: NewsProviderConfig) -> Result<Self> {
        let api_key = config.api_key.clone().ok_or_else(|| {
            QuantixError::Config("Tavily API key is required".to_string())
        })?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// 从环境变量创建
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("TAVILY_API_KEY")
            .or_else(|_| std::env::var("TAVILY_API_KEYS"))
            .map_err(|_| QuantixError::Config("TAVILY_API_KEY not set".to_string()))?;

        let config = NewsProviderConfig {
            enabled: true,
            priority: 1,
            api_key: Some(api_key),
            base_url: Some("https://api.tavily.com".to_string()),
            timeout_seconds: 30,
            daily_limit: None,
            ..Default::default()
        };

        Self::new(config)
    }
}

#[async_trait]
impl NewsProvider for TavilyProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
        let start = Instant::now();
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            QuantixError::Config("Tavily API key not configured".to_string())
        })?;

        let tavily_request = TavilyRequest {
            query: request.query.clone(),
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            search_depth: "basic".to_string(),
            max_results: request.max_results,
            include_answer: true,
            include_raw_content: request.include_content,
            include_images: false,
            topic: "news".to_string(),
        };

        let url = format!("{}/search", self.config.base_url.as_deref().unwrap_or("https://api.tavily.com"));

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&tavily_request)
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantixError::Other(format!("Tavily API error: {} - {}", status, body)));
        }

        let tavily_response: TavilyResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("Failed to parse Tavily response: {}", e)))?;

        let articles: Vec<NewsArticle> = tavily_response
            .results
            .into_iter()
            .map(|r| {
                let mut article = NewsArticle::new(r.title, r.url, "tavily".to_string());
                article.summary = r.content;
                article.content = r.raw_content;
                article.sentiment = r.score;
                article
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(NewsSearchResult::new(articles, "tavily".to_string()).with_elapsed(elapsed))
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }

    fn config(&self) -> &NewsProviderConfig {
        &self.config
    }
}
