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
fn market_leader_requires_one_filter_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "leader"]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(stdout.is_empty(), "expected empty stdout, stdout={stdout}");
    assert!(stderr.contains("the following required arguments were not provided"));
    assert!(stderr.contains("<--sector <SECTOR>|--concept <CONCEPT>|--all>"));
    assert!(stderr.contains("Usage: quantix market leader"));
}

#[test]
fn market_leader_rejects_conflicting_filters_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&[
        "market",
        "leader",
        "--sector",
        "银行",
        "--concept",
        "人工智能",
    ]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(stdout.is_empty(), "expected empty stdout, stdout={stdout}");
    assert!(stderr.contains("cannot be used with"));
    assert!(stderr.contains("--sector <SECTOR>"));
    assert!(stderr.contains("--concept <CONCEPT>"));
    assert!(stderr.contains("Usage: quantix market leader"));
}
