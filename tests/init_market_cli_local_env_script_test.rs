use std::fs;

#[test]
fn init_market_cli_local_env_script_covers_copy_and_placeholder_validation() {
    let script = fs::read_to_string("scripts/dev/init_market_cli_local_env.sh")
        .expect("should read scripts/dev/init_market_cli_local_env.sh");

    for expected in [
        ".env.market.local.example",
        ".env.market.local",
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
