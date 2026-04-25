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
fn market_foundation_help_succeeds_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "foundation", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("foundation"));
    assert!(stdout.contains("获取全市场 A 股与行业分类基础数据摘要"));
}

#[test]
fn market_strength_help_succeeds_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "strength", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("strength"));
    assert!(stdout.contains("--date"));
    assert!(stdout.contains("--strong-top"));
    assert!(stdout.contains("--weak-top"));
    assert!(stdout.contains("--stock-top"));
    assert!(stdout.contains("分析强势/弱势行业板块，并输出强势板块个股 Top10"));
}

#[test]
fn market_overview_help_succeeds_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "overview", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("overview"));
    assert!(stdout.contains("市场综合概览"));
    assert!(stdout.contains("--top"));
    assert!(stdout.contains("--date"));
}
