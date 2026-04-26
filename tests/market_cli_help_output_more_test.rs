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
fn market_sector_help_lists_core_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "sector", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("行业板块排名"));
    assert!(stdout.contains("--top"));
    assert!(stdout.contains("--date"));
    assert!(stdout.contains("--sort-by"));
}

#[test]
fn market_concept_help_lists_core_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "concept", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("概念板块排名"));
    assert!(stdout.contains("--top"));
    assert!(stdout.contains("--date"));
    assert!(stdout.contains("--sort-by"));
}

#[test]
fn market_north_help_lists_date_option() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "north", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("北向资金概览"));
    assert!(stdout.contains("--date"));
}

#[test]
fn market_sentiment_help_lists_date_option() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "sentiment", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("市场情绪概览"));
    assert!(stdout.contains("--date"));
}
