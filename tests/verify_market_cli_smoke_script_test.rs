use std::fs;
use std::process::Command;

#[test]
fn market_smoke_script_covers_foundation_strength_and_strength_stocks_acceptance_path() {
    let script = fs::read_to_string("scripts/dev/verify_market_cli_smoke.sh")
        .expect("should read scripts/dev/verify_market_cli_smoke.sh");

    assert!(
        script.contains("QUANTIX_BIN=\"${QUANTIX_BIN:-$ROOT_DIR/target/debug/quantix}\""),
        "expected market smoke script to support an overridable quantix binary path"
    );
    assert!(
        script.contains("run_expect_pass \"Market foundation help\" \"\\\"$QUANTIX_BIN\\\" market foundation --help\""),
        "expected market smoke script to cover market foundation help"
    );
    assert!(
        script.contains("run_expect_pass \"Market strength help\" \"\\\"$QUANTIX_BIN\\\" market strength --help\""),
        "expected market smoke script to cover market strength help"
    );
    assert!(
        script.contains("run_expect_pass \"Market strength-stocks help\" \"\\\"$QUANTIX_BIN\\\" market strength-stocks --help\""),
        "expected market smoke script to cover market strength-stocks help"
    );
    assert!(
        script.contains("Risk sync industry Shenwan (external dependency)"),
        "expected market smoke script to include Shenwan sync dependency check"
    );
    assert!(
        script.contains("\"\\\"$QUANTIX_BIN\\\" market foundation\""),
        "expected market smoke script to include market foundation execution"
    );
    assert!(
        script.contains("\"\\\"$QUANTIX_BIN\\\" market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10\""),
        "expected market smoke script to include market strength execution"
    );
    assert!(
        script.contains("\"\\\"$QUANTIX_BIN\\\" market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10\""),
        "expected market smoke script to include market strength-stocks execution"
    );
    assert!(
        script.contains("sync industry"),
        "expected market smoke script to hint the Shenwan prerequisite in warn matching"
    );
    assert!(
        script.contains("echo \"EXTERNAL WARN : $EXTERNAL_WARN\""),
        "expected market smoke script to print external dependency summary counts"
    );
}

#[test]
fn market_smoke_script_runs_fake_cargo_and_quantix() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let log_file = log_dir.join("verify_market_cli_smoke.log");
    let bin_dir = tempdir.path().join("bin");
    let fake_cargo = bin_dir.join("cargo");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::create_dir_all(&bin_dir).expect("should create bin dir");

    fs::write(
        &fake_cargo,
        "#!/usr/bin/env bash\nset -euo pipefail\necho '[FAKE] cargo build'\n",
    )
    .expect("should write fake cargo");
    fs::write(
        &fake_quantix,
        r#"#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  "market --help"|"market foundation --help"|"market strength --help"|"market strength-stocks --help")
    echo "help ok"
    ;;
  "risk sync industry --standard shenwan"|"market foundation"|"market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10"|"market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10")
    echo "Error: Connection refused"
    exit 1
    ;;
  *)
    echo "unexpected args: $*" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("should write fake quantix");

    for path in [&fake_cargo, &fake_quantix] {
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
        .arg("scripts/dev/verify_market_cli_smoke.sh")
        .env("PATH", combined_path)
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &log_file)
        .env("QUANTIX_BIN", &fake_quantix)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run smoke script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_file).expect("should read smoke log");
    assert!(log.contains("[FAKE] cargo build"));
    assert!(log.contains("[PASS] Market strength-stocks help"));
    assert!(log.contains("[WARN-EXPECTED] Risk sync industry Shenwan (external dependency)"));
    assert!(log.contains("[WARN-EXPECTED] Market strength-stocks (external dependency)"));
    assert!(log.contains("PASS : 5"));
    assert!(log.contains("WARN : 4"));
    assert!(log.contains("FAIL : 0"));
}
