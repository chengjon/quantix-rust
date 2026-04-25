use std::fs;
use std::process::Command;

#[test]
fn doctor_market_cli_env_script_covers_override_visibility() {
    let script = fs::read_to_string("scripts/dev/doctor_market_cli_env.sh")
        .expect("should read scripts/dev/doctor_market_cli_env.sh");

    for expected in [
        "CLICKHOUSE_URL",
        "QUANTIX_UPSTREAM_MYSQL_URL",
        "DOTENV_PATH=\"${DOTENV_PATH:-$ROOT_DIR/.env}\"",
        "LOCAL_ENV_PATH=\"${LOCAL_ENV_PATH:-$ROOT_DIR/.env.market.local}\"",
        ".env.market.local overrides .env",
        "runtime :",
        "mask_if_secret",
        "Market CLI Env Doctor",
    ] {
        assert!(
            script.contains(expected),
            "expected doctor script to contain {expected}"
        );
    }
}

#[test]
fn doctor_market_cli_env_script_reports_override_precedence_and_masks_passwords() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let dotenv_path = tempdir.path().join(".env");
    let local_env_path = tempdir.path().join(".env.market.local");

    fs::write(
        &dotenv_path,
        [
            "CLICKHOUSE_URL=http://dotenv-host:8123",
            "CLICKHOUSE_DB=dotenv_db",
            "CLICKHOUSE_PASSWORD=dotenv-secret",
            "QUANTIX_UPSTREAM_MYSQL_URL=mysql://dotenv:3306",
            "QUANTIX_UPSTREAM_MYSQL_PASSWORD=dotenv-mysql-secret",
        ]
        .join("\n"),
    )
    .expect("should write dotenv file");
    fs::write(
        &local_env_path,
        [
            "CLICKHOUSE_URL=http://local-host:8123",
            "CLICKHOUSE_PASSWORD=local-secret",
            "QUANTIX_UPSTREAM_MYSQL_URL=mysql://local:3306",
            "QUANTIX_UPSTREAM_MYSQL_PASSWORD=local-mysql-secret",
        ]
        .join("\n"),
    )
    .expect("should write local env file");

    let output = Command::new("bash")
        .arg("scripts/dev/doctor_market_cli_env.sh")
        .env("DOTENV_PATH", &dotenv_path)
        .env("LOCAL_ENV_PATH", &local_env_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run doctor script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("== Market CLI Env Doctor =="));
    assert!(stdout.contains(&format!(".env: {}", dotenv_path.display())));
    assert!(stdout.contains(&format!(".env.market.local: {}", local_env_path.display())));
    assert!(stdout.contains("CLICKHOUSE_URL"));
    assert!(stdout.contains("  .env    : http://dotenv-host:8123"));
    assert!(stdout.contains("  local   : http://local-host:8123"));
    assert!(stdout.contains("  runtime : http://local-host:8123"));
    assert!(stdout.contains("  note    : .env.market.local overrides .env"));
    assert!(stdout.contains("CLICKHOUSE_PASSWORD"));
    assert!(stdout.contains("QUANTIX_UPSTREAM_MYSQL_PASSWORD"));
    assert!(!stdout.contains("dotenv-secret"));
    assert!(!stdout.contains("local-secret"));
    assert!(!stdout.contains("dotenv-mysql-secret"));
    assert!(!stdout.contains("local-mysql-secret"));
    assert!(stdout.contains("  .env    : ***"));
    assert!(stdout.contains("  local   : ***"));
    assert!(stdout.contains("  runtime : ***"));
}
