//! Shared helpers for openstock_client test sibling modules.

use crate::core::runtime::OpenStockSettings;
use crate::sources::OpenStockClientConfig;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct Rec {
    pub code: String,
}

pub(super) fn fast_test_cfg(base_url: String) -> OpenStockClientConfig {
    OpenStockClientConfig {
        base_url,
        api_key: "test-key".to_string(),
        timeout: std::time::Duration::from_secs(1),
        max_retries: 2,
        retry_base_delay: std::time::Duration::from_millis(5),
        circuit_break_threshold: 5,
        circuit_break_cooldown: std::time::Duration::from_millis(50),
    }
}

pub(super) fn success_body() -> &'static str {
    r#"{"data":[{"code":"600000"}],"source":"eltdx"}"#
}

// Silence unused-import warning if Rec's fields are never read in some configs.
#[allow(dead_code)]
fn _silence_unused() {
    let _ = OpenStockSettings {
        base_url: None,
        api_key: None,
        timeout_secs: 0,
    };
}
