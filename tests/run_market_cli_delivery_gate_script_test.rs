use std::fs;
use std::process::Command;

#[test]
fn delivery_gate_script_references_acceptance_formal_and_report_steps() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_delivery_gate.sh")
        .expect("should read scripts/dev/run_market_cli_delivery_gate.sh");

    for expected in [
        "run_market_cli_acceptance.sh",
        "run_market_cli_formal_sequence.sh",
        "generate_market_cli_acceptance_report.sh",
        "Acceptance orchestration",
        "Formal sequence",
        "Acceptance report generation",
        "Market CLI delivery gate completed.",
        "Report path:",
    ] {
        assert!(
            script.contains(expected),
            "expected delivery gate script to contain {expected}"
        );
    }
}

#[test]
fn delivery_gate_script_runs_fake_acceptance_formal_and_report_scripts() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let gate_log = log_dir.join("run_market_cli_delivery_gate.log");
    let report_path = log_dir.join("market_cli_delivery_gate_report.md");
    let fake_acceptance = tempdir.path().join("fake-acceptance.sh");
    let fake_formal = tempdir.path().join("fake-formal.sh");
    let fake_report = tempdir.path().join("fake-report.sh");

    fs::create_dir_all(&log_dir).expect("should create log dir");

    fs::write(
        &fake_acceptance,
        "#!/usr/bin/env bash\nset -euo pipefail\necho \"[FAKE] acceptance LOG_DIR=$LOG_DIR\"\n",
    )
    .expect("should write fake acceptance");
    fs::write(
        &fake_formal,
        "#!/usr/bin/env bash\nset -euo pipefail\necho \"[FAKE] formal LOG_DIR=$LOG_DIR\"\n",
    )
    .expect("should write fake formal");
    fs::write(
        &fake_report,
        format!(
            "#!/usr/bin/env bash\nset -euo pipefail\necho \"[FAKE] report REPORT_PATH=$REPORT_PATH\"\nprintf '# fake report\\n' > \"$REPORT_PATH\"\nif [[ \"$REPORT_PATH\" != \"{}\" ]]; then\n  echo \"unexpected report path: $REPORT_PATH\" >&2\n  exit 64\nfi\n",
            report_path.display()
        ),
    )
    .expect("should write fake report");

    for path in [&fake_acceptance, &fake_formal, &fake_report] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/run_market_cli_delivery_gate.sh")
        .env("LOG_DIR", &log_dir)
        .env("LOG_FILE", &gate_log)
        .env("REPORT_PATH", &report_path)
        .env("ACCEPTANCE_SCRIPT", &fake_acceptance)
        .env("FORMAL_SEQUENCE_SCRIPT", &fake_formal)
        .env("REPORT_SCRIPT", &fake_report)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run delivery gate script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&gate_log).expect("should read gate log");
    assert!(log.contains("[STEP] Acceptance orchestration"));
    assert!(log.contains(&format!("[FAKE] acceptance LOG_DIR={}", log_dir.display())));
    assert!(log.contains("[STEP] Formal sequence"));
    assert!(log.contains(&format!("[FAKE] formal LOG_DIR={}", log_dir.display())));
    assert!(log.contains("[STEP] Acceptance report generation"));
    assert!(log.contains(&format!("[FAKE] report REPORT_PATH={}", report_path.display())));
    assert!(log.contains(&format!("Report path: {}", report_path.display())));

    let report = fs::read_to_string(&report_path).expect("should read generated report");
    assert!(report.contains("# fake report"));
}
