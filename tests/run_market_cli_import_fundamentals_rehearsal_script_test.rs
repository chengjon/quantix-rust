use std::fs;
use std::process::Command;

#[test]
fn import_fundamentals_rehearsal_script_covers_scratch_db_and_smoke_fallback() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh")
        .expect("should read scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh");

    for expected in [
        "MARKET_FUNDAMENTALS_REHEARSAL_INPUT",
        "MARKET_FUNDAMENTALS_SMOKE_INPUT",
        "MARKET_FUNDAMENTALS_REHEARSAL_DB",
        "MARKET_FUNDAMENTALS_REHEARSAL_STATUS_CMD",
        "using smoke fixture in scratch DB only",
        "Production quantix DB was not modified.",
        "data import-fundamentals --input",
        "rehearsal_table_rows",
        "rehearsal_latest_snapshot",
        "Scratch DB retained for inspection",
    ] {
        assert!(
            script.contains(expected),
            "expected rehearsal script to contain {expected}"
        );
    }
}

#[test]
fn import_fundamentals_rehearsal_script_runs_fake_quantix_and_curl() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("market_cli_import_fundamentals_rehearsal.log");
    let import_log = log_dir.join("market_cli_import_fundamentals_step.log");
    let bin_dir = tempdir.path().join("bin");
    let fake_curl = bin_dir.join("curl");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_smoke = tempdir.path().join("quantix_market_fundamentals_smoke.json");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::create_dir_all(&bin_dir).expect("should create bin dir");
    fs::write(
        &fake_smoke,
        r#"[{"code":"000021","snapshot_date":"2026-03-14","market_cap":1200.5,"latest_report_profit":18.6,"profit_source":"smoke","pe_dynamic":22.4}]"#,
    )
    .expect("should write fake smoke input");

    fs::write(
        &fake_quantix,
        format!(
            r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${{CLICKHOUSE_DB:-}}" != "quantix_mf_rehearsal_test" ]]; then
  echo "unexpected CLICKHOUSE_DB: ${{CLICKHOUSE_DB:-unset}}" >&2
  exit 71
fi
if [[ "$*" != "data import-fundamentals --input {}" ]]; then
  echo "unexpected args: $*" >&2
  exit 64
fi
cat <<'EOF'
📥 导入市场基础面快照
  文件: {}
  记录数: 1
✅ 市场基础面快照导入完成
  已写入: 1
  耗时(秒): 1
EOF
"#,
            fake_smoke.display(),
            fake_smoke.display()
        ),
    )
    .expect("should write fake quantix");
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
    fs::write(&fake_env, "").expect("should write fake env");
    fs::write(
        &fake_curl,
        r#"#!/usr/bin/env bash
set -euo pipefail
query=""
while (($#)); do
  if [[ "$1" == "query="* ]]; then
    query="${1#query=}"
  fi
  shift
done

if [[ "$query" == *"system.tables"* ]]; then
  printf '1\n'
elif [[ "$query" == *"count(), max(snapshot_date)"* ]]; then
  printf '1\t2026-03-14\n'
else
  echo "unexpected query: $query" >&2
  exit 65
fi
"#,
    )
    .expect("should write fake curl");

    for path in [&fake_quantix, &fake_init, &fake_curl] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let original_path = std::env::var("PATH").unwrap_or_default();
    let combined_path = format!("{}:{}", bin_dir.display(), original_path);

    let output = Command::new("bash")
        .arg("scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh")
        .env("PATH", combined_path)
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("IMPORT_LOG", &import_log)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("INIT_LOCAL_ENV_SCRIPT", &fake_init)
        .env("LOCAL_ENV_PATH", &fake_env)
        .env("MARKET_FUNDAMENTALS_SMOKE_INPUT", &fake_smoke)
        .env("MARKET_FUNDAMENTALS_REHEARSAL_DB", "quantix_mf_rehearsal_test")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run rehearsal script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read rehearsal log");
    assert!(log.contains("[WARN] No explicit fundamentals JSON configured; using smoke fixture in scratch DB only:"));
    assert!(log.contains("[FIELD] rehearsal_input_mode=smoke_fixture"));
    assert!(log.contains("[FIELD] rehearsal_clickhouse_db=quantix_mf_rehearsal_test"));
    assert!(log.contains("[FIELD] rehearsal_import_records=1"));
    assert!(log.contains("[FIELD] rehearsal_import_written=1"));
    assert!(log.contains("[FIELD] rehearsal_table_state=populated"));
    assert!(log.contains("[FIELD] rehearsal_table_rows=1"));
    assert!(log.contains("[FIELD] rehearsal_latest_snapshot=2026-03-14"));
    assert!(log.contains("Production quantix DB was not modified."));
}
