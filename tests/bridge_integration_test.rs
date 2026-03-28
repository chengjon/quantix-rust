//! Bridge Integration Test
//!
//! Run with: cargo test --test bridge_integration_test -- --nocapture
//!
//! Prerequisites:
//! 1. Bridge service running on Windows at http://127.0.0.1:17580
//! 2. Set QUANTIX_BRIDGE_BASE_URL env var if different

use quantix_cli::bridge::client::BridgeHttpClient;

fn get_bridge_client() -> BridgeHttpClient {
    let base_url = std::env::var("QUANTIX_BRIDGE_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:17580".to_string());
    BridgeHttpClient::new(base_url, None).expect("Failed to create bridge client")
}

#[tokio::test]
async fn test_bridge_health() {
    let client = get_bridge_client();

    // Test capabilities endpoint
    let caps = client
        .capabilities()
        .await
        .expect("Failed to get capabilities");

    println!("Bridge Capabilities:");
    println!("  TDX enabled: {}", caps.tdx.enabled);
    println!("  QMT enabled: {}", caps.qmt.enabled);
    println!("  QMT mode: {}", caps.qmt.mode);

    assert!(caps.tdx.enabled, "TDX should be enabled");
    assert!(caps.qmt.enabled, "QMT should be enabled");
    assert_eq!(caps.qmt.mode, "live", "QMT should be in live mode");
}

#[tokio::test]
async fn test_qmt_account_status() {
    let client = get_bridge_client();

    let status = client
        .qmt_account_status()
        .await
        .expect("Failed to get account status");

    println!("QMT Account Status:");
    println!("  Adapter: {}", status.adapter);
    println!("  Mode: {}", status.mode);
    println!("  SDK Available: {}", status.sdk_available);
    println!("  Connected: {}", status.connected);

    assert!(status.sdk_available, "QMT SDK should be available");
    // Note: connected may be false if QMT client is not logged in
}

#[tokio::test]
async fn test_qmt_positions() {
    let client = get_bridge_client();

    let positions = client
        .qmt_positions()
        .await
        .expect("Failed to get positions");

    println!("QMT Positions: {} positions", positions.len());
    for pos in positions.iter().take(5) {
        println!("  {} ({:?}): {} shares", pos.symbol, pos.name, pos.volume);
    }
}

#[tokio::test]
async fn test_qmt_asset() {
    let client = get_bridge_client();

    let asset = client.qmt_asset().await.expect("Failed to get asset");

    println!("QMT Asset:");
    println!("  Account: {}", asset.account_id);
    println!("  Total: {}", asset.total_asset);
    println!("  Cash: {}", asset.cash);
    println!("  Market Value: {}", asset.market_value);
}
