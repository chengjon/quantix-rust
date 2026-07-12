use super::helpers::*;

#[tokio::test]
async fn release_buy_lock_is_cleared_on_day_rollover() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let next_day_status = service
        .status(&snapshot(dec!(949000), &[]), now + Duration::days(1))
        .await
        .unwrap();

    assert!(!next_day_status.buy_locked);
    assert!(!next_day_status.manual_release_active);
    let state = store.snapshot().unwrap();
    assert_eq!(state.buy_lock.released_for_date, None);
}

#[tokio::test]
async fn release_buy_lock_errors_when_no_active_lock_exists() {
    let (service, _) = service();

    let err = service.release_buy_lock(fixed_ts()).await.unwrap_err();
    assert!(err.to_string().contains("当前无活动买入锁"));
}

#[tokio::test]
async fn list_log_returns_newest_first_rule_and_lock_events() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockReleased);
    assert_eq!(
        events[1].event_type,
        RiskLogEventType::DailyLossLockTriggered
    );
    assert_eq!(events[2].event_type, RiskLogEventType::RuleSet);
}

#[tokio::test]
async fn idempotent_release_does_not_append_duplicate_log_events() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(3))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    let release_count = events
        .iter()
        .filter(|event| event.event_type == RiskLogEventType::BuyLockReleased)
        .count();
    assert_eq!(release_count, 1);
}

#[tokio::test]
async fn day_rollover_clearing_lock_appends_log_event() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(949000), &[]), now + Duration::days(1))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockCleared);
    assert!(events[0].detail.contains("day rollover"));
}

#[tokio::test]
async fn list_log_filters_by_event_write_date() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .set_rule("position-limit", "20%", now + Duration::days(1))
        .await
        .unwrap();

    let events = service
        .list_log(Some(10), Some(now.date_naive()), None)
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, RiskLogEventType::RuleSet);
    assert!(events[0].detail.contains("daily-loss-limit"));
}

#[tokio::test]
async fn list_log_filters_by_type_and_date_together() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let events = service
        .list_log(
            Some(10),
            Some(now.date_naive()),
            Some(RiskLogEventType::BuyLockReleased),
        )
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockReleased);
}
