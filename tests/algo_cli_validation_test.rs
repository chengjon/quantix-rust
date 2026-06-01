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
fn algo_plan_rejects_invalid_side_before_emitting_preview() {
    let (stdout, stderr, success) = run_quantix(&[
        "algo",
        "plan",
        "--code",
        "600519.SH",
        "--side",
        "hold",
        "--quantity",
        "1000",
        "--algo-type",
        "twap",
        "--duration",
        "10",
        "--slices",
        "2",
        "--output",
        "json",
    ]);

    assert!(
        !success,
        "expected algo plan to fail for invalid side, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no slice preview on invalid side, stdout={stdout}"
    );
    assert!(
        stderr.contains("Side must be 'buy' or 'sell'"),
        "expected side validation guidance in stderr, stderr={stderr}"
    );
}
