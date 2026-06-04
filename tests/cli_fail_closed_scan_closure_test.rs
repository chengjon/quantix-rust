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
fn monitor_event_list_rejects_unsupported_type_as_unsupported() {
    let (stdout, stderr, success) =
        run_quantix(&["monitor", "event", "list", "--type", "dividend"]);

    assert!(
        !success,
        "expected monitor event list to fail for unsupported event type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no monitor event output for unsupported event type, stdout={stdout}"
    );
    assert!(
        stderr.contains("monitor event list 不支持的事件类型: dividend"),
        "expected event type guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported monitor event type, stderr={stderr}"
    );
}

#[test]
fn stop_history_rejects_unsupported_event_type_as_unsupported() {
    let (stdout, stderr, success) = run_quantix(&["stop", "history", "--type", "dividend"]);

    assert!(
        !success,
        "expected stop history to fail for unsupported event type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no stop history output for unsupported event type, stdout={stdout}"
    );
    assert!(
        stderr.contains("未知 stop history event_type: dividend"),
        "expected stop history event type guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported stop history event type, stderr={stderr}"
    );
}

#[test]
fn strategy_request_list_rejects_unsupported_status_as_unsupported() {
    let (stdout, stderr, success) =
        run_quantix(&["strategy", "request", "list", "--status", "archived"]);

    assert!(
        !success,
        "expected strategy request list to fail for unsupported status, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no strategy request output for unsupported status, stdout={stdout}"
    );
    assert!(
        stderr.contains("未知 request_status: archived"),
        "expected request status guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported request status, stderr={stderr}"
    );
}
