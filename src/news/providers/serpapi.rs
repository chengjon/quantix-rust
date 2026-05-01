//! SerpAPI 新闻搜索提供商
//!
//! Google 搜索结果的 API 接口

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Instant;

use super::super::provider::NewsProvider;
use super::super::types::{NewsArticle, NewsProviderConfig, NewsSearchRequest, NewsSearchResult};
use crate::core::{QuantixError, Result};

/// SerpAPI 提供商
pub struct SerpApiProvider {
    config: NewsProviderConfig,
    client: Client,
}

/// SerpAPI 响应
#[derive(Deserialize)]
struct SerpApiResponse {
    news_results: Option<Vec<SerpApiNewsItem>>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct SerpApiNewsItem {
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
    #[serde(rename = "date")]
    _date: Option<String>,
    source: Option<String>,
    thumbnail: Option<String>,
}

impl SerpApiProvider {
    /// 创建新的 SerpAPI 提供商
    pub fn new(config: NewsProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// 从环境变量创建
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("SERPAPI_API_KEY")
            .or_else(|_| std::env::var("SERPAPI_API_KEYS"))
            .map_err(|_| QuantixError::Config("SERPAPI_API_KEY not set".to_string()))?;

        let config = NewsProviderConfig {
            enabled: true,
            priority: 2,
            api_key: Some(api_key),
            base_url: Some("https://serpapi.com".to_string()),
            timeout_seconds: 30,
            daily_limit: None,
            ..Default::default()
        };

        Self::new(config)
    }
}

#[async_trait]
impl NewsProvider for SerpApiProvider {
    fn name(&self) -> &'static str {
        "serpapi"
    }

    async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
        let start = Instant::now();
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| QuantixError::Config("SerpAPI key not configured".to_string()))?;

        let query = if !request.codes.is_empty() {
            format!("{} {}", request.query, request.codes.join(" "))
        } else {
            request.query.clone()
        };

        let url = format!(
            "{}/search?engine=google_news&q={}&api_key={}&num={}",
            self.config
                .base_url
                .as_deref()
                .unwrap_or("https://serpapi.com"),
            urlencoding::encode(&query),
            api_key,
            request.max_results
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantixError::Other(format!(
                "SerpAPI error: {} - {}",
                status, body
            )));
        }

        let serp_response: SerpApiResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("Failed to parse SerpAPI response: {}", e)))?;

        if let Some(error) = serp_response.error {
            return Err(QuantixError::Other(format!("SerpAPI error: {}", error)));
        }

        let articles: Vec<NewsArticle> = serp_response
            .news_results
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let title = item.title?;
                let url = item.link?;
                let mut article = NewsArticle::new(
                    title,
                    url,
                    item.source.unwrap_or_else(|| "unknown".to_string()),
                );
                article.summary = item.snippet;
                article.image_url = item.thumbnail;
                Some(article)
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(NewsSearchResult::new(articles, "serpapi".to_string()).with_elapsed(elapsed))
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }

    fn config(&self) -> &NewsProviderConfig {
        &self.config
    }
}

// URL encoding module (simple implementation)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
