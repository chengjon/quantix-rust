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
