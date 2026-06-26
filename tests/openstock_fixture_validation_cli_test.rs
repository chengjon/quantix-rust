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
fn openstock_validate_fixture_reports_local_fixture_summary() {
    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "openstock",
        "validate-fixture",
        "--file",
        "tests/fixtures/openstock/daily_kline.json",
    ]);

    assert!(
        success,
        "expected fixture validation to pass, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.contains("OpenStock 本地 fixture 校验"),
        "expected local validation heading, stdout={stdout}"
    );
    assert!(
        stdout.contains("记录数: 2"),
        "expected parsed record count, stdout={stdout}"
    );
    assert!(
        stdout.contains("代码: 600000"),
        "expected canonical code summary, stdout={stdout}"
    );
    assert!(
        stdout.contains("日期范围: 2026-06-22..2026-06-23"),
        "expected fixture date range, stdout={stdout}"
    );
    assert!(
        stdout.contains("来源: local_fixture"),
        "expected local-only source marker, stdout={stdout}"
    );
    assert!(
        stderr.is_empty(),
        "expected no stderr for valid fixture, stderr={stderr}"
    );
}

#[test]
fn openstock_validate_fixture_fails_closed_without_file() {
    let (stdout, stderr, success) = run_quantix(&["data", "openstock", "validate-fixture"]);

    assert!(
        !success,
        "expected missing fixture path to fail, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no success output without a fixture path, stdout={stdout}"
    );
    assert!(
        stderr.contains("--file"),
        "expected clap to require --file, stderr={stderr}"
    );
}
