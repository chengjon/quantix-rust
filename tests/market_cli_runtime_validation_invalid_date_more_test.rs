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

fn assert_invalid_date_failure(args: &[&str], normal_output_marker: &str) {
    let (stdout, stderr, success) = run_quantix(args);

    assert!(!success, "expected failure, stdout={stdout}, stderr={stderr}");
    assert!(
        stderr.contains("无效日期格式: 20260309，请使用 YYYY-MM-DD"),
        "expected invalid date guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains(normal_output_marker),
        "expected no normal output marker {normal_output_marker:?}, stdout={stdout}"
    );
}

#[test]
fn market_sector_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(&["market", "sector", "--date", "20260309"], "涨跌幅");
}

#[test]
fn market_concept_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(&["market", "concept", "--date", "20260309"], "涨跌幅");
}

#[test]
fn market_leader_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(&["market", "leader", "--all", "--date", "20260309"], "龙头股");
}

#[test]
fn market_overview_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(&["market", "overview", "--date", "20260309"], "== 市场概览 ==");
}

#[test]
fn market_strength_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(&["market", "strength", "--date", "20260309"], "== 强弱板块分析 ==");
}

#[test]
fn market_strength_stocks_rejects_invalid_date_at_binary_entry() {
    assert_invalid_date_failure(
        &["market", "strength-stocks", "--date", "20260309"],
        "== 强势板块个股排行 ==",
    );
}
