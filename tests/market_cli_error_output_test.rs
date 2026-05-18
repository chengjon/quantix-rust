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
fn market_strength_rejects_invalid_date_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "strength", "--date", "20260309"]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("无效日期格式: 20260309，请使用 YYYY-MM-DD"),
        "expected invalid date guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains("== 强弱板块分析 =="),
        "expected no normal strength report on invalid date, stdout={stdout}"
    );
}

#[test]
fn market_strength_stocks_rejects_invalid_date_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&[
        "market",
        "strength-stocks",
        "--date",
        "2026/03/09",
        "--metric",
        "profit",
    ]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("无效日期格式: 2026/03/09，请使用 YYYY-MM-DD"),
        "expected invalid date guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains("== 强势板块个股排行 =="),
        "expected no normal strength-stocks report on invalid date, stdout={stdout}"
    );
}
