use super::helpers::*;

#[tokio::test]
async fn set_rule_upserts_position_limit_percentage_values() {
    let (service, store) = service();
    let now = fixed_ts();

    let first = service
        .set_rule("position-limit", "20%", now)
        .await
        .unwrap();
    assert_eq!(first.rule_type, RiskRuleType::PositionLimit);
    assert_eq!(first.value, RuleValue::Percentage(dec!(20)));
    assert!(first.enabled);

    let updated = service
        .set_rule("position-limit", "25%", now + Duration::minutes(1))
        .await
        .unwrap();
    assert_eq!(updated.value, RuleValue::Percentage(dec!(25)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(25)));
}

#[tokio::test]
async fn set_rule_upserts_daily_loss_amount_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("daily-loss-limit", "50000", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.rule_type, RiskRuleType::DailyLossLimit);
    assert_eq!(rule.value, RuleValue::Amount(dec!(50000)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Amount(dec!(50000)));
}

#[tokio::test]
async fn set_rule_upserts_daily_loss_percentage_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("daily-loss-limit", "5%", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.value, RuleValue::Percentage(dec!(5)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(5)));
}

#[tokio::test]
async fn set_rule_upserts_volatility_limit_percentage_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.rule_type, RiskRuleType::VolatilityLimit);
    assert_eq!(rule.value, RuleValue::Percentage(dec!(4)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::VolatilityLimit);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(4)));
}

#[tokio::test]
async fn set_rule_upserts_industry_blocklist_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("industry-blocklist", "银行,地产", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.rule_type, RiskRuleType::IndustryBlocklist);
    assert_eq!(
        rule.value,
        RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()])
    );

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::IndustryBlocklist);
    assert_eq!(
        state.rules[0].value,
        RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()])
    );
}

#[tokio::test]
async fn set_rule_industry_blocklist_ignores_empty_segments() {
    let cases = vec![
        ("银行, ,地产", vec!["银行".to_string(), "地产".to_string()]),
        ("银行,", vec!["银行".to_string()]),
        (",地产", vec!["地产".to_string()]),
        ("银行,,地产", vec!["银行".to_string(), "地产".to_string()]),
    ];

    for (raw, expected) in cases {
        let (service, store) = service();

        let rule = service
            .set_rule("industry-blocklist", raw, fixed_ts())
            .await
            .unwrap();

        assert_eq!(rule.rule_type, RiskRuleType::IndustryBlocklist);
        assert_eq!(rule.value, RuleValue::TextList(expected.clone()));

        let state = store.snapshot().unwrap();
        assert_eq!(state.rules.len(), 1);
        assert_eq!(state.rules[0].value, RuleValue::TextList(expected));
    }
}

#[tokio::test]
async fn set_rule_industry_blocklist_rejects_empty_names() {
    let (service, _) = service();

    let err = service
        .set_rule("industry-blocklist", " , , ", fixed_ts())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("industry-blocklist"));
}

#[tokio::test]
async fn industry_blocklist_rejects_buy_when_resolved_industry_is_blocked_without_lock_or_log() {
    let (service, store) =
        service_with_industry_resolver(FakeIndustryResolver::with_rows(&[("000001", "银行")]));
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    let before = store.snapshot().unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("industry-blocklist"));
    assert!(err.to_string().contains("银行"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before.events.len());
}

#[tokio::test]
async fn industry_blocklist_allows_buy_when_resolved_industry_is_not_blocked() {
    let (service, _) =
        service_with_industry_resolver(FakeIndustryResolver::with_rows(&[("000001", "有色金属")]));
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();

    service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn industry_blocklist_returns_hard_check_failure_when_resolver_misses() {
    let (service, store) = service_with_industry_resolver(FakeIndustryResolver::default());
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();
    let before = store.snapshot().unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, QuantixError::DataSource(_)));
    assert!(err.to_string().contains("industry-blocklist"));
    assert!(err.to_string().contains("检查失败"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before.events.len());
}

#[test]
fn text_list_display_joins_values_with_commas() {
    let value = RuleValue::TextList(vec![
        "银行".to_string(),
        "地产".to_string(),
        "煤炭".to_string(),
    ]);
    assert_eq!(value.display(), "银行,地产,煤炭");
}

#[tokio::test]
async fn status_rejects_invalid_daily_loss_rule_value_type() {
    let (service, store) = service();
    let now = fixed_ts();
    let mut state = RiskState::default();
    state.rules.push(RiskRule {
        rule_type: RiskRuleType::DailyLossLimit,
        value: RuleValue::TextList(vec!["银行".to_string()]),
        enabled: true,
        created_at: now,
        updated_at: now,
    });
    store.set_snapshot(state);

    let err = service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("daily-loss-limit"));
    assert!(err.to_string().contains("配置无效"));
}

#[tokio::test]
async fn position_limit_rejects_amount_syntax() {
    let (service, _) = service();

    let err = service
        .set_rule("position-limit", "50000", fixed_ts())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("position-limit"));
}

#[tokio::test]
async fn volatility_limit_rejects_amount_syntax() {
    let (service, _) = service();

    let err = service
        .set_rule("volatility-limit", "50000", fixed_ts())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("volatility-limit"));
}

#[tokio::test]
async fn set_rule_rejects_unknown_rule_type_or_malformed_value() {
    let (service, _) = service();

    let unknown_rule = service
        .set_rule("unknown-rule", "5%", fixed_ts())
        .await
        .unwrap_err();
    assert!(unknown_rule.to_string().contains("unknown-rule"));

    let malformed_value = service
        .set_rule("daily-loss-limit", "abc%", fixed_ts())
        .await
        .unwrap_err();
    assert!(malformed_value.to_string().contains("abc%"));
}

#[tokio::test]
async fn enable_and_disable_rule_update_existing_rule() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "5%", now)
        .await
        .unwrap();

    let disabled = service
        .disable_rule("daily-loss-limit", now + Duration::minutes(1))
        .await
        .unwrap();
    assert!(!disabled.enabled);

    let enabled = service
        .enable_rule("daily-loss-limit", now + Duration::minutes(2))
        .await
        .unwrap();
    assert!(enabled.enabled);
}

#[tokio::test]
async fn status_initializes_the_current_day_baseline_when_missing() {
    let (service, store) = service();

    let status = service
        .status(
            &snapshot(dec!(1000000), &[("000001", dec!(200000))]),
            fixed_ts(),
        )
        .await
        .unwrap();

    assert_eq!(status.starting_total_assets, dec!(1000000));
    assert_eq!(status.current_total_assets, dec!(1000000));
    assert_eq!(status.daily_pnl, dec!(0));
    assert_eq!(status.daily_pnl_pct, dec!(0));
    assert!(!status.buy_locked);
    assert_eq!(status.lock_state_source, RiskLockStateSource::Open);
    assert_eq!(status.lock_trigger_reason, None);
    assert_eq!(status.lock_triggered_at, None);
    assert_eq!(status.lock_effective_trading_date, None);
    assert_eq!(status.position_ratios.len(), 1);
    assert_eq!(status.position_ratios[0].code, "000001");
    assert_eq!(status.position_ratios[0].ratio_pct, dec!(20));

    let state = store.snapshot().unwrap();
    assert_eq!(
        state.daily_baseline.unwrap().trading_date,
        fixed_ts().date_naive()
    );
    assert!(!state.buy_lock.locked);
}
