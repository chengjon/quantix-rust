use std::process::Command;

fn run_quantix(args: &[&str]) -> (String, String, bool) {
    let temp_home = tempfile::tempdir().expect("should create isolated HOME for account tests");
    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .env("HOME", temp_home.path())
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
