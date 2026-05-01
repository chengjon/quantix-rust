use std::fs;
use std::process::Command;

fn run_quantix(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args(args)
        .output()
        .expect("should run quantix binary");

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

#[test]
fn data_validate_fundamentals_help_lists_input_option() {
    let (stdout, stderr, success) =
        run_quantix(&["data", "validate-fundamentals", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("validate-fundamentals"));
    assert!(stdout.contains("校验本地市场基础面 JSON 文件，不写入 ClickHouse"));
    assert!(stdout.contains("--input <INPUT>"));
}

#[test]
fn data_validate_fundamentals_outputs_summary_fields_for_valid_json() {
    let dir = tempfile::tempdir().expect("should create tempdir");
    let input_path = dir.path().join("market_fundamentals.json");
    fs::write(
        &input_path,
        r#"[
  {"code":"600519","snapshot_date":"2026-03-14","market_cap":23000.5,"latest_report_profit":862.1,"profit_source":"report","pe_dynamic":27.4},
  {"code":"601398","snapshot_date":"2026-03-15","market_cap":18000.0,"latest_report_profit":95.4,"profit_source":"manual","pe_dynamic":6.2}
]"#,
    )
    .expect("should write market fundamentals fixture");

    let input = input_path.to_string_lossy().into_owned();
    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "validate-fundamentals",
        "--input",
        input.as_str(),
    ]);

    assert!(success, "expected success, stdout={stdout}, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("[FIELD] validation_total_records=2"));
    assert!(stdout.contains("[FIELD] validation_unique_codes=2"));
    assert!(stdout.contains("[FIELD] validation_snapshot_min=2026-03-14"));
    assert!(stdout.contains("[FIELD] validation_snapshot_max=2026-03-15"));
    assert!(stdout.contains("[FIELD] validation_market_cap_coverage=2/2"));
    assert!(stdout.contains("[FIELD] validation_latest_report_profit_coverage=2/2"));
    assert!(stdout.contains("[FIELD] validation_profit_sources=manual=1,report=1"));
    assert!(stdout.contains("[PASS] No blocking data-shape issues detected."));
}

#[test]
fn data_validate_fundamentals_rejects_empty_array_at_binary_entry() {
    let dir = tempfile::tempdir().expect("should create tempdir");
    let input_path = dir.path().join("market_fundamentals.json");
    fs::write(&input_path, "[]").expect("should write empty fixture");

    let input = input_path.to_string_lossy().into_owned();
    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "validate-fundamentals",
        "--input",
        input.as_str(),
    ]);

    assert!(
        !success,
        "expected failure for empty array, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("没有可校验的市场基础面记录"),
        "expected empty-input guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains("[FIELD] validation_total_records="),
        "expected no validation summary on blocking failure, stdout={stdout}"
    );
}
