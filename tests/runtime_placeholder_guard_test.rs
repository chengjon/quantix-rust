#[test]
fn runtime_paths_do_not_use_zeroed_placeholders() {
    let etl = include_str!("../src/sync/etl.rs");
    let auction = include_str!("../src/sources/auction_collector.rs");

    assert!(
        !etl.contains("zeroed()"),
        "src/sync/etl.rs still contains zeroed() placeholder construction"
    );
    assert!(
        !auction.contains("zeroed()"),
        "src/sources/auction_collector.rs still contains zeroed() placeholder construction"
    );
}
