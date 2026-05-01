use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_alert_manager_creation() {
    let manager = AlertManager::with_defaults();
    assert_eq!(manager.get_active_alerts().len(), 0);
    assert_eq!(manager.get_alert_history().len(), 0);
}

#[test]
fn test_send_alert() {
    let mut manager = AlertManager::with_defaults();

    let alert = Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试告警".to_string(),
        },
        "这是一个测试告警".to_string(),
    );

    manager.send_alert(alert);

    assert_eq!(manager.get_active_alerts().len(), 1);
    assert_eq!(manager.get_alert_history().len(), 1);
}

#[test]
fn test_check_and_alert() {
    let mut manager = AlertManager::with_defaults();

    let threshold = AlertThreshold::new(
        "test_threshold".to_string(),
        "测试阈值".to_string(),
        Decimal::from(10),
        AlertLevel::Warning,
    );

    manager.add_threshold(threshold);

    // 未超过阈值
    let alert = manager.check_and_alert("test_threshold", Decimal::from(5));
    assert!(alert.is_none());

    // 超过阈值
    let alert = manager.check_and_alert("test_threshold", Decimal::from(15));
    assert!(alert.is_some());
}

#[test]
fn test_alert_cooldown() {
    let mut manager = AlertManager::with_defaults();

    let mut threshold = AlertThreshold::new(
        "test_threshold".to_string(),
        "测试阈值".to_string(),
        Decimal::from(10),
        AlertLevel::Warning,
    );
    threshold.cooldown_secs = 10; // 10秒冷却

    manager.add_threshold(threshold);

    // 第一次触发
    let alert1 = manager.check_and_alert("test_threshold", Decimal::from(15));
    assert!(alert1.is_some());

    // 冷却期内，不应该再次触发
    let alert2 = manager.check_and_alert("test_threshold", Decimal::from(15));
    assert!(alert2.is_none(), "Should not alert during cooldown period");
}

#[test]
fn test_acknowledge_alert() {
    let mut manager = AlertManager::with_defaults();

    let alert = Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试告警".to_string(),
        },
        "这是一个测试告警".to_string(),
    );

    let alert = manager.send_alert(alert);
    let alert_id = alert.id.clone();

    assert!(!manager.get_active_alerts()[0].acknowledged);

    manager.acknowledge_alert(&alert_id);

    assert!(manager.get_active_alerts()[0].acknowledged);
}

#[test]
fn test_get_unacknowledged_alerts() {
    let mut manager = AlertManager::with_defaults();

    let alert1 = Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试告警1".to_string(),
        },
        "告警1".to_string(),
    );

    let alert2 = Alert::new(
        AlertLevel::Info,
        AlertType::System {
            message: "测试告警2".to_string(),
        },
        "告警2".to_string(),
    );

    manager.send_alert(alert1);
    manager.send_alert(alert2);

    let unacknowledged = manager.get_unacknowledged_alerts();
    assert_eq!(unacknowledged.len(), 2);
}

#[test]
fn test_alert_threshold_builder() {
    let drawdown_threshold = AlertThresholdBuilder::drawdown_warning(0.1);
    assert_eq!(
        drawdown_threshold.threshold,
        Decimal::from_f64_retain(0.1).unwrap()
    );
    assert_eq!(drawdown_threshold.level, AlertLevel::Warning);

    let critical_threshold = AlertThresholdBuilder::drawdown_critical(0.2);
    assert_eq!(critical_threshold.level, AlertLevel::Critical);
}

#[test]
fn test_drawdown_threshold_value() {
    let threshold = AlertThresholdBuilder::drawdown_warning(0.1);
    // Test with actual decimal value
    let expected = Decimal::from_f64_retain(0.1).unwrap();
    assert_eq!(threshold.threshold, expected);
}

#[test]
fn test_critical_threshold_value() {
    let threshold = AlertThresholdBuilder::drawdown_critical(0.2);
    let expected = Decimal::from_f64_retain(0.2).unwrap();
    assert_eq!(threshold.threshold, expected);
}

#[test]
fn test_position_ratio_threshold_value() {
    let threshold = AlertThresholdBuilder::position_ratio(0.3);
    let expected = Decimal::from_f64_retain(0.3).unwrap();
    assert_eq!(threshold.threshold, expected);
}

#[test]
fn test_signal_frequency_threshold_value() {
    let threshold = AlertThresholdBuilder::signal_frequency(5.0);
    let expected = Decimal::from_f64_retain(5.0).unwrap();
    assert_eq!(threshold.threshold, expected);
}

#[test]
fn test_enable_disable_threshold() {
    let mut threshold = AlertThreshold::new(
        "test".to_string(),
        "测试".to_string(),
        Decimal::from(10),
        AlertLevel::Warning,
    );

    assert!(threshold.enabled);

    threshold.disable();
    assert!(!threshold.enabled);

    threshold.enable();
    assert!(threshold.enabled);
}

#[test]
fn test_alert_format_message() {
    let alert = Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试告警".to_string(),
        },
        "这是一个测试告警".to_string(),
    )
    .with_current_value(dec!(15.5))
    .with_threshold(dec!(10.0));

    let msg = alert.format_message();
    assert!(msg.contains("[WARNING]"));
    assert!(msg.contains("15.50"));
    assert!(msg.contains("10.00"));
}

#[test]
fn test_clear_active_alerts() {
    let mut manager = AlertManager::with_defaults();

    manager.send_alert(Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试".to_string(),
        },
        "测试告警".to_string(),
    ));

    assert_eq!(manager.get_active_alerts().len(), 1);

    manager.clear_active_alerts();
    assert_eq!(manager.get_active_alerts().len(), 0);
}

#[test]
fn test_alert_stats() {
    let mut manager = AlertManager::with_defaults();

    manager.send_alert(Alert::new(
        AlertLevel::Warning,
        AlertType::System {
            message: "测试".to_string(),
        },
        "测试告警".to_string(),
    ));

    manager.send_alert(Alert::new(
        AlertLevel::Error,
        AlertType::System {
            message: "测试".to_string(),
        },
        "测试告警".to_string(),
    ));

    let stats = manager.get_alert_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats.values().next().unwrap(), &2);
}
