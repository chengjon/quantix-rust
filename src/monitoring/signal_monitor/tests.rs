use super::*;
use chrono::NaiveDateTime;

fn create_test_signal(strategy: &str, code: &str, signal: Signal, price: f64) -> SignalEvent {
    SignalEvent::new(
        strategy.to_string(),
        code.to_string(),
        signal,
        Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO),
        NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
    )
}

#[test]
fn test_signal_monitor_creation() {
    let monitor = SignalMonitor::with_defaults();
    assert_eq!(monitor.history_size(), 0);
}

#[test]
fn test_record_signal() {
    let mut monitor = SignalMonitor::with_defaults();

    let event = create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0);
    monitor.record_signal(event);

    assert_eq!(monitor.history_size(), 1);

    let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
    assert_eq!(stats.buy_count, 1);
    assert_eq!(stats.total_count, 1);
}

#[test]
fn test_multiple_signals() {
    let mut monitor = SignalMonitor::with_defaults();

    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
    monitor.record_signal(create_test_signal(
        "MA_Cross",
        "000001",
        Signal::Sell,
        105.0,
    ));
    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 98.0));

    let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
    assert_eq!(stats.buy_count, 2);
    assert_eq!(stats.sell_count, 1);
    assert_eq!(stats.total_count, 3);
}

#[test]
fn test_history_limit() {
    let config = SignalMonitorConfig {
        max_history_size: 5,
        ..Default::default()
    };
    let mut monitor = SignalMonitor::new(config);

    for i in 0..10 {
        monitor.record_signal(create_test_signal(
            "MA_Cross",
            "000001",
            Signal::Buy,
            100.0 + i as f64,
        ));
    }

    assert_eq!(monitor.history_size(), 5);
}

#[test]
fn test_get_recent_signals() {
    let mut monitor = SignalMonitor::with_defaults();

    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
    monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 101.0));
    monitor.record_signal(create_test_signal("MA_Cross", "000003", Signal::Buy, 102.0));

    let recent = monitor.get_recent_signals(2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].code, "000003");
    assert_eq!(recent[1].code, "000002");
}

#[test]
fn test_code_stats() {
    let mut monitor = SignalMonitor::with_defaults();

    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
    monitor.record_signal(create_test_signal(
        "MA_Cross",
        "000001",
        Signal::Sell,
        105.0,
    ));
    monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 50.0));

    let stats1 = monitor.get_code_stats("000001").unwrap();
    assert_eq!(stats1.buy_count, 1);
    assert_eq!(stats1.sell_count, 1);

    let stats2 = monitor.get_code_stats("000002").unwrap();
    assert_eq!(stats2.buy_count, 1);
    assert_eq!(stats2.sell_count, 0);
}

#[test]
fn test_clear_history() {
    let mut monitor = SignalMonitor::with_defaults();

    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
    monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 101.0));

    assert_eq!(monitor.history_size(), 2);

    monitor.clear_history();
    assert_eq!(monitor.history_size(), 0);
}

#[test]
fn test_signal_event_with_metadata() {
    let event = create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0)
        .with_metadata("rsi".to_string(), "30".to_string())
        .with_metadata("volume_ratio".to_string(), "2.5".to_string());

    assert_eq!(event.metadata.len(), 2);
    assert_eq!(event.metadata.get("rsi"), Some(&"30".to_string()));
}

#[test]
fn test_reset_stats() {
    let mut monitor = SignalMonitor::with_defaults();

    monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
    monitor.record_signal(create_test_signal(
        "MA_Cross",
        "000001",
        Signal::Sell,
        105.0,
    ));

    let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
    assert_eq!(stats.total_count, 2);

    monitor.reset_stats();

    let stats = monitor.get_strategy_stats("MA_Cross");
    assert!(stats.is_none());
}
