use crate::bridge::client::BridgeHttpClient;
use crate::bridge::models::BridgeQmtPreviewRequest;
use crate::core::{QuantixError, Result};
use crate::execution::models::{ExecutionRequestRecord, OrderStatus};

/// QMT bridge preview 适配器：将 ExecutionRequestRecord 翻译成 bridge preview 请求并解析响应，仅做 dry-run 预演，不提交真实订单。
#[derive(Debug, Clone)]
pub struct QmtBridgePreviewAdapter {
    client: BridgeHttpClient,
}

/// QMT bridge preview 响应：adapter_order_id broker 订单号、latest_status 最新状态、filled_quantity 已成交数量、avg_fill_price 均价、fill_details 成交明细、rejection_reason 拒单原因、broker_payload 原始 broker 载荷。
#[derive(Debug, Clone, PartialEq)]
pub struct QmtBridgePreviewResponse {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<String>,
    pub fill_details: Option<serde_json::Value>,
    pub rejection_reason: Option<String>,
    pub broker_payload: serde_json::Value,
}

impl QmtBridgePreviewAdapter {
    /// 构造 preview 适配器：注入已鉴权的 BridgeHttpClient。
    pub fn new(client: BridgeHttpClient) -> Self {
        Self { client }
    }

    /// 对 ExecutionRequestRecord 执行 dry-run 预演：从 payload.execution_snapshot 取 order_intent，调用 bridge preview 接口并解析为标准化响应。
    pub async fn preview_request(
        &self,
        request: &ExecutionRequestRecord,
    ) -> Result<QmtBridgePreviewResponse> {
        let snapshot = request
            .payload_json
            .get("execution_snapshot")
            .ok_or_else(|| QuantixError::Other("request 缺少 execution_snapshot".to_string()))?;
        let order_intent = snapshot
            .get("order_intent")
            .ok_or_else(|| QuantixError::Other("request 缺少 order_intent".to_string()))?;

        let symbol = snapshot
            .get("symbol")
            .and_then(|value| value.as_str())
            .ok_or_else(|| QuantixError::Other("request 缺少 symbol".to_string()))?;
        let side = order_intent
            .get("side")
            .and_then(|value| value.as_str())
            .ok_or_else(|| QuantixError::Other("request 缺少 side".to_string()))?;
        let quantity = order_intent
            .get("requested_quantity")
            .and_then(|value| value.as_i64())
            .ok_or_else(|| QuantixError::Other("request 缺少 requested_quantity".to_string()))?;
        let price = order_intent
            .get("requested_price")
            .and_then(|value| value.as_str())
            .ok_or_else(|| QuantixError::Other("request 缺少 requested_price".to_string()))?;
        let order_type = order_intent
            .get("order_type")
            .and_then(|value| value.as_str())
            .ok_or_else(|| QuantixError::Other("request 缺少 order_type".to_string()))?;

        let response = self
            .client
            .qmt_preview_order(&BridgeQmtPreviewRequest {
                request_id: request.request_id.clone(),
                client_order_id: request.request_id.clone(),
                symbol: normalize_symbol(symbol),
                side: side.to_string(),
                quantity,
                price: price.to_string(),
                order_type: order_type.to_string(),
                snapshot_metadata: serde_json::json!({
                    "source": "execution_request"
                }),
            })
            .await
            .map_err(|err| QuantixError::Other(err.to_string()))?;

        Ok(QmtBridgePreviewResponse {
            adapter_order_id: response.adapter_order_id,
            latest_status: OrderStatus::from_str(&response.latest_status).ok_or_else(|| {
                QuantixError::Other(format!("未知 preview 状态: {}", response.latest_status))
            })?,
            filled_quantity: response.filled_quantity,
            avg_fill_price: response.avg_fill_price,
            fill_details: response.fill_details,
            rejection_reason: response.rejection_reason,
            broker_payload: response.broker_payload,
        })
    }
}

fn normalize_symbol(symbol: &str) -> String {
    if symbol.contains('.') {
        return symbol.to_string();
    }

    if symbol.starts_with('6') {
        format!("{symbol}.SH")
    } else {
        format!("{symbol}.SZ")
    }
}
