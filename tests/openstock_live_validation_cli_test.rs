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

const VALID_PAYLOAD: &str = r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[
    {"symbol":"600000","time":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"amount":"12345678.90","period":"daily"},
    {"symbol":"600000","time":"2026-06-23","open":"10.05","high":"10.30","low":"9.95","close":"10.20","volume":2345678,"amount":"23456789.01","period":"daily"}
]}"#;

fn write_payload(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("openstock_live_validation_cli_test");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write payload");
    path.to_string_lossy().into_owned()
}

#[test]
fn validate_live_emits_dry_run_report_for_valid_payload() {
    let path = write_payload("valid.json", VALID_PAYLOAD);
    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "openstock",
        "validate-live",
        "--payload",
        &path,
        "--symbol",
        "600000",
        "--period",
        "daily",
        "--start",
        "2026-06-22",
        "--end",
        "2026-06-23",
    ]);

    assert!(
        success,
        "expected validate-live to succeed, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.contains("OpenStock live shadow validation"),
        "expected report heading, stdout={stdout}"
    );
    assert!(
        stdout.contains("dry_run: true"),
        "expected dry_run marker, stdout={stdout}"
    );
    assert!(
        stdout.contains("status: ok"),
        "expected ok status, stdout={stdout}"
    );
    assert!(
        stdout.contains("symbol: 600000"),
        "expected symbol summary, stdout={stdout}"
    );
    assert!(
        stderr.is_empty(),
        "expected no stderr for valid payload, stderr={stderr}"
    );
}

#[test]
fn validate_live_reports_drift_when_limit_is_smaller_than_returned_count() {
    let path = write_payload("drift.json", VALID_PAYLOAD);
    let (stdout, _stderr, success) = run_quantix(&[
        "data",
        "openstock",
        "validate-live",
        "--payload",
        &path,
        "--symbol",
        "600000",
        "--start",
        "2026-06-22",
        "--end",
        "2026-06-23",
        "--limit",
        "1",
    ]);

    assert!(success, "drift should not abort the CLI");
    assert!(
        stdout.contains("status: drift"),
        "expected drift status, stdout={stdout}"
    );
    assert!(
        stdout.contains("received_count_exceeds_limit"),
        "expected limit drift rule, stdout={stdout}"
    );
}

#[test]
fn validate_live_fails_closed_on_missing_symbol_field() {
    let broken = r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[
        {"symbol":null,"time":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"period":"daily"}
    ]}"#;
    let path = write_payload("broken.json", broken);
    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "openstock",
        "validate-live",
        "--payload",
        &path,
        "--symbol",
        "600000",
        "--start",
        "2026-06-22",
        "--end",
        "2026-06-23",
    ]);

    assert!(
        success,
        "fail-closed should still exit 0 because the report was produced; stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.contains("status: fail_closed"),
        "expected fail_closed status, stdout={stdout}"
    );
    assert!(
        stdout.contains("fail_closed_errors"),
        "expected fail_closed errors section, stdout={stdout}"
    );
}

#[test]
fn validate_live_fails_when_payload_file_is_missing() {
    let (_stdout, stderr, success) = run_quantix(&[
        "data",
        "openstock",
        "validate-live",
        "--payload",
        "/nonexistent/openstock-payload.json",
        "--symbol",
        "600000",
        "--start",
        "2026-06-22",
        "--end",
        "2026-06-23",
    ]);

    assert!(
        !success,
        "missing payload file must abort the CLI, stderr={stderr}"
    );
    assert!(
        stderr.contains("读取 OpenStock 线上响应失败"),
        "expected io error message, stderr={stderr}"
    );
}
