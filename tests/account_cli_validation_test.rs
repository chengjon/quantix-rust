use std::path::Path;
use std::process::Command;

fn run_quantix(args: &[&str]) -> (String, String, bool) {
    let temp_home = tempfile::tempdir().expect("should create isolated HOME for account tests");
    run_quantix_with_home(args, temp_home.path())
}

fn run_quantix_with_home(args: &[&str], home: &Path) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .env("HOME", home)
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
fn account_register_rejects_unsupported_account_type_before_registry_write() {
    let (stdout, stderr, success) = run_quantix(&[
        "account",
        "register",
        "--id",
        "bad-account-type",
        "--account-type",
        "crypto",
        "--capital",
        "10000",
    ]);

    assert!(
        !success,
        "expected account register to fail for invalid account type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no success output for invalid account type, stdout={stdout}"
    );
    assert!(
        stderr.contains("无效的账户类型: crypto，支持: paper, mock_live, qmt_live"),
        "expected account type guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for invalid account type, stderr={stderr}"
    );
}

#[test]
fn account_split_rejects_unsupported_target_type_before_split_output() {
    let temp_home = tempfile::tempdir().expect("should create isolated HOME for account tests");
    let (register_stdout, register_stderr, register_success) = run_quantix_with_home(
        &[
            "account",
            "register",
            "--id",
            "paper-main",
            "--account-type",
            "paper",
            "--capital",
            "10000",
        ],
        temp_home.path(),
    );
    assert!(
        register_success,
        "expected account fixture registration to succeed, stdout={register_stdout}, stderr={register_stderr}"
    );

    let (stdout, stderr, success) = run_quantix_with_home(
        &[
            "account",
            "split",
            "--code",
            "000001",
            "--side",
            "buy",
            "--quantity",
            "100",
            "--target-type",
            "desk",
            "--target-id",
            "paper-main",
        ],
        temp_home.path(),
    );

    assert!(
        !success,
        "expected account split to fail for invalid target type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        !stdout.contains("订单拆分预览"),
        "expected no split preview on invalid target type, stdout={stdout}"
    );
    assert!(
        stderr.contains("无效的目标类型: desk，支持: single, group"),
        "expected target type guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for invalid target type, stderr={stderr}"
    );
}

#[test]
fn account_group_set_strategy_rejects_unsupported_strategy_as_unsupported() {
    let temp_home = tempfile::tempdir().expect("should create isolated HOME for account tests");
    let (create_stdout, create_stderr, create_success) = run_quantix_with_home(
        &[
            "account", "group", "create", "--id", "desk-a", "--name", "Desk A",
        ],
        temp_home.path(),
    );
    assert!(
        create_success,
        "expected account group fixture creation to succeed, stdout={create_stdout}, stderr={create_stderr}"
    );

    let (stdout, stderr, success) = run_quantix_with_home(
        &[
            "account",
            "group",
            "set-strategy",
            "--group-id",
            "desk-a",
            "--strategy",
            "round_robin",
        ],
        temp_home.path(),
    );

    assert!(
        !success,
        "expected account group set-strategy to fail for invalid strategy, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no success output for invalid strategy, stdout={stdout}"
    );
    assert!(
        stderr.contains(
            "无效的分配策略: round_robin，支持: equal, proportional, weighted, primary_first"
        ),
        "expected allocation strategy guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for invalid strategy, stderr={stderr}"
    );
}
