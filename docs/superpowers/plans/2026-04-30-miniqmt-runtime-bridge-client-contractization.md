# miniQMT Runtime Bridge Client Contractization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Rust bridge runtime and HTTP client contract-aware for miniQMT `task/execute` and `task/result` without changing adapter or daemon semantics in this slice.

**Architecture:** Extend `BridgeRuntimeSettings` so task-contract auth, version, and polling knobs are explicit. Keep legacy broker-style client methods unchanged while adding task-contract request/response models, explicit bridge error kinds, and new client methods for submit receipt and task result lookup.

**Tech Stack:** Rust, `reqwest`, `serde`, `tokio`, `wiremock`, existing `quantix-cli` bridge/runtime modules

---

### Task 1: Runtime Config Contractization

**Files:**
- Modify: `src/core/runtime.rs`
- Test: `src/core/runtime.rs`

- [ ] **Step 1: Write the failing runtime env tests**

```rust
#[test]
fn test_bridge_runtime_settings_default_contract_values() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(BRIDGE_BASE_URL_ENV);
        std::env::remove_var(BRIDGE_API_KEY_ENV);
        std::env::remove_var(BRIDGE_BEARER_TOKEN_ENV);
        std::env::remove_var(BRIDGE_CONTRACT_VERSION_ENV);
        std::env::remove_var(BRIDGE_TIMEOUT_MS_ENV);
        std::env::remove_var(BRIDGE_POLL_INTERVAL_MS_ENV);
        std::env::remove_var(BRIDGE_POLL_TIMEOUT_MS_ENV);
    }

    let settings = BridgeRuntimeSettings::from_env();

    assert_eq!(settings.base_url, "http://127.0.0.1:17580");
    assert_eq!(settings.bearer_token, None);
    assert_eq!(settings.api_key_fallback, None);
    assert_eq!(settings.contract_version, "miniqmt.v1");
    assert_eq!(settings.timeout_ms, 30_000);
    assert_eq!(settings.poll_interval_ms, 1_000);
    assert_eq!(settings.poll_timeout_ms, 30_000);
}

#[test]
fn test_bridge_runtime_settings_contract_env_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, "http://bridge.internal:18080");
        std::env::set_var(BRIDGE_API_KEY_ENV, "legacy-key");
        std::env::set_var(BRIDGE_BEARER_TOKEN_ENV, "bearer-123");
        std::env::set_var(BRIDGE_CONTRACT_VERSION_ENV, "miniqmt.v1beta");
        std::env::set_var(BRIDGE_TIMEOUT_MS_ENV, "45000");
        std::env::set_var(BRIDGE_POLL_INTERVAL_MS_ENV, "1500");
        std::env::set_var(BRIDGE_POLL_TIMEOUT_MS_ENV, "90000");
    }

    let settings = BridgeRuntimeSettings::from_env();

    assert_eq!(settings.base_url, "http://bridge.internal:18080");
    assert_eq!(settings.api_key.as_deref(), Some("legacy-key"));
    assert_eq!(settings.api_key_fallback.as_deref(), Some("legacy-key"));
    assert_eq!(settings.bearer_token.as_deref(), Some("bearer-123"));
    assert_eq!(settings.contract_version, "miniqmt.v1beta");
    assert_eq!(settings.timeout_ms, 45_000);
    assert_eq!(settings.poll_interval_ms, 1_500);
    assert_eq!(settings.poll_timeout_ms, 90_000);
}
```

- [ ] **Step 2: Run the runtime tests to verify they fail**

Run: `cargo test runtime_loads_bridge_settings_from_env test_bridge_runtime_settings_default_contract_values test_bridge_runtime_settings_contract_env_override -- --nocapture`

Expected: compile or assertion failure because the new bridge runtime fields and env constants do not exist yet.

- [ ] **Step 3: Write the minimal runtime implementation**

```rust
pub const BRIDGE_BEARER_TOKEN_ENV: &str = "QUANTIX_BRIDGE_BEARER_TOKEN";
pub const BRIDGE_CONTRACT_VERSION_ENV: &str = "QUANTIX_BRIDGE_CONTRACT_VERSION";
pub const BRIDGE_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_TIMEOUT_MS";
pub const BRIDGE_POLL_INTERVAL_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_INTERVAL_MS";
pub const BRIDGE_POLL_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_TIMEOUT_MS";

pub struct BridgeRuntimeSettings {
    pub base_url: String,
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub api_key_fallback: Option<String>,
    pub contract_version: String,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub poll_timeout_ms: u64,
    pub tdx_enabled: bool,
    pub qmt_preview_enabled: bool,
}
```

- [ ] **Step 4: Run the runtime tests to verify they pass**

Run: `cargo test runtime_loads_bridge_settings_from_env test_bridge_runtime_settings_default_contract_values test_bridge_runtime_settings_contract_env_override -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the runtime slice**

```bash
git add src/core/runtime.rs
git commit -m "feat: add bridge task contract runtime settings"
```

### Task 2: Task-Contract Models and Error Kinds

**Files:**
- Modify: `src/bridge/error.rs`
- Modify: `src/bridge/models.rs`
- Test: `tests/bridge_client_test.rs`

- [ ] **Step 1: Write the failing model parsing tests**

```rust
#[test]
fn bridge_task_execute_request_serializes_expected_shape() {
    let payload = BridgeTaskExecuteRequest {
        provider: "qmt".to_string(),
        method: "submit_order".to_string(),
        params: BridgeTaskExecuteParams {
            request_id: "req-1".to_string(),
            client_order_id: "cli-1".to_string(),
            local_submission_id: "local-1".to_string(),
            symbol: "600000.SH".to_string(),
            side: "buy".to_string(),
            quantity: 100,
            price: "10.50".to_string(),
            order_type: "limit".to_string(),
            strategy_name: Some("alpha".to_string()),
            order_remark: Some("manual".to_string()),
            snapshot_metadata: Some(serde_json::json!({"source":"test"})),
        },
    };

    let value = serde_json::to_value(payload).unwrap();
    assert_eq!(value["provider"], "qmt");
    assert_eq!(value["method"], "submit_order");
    assert_eq!(value["params"]["local_submission_id"], "local-1");
}

#[test]
fn bridge_task_result_response_parses_failure_code() {
    let payload = serde_json::json!({
        "task_id": "task-1",
        "status": "failed",
        "bridge_contract_version": "miniqmt.v1",
        "result": {
            "client_order_id": "cli-1",
            "local_submission_id": "local-1",
            "account_scope": "sim",
            "event_id": "evt-1",
            "occurred_at": "2026-04-30T00:00:00Z",
            "source_name": "miniqmt",
            "reason_code": "live_bridge_timeout",
            "reason_detail": "deadline exceeded"
        }
    });

    let parsed: BridgeTaskResultResponse = serde_json::from_value(payload).unwrap();
    assert_eq!(parsed.result.reason_code, Some(BridgeFailureCode::LiveBridgeTimeout));
}
```

- [ ] **Step 2: Run the model tests to verify they fail**

Run: `cargo test bridge_task_execute_request_serializes_expected_shape bridge_task_result_response_parses_failure_code -- --nocapture`

Expected: compile failure because the new task-contract types and enums do not exist yet.

- [ ] **Step 3: Add minimal contract types and bridge error variants**

```rust
pub enum BridgeError {
    Config(String),
    Timeout(String),
    Unavailable(String),
    Unauthorized(String),
    UnsupportedContractVersion(String),
    UnsupportedMethod(String),
    InvalidResult(String),
    Protocol(String),
    Http(reqwest::Error),
}
```

- [ ] **Step 4: Run the model tests to verify they pass**

Run: `cargo test bridge_task_execute_request_serializes_expected_shape bridge_task_result_response_parses_failure_code -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the model/error slice**

```bash
git add src/bridge/error.rs src/bridge/models.rs tests/bridge_client_test.rs
git commit -m "feat: add miniqmt task contract bridge models"
```

### Task 3: Contract-Aware Bridge Client

**Files:**
- Modify: `src/bridge/client.rs`
- Modify: `tests/bridge_client_test.rs`

- [ ] **Step 1: Write the failing client tests**

```rust
#[tokio::test]
async fn bridge_client_submits_task_execute_with_bearer_and_contract_version() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "bridge_task_accepted",
            "receipt_timestamp": "2026-04-30T00:00:00Z",
            "bridge_contract_version": "miniqmt.v1",
            "source_name": "miniqmt"
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let receipt = client.task_execute_qmt_submit(&sample_task_execute_request()).await.unwrap();
    assert_eq!(receipt.task_id, "task-1");
}

#[tokio::test]
async fn bridge_client_task_result_falls_back_to_api_key_without_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("x-quantix-api-key", "legacy-key"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_task_result_json()))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(
        server.uri(),
        Some("legacy-key".to_string()),
        None,
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let result = client.task_result("task-1").await.unwrap();
    assert_eq!(result.task_id, "task-1");
}
```

- [ ] **Step 2: Run the client tests to verify they fail**

Run: `cargo test bridge_client_submits_task_execute_with_bearer_and_contract_version bridge_client_task_result_falls_back_to_api_key_without_bearer bridge_client_fetches_capabilities_with_api_key -- --nocapture`

Expected: compile failure because `BridgeHttpClient::new(...)` and the task-contract methods do not yet support the new signature and behavior.

- [ ] **Step 3: Implement the minimal contract-aware client**

```rust
pub async fn task_execute_qmt_submit(
    &self,
    payload: &BridgeTaskExecuteRequest,
) -> Result<BridgeTaskExecuteReceipt> { ... }

pub async fn task_result(&self, task_id: &str) -> Result<BridgeTaskResultResponse> { ... }
```

Rules:
- task-contract calls prefer `Authorization: Bearer`
- if Bearer is absent and API key exists, use `X-Quantix-Api-Key`
- always send `X-Bridge-Contract-Version`
- keep legacy methods on the old API-key path unchanged

- [ ] **Step 4: Run the client tests to verify they pass**

Run: `cargo test --test bridge_client_test -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the client slice**

```bash
git add src/bridge/client.rs tests/bridge_client_test.rs
git commit -m "feat: add task contract bridge client methods"
```

### Task 4: Final Verification and Scope Check

**Files:**
- Modify: none expected beyond prior tasks

- [ ] **Step 1: Run the focused verification suite**

Run: `cargo test runtime_loads_bridge_settings_from_env test_bridge_runtime_settings_default_contract_values test_bridge_runtime_settings_contract_env_override bridge_task_execute_request_serializes_expected_shape bridge_task_result_response_parses_failure_code -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run the bridge integration tests**

Run: `cargo test --test bridge_client_test -- --nocapture`

Expected: PASS

- [ ] **Step 3: Inspect scope before commit**

Run: `git -C /opt/claude/quantix-rust/.worktrees/manual-qmt-live-diag-gap diff --stat`

Expected: only `src/core/runtime.rs`, `src/bridge/error.rs`, `src/bridge/models.rs`, `src/bridge/client.rs`, `tests/bridge_client_test.rs`, and this plan doc changed for this slice.

- [ ] **Step 4: Run GitNexus scope check**

Run: `gitnexus_detect_changes({scope: "all"})`

Expected: changed symbols stay within runtime and bridge contractization expectations.
