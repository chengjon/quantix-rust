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
fn data_export_fails_closed_for_unknown_format_before_output() {
    let output_dir = tempfile::tempdir().expect("should create temp output dir");
    let output_dir = output_dir.path().to_string_lossy().into_owned();

    let (stdout, stderr, success) = run_quantix(&[
        "data",
        "export",
        "--code",
        "600519",
        "--format",
        "xml",
        "--output",
        &output_dir,
    ]);

    assert!(
        !success,
        "expected data export to fail for unknown format, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no data export output before format validation failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("data export format 不支持"),
        "expected explicit data export format boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("csv") && stderr.contains("parquet"),
        "expected supported formats in stderr, stderr={stderr}"
    );
}
