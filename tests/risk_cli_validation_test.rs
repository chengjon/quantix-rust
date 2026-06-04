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
fn risk_status_rejects_unsupported_source_as_unsupported() {
    let (stdout, stderr, success) = run_quantix(&["risk", "status", "--source", "warehouse"]);

    assert!(
        !success,
        "expected risk status to fail for unsupported source, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no risk status output for unsupported source, stdout={stdout}"
    );
    assert!(
        stderr.contains("risk --source 不支持的值: warehouse"),
        "expected source guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported source, stderr={stderr}"
    );
}
