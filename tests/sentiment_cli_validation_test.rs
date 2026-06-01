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
fn sentiment_show_fails_closed_without_wired_provider() {
    let (stdout, stderr, success) = run_quantix(&["sentiment", "show", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected sentiment show to fail without a wired provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder sentiment output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("sentiment provider 尚未接线"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn sentiment_history_fails_closed_without_wired_provider() {
    let (stdout, stderr, success) = run_quantix(&["sentiment", "history", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected sentiment history to fail without a wired provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder history output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("sentiment provider 尚未接线"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn sentiment_mentions_fails_closed_without_wired_provider() {
    let (stdout, stderr, success) = run_quantix(&["sentiment", "mentions", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected sentiment mentions to fail without a wired provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder mentions output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("sentiment provider 尚未接线"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}
