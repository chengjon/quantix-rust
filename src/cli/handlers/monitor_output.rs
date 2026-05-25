use super::stop_output::format_triggered_stop_message;
use super::*;

use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorConfig,
    MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorIterationOutput,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorUserServiceInstaller,
    MonitorWatchlistReader, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
use crate::stop::{
    SqliteStopRuleStore, StopHistoryEvent, StopHistoryEventType, StopRule, StopRuleStore,
    StopRuleUpdate, StopService, StopStatusRow, StopTriggerKind, TriggeredStop,
};

pub(super) fn print_monitor_command_output(output: &MonitorCommandOutput) {
    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => print_monitor_watchlist_snapshot(snapshot, triggered_stops),
        MonitorCommandOutput::AutomationIteration {
            run_mode: _,
            output,
        } => {
            print_monitor_watchlist_snapshot(&output.snapshot, &output.triggered_stops);
            if !output.new_events.is_empty() {
                println!();
                print_monitor_events(&output.new_events);
            }
        }
        MonitorCommandOutput::AlertAdded(alert) => println!(
            "✅ 已添加价格告警 #{} {} {} {:.2}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            alert.target_price
        ),
        MonitorCommandOutput::AlertList(alerts) => print_monitor_alerts(alerts),
        MonitorCommandOutput::Config(config) => {
            println!("轮询间隔(秒): {}", config.interval_seconds);
            println!(
                "分组过滤: {}",
                config.watchlist_group.as_deref().unwrap_or("-")
            );
            println!("持久化事件: {}", config.persist_events);
            println!("自动通知: {}", config.notify_enabled);
            println!("最大历史条数: {}", config.max_event_history);
        }
        MonitorCommandOutput::EventList(rows) => print_monitor_events(rows),
        MonitorCommandOutput::ServiceConfig(config) => {
            println!("quantix_bin_path: {}", config.quantix_bin_path.display());
        }
        MonitorCommandOutput::ServiceStatus(summary) => {
            print_monitor_service_status_summary(summary);
        }
        MonitorCommandOutput::ServiceMessage(message) => println!("{}", message),
        MonitorCommandOutput::AlertRemoved { id, removed } => {
            if *removed {
                println!("✅ 已删除价格告警 #{}", id);
            } else {
                println!("⚠️  未找到价格告警 #{}", id);
            }
        }
    }
}

pub(super) fn print_monitor_watchlist_snapshot(
    snapshot: &MonitorWatchlistSnapshot,
    triggered_stops: &[TriggeredStop],
) {
    if snapshot.rows.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!(
        "{:<10} {:<12} {:<16} {:<10} {:<10} 备注",
        "代码", "分组", "标签", "最新价", "涨跌幅"
    );
    println!("{}", "-".repeat(80));

    for row in &snapshot.rows {
        println!(
            "{:<10} {:<12} {:<16} {:<10} {:<10} {}",
            row.code,
            row.group,
            format_tags(&row.tags),
            row.last_price
                .map(|value| format!("{:.2}", value))
                .unwrap_or_else(|| "-".to_string()),
            row.change_pct
                .map(|value| format!("{:.2}%", value))
                .unwrap_or_else(|| "-".to_string()),
            row.note.as_deref().unwrap_or("-")
        );
    }

    if !snapshot.triggered_alerts.is_empty() {
        println!();
        println!("== 触发告警 ==");
        for alert in &snapshot.triggered_alerts {
            println!(
                "[#{}] {} 当前价 {:.2} {} {:.2}",
                alert.alert_id,
                alert.code,
                alert.current_price,
                format_monitor_alert_kind(alert.kind),
                alert.target_price
            );
        }
    }

    if !triggered_stops.is_empty() {
        println!();
        println!("== 止盈止损 ==");
        for triggered_stop in triggered_stops {
            println!("{}", format_triggered_stop_message(triggered_stop));
        }
    }

    if !snapshot.warnings.is_empty() {
        println!();
        println!("== 警告 ==");
        for warning in &snapshot.warnings {
            println!("{}", warning);
        }
    }
}

pub(super) fn print_monitor_alerts(alerts: &[PriceAlert]) {
    if alerts.is_empty() {
        println!("📭 暂无价格告警");
        return;
    }

    println!(
        "{:<6} {:<10} {:<8} {:<12} 最后触发",
        "ID", "代码", "类型", "目标价"
    );
    println!("{}", "-".repeat(64));

    for alert in alerts {
        println!(
            "{:<6} {:<10} {:<8} {:<12} {}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            format!("{:.2}", alert.target_price),
            alert
                .last_triggered_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

pub(super) fn print_monitor_events(rows: &[MonitorEventRow]) {
    if rows.is_empty() {
        println!("📭 暂无监控事件");
        return;
    }

    println!(
        "{:<20} {:<14} {:<8} {:<8} {:<10}",
        "时间", "类型", "代码", "价格", "模式"
    );
    println!("{}", "-".repeat(72));

    for row in rows {
        println!(
            "{:<20} {:<14} {:<8} {:<8} {:<10}",
            row.event_time.format("%Y-%m-%d %H:%M:%S"),
            format_monitor_event_type(row.event_type),
            row.code,
            row.price
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "-".to_string()),
            format_monitor_run_mode(row.run_mode),
        );
        println!("  {}", row.message);
    }
}

pub(super) fn print_monitor_service_status_summary(summary: &MonitorServiceStatusSummary) {
    println!(
        "installed: {}",
        if summary.installed { "yes" } else { "no" }
    );
    println!("enabled: {}", if summary.enabled { "yes" } else { "no" });
    println!("active: {}", summary.active);
    println!("unit_path: {}", summary.unit_path.display());
    println!("wrapper_path: {}", summary.wrapper_path.display());
    println!("quantix_bin_path: {}", summary.quantix_bin_path.display());

    if let Some(raw_status) = &summary.raw_status {
        println!();
        print!("{}", raw_status);
    }
}

pub(super) fn build_unconfigured_monitor_service_status_summary() -> MonitorServiceStatusSummary {
    MonitorServiceStatusSummary {
        installed: false,
        enabled: false,
        active: "unconfigured".to_string(),
        unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-monitor.service"),
        wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-monitor-run"),
        quantix_bin_path: std::path::PathBuf::from("<unconfigured>"),
        raw_status: None,
    }
}

pub(super) fn format_monitor_alert_kind(kind: PriceAlertKind) -> &'static str {
    match kind {
        PriceAlertKind::Above => "above",
        PriceAlertKind::Below => "below",
    }
}

pub(super) fn format_monitor_event_type(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price-alert",
        MonitorEventType::StopLoss => "stop-loss",
        MonitorEventType::StopProfit => "stop-profit",
        MonitorEventType::TrailingStop => "trailing-stop",
    }
}

pub(super) fn format_monitor_run_mode(run_mode: MonitorRunMode) -> &'static str {
    match run_mode {
        MonitorRunMode::Foreground => "foreground",
        MonitorRunMode::Daemon => "daemon",
    }
}
