use super::*;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

fn create_test_position(code: &str, quantity: i64, price: f64) -> Position {
    Position::new(
        code.to_string(),
        quantity,
        Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    )
}

#[test]
fn test_position_monitor_creation() {
    let monitor = PositionMonitor::with_defaults(dec!(1000000));
    assert_eq!(monitor.get_position_count(), 0);
    assert_eq!(monitor.get_current_equity(), dec!(1000000));
}

#[test]
fn test_update_positions() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    assert_eq!(monitor.get_position_count(), 1);
    assert!(monitor.get_position("000001").is_some());
}

#[test]
fn test_new_position_detection() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    let events = monitor.get_change_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].change_type, PositionChangeType::New);
}

#[test]
fn test_position_increase_detection() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    let pos2 = create_test_position("000001", 1500, 10.0);
    monitor.update_positions(&[pos2]);

    let events = monitor.get_change_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].change_type, PositionChangeType::Increased);
    assert_eq!(events[1].quantity_change, 500);
}

#[test]
fn test_position_close_detection() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    // 平仓
    monitor.update_positions(&[]);

    let events = monitor.get_change_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].change_type, PositionChangeType::Closed);
}

#[test]
fn test_create_snapshot() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    let snapshot = monitor.create_snapshot();
    assert_eq!(snapshot.position_count, 1);
    assert_eq!(snapshot.total_market_value, dec!(10000));
}

#[test]
fn test_total_pnl_calculation() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let mut pos1 = create_test_position("000001", 1000, 10.0);
    pos1.update_price(dec!(11.0)); // 盈利1000

    monitor.update_positions(&[pos1]);

    assert_eq!(monitor.get_total_pnl(), dec!(1000));
}

#[test]
fn test_check_position_ratio() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let mut pos1 = create_test_position("000001", 1000, 10.0);
    pos1.update_price(dec!(250.0)); // 市值250000，比例25%

    monitor.update_positions(&[pos1]);

    assert!(monitor.check_position_ratio("000001")); // 超过20%阈值
}

#[test]
fn test_get_recent_changes() {
    let mut monitor = PositionMonitor::with_defaults(Decimal::from(1000000));

    // Add first position
    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    // Add second position (replaces first)
    let pos2 = create_test_position("000002", 1000, 10.0);
    monitor.update_positions(&[pos2]);

    // Check change history
    println!("Total changes: {}", monitor.change_history.len());
    for (i, change) in monitor.change_history.iter().enumerate() {
        println!(
            "Change {}: code={}, change_type={:?}",
            i, change.code, change.change_type
        );
    }

    let recent = monitor.get_recent_changes(1);
    assert_eq!(recent.len(), 1);
    // The last change should be for 000001 (closed) or 000002 (new)
    // Since we replaced the entire position list, 000001 gets closed last
    assert_eq!(recent[0].code, "000001");
    assert_eq!(recent[0].change_type, PositionChangeType::Closed);
}

#[test]
fn test_clear_snapshots() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);
    monitor.create_snapshot();
    monitor.create_snapshot();

    assert_eq!(monitor.get_snapshots().len(), 2);

    monitor.clear_snapshots();
    assert_eq!(monitor.get_snapshots().len(), 0);
}

#[test]
fn test_clear_change_history() {
    let mut monitor = PositionMonitor::with_defaults(dec!(1000000));

    let pos1 = create_test_position("000001", 1000, 10.0);
    monitor.update_positions(&[pos1]);

    assert_eq!(monitor.get_change_events().len(), 1);

    monitor.clear_change_history();
    assert_eq!(monitor.get_change_events().len(), 0);
}
