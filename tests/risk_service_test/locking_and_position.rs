use super::helpers::*;

#[tokio::test]
async fn day_rollover_replaces_baseline_and_clears_daily_lock() {
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

    let locked = service
        .sync_after_trade_snapshot(&snapshot(dec!(940000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    assert!(locked.buy_locked);

    let next_day = now + Duration::days(1);
    let reset = service
        .status(&snapshot(dec!(930000), &[]), next_day)
        .await
        .unwrap();

    assert!(!reset.buy_locked);
    assert_eq!(reset.starting_total_assets, dec!(930000));
    assert_eq!(reset.daily_pnl, dec!(0));

    let state = store.snapshot().unwrap();
    assert_eq!(
        state.daily_baseline.unwrap().trading_date,
        next_day.date_naive()
    );
    assert!(!state.buy_lock.locked);
}

#[tokio::test]
async fn daily_loss_amount_triggers_buy_lock() {
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

    let status = service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(2))
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert!(status.lock_reason.unwrap().contains("50000"));
}

#[tokio::test]
async fn daily_loss_percentage_triggers_buy_lock() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "5%", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();

    let status = service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(2))
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert_eq!(status.daily_pnl_pct, dec!(-5.1));
    assert!(status.lock_reason.unwrap().contains("5%"));
    assert_eq!(
        status.lock_state_source,
        RiskLockStateSource::DailyLossLocked
    );
    assert!(status.lock_trigger_reason.as_ref().unwrap().contains("5%"));
    assert_eq!(status.lock_triggered_at, Some(now + Duration::hours(2)));
    assert_eq!(status.lock_effective_trading_date, Some(now.date_naive()));
}

#[tokio::test]
async fn current_lock_blocks_new_buys() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[("000001", dec!(100000))]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(
            &snapshot(dec!(949000), &[("000001", dec!(100000))]),
            now + Duration::hours(1),
        )
        .await
        .unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(949000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(200000), dec!(948500)),
            now + Duration::hours(2),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("锁定"));
}

#[tokio::test]
async fn position_limit_rejects_a_projected_buy_that_would_exceed_the_cap() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("position-limit", "20%", now)
        .await
        .unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(150000))]),
            &ProjectedBuyImpact::new("000001", dec!(250000), dec!(999500)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("position-limit"));
}

#[tokio::test]
async fn sell_sync_remains_allowed_while_buy_lock_is_active() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[("000001", dec!(200000))]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(
            &snapshot(dec!(949000), &[("000001", dec!(180000))]),
            now + Duration::hours(1),
        )
        .await
        .unwrap();

    let status = service
        .sync_after_trade_snapshot(
            &snapshot(dec!(960000), &[("000001", dec!(120000))]),
            now + Duration::hours(2),
        )
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert_eq!(status.position_ratios[0].ratio_pct, dec!(12.5));
    assert!(store.snapshot().unwrap().buy_lock.locked);
}

#[tokio::test]
async fn release_buy_lock_suppresses_same_day_relock() {
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

    let released = service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();
    assert!(!released.locked);
    assert_eq!(released.released_for_date, Some(now.date_naive()));

    let status = service
        .status(&snapshot(dec!(949000), &[]), now + Duration::hours(3))
        .await
        .unwrap();
    assert!(!status.buy_locked);
    assert!(status.manual_release_active);
    assert_eq!(
        status.lock_state_source,
        RiskLockStateSource::ManualReleaseActive
    );
    assert!(
        status
            .lock_trigger_reason
            .as_ref()
            .unwrap()
            .contains("50000")
    );
    assert_eq!(status.lock_triggered_at, Some(now + Duration::hours(1)));
    assert_eq!(status.lock_effective_trading_date, Some(now.date_naive()));

    let state = store.snapshot().unwrap();
    assert_eq!(state.buy_lock.released_for_date, Some(now.date_naive()));
    assert!(!state.buy_lock.locked);
}
