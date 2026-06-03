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
fn screener_run_rejects_unsupported_sort_by_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&[
        "analyze",
        "screener",
        "run",
        "--codes",
        "000001",
        "--preset",
        "close_above_ma:period=3",
        "--sort-by",
        "volume",
    ]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("不支持的 sort_by: volume，仅支持 code 或 score"),
        "expected sort_by guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for screener sort_by boundary, stderr={stderr}"
    );
    assert!(
        !stdout.contains("筛选结果"),
        "expected no normal screener output on invalid sort_by, stdout={stdout}"
    );
}
