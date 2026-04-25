use std::fs;
use std::process::Command;

#[test]
fn init_market_cli_local_env_script_covers_copy_and_placeholder_validation() {
    let script = fs::read_to_string("scripts/dev/init_market_cli_local_env.sh")
        .expect("should read scripts/dev/init_market_cli_local_env.sh");

    for expected in [
        ".env.market.local.example",
        ".env.market.local",
        "EXAMPLE_PATH=\"${EXAMPLE_PATH:-$ROOT_DIR/.env.market.local.example}\"",
        "LOCAL_PATH=\"${LOCAL_PATH:-$ROOT_DIR/.env.market.local}\"",
        "cp \"$EXAMPLE_PATH\" \"$LOCAL_PATH\"",
        "replace-me",
        "[WARN] placeholder values still present",
        "[PASS] local market env ready",
    ] {
        assert!(
            script.contains(expected),
            "expected init helper to contain {expected}"
        );
    }
}

#[test]
fn init_market_cli_local_env_script_copies_example_and_returns_warn_for_placeholders() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let example_path = tempdir.path().join(".env.market.local.example");
    let local_path = tempdir.path().join(".env.market.local");

    fs::write(
        &example_path,
        "QUANTIX_UPSTREAM_MYSQL_URL=replace-me\nCLICKHOUSE_URL=replace-me\n",
    )
    .expect("should write example env");

    let output = Command::new("bash")
        .arg("scripts/dev/init_market_cli_local_env.sh")
        .env("EXAMPLE_PATH", &example_path)
        .env("LOCAL_PATH", &local_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run init env script");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected warning exit code, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let local = fs::read_to_string(&local_path).expect("should read copied local env");
    assert!(local.contains("replace-me"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("[INFO] created"));
    assert!(stderr.contains("[WARN] placeholder values still present"));
    assert!(stderr.contains("[NEXT] edit"));
}

#[test]
fn init_market_cli_local_env_script_passes_when_local_env_is_ready() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let example_path = tempdir.path().join(".env.market.local.example");
    let local_path = tempdir.path().join(".env.market.local");

    fs::write(&example_path, "QUANTIX_UPSTREAM_MYSQL_URL=replace-me\n")
        .expect("should write example env");
    fs::write(
        &local_path,
        "QUANTIX_UPSTREAM_MYSQL_URL=mysql://ready\nCLICKHOUSE_URL=http://localhost:8123\n",
    )
    .expect("should write local env");

    let output = Command::new("bash")
        .arg("scripts/dev/init_market_cli_local_env.sh")
        .env("EXAMPLE_PATH", &example_path)
        .env("LOCAL_PATH", &local_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run init env script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[INFO] local env already exists:"));
    assert!(stdout.contains("[PASS] local market env ready:"));
}
