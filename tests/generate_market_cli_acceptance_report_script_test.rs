use std::fs;
use std::process::Command;

#[test]
fn acceptance_report_generator_references_expected_logs_and_sections() {
    let script = fs::read_to_string("scripts/dev/generate_market_cli_acceptance_report.sh")
        .expect("should read scripts/dev/generate_market_cli_acceptance_report.sh");

    for expected in [
        "run_market_cli_acceptance_*.log",
        "check_market_cli_prereqs_*.log",
        "verify_market_cli_smoke_*.log",
        "market_cli_formal_sequence_*.log",
        "Market CLI Acceptance Report",
        "precheck: PASS=",
        "fundamentals_state:",
        "fundamentals_rows:",
        "fundamentals_latest_snapshot:",
        "smoke: PASS=",
        "sync industry exit=",
        "market foundation exit=",
        "market strength exit=",
        "summary:",
        "market fundamentals validate exit=",
        "total_stocks:",
        "total_records:",
        "unique_codes:",
        "top_sector:",
        "candidate_stock_count:",
        "snapshot_source:",
        "tdx_coverage:",
        "top_market_cap_stock:",
        "market strength-stocks exit=",
        "sector_filter:",
        "metric:",
        "coverage:",
        "top_row:",
        "FORMAL_MARKET_DATE",
        "REHEARSAL_CMD",
        "STRENGTH_CMD=",
        "STRENGTH_STOCKS_CMD=",
        "quantix risk sync industry --standard shenwan",
        "quantix market foundation",
        "quantix data validate-fundamentals --input /abs/path/market_fundamentals.json",
        "quantix data import-fundamentals --input /abs/path/market_fundamentals.json",
        "quantix market strength --date ${STRENGTH_CMD_DATE} --strong-top 3 --weak-top 3 --stock-top 10",
        "quantix market strength-stocks --date ${STRENGTH_CMD_DATE} --strong-top 3",
        "--metric ${STRENGTH_STOCKS_METRIC_FLAG} --top 10",
        "Generated report:",
    ] {
        assert!(
            script.contains(expected),
            "expected acceptance report generator to contain {expected}"
        );
    }
}

#[test]
fn acceptance_report_generator_extracts_strength_stocks_fields_from_formal_log() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let acceptance_log = tempdir.path().join("run_market_cli_acceptance.log");
    let precheck_log = tempdir.path().join("check_market_cli_prereqs.log");
    let smoke_log = tempdir.path().join("verify_market_cli_smoke.log");
    let formal_log = tempdir.path().join("market_cli_formal_sequence.log");
    let report_path = tempdir.path().join("market_cli_acceptance_report.md");

    fs::write(&acceptance_log, "[INFO] acceptance\n").expect("should write acceptance log");
    fs::write(
        &precheck_log,
        "\
PASS : 3
WARN : 1
FAIL : 0
[FIELD] precheck_market_fundamentals_state=empty
[FIELD] precheck_market_fundamentals_rows=0
[FIELD] precheck_market_fundamentals_latest_snapshot=N/A
",
    )
    .expect("should write precheck log");
    fs::write(&smoke_log, "PASS : 4\nWARN : 2\nFAIL : 0\n").expect("should write smoke log");
    fs::write(
        &formal_log,
        "\
[INFO] Using market date for formal sequence: 2026-03-14
[RESULT] sync_industry_exit=0
[LOG] sync_industry_log=/tmp/sync.log
[SUMMARY] sync_industry_summary=ok
[RESULT] market_foundation_exit=0
[LOG] market_foundation_log=/tmp/foundation.log
[SUMMARY] market_foundation_summary=ok
[RESULT] market_fundamentals_validate_exit=0
[LOG] market_fundamentals_validate_log=/tmp/validate.log
[SUMMARY] market_fundamentals_validate_summary=记录数=2 唯一股票=2 快照区间=2026-03-14~2026-03-15 总市值覆盖=2/2 净利润覆盖=2/2 warnings=0
[FIELD] market_fundamentals_validate_total_records=2
[FIELD] market_fundamentals_validate_unique_codes=2
[FIELD] market_fundamentals_validate_snapshot_min=2026-03-14
[FIELD] market_fundamentals_validate_snapshot_max=2026-03-15
[FIELD] market_fundamentals_validate_market_cap_coverage=2/2
[FIELD] market_fundamentals_validate_latest_report_profit_coverage=2/2
[FIELD] market_fundamentals_validate_warning_count=0
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
[FIELD] market_strength_snapshot_source=tdx_fallback
[FIELD] market_strength_tdx_coverage=4205/4430
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

    let output = Command::new("bash")
        .arg("scripts/dev/generate_market_cli_acceptance_report.sh")
        .env("ACCEPTANCE_LOG", &acceptance_log)
        .env("PRECHECK_LOG", &precheck_log)
        .env("SMOKE_LOG", &smoke_log)
        .env("FORMAL_LOG", &formal_log)
        .env("REPORT_PATH", &report_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run acceptance report generator");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = fs::read_to_string(&report_path).expect("should read generated report");
    assert!(report.contains("fundamentals_state: empty"));
    assert!(report.contains("fundamentals_rows: 0"));
    assert!(report.contains("fundamentals_latest_snapshot: N/A"));
    assert!(report.contains(
        "`scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh` 做 scratch ClickHouse 导入演练"
    ));
    assert!(report.contains("market fundamentals validate exit=0"));
    assert!(report.contains("total_records: 2"));
    assert!(report.contains("unique_codes: 2"));
    assert!(report.contains(
        "`quantix data validate-fundamentals --input /abs/path/market_fundamentals.json`"
    ));
    assert!(report.contains(
        "`quantix data import-fundamentals --input /abs/path/market_fundamentals.json`"
    ));
    assert!(report.contains("snapshot_source: tdx_fallback"));
    assert!(report.contains("tdx_coverage: 4205/4430"));
    assert!(report.contains("market strength-stocks exit=0"));
    assert!(report.contains("sector_filter: 银行"));
    assert!(report.contains("metric: 上一会计周期净利润"));
    assert!(report.contains("coverage: 1/1"));
    assert!(report.contains("top_row: 1 银行 601398 工商银行 7.00 100.00"));
    assert!(report.contains(
        "`quantix market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10`"
    ));
    assert!(report.contains("`quantix market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10`"));
}
