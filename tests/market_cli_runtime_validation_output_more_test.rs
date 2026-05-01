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
fn market_concept_rejects_unsupported_sort_by_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "concept", "--sort-by", "volume"]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("不支持的 sort_by: volume，仅支持 change 或 change_pct"),
        "expected sort_by guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains("涨跌幅"),
        "expected no normal board output on invalid sort_by, stdout={stdout}"
    );
}

#[test]
fn market_sentiment_rejects_invalid_date_at_binary_entry() {
    let (stdout, stderr, success) = run_quantix(&["market", "sentiment", "--date", "20260309"]);

    assert!(
        !success,
        "expected failure, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("无效日期格式: 20260309，请使用 YYYY-MM-DD"),
        "expected invalid date guidance in stderr, stderr={stderr}"
    );
    assert!(
        !stdout.contains("涨停"),
        "expected no normal sentiment output on invalid date, stdout={stdout}"
    );
}
