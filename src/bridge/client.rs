use reqwest::Client;
use serde_json::json;

use crate::bridge::error::{BridgeError, Result};
use crate::bridge::models::{
    BridgeCapabilitiesResponse,
    BridgeKlineResponse,
    BridgeQmtAccountStatusResponse,
    BridgeQmtAsset,
    BridgeQmtCancelResponse,
    BridgeQmtOrderQueryResponse,
    // Live order models
    BridgeQmtOrderRequest,
    BridgeQmtOrderResponse,
    BridgeQmtPosition,
    BridgeQmtPreviewRequest,
    BridgeQmtPreviewResponse,
    BridgeQuotesResponse,
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
            return Err(BridgeError::Config(
                "bridge base_url cannot be empty".to_string(),
            ));
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
            .get(format!(
                "{}/api/v1/data/tdx/kline/{}",
                self.base_url, symbol
            ))
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
            .post(format!(
                "{}/api/v1/broker/qmt/orders/preview",
                self.base_url
            ))
            .json(payload);

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtPreviewResponse>().await?)
    }

    // ============ Live Order Methods ============

    /// Submit a real order to QMT (LIVE TRADING)
    pub async fn qmt_submit_order(
        &self,
        payload: &BridgeQmtOrderRequest,
    ) -> Result<BridgeQmtOrderResponse> {
        let mut request = self
            .client
            .post(format!("{}/api/v1/broker/qmt/orders", self.base_url))
            .json(payload);

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtOrderResponse>().await?)
    }

    /// Query order status
    pub async fn qmt_query_order(&self, order_id: &str) -> Result<BridgeQmtOrderQueryResponse> {
        let mut request = self.client.get(format!(
            "{}/api/v1/broker/qmt/orders/{}",
            self.base_url, order_id
        ));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtOrderQueryResponse>().await?)
    }

    /// Cancel an order
    pub async fn qmt_cancel_order(&self, order_id: &str) -> Result<BridgeQmtCancelResponse> {
        let mut request = self.client.delete(format!(
            "{}/api/v1/broker/qmt/orders/{}",
            self.base_url, order_id
        ));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtCancelResponse>().await?)
    }

    /// Get account status
    pub async fn qmt_account_status(&self) -> Result<BridgeQmtAccountStatusResponse> {
        let mut request = self.client.get(format!(
            "{}/api/v1/broker/qmt/account/status",
            self.base_url
        ));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtAccountStatusResponse>().await?)
    }

    /// Get all positions
    pub async fn qmt_positions(&self) -> Result<Vec<BridgeQmtPosition>> {
        let mut request = self
            .client
            .get(format!("{}/api/v1/broker/qmt/positions", self.base_url));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<Vec<BridgeQmtPosition>>().await?)
    }

    /// Get account asset
    pub async fn qmt_asset(&self) -> Result<BridgeQmtAsset> {
        let mut request = self
            .client
            .get(format!("{}/api/v1/broker/qmt/account/asset", self.base_url));

        if let Some(api_key) = &self.api_key {
            request = request.header("X-Quantix-Api-Key", api_key);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<BridgeQmtAsset>().await?)
    }
}
