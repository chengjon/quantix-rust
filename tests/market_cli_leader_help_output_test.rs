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
fn market_leader_help_lists_filters_and_core_options() {
    let (stdout, stderr, success) = run_quantix_help(&["market", "leader", "--help"]);

    assert!(success, "expected success, stderr={stderr}");
    assert!(stderr.is_empty(), "expected empty stderr, stderr={stderr}");
    assert!(stdout.contains("龙头股识别"));
    assert!(stdout.contains("Usage: quantix market leader"));
    assert!(stdout.contains("<--sector <SECTOR>|--concept <CONCEPT>|--all>"));
    assert!(stdout.contains("--sector <SECTOR>"));
    assert!(stdout.contains("--concept <CONCEPT>"));
    assert!(stdout.contains("--all"));
    assert!(stdout.contains("--limit <LIMIT>"));
    assert!(stdout.contains("--date <DATE>"));
}
