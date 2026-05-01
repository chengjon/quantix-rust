use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_performance_monitor_creation() {
    let monitor = PerformanceMonitor::with_defaults(dec!(1000000));
    assert_eq!(monitor.get_current_metrics().current_equity, dec!(1000000));
}

#[test]
fn test_update_equity() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1010000), dec!(10000), dec!(1000000));

    assert_eq!(monitor.get_current_metrics().current_equity, dec!(1010000));
    assert_eq!(monitor.get_current_metrics().total_return, dec!(0.01));
}

#[test]
fn test_drawdown_calculation() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000)); // 新高
    monitor.update_equity(dec!(1000000), dec!(0), dec!(1000000)); // 回撤

    assert!(monitor.get_current_metrics().current_drawdown > dec!(0.09));
    assert!(monitor.get_current_metrics().current_drawdown < dec!(0.091));
}

#[test]
fn test_max_drawdown() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
    monitor.update_equity(dec!(900000), dec!(0), dec!(900000));
    monitor.update_equity(dec!(950000), dec!(0), dec!(950000));

    assert!(monitor.get_current_metrics().max_drawdown > dec!(0.18));
}

#[test]
fn test_record_trade_pnl() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.record_trade_pnl(dec!(1000));
    monitor.record_trade_pnl(dec!(-500));

    assert_eq!(monitor.get_current_metrics().total_trades, 2);
    assert_eq!(monitor.get_current_metrics().win_trades, 1);
    assert_eq!(monitor.get_current_metrics().loss_trades, 1);
}

#[test]
fn test_win_rate_calculation() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.record_trade_pnl(dec!(1000));
    monitor.record_trade_pnl(dec!(500));
    monitor.record_trade_pnl(dec!(-500));

    let metrics = monitor.get_current_metrics();
    assert!(metrics.win_rate > dec!(66));
    assert!(metrics.win_rate < dec!(67));
}

#[test]
fn test_check_drawdown_alert() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
    monitor.update_equity(dec!(950000), dec!(0), dec!(950000)); // 13.6% 回撤

    assert!(monitor.check_drawdown_alert());
}

#[test]
fn test_get_drawdown_status() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
    monitor.update_equity(dec!(800000), dec!(0), dec!(800000)); // 27% 回撤

    assert_eq!(monitor.get_drawdown_status(), DrawdownStatus::Critical);
}

#[test]
fn test_profit_loss_ratio() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.record_trade_pnl(dec!(2000));
    monitor.record_trade_pnl(dec!(1000));
    monitor.record_trade_pnl(dec!(-1000));
    monitor.record_trade_pnl(dec!(-500));

    let metrics = monitor.get_current_metrics();
    assert_eq!(metrics.profit_loss_ratio, dec!(2));
}

#[test]
fn test_reset() {
    let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

    monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
    monitor.record_trade_pnl(dec!(1000));

    monitor.reset();

    assert_eq!(monitor.get_current_metrics().current_equity, dec!(1000000));
    assert_eq!(monitor.get_current_metrics().total_trades, 0);
    assert_eq!(monitor.get_equity_history().len(), 0);
}

#[test]
fn test_equity_history_limit() {
    let config = PerformanceMonitorConfig {
        max_equity_history: 5,
        ..Default::default()
    };
    let mut monitor = PerformanceMonitor::new(config, dec!(1000000));

    for i in 0..10 {
        let equity = Decimal::from(1_000_000 + i * 1_000);
        monitor.update_equity(equity, Decimal::ZERO, equity);
    }

    assert_eq!(monitor.get_equity_history().len(), 5);
}
