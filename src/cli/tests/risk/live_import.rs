use super::*;
use crate::risk::{
    JsonRiskStore, RiskRule, RiskRuleType, RiskState, RiskStore, RuleValue, SqliteLiveImportStore,
};
use std::fs;

#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn run_risk_import_rebuild_and_live_import_status_dispatch_to_handlers() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    let input_path = dir.path().join("live.csv");
    fs::write(
        &input_path,
        "record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at\ncash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z\ntrade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,\n",
    )
    .unwrap();
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

    Cli::try_parse_from([
        "quantix",
        "risk",
        "import",
        "live-trades",
        "--account",
        "live-001",
        "--input",
        input_path.to_str().unwrap(),
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "rebuild",
        "live-account",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    let live_import_path = risk_path.with_file_name("live_import.db");
    let store = SqliteLiveImportStore::new(&live_import_path).await.unwrap();
    let mirror = store
        .get_latest_mirror_account("live-001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mirror.account_id, "live-001");
    assert_eq!(mirror.positions.len(), 1);
}

#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn run_risk_live_import_status_rejects_invalid_daily_loss_rule_value_type() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    let input_path = dir.path().join("live.csv");
    fs::write(
        &input_path,
        "record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at\ncash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z\ntrade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,\n",
    )
    .unwrap();
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

    Cli::try_parse_from([
        "quantix",
        "risk",
        "import",
        "live-trades",
        "--account",
        "live-001",
        "--input",
        input_path.to_str().unwrap(),
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "rebuild",
        "live-account",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    let now = chrono::Utc::now();
    let mut state = RiskState::default();
    state.rules.push(RiskRule {
        rule_type: RiskRuleType::DailyLossLimit,
        value: RuleValue::TextList(vec!["银行".to_string()]),
        enabled: true,
        created_at: now,
        updated_at: now,
    });
    JsonRiskStore::new(&risk_path)
        .save_state(&state)
        .await
        .unwrap();

    let err = Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap_err();

    assert!(err.to_string().contains("daily-loss-limit"));
    assert!(err.to_string().contains("配置无效"));
}

#[tokio::test]
async fn run_risk_live_import_status_requires_account_flag() {
    let cli =
        Cli::try_parse_from(["quantix", "risk", "status", "--source", "live_import"]).unwrap();

    let err = cli.run().await.unwrap_err();
    assert!(err.to_string().contains("--account"));
}
