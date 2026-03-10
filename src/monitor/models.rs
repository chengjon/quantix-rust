use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorQuoteRow {
    pub code: String,
    pub group: String,
    pub tags: Vec<String>,
    pub last_price: Option<f64>,
    pub change_pct: Option<f64>,
    pub quote_time: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceAlertKind {
    Above,
    Below,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: i64,
    pub code: String,
    pub kind: PriceAlertKind,
    pub target_price: f64,
    pub created_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggeredAlert {
    pub alert_id: i64,
    pub code: String,
    pub kind: PriceAlertKind,
    pub target_price: f64,
    pub current_price: f64,
    pub triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MonitorWatchlistSnapshot {
    pub rows: Vec<MonitorQuoteRow>,
    pub triggered_alerts: Vec<TriggeredAlert>,
    pub warnings: Vec<String>,
}
