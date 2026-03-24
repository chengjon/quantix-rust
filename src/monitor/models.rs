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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorEventType {
    PriceAlert,
    StopLoss,
    StopProfit,
    TrailingStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorRunMode {
    Foreground,
    Daemon,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewMonitorEvent {
    pub event_time: DateTime<Utc>,
    pub event_type: MonitorEventType,
    pub code: String,
    pub price: Option<f64>,
    pub message: String,
    pub source_type: String,
    pub source_key: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub run_mode: MonitorRunMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorEventRow {
    pub id: i64,
    pub event_time: DateTime<Utc>,
    pub event_type: MonitorEventType,
    pub code: String,
    pub price: Option<f64>,
    pub message: String,
    pub source_type: String,
    pub source_key: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub run_mode: MonitorRunMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorEventFilter {
    pub limit: usize,
    pub code: Option<String>,
    pub event_type: Option<MonitorEventType>,
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
