use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeCapabilitySection {
    pub enabled: bool,
    #[serde(default)]
    pub supports: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtCapabilitySection {
    pub enabled: bool,
    pub mode: String,
    #[serde(default)]
    pub supports: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeCapabilitiesResponse {
    pub tdx: BridgeCapabilitySection,
    pub qmt: BridgeQmtCapabilitySection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQuotePayload {
    pub symbol: String,
    pub name: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub volume: i64,
    pub turnover: f64,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQuotesResponse {
    pub quotes: Vec<BridgeQuotePayload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeKlineBarPayload {
    pub datetime: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub turnover: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeKlineResponse {
    pub symbol: String,
    pub period: String,
    pub bars: Vec<BridgeKlineBarPayload>,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BridgeQmtPreviewRequest {
    pub request_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i64,
    pub price: String,
    pub order_type: String,
    pub snapshot_metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtPreviewResponse {
    pub adapter_order_id: String,
    pub latest_status: String,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<String>,
    pub fill_details: Option<serde_json::Value>,
    pub rejection_reason: Option<String>,
    pub broker_payload: serde_json::Value,
}
