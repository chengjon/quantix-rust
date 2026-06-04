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

#[test]
fn algo_plan_rejects_unknown_output_format_before_emitting_preview() {
    let (stdout, stderr, success) = run_quantix(&[
        "algo",
        "plan",
        "--code",
        "600519.SH",
        "--side",
        "buy",
        "--quantity",
        "1000",
        "--algo-type",
        "twap",
        "--duration",
        "10",
        "--slices",
        "2",
        "--output",
        "yaml",
    ]);

    assert!(
        !success,
        "expected algo plan to fail for unsupported output format, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no slice preview on unsupported output format, stdout={stdout}"
    );
    assert!(
        stderr.contains("不支持的输出格式: yaml，仅支持 table 或 json"),
        "expected output format validation guidance in stderr, stderr={stderr}"
    );
}

#[test]
fn algo_plan_rejects_unsupported_algo_type_before_emitting_preview() {
    let (stdout, stderr, success) = run_quantix(&[
        "algo",
        "plan",
        "--code",
        "600519.SH",
        "--side",
        "buy",
        "--quantity",
        "1000",
        "--algo-type",
        "iceberg",
        "--duration",
        "10",
        "--slices",
        "2",
        "--output",
        "json",
    ]);

    assert!(
        !success,
        "expected algo plan to fail for unsupported algo type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no slice preview on unsupported algo type, stdout={stdout}"
    );
    assert!(
        stderr.contains("不支持的算法类型: iceberg"),
        "expected algo type validation guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported algo type, stderr={stderr}"
    );
}

#[test]
fn algo_create_rejects_unsupported_algo_type_before_task_output() {
    let (stdout, stderr, success) = run_quantix(&[
        "algo",
        "create",
        "--code",
        "600519.SH",
        "--side",
        "buy",
        "--quantity",
        "1000",
        "--algo-type",
        "iceberg",
        "--duration",
        "10",
    ]);

    assert!(
        !success,
        "expected algo create to fail for unsupported algo type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        !stdout.contains("算法任务已创建"),
        "expected no task creation output on unsupported algo type, stdout={stdout}"
    );
    assert!(
        stderr.contains("不支持的算法类型: iceberg"),
        "expected algo type validation guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported algo type, stderr={stderr}"
    );
}
