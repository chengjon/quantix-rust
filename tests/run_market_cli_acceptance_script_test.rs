use std::fs;
use std::process::Command;

#[test]
fn acceptance_orchestrator_references_template_precheck_and_smoke() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_acceptance.sh")
        .expect("should read scripts/dev/run_market_cli_acceptance.sh");

    for expected in [
        "market_cli_env.example.sh",
        ".env.market.local",
        "init_market_cli_local_env.sh",
        "check_market_cli_prereqs.sh",
        "verify_market_cli_smoke.sh",
        "Suggested first step: source",
        "Environment precheck",
        "Smoke verification",
        "quantix risk sync industry --standard shenwan",
        "quantix market foundation",
        "quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10",
        "quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10",
        "Market CLI acceptance orchestration completed.",
    ] {
        assert!(
            script.contains(expected),
            "expected acceptance orchestrator to contain {expected}"
        );
    }
}

#[test]
fn acceptance_orchestrator_runs_fake_precheck_and_smoke_scripts() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let acceptance_log = log_dir.join("run_market_cli_acceptance.log");
    let fake_env_template = tempdir.path().join("market_cli_env.example.sh");
    let fake_local_env = tempdir.path().join(".env.market.local");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_precheck = tempdir.path().join("fake-precheck.sh");
    let fake_smoke = tempdir.path().join("fake-smoke.sh");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(&fake_env_template, "# fake env template\n").expect("should write env template");
    fs::write(&fake_local_env, "export MARKET_FAKE_ENV=1\n").expect("should write local env");
    fs::write(
        &fake_init,
        "#!/usr/bin/env bash\nset -euo pipefail\necho '[FAKE] init'\n",
    )
    .expect("should write fake init");
    fs::write(
        &fake_precheck,
        "#!/usr/bin/env bash\nset -euo pipefail\necho '[FAKE] precheck'\necho 'PASS : 2'\necho 'WARN : 1'\necho 'FAIL : 0'\n",
    )
    .expect("should write fake precheck");
    fs::write(
        &fake_smoke,
        "#!/usr/bin/env bash\nset -euo pipefail\necho '[FAKE] smoke'\necho 'PASS : 3'\necho 'WARN : 2'\necho 'FAIL : 0'\n",
    )
    .expect("should write fake smoke");

    for path in [&fake_init, &fake_precheck, &fake_smoke] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/run_market_cli_acceptance.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &acceptance_log)
        .env("ENV_TEMPLATE_PATH", &fake_env_template)
        .env("LOCAL_ENV_PATH", &fake_local_env)
        .env("INIT_LOCAL_ENV_SCRIPT", &fake_init)
        .env("PRECHECK_SCRIPT", &fake_precheck)
        .env("SMOKE_SCRIPT", &fake_smoke)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run acceptance orchestrator");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&acceptance_log).expect("should read acceptance log");
    assert!(log.contains("[FAKE] init"));
    assert!(log.contains("[STEP] Environment precheck"));
    assert!(log.contains("[FAKE] precheck"));
    assert!(log.contains("[STEP] Smoke verification"));
    assert!(log.contains("[FAKE] smoke"));
    assert!(log.contains("quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10"));
}
