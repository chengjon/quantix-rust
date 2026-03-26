use reqwest::Client;
use serde_json::json;

use crate::bridge::error::{BridgeError, Result};
use crate::bridge::models::{
    BridgeCapabilitiesResponse, BridgeKlineResponse, BridgeQmtPreviewRequest,
    BridgeQmtPreviewResponse, BridgeQuotesResponse,
};

#[derive(Debug, Clone)]
pub struct BridgeHttpClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl BridgeHttpClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        if base_url.trim().is_empty() {
            return Err(BridgeError::Config("bridge base_url cannot be empty".to_string()));
        }

        Ok(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    pub async fn capabilities(&self) -> Result<BridgeCapabilitiesResponse> {
        let mut request = self
            .client
            .get(format!("{}/api/v1/capabilities", self.base_url));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeCapabilitiesResponse>().await?)
    }

    pub async fn fetch_tdx_quotes(&self, symbols: &[String]) -> Result<BridgeQuotesResponse> {
        let mut request = self
            .client
            .post(format!("{}/api/v1/data/tdx/quotes", self.base_url))
            .json(&json!({ "symbols": symbols }));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQuotesResponse>().await?)
    }

    pub async fn fetch_tdx_kline(
        &self,
        symbol: &str,
        period: &str,
        start: &str,
        end: &str,
    ) -> Result<BridgeKlineResponse> {
        let mut request = self
            .client
            .get(format!("{}/api/v1/data/tdx/kline/{}", self.base_url, symbol))
            .query(&[("period", period), ("start", start), ("end", end)]);

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeKlineResponse>().await?)
    }

    pub async fn qmt_preview_order(
        &self,
        payload: &BridgeQmtPreviewRequest,
    ) -> Result<BridgeQmtPreviewResponse> {
        let mut request = self
            .client
            .post(format!("{}/api/v1/broker/qmt/orders/preview", self.base_url))
            .json(payload);

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtPreviewResponse>().await?)
    }
}
