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
        script.contains("MARKET_SNAPSHOT_PROBE_URL"),
        "expected precheck script to define an overridable A-share snapshot probe url"
    );
    assert!(
        script.contains("MARKET_SNAPSHOT_PROBE_CMD"),
        "expected precheck script to define an overridable A-share snapshot probe command"
    );
    assert!(
        script.contains("MARKET_FUNDAMENTALS_INPUT"),
        "expected precheck script to surface the optional market fundamentals import input"
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
        script.contains("QUANTIX_TDX_ROOT"),
        "expected precheck script to surface the configured TDX root for local day-file workflows"
    );
    assert!(
        script.contains("QUANTIX_TDX_MARKET"),
        "expected precheck script to surface the configured TDX market hint for local day-file workflows"
    );
    assert!(
        script.contains("QUANTIX_MARKET_SNAPSHOT_SOURCE"),
        "expected precheck script to expose the market snapshot source override for TDX-only runtime mode"
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
        script.contains("EastMoney A-share snapshot upstream reachable"),
        "expected precheck script to validate the upstream A-share snapshot reachability"
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
    assert!(
        script.contains("A股全市场快照上游当前不可达"),
        "expected precheck script to explain the runtime impact when the upstream snapshot source is blocked"
    );
    assert!(
        script.contains("market foundation 与 market strength 会尝试退回 TDX 实时行情"),
        "expected precheck script to explain the new TDX fallback behavior when the A-share snapshot upstream is blocked"
    );
    assert!(
        script.contains("market_fundamentals_daily"),
        "expected precheck script to explain the remaining top-n dependency on local fundamentals coverage"
    );
    assert!(
        script.contains("quantix data validate-fundamentals --input"),
        "expected precheck script to recommend the local fundamentals validate command before import when operators have a JSON snapshot file"
    );
    assert!(
        script.contains("quantix data import-fundamentals --input"),
        "expected precheck script to recommend the local fundamentals import command when operators have a JSON snapshot file"
    );
    assert!(
        script.contains("formal sequence 会自动先运行 quantix data validate-fundamentals"),
        "expected precheck script to explain that the formal sequence validates a fundamentals JSON before importing it"
    );
    assert!(
        script.contains("run_market_cli_import_fundamentals_rehearsal.sh"),
        "expected precheck script to recommend the scratch fundamentals rehearsal helper before production import"
    );
    assert!(
        script.contains("direct TDX quote-server fallback"),
        "expected precheck script to explain that market fallback uses direct TDX quote servers instead of the GUI client"
    );
    assert!(
        script.contains("Market snapshot mode"),
        "expected precheck script to show the effective market snapshot source mode in its runtime summary"
    );
    assert!(
        script.contains("MARKET_FUNDAMENTALS_STATUS_CMD"),
        "expected precheck script to support an overridable fundamentals table status probe for testing and operator diagnostics"
    );
    assert!(
        script.contains("[FIELD] precheck_market_fundamentals_state="),
        "expected precheck script to emit a machine-readable fundamentals table state field"
    );
    assert!(
        script.contains("[FIELD] precheck_market_fundamentals_rows="),
        "expected precheck script to emit a machine-readable fundamentals table row-count field"
    );
    assert!(
        script.contains("[FIELD] precheck_market_fundamentals_latest_snapshot="),
        "expected precheck script to emit a machine-readable fundamentals latest snapshot field"
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
    let fake_snapshot_probe = tempdir.path().join("fake-snapshot-probe.sh");
    let fake_fundamentals_status = tempdir.path().join("fake-fundamentals-status.sh");

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
    fs::write(
        &fake_snapshot_probe,
        "#!/usr/bin/env bash\nset -euo pipefail\necho 'curl: (52) Empty reply from server'\nexit 52\n",
    )
    .expect("should write fake snapshot probe");
    fs::write(
        &fake_fundamentals_status,
        "#!/usr/bin/env bash\nset -euo pipefail\necho 'probe unavailable' >&2\nexit 70\n",
    )
    .expect("should write fake fundamentals status probe");

    for path in [&fake_quantix, &fake_snapshot_probe, &fake_fundamentals_status] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/check_market_cli_prereqs.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("QUANTIX_INDUSTRY_DB_PATH", &missing_industry_db)
        .env("QUANTIX_TDX_ROOT", "/mnt/d/mystocks/tdx/tdx-quant")
        .env("QUANTIX_TDX_MARKET", "sh")
        .env("CLICKHOUSE_URL", "http://localhost:8123")
        .env("CLICKHOUSE_DB", "quantix")
        .env("MARKET_SNAPSHOT_PROBE_CMD", &fake_snapshot_probe)
        .env("MARKET_FUNDAMENTALS_STATUS_CMD", &fake_fundamentals_status)
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
    assert!(log.contains("[WARN] EastMoney A-share snapshot upstream reachable"));
    assert!(log.contains("缺少本地行业 SQLite：先 source "));
    assert!(log.contains("quantix risk sync industry --standard shenwan"));
    assert!(log.contains("缺少上游 MySQL 环境变量：请 source "));
    assert!(log.contains("A股全市场快照上游当前不可达"));
    assert!(log.contains("market foundation 与 market strength 会尝试退回 TDX 实时行情"));
    assert!(log.contains("strength / strength-stocks 的总市值、净利润 TopN 可能为空"));
    assert!(log.contains("TDX root       : /mnt/d/mystocks/tdx/tdx-quant"));
    assert!(log.contains("TDX market     : sh"));
    assert!(log.contains("TDX quote mode : direct TDX quote-server fallback for market foundation/strength; GUI not required"));
    assert!(log.contains("[FIELD] precheck_market_fundamentals_state=unavailable"));
    assert!(log.contains("Scratch rehearsal: scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh"));
    assert!(log.contains("quantix data validate-fundamentals --input /abs/path/market_fundamentals.json"));
    assert!(log.contains("quantix data import-fundamentals --input /abs/path/market_fundamentals.json"));
    assert!(log.contains("PASS : 4"));
    assert!(log.contains("WARN : 3"));
    assert!(log.contains("FAIL : 0"));
}

#[test]
fn market_prereq_script_falls_back_to_repo_root_env_for_tdx_settings() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("check_market_cli_prereqs.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_root_env = tempdir.path().join("repo.env");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let industry_db = tempdir.path().join("industry.db");
    let fake_snapshot_probe = tempdir.path().join("fake-snapshot-probe.sh");

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
    fs::write(
        &fake_root_env,
        "QUANTIX_TDX_ROOT=/mnt/d/mystocks/tdx/tdx-quant\nQUANTIX_TDX_MARKET=sh\n",
    )
    .expect("should write fake repo env");
    fs::write(&fake_env_template, "# fake env template\n").expect("should write fake env template");
    fs::write(&industry_db, "sqlite").expect("should write fake industry db");
    fs::write(
        &fake_snapshot_probe,
        "#!/usr/bin/env bash\nset -euo pipefail\nexit 0\n",
    )
    .expect("should write fake snapshot probe");

    for path in [&fake_quantix, &fake_snapshot_probe] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/check_market_cli_prereqs.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("ROOT_ENV_PATH", &fake_root_env)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("QUANTIX_INDUSTRY_DB_PATH", &industry_db)
        .env("MARKET_SNAPSHOT_PROBE_CMD", &fake_snapshot_probe)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run prerequisite script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read prerequisite log");
    assert!(log.contains("TDX root       : /mnt/d/mystocks/tdx/tdx-quant"));
    assert!(log.contains("TDX market     : sh"));
}

#[test]
fn market_prereq_script_skips_eastmoney_probe_when_tdx_snapshot_mode_forced() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("check_market_cli_prereqs.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let industry_db = tempdir.path().join("industry.db");
    let fake_snapshot_probe = tempdir.path().join("fake-snapshot-probe.sh");

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
    fs::write(&industry_db, "sqlite").expect("should write fake industry db");
    fs::write(
        &fake_snapshot_probe,
        "#!/usr/bin/env bash\nset -euo pipefail\necho 'snapshot probe should have been skipped' >&2\nexit 52\n",
    )
    .expect("should write fake snapshot probe");

    for path in [&fake_quantix, &fake_snapshot_probe] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/check_market_cli_prereqs.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("QUANTIX_INDUSTRY_DB_PATH", &industry_db)
        .env("QUANTIX_TDX_ROOT", "/mnt/d/mystocks/tdx/tdx-quant")
        .env("QUANTIX_TDX_MARKET", "sh")
        .env("QUANTIX_MARKET_SNAPSHOT_SOURCE", "tdx")
        .env("CLICKHOUSE_URL", "http://localhost:8123")
        .env("CLICKHOUSE_DB", "quantix")
        .env("MARKET_SNAPSHOT_PROBE_CMD", &fake_snapshot_probe)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run prerequisite script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read prerequisite log");
    assert!(log.contains("Market snapshot mode : tdx"));
    assert!(log.contains("跳过 EastMoney 连通性探测"));
    assert!(log.contains("[PASS] EastMoney A-share snapshot upstream reachable"));
    assert!(!log.contains("snapshot probe should have been skipped"));
}

#[test]
fn market_prereq_script_warns_when_market_fundamentals_table_is_empty() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("check_market_cli_prereqs.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let industry_db = tempdir.path().join("industry.db");
    let fake_fundamentals_status = tempdir.path().join("fake-fundamentals-status.sh");

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
    fs::write(&industry_db, "sqlite").expect("should write fake industry db");
    fs::write(
        &fake_fundamentals_status,
        "#!/usr/bin/env bash\nset -euo pipefail\necho 'state=empty'\necho 'rows=0'\necho 'latest_snapshot=1970-01-01'\n",
    )
    .expect("should write fake fundamentals status probe");

    for path in [&fake_quantix, &fake_fundamentals_status] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/check_market_cli_prereqs.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("QUANTIX_INDUSTRY_DB_PATH", &industry_db)
        .env("QUANTIX_UPSTREAM_MYSQL_URL", "mysql://localhost:3306")
        .env("QUANTIX_UPSTREAM_MYSQL_DB", "quantix")
        .env("QUANTIX_UPSTREAM_MYSQL_USER", "tester")
        .env("CLICKHOUSE_URL", "http://localhost:8123")
        .env("CLICKHOUSE_DB", "quantix")
        .env("QUANTIX_MARKET_SNAPSHOT_SOURCE", "tdx")
        .env("MARKET_FUNDAMENTALS_STATUS_CMD", &fake_fundamentals_status)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run prerequisite script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read prerequisite log");
    assert!(log.contains("[WARN] Local market fundamentals table ready for TopN ranking"));
    assert!(log.contains("[FIELD] precheck_market_fundamentals_state=empty"));
    assert!(log.contains("[FIELD] precheck_market_fundamentals_rows=0"));
    assert!(log.contains("[FIELD] precheck_market_fundamentals_latest_snapshot=N/A"));
    assert!(log.contains("本地 market_fundamentals_daily 已建表但仍为空"));
    assert!(log.contains("建议先运行 scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh 做 scratch DB 导入演练"));
    assert!(log.contains("PASS : 7"));
    assert!(log.contains("WARN : 1"));
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
        "export QUANTIX_MARKET_SNAPSHOT_SOURCE=",
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
