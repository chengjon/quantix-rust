use super::*;

#[test]
fn test_reconciliation_action_serialization() {
    let action = ReconciliationAction::Recovered;
    assert_eq!(action.as_str(), "recovered");
    assert_eq!(ReconciliationAction::from_str("recovered"), Some(action));
}

#[test]
fn test_reconciliation_summary_creation() {
    let summary = ReconciliationSummary {
        reconciled_at: Utc::now(),
        total_open_orders: 10,
        matched_orders: 8,
        mismatched_orders: 1,
        recovered_orders: 1,
        failed_orders: 0,
        duration_ms: 150,
    };

    assert_eq!(summary.total_open_orders, 10);
    assert_eq!(summary.matched_orders, 8);
}

#[test]
fn test_open_order_summary() {
    let mut by_status = HashMap::new();
    by_status.insert("accepted".to_string(), 5);
    by_status.insert("partially_filled".to_string(), 3);

    let summary = OpenOrderSummary {
        total_open: 8,
        by_status,
        stale_count: 2,
        unknown_count: 1,
        stale_threshold_seconds: 3600,
        unknown_timeout_seconds: 300,
    };

    assert_eq!(summary.total_open, 8);
    assert_eq!(summary.stale_count, 2);
    assert_eq!(summary.unknown_count, 1);
}
