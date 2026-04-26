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
fn market_strength_stocks_rejects_invalid_metric_value() {
    let (stdout, stderr, success) = run_quantix(&[
        "market",
        "strength-stocks",
        "--metric",
        "invalid",
    ]);

    assert!(!success, "expected failure, stdout={stdout}, stderr={stderr}");
    assert!(stdout.is_empty(), "expected empty stdout, stdout={stdout}");
    assert!(
        stderr.contains("invalid value 'invalid' for '--metric <METRIC>'"),
        "expected clap invalid metric guidance, stderr={stderr}"
    );
    assert!(
        stderr.contains("[possible values: market-cap, profit]"),
        "expected clap possible values in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("For more information, try '--help'."),
        "expected clap help hint in stderr, stderr={stderr}"
    );
}
