use std::process::Command;

fn run_quantix_help(args: &[&str]) -> (String, String, bool) {
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
fn market_foundation_help_lists_title_and_usage() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "foundation", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("获取全市场 A 股与行业分类基础数据摘要"));
    assert!(stdout.contains("Usage: quantix market foundation"));
}

#[test]
fn market_overview_help_lists_title_and_core_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "overview", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("市场综合概览"));
    assert!(stdout.contains("Usage: quantix market overview [OPTIONS]"));
    assert!(stdout.contains("--top <TOP>"));
    assert!(stdout.contains("--date <DATE>"));
}

#[test]
fn market_strength_help_lists_title_and_core_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "strength", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("分析强势/弱势行业板块，并输出强势板块个股 Top10"));
    assert!(stdout.contains("Usage: quantix market strength [OPTIONS]"));
    assert!(stdout.contains("--date <DATE>"));
    assert!(stdout.contains("--strong-top <STRONG_TOP>"));
    assert!(stdout.contains("--weak-top <WEAK_TOP>"));
    assert!(stdout.contains("--stock-top <STOCK_TOP>"));
}
