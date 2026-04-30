use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::bridge::error::{BridgeError, Result};
use crate::bridge::models::{
    BridgeCapabilitiesResponse, BridgeKlineResponse, BridgeQmtAccountStatusResponse,
    BridgeQmtAsset, BridgeQmtCancelResponse, BridgeQmtOrderQueryResponse, BridgeQmtOrderRequest,
    BridgeQmtOrderResponse, BridgeQmtPosition, BridgeQmtPreviewRequest, BridgeQmtPreviewResponse,
    BridgeQuotesResponse, BridgeTaskExecuteReceipt, BridgeTaskExecuteRequest,
    BridgeTaskResultResponse,
};
use crate::core::runtime::{DEFAULT_BRIDGE_CONTRACT_VERSION, DEFAULT_BRIDGE_TIMEOUT_MS};

#[derive(Debug, Clone)]
pub struct BridgeHttpClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    bearer_token: Option<String>,
    contract_version: String,
}

impl BridgeHttpClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        Self::new_with_contract(
            base_url,
            api_key,
            None,
            DEFAULT_BRIDGE_CONTRACT_VERSION.to_string(),
            DEFAULT_BRIDGE_TIMEOUT_MS,
        )
    }

    pub fn new_with_contract(
        base_url: String,
        api_key: Option<String>,
        bearer_token: Option<String>,
        contract_version: String,
        timeout_ms: u64,
    ) -> Result<Self> {
        if base_url.trim().is_empty() {
            return Err(BridgeError::Config(
                "bridge base_url cannot be empty".to_string(),
            ));
        }

        let contract_version = contract_version.trim().to_string();
        if contract_version.is_empty() {
            return Err(BridgeError::Config(
                "bridge contract_version cannot be empty".to_string(),
            ));
        }

        let timeout_ms = timeout_ms.max(1);
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: normalize_optional(api_key),
            bearer_token: normalize_optional(bearer_token),
            contract_version,
        })
    }

    pub async fn capabilities(&self) -> Result<BridgeCapabilitiesResponse> {
        let request = self
            .client
            .get(format!("{}/api/v1/capabilities", self.base_url));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    pub async fn fetch_tdx_quotes(&self, symbols: &[String]) -> Result<BridgeQuotesResponse> {
        let request = self
            .client
            .post(format!("{}/api/v1/data/tdx/quotes", self.base_url))
            .json(&json!({ "symbols": symbols }));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    pub async fn fetch_tdx_kline(
        &self,
        symbol: &str,
        period: &str,
        start: &str,
        end: &str,
    ) -> Result<BridgeKlineResponse> {
        let request = self
            .client
            .get(format!(
                "{}/api/v1/data/tdx/kline/{}",
                self.base_url, symbol
            ))
            .query(&[("period", period), ("start", start), ("end", end)]);

        self.send_legacy_json(self.with_api_key(request)).await
    }

    pub async fn qmt_preview_order(
        &self,
        payload: &BridgeQmtPreviewRequest,
    ) -> Result<BridgeQmtPreviewResponse> {
        let request = self
            .client
            .post(format!(
                "{}/api/v1/broker/qmt/orders/preview",
                self.base_url
            ))
            .json(payload);

        self.send_legacy_json(self.with_api_key(request)).await
    }

    pub async fn task_execute_qmt_submit(
        &self,
        payload: &BridgeTaskExecuteRequest,
    ) -> Result<BridgeTaskExecuteReceipt> {
        let request = self
            .client
            .post(format!("{}/api/v1/task/execute", self.base_url))
            .json(payload);

        let request = self.with_task_contract_headers(request)?;
        self.send_task_contract_json(request).await
    }

    pub async fn task_result(&self, task_id: &str) -> Result<BridgeTaskResultResponse> {
        if task_id.trim().is_empty() {
            return Err(BridgeError::Config(
                "bridge task_id cannot be empty".to_string(),
            ));
        }

        let request = self
            .client
            .get(format!("{}/api/v1/task/result/{}", self.base_url, task_id));

        let request = self.with_task_contract_headers(request)?;
        self.send_task_contract_json(request).await
    }

    // ============ Live Order Methods ============

    /// Submit a real order to QMT (LIVE TRADING)
    pub async fn qmt_submit_order(
        &self,
        payload: &BridgeQmtOrderRequest,
    ) -> Result<BridgeQmtOrderResponse> {
        let request = self
            .client
            .post(format!("{}/api/v1/broker/qmt/orders", self.base_url))
            .json(payload);

        self.send_legacy_json(self.with_api_key(request)).await
    }

    /// Query order status
    pub async fn qmt_query_order(&self, order_id: &str) -> Result<BridgeQmtOrderQueryResponse> {
        let request = self.client.get(format!(
            "{}/api/v1/broker/qmt/orders/{}",
            self.base_url, order_id
        ));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    /// Cancel an order
    pub async fn qmt_cancel_order(&self, order_id: &str) -> Result<BridgeQmtCancelResponse> {
        let request = self.client.delete(format!(
            "{}/api/v1/broker/qmt/orders/{}",
            self.base_url, order_id
        ));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    /// Get account status
    pub async fn qmt_account_status(&self) -> Result<BridgeQmtAccountStatusResponse> {
        let request = self.client.get(format!(
            "{}/api/v1/broker/qmt/account/status",
            self.base_url
        ));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    /// Get all positions
    pub async fn qmt_positions(&self) -> Result<Vec<BridgeQmtPosition>> {
        let request = self
            .client
            .get(format!("{}/api/v1/broker/qmt/positions", self.base_url));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    /// Get account asset
    pub async fn qmt_asset(&self) -> Result<BridgeQmtAsset> {
        let request = self
            .client
            .get(format!("{}/api/v1/broker/qmt/account/asset", self.base_url));

        self.send_legacy_json(self.with_api_key(request)).await
    }

    fn with_api_key(&self, request: RequestBuilder) -> RequestBuilder {
        if let Some(api_key) = &self.api_key {
            return request.header("X-Quantix-Api-Key", api_key);
        }

        request
    }

    fn with_task_contract_headers(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let request = request.header("X-Bridge-Contract-Version", &self.contract_version);

        if let Some(bearer_token) = &self.bearer_token {
            return Ok(request.header("Authorization", format!("Bearer {}", bearer_token)));
        }

        if let Some(api_key) = &self.api_key {
            return Ok(request.header("X-Quantix-Api-Key", api_key));
        }

        Err(BridgeError::Config(
            "task contract requests require bearer token or bridge api key".to_string(),
        ))
    }

    async fn send_legacy_json<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = request.send().await?.error_for_status()?;
        response.json::<T>().await.map_err(BridgeError::from)
    }

    async fn send_task_contract_json<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = request.send().await?;
        self.parse_task_contract_response(response).await
    }

    async fn parse_task_contract_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        if status.is_success() {
            return response
                .json::<T>()
                .await
                .map_err(|error| BridgeError::Protocol(error.to_string()));
        }

        let body = response.text().await.unwrap_or_default();
        Err(map_task_contract_error(status, &body))
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn map_task_contract_error(status: StatusCode, body: &str) -> BridgeError {
    let reason_code = parse_error_field(body, "reason_code");
    let reason_detail = parse_error_field(body, "reason_detail");
    let message = reason_detail
        .or_else(|| reason_code.clone())
        .unwrap_or_else(|| format!("bridge returned {}", status));

    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => BridgeError::Unauthorized(message),
        StatusCode::BAD_REQUEST => match reason_code.as_deref() {
            Some("live_bridge_unsupported_contract_version") => {
                BridgeError::UnsupportedContractVersion(message)
            }
            Some("live_bridge_unsupported_method") => BridgeError::UnsupportedMethod(message),
            Some("live_bridge_invalid_result") => BridgeError::InvalidResult(message),
            _ => BridgeError::Http(format!("bridge returned {}: {}", status, message)),
        },
        _ if status.is_server_error() => BridgeError::Unavailable(message),
        _ => BridgeError::Http(format!("bridge returned {}: {}", status, message)),
    }
}

fn parse_error_field(body: &str, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get(key)
                .and_then(|field| field.as_str())
                .map(str::to_string)
        })
}
