use std::fs;

#[test]
fn parquet_placeholder_warn_hint_covers_future_no_data_messages() {
    let script = fs::read_to_string("scripts/verify_features.sh")
        .expect("should read scripts/verify_features.sh");

    let parquet_line = script
        .lines()
        .find(|line| line.contains("Parquet export placeholder"))
        .expect("should contain Parquet export placeholder check");

    assert!(
        parquet_line.contains("未找到数据"),
        "expected Parquet placeholder hint to cover future no-data wording"
    );
    assert!(
        parquet_line.contains("empty"),
        "expected Parquet placeholder hint to cover future empty wording"
    );
    assert!(
        parquet_line.contains("no data"),
        "expected Parquet placeholder hint to cover future no data wording"
    );
}

#[test]
fn script_covers_shipped_execution_mainline_smoke_checks() {
    let script = fs::read_to_string("scripts/verify_features.sh")
        .expect("should read scripts/verify_features.sh");

    assert!(
        script.contains("QUANTIX_BIN=\"$ROOT_DIR/target/debug/quantix\""),
        "expected verify_features script to build and use the quantix binary"
    );
    assert!(
        script
            .contains("run_expect_pass \"Build quantix binary\" \"cargo build -q --bin quantix\""),
        "expected verify_features script to build quantix before smoke checks"
    );
    assert!(
        script.contains(
            "run_expect_pass \"Execution config show\" \"\\\"$QUANTIX_BIN\\\" execution config show\""
        ),
        "expected verify_features script to cover execution config show"
    );
    assert!(
        script.contains(
            "run_expect_pass \"Execution daemon run once\" \"\\\"$QUANTIX_BIN\\\" execution daemon run --once\""
        ),
        "expected verify_features script to cover execution daemon run --once"
    );
    assert!(
        script.contains("# 3) Local binary smoke checks"),
        "expected verify_features script to separate local binary smoke checks"
    );
    assert!(
        script.contains("# 4) External dependency smoke checks"),
        "expected verify_features script to separate external dependency checks"
    );
    assert!(
        script.contains(
            "run_expect_warn \"Execution bridge status (external dependency)\" \"\\\"$QUANTIX_BIN\\\" execution bridge status\" \"Connection refused|bridge request failed|timeout|timed out\" external"
        ),
        "expected execution bridge status to be categorized as an external dependency check"
    );
    assert!(
        script.contains("echo \"EXTERNAL PASS : $EXTERNAL_PASS\""),
        "expected verify_features script to print external dependency summary counts"
    );
}
