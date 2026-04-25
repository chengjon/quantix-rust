use std::fs;
use std::process::Command;

#[test]
fn market_cli_acceptance_orchestrator_output_feeds_report_generator() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let acceptance_log = log_dir.join("run_market_cli_acceptance.log");
    let precheck_log = log_dir.join("check_market_cli_prereqs.log");
    let smoke_log = log_dir.join("verify_market_cli_smoke.log");
    let formal_log = log_dir.join("market_cli_formal_sequence.log");
    let report_path = log_dir.join("market_cli_acceptance_report.md");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let fake_local_env = tempdir.path().join(".env.market.local");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_precheck = tempdir.path().join("fake-precheck.sh");
    let fake_smoke = tempdir.path().join("fake-smoke.sh");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(&fake_env_template, "# fake env template\n").expect("should write env template");
    fs::write(&fake_local_env, "export MARKET_FAKE_ENV=1\n").expect("should write local env");
    fs::write(
        &fake_init,
        "#!/usr/bin/env bash\nset -euo pipefail\necho '[FAKE] init'\n",
    )
    .expect("should write fake init");
    fs::write(
        &fake_precheck,
        format!(
            "#!/usr/bin/env bash\nset -euo pipefail\ncat <<'EOF' | tee \"{}\"\n[FAKE] precheck\nPASS : 2\nWARN : 1\nFAIL : 0\nEOF\n",
            precheck_log.display()
        ),
    )
    .expect("should write fake precheck");
    fs::write(
        &fake_smoke,
        format!(
            "#!/usr/bin/env bash\nset -euo pipefail\ncat <<'EOF' | tee \"{}\"\n[FAKE] smoke\nPASS : 3\nWARN : 2\nFAIL : 0\nEOF\n",
            smoke_log.display()
        ),
    )
    .expect("should write fake smoke");

    for path in [&fake_init, &fake_precheck, &fake_smoke] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let acceptance_output = Command::new("bash")
        .arg("scripts/dev/run_market_cli_acceptance.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &acceptance_log)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("LOCAL_ENV_PATH", &fake_local_env)
        .env("INIT_LOCAL_ENV_SCRIPT", &fake_init)
        .env("PRECHECK_SCRIPT", &fake_precheck)
        .env("SMOKE_SCRIPT", &fake_smoke)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run acceptance orchestrator");

    assert!(
        acceptance_output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&acceptance_output.stderr)
    );

    fs::write(
        &formal_log,
        "\
[RESULT] sync_industry_exit=0
[LOG] sync_industry_log=/tmp/sync.log
[SUMMARY] sync_industry_summary=ok
[RESULT] market_foundation_exit=0
[LOG] market_foundation_log=/tmp/foundation.log
[SUMMARY] market_foundation_summary=ok
[FIELD] market_foundation_total_stocks=5300
[FIELD] market_foundation_classified_stocks=5200
[FIELD] market_foundation_unclassified_stocks=100
[FIELD] market_foundation_sector_count=31
[FIELD] market_foundation_top_sector=1 银行 42
[RESULT] market_strength_exit=0
[LOG] market_strength_log=/tmp/strength.log
[SUMMARY] market_strength_summary=ok
[FIELD] market_strength_base=A股=5300 行业覆盖=5200 未覆盖=100
[FIELD] market_strength_candidate_stock_count=12
[FIELD] market_strength_top_strong_sector=1 BK001 银行 2.10%
[FIELD] market_strength_top_weak_sector=1 BK999 有色金属 -1.80%
[FIELD] market_strength_top_market_cap_stock=1 银行 601398 工商银行 7.00 7000.00
[FIELD] market_strength_top_profit_stock=1 银行 601398 工商银行 7.00 100.00
[RESULT] market_strength_stocks_exit=0
[LOG] market_strength_stocks_log=/tmp/strength_stocks.log
[SUMMARY] market_strength_stocks_summary=行业过滤=银行; 指标=上一会计周期净利润; 覆盖=1/1; 首行=1 银行 601398 工商银行 7.00 100.00
[FIELD] market_strength_stocks_sector_filter=银行
[FIELD] market_strength_stocks_metric=上一会计周期净利润
[FIELD] market_strength_stocks_coverage=1/1
[FIELD] market_strength_stocks_top_row=1 银行 601398 工商银行 7.00 100.00
",
    )
    .expect("should write formal log");

    let report_output = Command::new("bash")
        .arg("scripts/dev/generate_market_cli_acceptance_report.sh")
        .env("ACCEPTANCE_LOG", &acceptance_log)
        .env("PRECHECK_LOG", &precheck_log)
        .env("SMOKE_LOG", &smoke_log)
        .env("FORMAL_LOG", &formal_log)
        .env("REPORT_PATH", &report_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run report generator");

    assert!(
        report_output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&report_output.stderr)
    );

    let acceptance = fs::read_to_string(&acceptance_log).expect("should read acceptance log");
    assert!(acceptance.contains("[STEP] Environment precheck"));
    assert!(acceptance.contains("[FAKE] precheck"));
    assert!(acceptance.contains("[STEP] Smoke verification"));
    assert!(acceptance.contains("[FAKE] smoke"));

    let report = fs::read_to_string(&report_path).expect("should read generated report");
    assert!(report.contains(&format!(
        "- acceptance orchestrator: {}",
        acceptance_log.display()
    )));
    assert!(report.contains(&format!("- precheck: {}", precheck_log.display())));
    assert!(report.contains(&format!("- smoke: {}", smoke_log.display())));
    assert!(report.contains(&format!("- formal sequence: {}", formal_log.display())));
    assert!(report.contains("- precheck: PASS=2 WARN=1 FAIL=0"));
    assert!(report.contains("- smoke: PASS=3 WARN=2 FAIL=0"));
    assert!(report.contains("market strength-stocks exit=0"));
    assert!(report.contains("sector_filter: 银行"));
    assert!(report.contains("metric: 上一会计周期净利润"));
    assert!(report.contains("top_row: 1 银行 601398 工商银行 7.00 100.00"));
}
