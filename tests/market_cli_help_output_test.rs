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
fn root_help_lists_market_command() {
    let (stdout, stderr, success) = run_quantix_help(&["--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("market"));
    assert!(stdout.contains("市场分析命令"));
}

#[test]
fn market_help_lists_strength_and_strength_stocks() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("strength"));
    assert!(stdout.contains("strength-stocks"));
    assert!(stdout.contains("分析强势/弱势行业板块，并输出强势板块个股 Top10"));
    assert!(stdout.contains("仅输出强势板块个股排行"));
}

#[test]
fn strength_stocks_help_lists_key_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "strength-stocks", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("--strong-top"));
    assert!(stdout.contains("--sector"));
    assert!(stdout.contains("--metric"));
    assert!(stdout.contains("--top"));
    assert!(stdout.contains("market-cap"));
    assert!(stdout.contains("profit"));
}
