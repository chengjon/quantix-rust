use std::fs;

#[test]
fn market_smoke_script_covers_foundation_strength_acceptance_path() {
    let script = fs::read_to_string("scripts/dev/verify_market_cli_smoke.sh")
        .expect("should read scripts/dev/verify_market_cli_smoke.sh");

    assert!(
        script.contains("QUANTIX_BIN=\"$ROOT_DIR/target/debug/quantix\""),
        "expected market smoke script to build and use the quantix binary"
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
        script.contains("sync industry"),
        "expected market smoke script to hint the Shenwan prerequisite in warn matching"
    );
    assert!(
        script.contains("echo \"EXTERNAL WARN : $EXTERNAL_WARN\""),
        "expected market smoke script to print external dependency summary counts"
    );
}
