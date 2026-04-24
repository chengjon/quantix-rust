use std::fs;

#[test]
fn doctor_market_cli_env_script_covers_override_visibility() {
    let script = fs::read_to_string("scripts/dev/doctor_market_cli_env.sh")
        .expect("should read scripts/dev/doctor_market_cli_env.sh");

    for expected in [
        "CLICKHOUSE_URL",
        "QUANTIX_UPSTREAM_MYSQL_URL",
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
