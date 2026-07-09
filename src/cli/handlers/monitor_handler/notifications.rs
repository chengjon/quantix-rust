use super::*;

pub(super) async fn send_monitor_notifications(output: &MonitorIterationOutput) -> Result<()> {
    let config = crate::monitoring::NotificationConfig::from_env();
    let mut service = crate::monitoring::NotificationService::new(config);
    send_monitor_notifications_with_service(output, &mut service).await
}

pub(crate) async fn send_monitor_notifications_with_service<S>(
    output: &MonitorIterationOutput,
    service: &mut S,
) -> Result<()>
where
    S: MonitorNotificationSender,
{
    for event in &output.new_events {
        let notification = crate::monitoring::Notification::new(
            format!(
                "Monitor {} {}",
                monitor_notification_event_label(event.event_type),
                event.code
            ),
            format!(
                "{}\n模式: {}\n时间: {}",
                event.message,
                monitor_notification_run_mode_label(event.run_mode),
                event.event_time.format("%Y-%m-%d %H:%M:%S")
            ),
            monitor_notification_level(event.event_type),
        )
        .with_metadata(
            "event_type",
            monitor_notification_event_key(event.event_type).to_string(),
        )
        .with_metadata("code", event.code.clone())
        .with_metadata(
            "run_mode",
            monitor_notification_run_mode_label(event.run_mode).to_string(),
        );

        service.notify(notification).await?;
    }

    Ok(())
}

pub(crate) async fn persist_triggered_monitor_alerts<RS>(
    store: &RS,
    snapshot: &MonitorWatchlistSnapshot,
    observed_at: chrono::DateTime<Utc>,
) -> Result<()>
where
    RS: MonitorAlertStore,
{
    for alert in &snapshot.triggered_alerts {
        let triggered_at = alert.triggered_at.unwrap_or(observed_at);
        store.mark_triggered(alert.alert_id, triggered_at).await?;
    }
    Ok(())
}
