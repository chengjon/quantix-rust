use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MockLiveFillStep {
    #[serde(default)]
    pub quantity: i64,
    #[serde(default)]
    pub delay_secs: i64,
}

/// Fault injection modes for mock_live testing
///
/// Supported modes:
/// - `unknown_once`: Return Unknown status once, then clear
/// - `unknown_always`: Always return Unknown status
/// - `network_timeout`: Simulate network timeout on query
/// - `network_disconnect`: Simulate network disconnection
/// - `delayed_response:<secs>`: Delay response by specified seconds
/// - `simulated_rejection:<reason>`: Reject order with specified reason
/// - `partial_timeout`: Return timeout on first query, then normal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MockLiveFaultInjection {
    #[serde(default)]
    pub mode: Option<String>,
    /// For delayed_response mode: delay in seconds
    #[serde(default)]
    pub delay_seconds: Option<i64>,
    /// For simulated_rejection mode: rejection reason
    #[serde(default)]
    pub rejection_reason: Option<String>,
    /// For network_timeout mode: timeout duration in seconds
    #[serde(default)]
    pub timeout_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MockLiveOrderState {
    #[serde(default)]
    pub fill_plan: Vec<MockLiveFillStep>,
    #[serde(default)]
    pub next_step_index: usize,
    #[serde(default)]
    pub simulated_fill_price: Option<Decimal>,
    #[serde(default)]
    pub planned_fill_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub fault_injection: Option<MockLiveFaultInjection>,
    #[serde(default)]
    pub unknown_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub cancel_requested: bool,
    #[serde(default)]
    pub last_applied_fill_id: u64,
    #[serde(default)]
    pub unknown_retries: u32,
    #[serde(default)]
    pub recovery_exhausted: bool,
    #[serde(default)]
    pub exhausted_reason: Option<String>,
    #[serde(default)]
    pub query_script_index: usize,
    #[serde(default)]
    pub query_script_fill_started: bool,
}
