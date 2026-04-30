# miniQMT QmtTaskSubmitService And Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a task-contract-aware QMT submit service and switch `QmtLiveExecutionAdapter` from broker-submit semantics to receipt/result semantics without touching CLI or daemon flows yet.

**Architecture:** Introduce a dedicated execution-layer `QmtTaskSubmitService` that shapes `task/execute` requests, validates receipt/result identity, and offers both single-shot result lookup and poll-until-terminal helpers. Refactor `QmtLiveExecutionAdapter` to run the existing live capability gate, return a receipt-based `OrderInitialResponse` from submit, and translate `task/result` payloads into existing `OrderQueryResponse` / `AdapterError` shapes while preserving the existing `ExecutionAdapter` trait.

**Tech Stack:** Rust, `tokio`, `reqwest`, `wiremock`, existing bridge task-contract client/models, execution adapter abstractions

---

### Task 1: Tighten Task-Result Model For Pending Responses

**Files:**
- Modify: `src/bridge/models.rs`
- Modify: `tests/bridge_client_test.rs`

- [ ] **Step 1: Write the failing pending-response parsing test**

```rust
#[test]
fn bridge_task_result_response_allows_pending_without_result_payload() {
    let parsed: BridgeTaskResultResponse = serde_json::from_value(serde_json::json!({
        "task_id": "task-1",
        "status": "pending",
        "bridge_contract_version": "miniqmt.v1",
        "result": null
    }))
    .unwrap();

    assert_eq!(parsed.task_id, "task-1");
    assert_eq!(parsed.status, BridgeTaskLifecycleStatus::Pending);
    assert!(parsed.result.is_none());
}
```

- [ ] **Step 2: Run the parsing test to verify it fails**

Run: `cargo test bridge_task_result_response_allows_pending_without_result_payload --test bridge_client_test -- --nocapture`

Expected: FAIL because `BridgeTaskResultResponse.result` is currently required.

- [ ] **Step 3: Write the minimal model change**

```rust
pub struct BridgeTaskResultResponse {
    pub task_id: String,
    pub status: BridgeTaskLifecycleStatus,
    pub bridge_contract_version: String,
    pub result: Option<BridgeTaskResultPayload>,
}
```

- [ ] **Step 4: Run the parsing test to verify it passes**

Run: `cargo test bridge_task_result_response_allows_pending_without_result_payload --test bridge_client_test -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the pending-response model slice**

```bash
git add src/bridge/models.rs tests/bridge_client_test.rs
git commit -m "feat: allow pending task result payloads"
```

### Task 2: Add QmtTaskSubmitService

**Files:**
- Create: `src/execution/qmt_task_submit_service.rs`
- Modify: `src/execution/mod.rs`
- Test: `tests/qmt_task_contract_test.rs`

- [ ] **Step 1: Write the failing service tests**

```rust
#[tokio::test]
async fn qmt_task_submit_service_returns_receipt_with_local_submission_id() {
    let server = MockServer::start().await;
    mock_live_capabilities(&server).await;
    mock_task_execute_accepted(&server).await;

    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let service = QmtTaskSubmitService::new(client, 1, 10).unwrap();
    let receipt = service
        .submit_order(&sample_adapter_request("cli-1"))
        .await
        .unwrap();

    assert_eq!(receipt.task_id, "task-1");
    assert_eq!(receipt.client_order_id, "cli-1");
    assert!(!receipt.local_submission_id.is_empty());
}

#[tokio::test]
async fn qmt_task_submit_service_rejects_identity_mismatch() {
    let server = MockServer::start().await;
    mock_task_result_identity_mismatch(&server).await;

    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let service = QmtTaskSubmitService::new(client, 1, 10).unwrap();
    let err = service
        .query_task_result_once("task-1", "cli-1", "local-1")
        .await
        .unwrap_err();

    assert!(matches!(err, BridgeError::InvalidResult(_)));
}
```

- [ ] **Step 2: Run the service tests to verify they fail**

Run: `cargo test --test qmt_task_contract_test -- --nocapture`

Expected: compile failure because `QmtTaskSubmitService` and its helper types do not exist yet.

- [ ] **Step 3: Write the minimal service implementation**

```rust
pub struct QmtTaskSubmitService {
    client: BridgeHttpClient,
    poll_interval: Duration,
    poll_timeout: Duration,
}

impl QmtTaskSubmitService {
    pub fn new(client: BridgeHttpClient, poll_interval_ms: u64, poll_timeout_ms: u64) -> Result<Self> { ... }
    pub async fn submit_order(&self, request: &AdapterOrderRequest) -> Result<QmtTaskSubmitReceipt, BridgeError> { ... }
    pub async fn query_task_result_once(&self, task_id: &str, client_order_id: &str, local_submission_id: &str) -> Result<QmtTaskResolvedResult, BridgeError> { ... }
}
```

Rules:
- generate `local_submission_id`
- use `BridgeTaskExecuteRequest`
- validate `bridge_task_accepted` receipt path
- validate `client_order_id` and `local_submission_id` on result payload
- allow pending result without payload

- [ ] **Step 4: Run the service tests to verify they pass**

Run: `cargo test --test qmt_task_contract_test -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the service slice**

```bash
git add src/execution/mod.rs src/execution/qmt_task_submit_service.rs tests/qmt_task_contract_test.rs
git commit -m "feat: add qmt task submit service"
```

### Task 3: Switch QmtLiveExecutionAdapter To Receipt/Result Semantics

**Files:**
- Modify: `src/execution/qmt_live_adapter.rs`
- Test: `tests/qmt_live_adapter_test.rs`

- [ ] **Step 1: Write the failing adapter tests**

```rust
#[tokio::test]
async fn qmt_live_adapter_submit_returns_pending_submit_task_receipt() {
    let server = MockServer::start().await;
    mock_live_capabilities(&server).await;
    mock_task_execute_accepted(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.submit_order(sample_adapter_request("cli-1")).await.unwrap();

    assert_eq!(response.adapter_order_id, "task-1");
    assert_eq!(response.latest_status, OrderStatus::PendingSubmit);
    assert_eq!(response.filled_quantity, 0);
    assert!(response.avg_fill_price.is_none());
}

#[tokio::test]
async fn qmt_live_adapter_query_maps_acknowledgement_to_accepted() {
    let server = MockServer::start().await;
    mock_task_result_ack(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.query_order("task-1").await.unwrap();

    assert_eq!(response.adapter_order_id, "task-1");
    assert_eq!(response.latest_status, OrderStatus::Accepted);
}

#[tokio::test]
async fn qmt_live_adapter_query_maps_reject_to_rejection_reason() {
    let server = MockServer::start().await;
    mock_task_result_reject(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.query_order("task-1").await.unwrap();

    assert_eq!(response.latest_status, OrderStatus::Rejected);
    assert_eq!(response.rejection_reason.as_deref(), Some("price rejected"));
}
```

- [ ] **Step 2: Run the adapter tests to verify they fail**

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected: FAIL because the adapter still submits to `/broker/qmt/orders` and queries `/broker/qmt/orders/{id}`.

- [ ] **Step 3: Write the minimal adapter refactor**

```rust
pub struct QmtLiveExecutionAdapter {
    client: BridgeHttpClient,
    submit_service: QmtTaskSubmitService,
    adapter_name: &'static str,
}
```

Rules:
- `submit_order(...)` must run `ensure_bridge_qmt_live_mode(...)`
- submit path returns task receipt mapped to `PendingSubmit`
- `query_order(...)` must call `task_result(...)` through the service
- `reject` maps to `OrderStatus::Rejected`
- `acknowledgement` maps to `OrderStatus::Accepted`
- `execution` maps to the existing closest terminal semantic for now
- cancellation stays on the existing compatibility endpoint

- [ ] **Step 4: Run the adapter tests to verify they pass**

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit the adapter slice**

```bash
git add src/execution/qmt_live_adapter.rs tests/qmt_live_adapter_test.rs
git commit -m "feat: switch qmt live adapter to task receipt semantics"
```

### Task 4: Final Verification And Scope Check

**Files:**
- Modify: none expected beyond prior tasks

- [ ] **Step 1: Run focused task-contract verification**

Run: `cargo test --test bridge_client_test -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run new service and adapter tests**

Run: `cargo test --test qmt_task_contract_test -- --nocapture`

Expected: PASS

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected: PASS

- [ ] **Step 3: Re-run the prior compatibility checks touched by runtime config**

Run: `cargo test --test monitor_systemd_test -- --nocapture`

Expected: PASS

Run: `cargo test --test strategy_systemd_test -- --nocapture`

Expected: PASS

- [ ] **Step 4: Inspect scope**

Run: `git -C /opt/claude/quantix-rust/.worktrees/manual-qmt-live-diag-gap diff --stat`

Expected: changes limited to `src/bridge/models.rs`, `src/execution/mod.rs`, `src/execution/qmt_task_submit_service.rs`, `src/execution/qmt_live_adapter.rs`, `tests/bridge_client_test.rs`, `tests/qmt_task_contract_test.rs`, `tests/qmt_live_adapter_test.rs`, and this plan doc.

- [ ] **Step 5: Run GitNexus scope check**

Run: `gitnexus_detect_changes({scope: "all"})`

Expected: changed execution flows stay within the expected bridge task-contract and qmt live adapter surface.
