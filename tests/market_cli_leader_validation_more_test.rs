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
fn market_leader_missing_filter_includes_help_hint() {
    let (stdout, stderr, success) = run_quantix(&["market", "leader"]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(stdout.is_empty(), "expected empty stdout, stdout={stdout}");
    assert!(
        stderr.contains("error: the following required arguments were not provided:"),
        "expected missing filter error in stderr, stderr={stderr}"
    );
    assert!(stderr.contains("<--sector <SECTOR>|--concept <CONCEPT>|--all>"));
    assert!(
        stderr
            .contains("Usage: quantix market leader <--sector <SECTOR>|--concept <CONCEPT>|--all>")
    );
    assert!(
        stderr.contains("For more information, try '--help'."),
        "expected clap help hint in stderr, stderr={stderr}"
    );
}

#[test]
fn market_leader_conflicting_filters_include_help_hint() {
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
    assert!(
        stderr.contains(
            "error: the argument '--sector <SECTOR>' cannot be used with '--concept <CONCEPT>'"
        ),
        "expected conflicting filter error in stderr, stderr={stderr}"
    );
    assert!(
        stderr
            .contains("Usage: quantix market leader <--sector <SECTOR>|--concept <CONCEPT>|--all>")
    );
    assert!(
        stderr.contains("For more information, try '--help'."),
        "expected clap help hint in stderr, stderr={stderr}"
    );
}
