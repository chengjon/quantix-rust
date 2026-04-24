use std::fs;

#[test]
fn market_prereq_script_covers_expected_environment_checks() {
    let script = fs::read_to_string("scripts/dev/check_market_cli_prereqs.sh")
        .expect("should read scripts/dev/check_market_cli_prereqs.sh");

    assert!(
        script.contains("QUANTIX_BIN=\"$ROOT_DIR/target/debug/quantix\""),
        "expected precheck script to reference quantix binary"
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
