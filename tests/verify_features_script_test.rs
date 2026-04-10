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
        script.contains("run_expect_pass \"Execution config show\" \"cargo run -- execution config show\""),
        "expected verify_features script to cover execution config show"
    );
    assert!(
        script.contains(
            "run_expect_pass \"Execution daemon run once\" \"cargo run -- execution daemon run --once\""
        ),
        "expected verify_features script to cover execution daemon run --once"
    );
}
