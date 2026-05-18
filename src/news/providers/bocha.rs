//! 博查新闻搜索提供商
//!
//! 中文优化的新闻搜索 API

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Instant;

use super::super::provider::NewsProvider;
use super::super::types::{NewsArticle, NewsProviderConfig, NewsSearchRequest, NewsSearchResult};
use crate::core::{QuantixError, Result};

/// 博查提供商
pub struct BochaProvider {
    config: NewsProviderConfig,
    client: Client,
}

/// 博查 API 响应
#[derive(Deserialize)]
struct BochaResponse {
    code: Option<i32>,
    message: Option<String>,
    data: Option<BochaData>,
}

#[derive(Deserialize)]
struct BochaData {
    list: Option<Vec<BochaNewsItem>>,
    total: Option<i32>,
}

#[derive(Deserialize)]
struct BochaNewsItem {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
    source: Option<String>,
    pub_time: Option<String>,
    img_url: Option<String>,
}

impl BochaProvider {
    /// 创建新的博查提供商
    pub fn new(config: NewsProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// 从环境变量创建
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("BOCHA_API_KEY")
            .or_else(|_| std::env::var("BOCHA_API_KEYS"))
            .map_err(|_| QuantixError::Config("BOCHA_API_KEY not set".to_string()))?;

        let config = NewsProviderConfig {
            enabled: true,
            priority: 3,
            api_key: Some(api_key),
            base_url: Some("https://api.bocha.io".to_string()),
            timeout_seconds: 30,
            daily_limit: None,
            ..Default::default()
        };

        Self::new(config)
    }
}

#[async_trait]
impl NewsProvider for BochaProvider {
    fn name(&self) -> &'static str {
        "bocha"
    }

    async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
        let start = Instant::now();
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| QuantixError::Config("Bocha API key not configured".to_string()))?;

        let query = if !request.codes.is_empty() {
            format!("{} {}", request.query, request.codes.join(" "))
        } else {
            request.query.clone()
        };

        let url = format!(
            "{}/news/search?keyword={}&count={}",
            self.config
                .base_url
                .as_deref()
                .unwrap_or("https://api.bocha.io"),
            urlencoding::encode(&query),
            request.max_results
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantixError::Other(format!(
                "Bocha API error: {} - {}",
                status, body
            )));
        }

        let bocha_response: BochaResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("Failed to parse Bocha response: {}", e)))?;

        if bocha_response.code.map(|c| c != 0).unwrap_or(false) {
            return Err(QuantixError::Other(format!(
                "Bocha API error: {}",
                bocha_response
                    .message
                    .unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let articles: Vec<NewsArticle> = bocha_response
            .data
            .and_then(|d| d.list)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let title = item.title?;
                let url = item.url?;
                let mut article = NewsArticle::new(
                    title,
                    url,
                    item.source.unwrap_or_else(|| "unknown".to_string()),
                );
                article.summary = item.content;
                article.image_url = item.img_url;
                // Parse pub_time if available
                article.language = "zh".to_string();
                Some(article)
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(NewsSearchResult::new(articles, "bocha".to_string()).with_elapsed(elapsed))
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
