use std::fs;
use std::process::Command;

#[test]
fn market_prereq_script_covers_expected_environment_checks() {
    let script = fs::read_to_string("scripts/dev/check_market_cli_prereqs.sh")
        .expect("should read scripts/dev/check_market_cli_prereqs.sh");

    assert!(
        script.contains("QUANTIX_BIN=\"${QUANTIX_BIN:-$ROOT_DIR/target/debug/quantix}\""),
        "expected precheck script to support an overridable quantix binary path"
    );
    assert!(
        script.contains("INDUSTRY_DB_PATH"),
        "expected precheck script to inspect the local industry sqlite path"
    );
    assert!(
        script.contains("QUANTIX_UPSTREAM_MYSQL_URL"),
        "expected precheck script to inspect upstream MySQL environment"
    );
    assert!(
        script.contains("CLICKHOUSE_URL"),
        "expected precheck script to inspect ClickHouse environment"
    );
    assert!(
        script.contains("market_cli_env.example.sh"),
        "expected precheck script to point operators to the reusable environment template"
    );
    assert!(
        script.contains(".env.market.local"),
        "expected precheck script to support a local-only market env override file"
    );
    assert!(
        script.contains("Shenwan SQLite reference DB present"),
        "expected precheck script to validate the local industry reference db"
    );
    assert!(
        script.contains("Upstream MySQL env configured for risk sync"),
        "expected precheck script to validate risk sync prerequisites"
    );
    assert!(
        script.contains("ClickHouse env resolved for market strength"),
        "expected precheck script to validate ClickHouse prerequisites"
    );
    assert!(
        script.contains("[REMEDIATION]"),
        "expected precheck script to print remediation guidance for warnings"
    );
    assert!(
        script.contains("quantix risk sync industry --standard shenwan"),
        "expected precheck script to recommend the Shenwan sync command when sqlite is missing"
    );
    assert!(
        script.contains("QUANTIX_UPSTREAM_MYSQL_PASSWORD"),
        "expected precheck script to name the MySQL env variables required for risk sync"
    );
    assert!(
        script.contains("Market CLI prerequisite checks passed"),
        "expected precheck script to emit a clear terminal summary"
    );
}

#[test]
fn market_prereq_script_runs_with_fake_quantix_and_expected_warnings() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("check_market_cli_prereqs.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let missing_industry_db = tempdir.path().join("missing-industry.db");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(
        &fake_quantix,
        r#"#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  "market --help"|"risk --help")
    echo "help ok"
    ;;
  *)
    echo "unexpected args: $*" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("should write fake quantix");
    fs::write(&fake_env, "").expect("should write fake env");
    fs::write(&fake_env_template, "# fake env template\n").expect("should write fake env template");

    let mut perms = fs::metadata(&fake_quantix)
        .expect("metadata")
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(&fake_quantix, perms).expect("set permissions");

    let output = Command::new("bash")
        .arg("scripts/dev/check_market_cli_prereqs.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("QUANTIX_INDUSTRY_DB_PATH", &missing_industry_db)
        .env("CLICKHOUSE_URL", "http://localhost:8123")
        .env("CLICKHOUSE_DB", "quantix")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run prerequisite script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read prerequisite log");
    assert!(log.contains("[INFO] Market CLI prerequisite log:"));
    assert!(log.contains("[PASS] Quantix binary exists"));
    assert!(log.contains("[PASS] Market command tree reachable"));
    assert!(log.contains("[PASS] Risk command tree reachable"));
    assert!(log.contains("[WARN] Shenwan SQLite reference DB present"));
    assert!(log.contains("[WARN] Upstream MySQL env configured for risk sync"));
    assert!(log.contains("[PASS] ClickHouse env resolved for market strength"));
    assert!(log.contains("缺少本地行业 SQLite：先 source "));
    assert!(log.contains("quantix risk sync industry --standard shenwan"));
    assert!(log.contains("缺少上游 MySQL 环境变量：请 source "));
    assert!(log.contains("PASS : 4"));
    assert!(log.contains("WARN : 2"));
    assert!(log.contains("FAIL : 0"));
}

#[test]
fn market_env_template_lists_required_exports() {
    let template = fs::read_to_string("scripts/dev/market_cli_env.example.sh")
        .expect("should read scripts/dev/market_cli_env.example.sh");

    for expected in [
        "export CLICKHOUSE_URL=",
        "export CLICKHOUSE_DB=",
        "export QUANTIX_UPSTREAM_MYSQL_URL=",
        "export QUANTIX_UPSTREAM_MYSQL_DB=",
        "export QUANTIX_UPSTREAM_MYSQL_USER=",
        "export QUANTIX_UPSTREAM_MYSQL_PASSWORD=",
        "export QUANTIX_INDUSTRY_DB_PATH=",
    ] {
        assert!(
            template.contains(expected),
            "expected market env template to contain {expected}"
        );
    }
}

#[test]
fn local_market_env_example_lists_required_secret_overrides() {
    let template = fs::read_to_string(".env.market.local.example")
        .expect("should read .env.market.local.example");

    for expected in [
        "QUANTIX_UPSTREAM_MYSQL_URL=",
        "QUANTIX_UPSTREAM_MYSQL_DB=",
        "QUANTIX_UPSTREAM_MYSQL_USER=",
        "QUANTIX_UPSTREAM_MYSQL_PASSWORD=",
        "CLICKHOUSE_URL=",
        "CLICKHOUSE_DB=",
    ] {
        assert!(
            template.contains(expected),
            "expected local market env example to contain {expected}"
        );
    }
}
