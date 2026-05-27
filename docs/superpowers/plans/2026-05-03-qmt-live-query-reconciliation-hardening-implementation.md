# QMT Live Query Reconciliation Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist `qmt_live` task identity and reconciliation summaries into runtime orders, then use those persisted facts to converge recoverable order state by `task_id` and expose the result through CLI surfaces.

**Architecture:** Keep `orders.payload_json` as the only persistence surface for `qmt_live` runtime metadata. First close the current boundary mismatch by moving the real operator-facing `execute_execution_bridge_qmt_live(...)` path onto `QmtTaskSubmitService` task-receipt semantics and by creating/updating the related runtime `OrderRecord`; then add a typed runtime-store payload update path plus a `ReconciliationService` branch that queries by `task_id` and persists last-writer-wins recovery summaries.

**Tech Stack:** Rust, tokio, sqlx/sqlite, serde/serde_json, chrono, wiremock, cargo test

---

## File Map

- `src/execution/models.rs`
  - Add typed `qmt_live` metadata structs that serialize cleanly into `OrderRecord.payload_json["qmt_live"]`.
- `src/execution/runtime_store/orders.rs`
  - Add version-checked payload update helpers and a typed `qmt_live` metadata write path that preserves unrelated payload keys.
- `src/execution/qmt_task_submit_service.rs`
  - Keep task-receipt and task-result mapping as the single local contract adapter for `qmt_live`.
- `src/cli/handlers/execution_handler.rs`
  - Switch the manual `qmt_live` submit path from legacy broker-submit semantics to task receipt semantics, create/update the related order record, and change operator guidance away from legacy `qmt-query`.
- `src/execution/reconciliation.rs`
  - Add the `qmt_live` recovery branch that reads `task_id`, calls `query_task_result_by_task_id(...)`, and persists `last_query` plus reconciliation state.
- `src/cli/handlers/strategy_handler/requests.rs`
  - Read related runtime-order facts and render compact plus detailed `qmt_live` reconciliation visibility.
- `tests/execution_runtime_store_test.rs`
  - Lock payload-preserving metadata updates.
- `tests/qmt_live_reconciliation_test.rs`
  - Add first-pass integration coverage for pending, reject, execution/fill, bridge failure, and missing-`task_id` reconciliation outcomes.
- `src/cli/handlers/tests/strategy_execution.rs`
  - Lock the manual `qmt_live` submit path and related order persistence.
- `src/cli/handlers/tests/strategy_requests.rs`
  - Lock request row/detail rendering and post-submit guidance wording.

## Boundary Notes

- The current real operator-facing `qmt_live` path is `execute_execution_bridge_qmt_live(...)`, not the generic daemon/kernel path.
- The current handler still posts to `/api/v1/broker/qmt/orders` and prints legacy `qmt-query` guidance.
- `task_id`-driven reconciliation cannot work end-to-end for actual live submissions until that handler is moved onto `QmtTaskSubmitService` and until a runtime `OrderRecord` exists for the request.
- `QmtTaskResolvedResult` already has `filled_quantity` and `avg_fill_price` fields, but the current bridge task-result payload does not populate them. First pass should keep the storage shape stable and persist `0` / `null` when the bridge has no fill snapshot yet, rather than inventing values.

### Task 1: Add Typed `qmt_live` Runtime Metadata And Payload Update Helpers

**Files:**
- Modify: `src/execution/models.rs`
- Modify: `src/execution/runtime_store/orders.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write the failing runtime-store test**

```rust
#[tokio::test]
async fn qmt_live_runtime_metadata_update_preserves_unrelated_payload_keys() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let mut order = sample_order(&run.run_id, "run_qmt_live_metadata");
    order.adapter = "qmt_live".to_string();
    order.payload_json = json!({
        "reason": "ma_cross_buy",
        "nested": { "keep": true }
    });
    store.insert_order(&order).await.unwrap();

    let metadata = QmtLiveRuntimeMetadata {
        task_identity: Some(QmtLiveTaskIdentity {
            task_id: "task-1".to_string(),
            client_order_id: "run_qmt_live_metadata".to_string(),
            local_submission_id: "local-1".to_string(),
            external_order_id: None,
        }),
        last_query: None,
        reconciliation: None,
    };

    let updated = store
        .try_update_order_qmt_live_metadata(&order, &metadata, fixed_ts() + chrono::Duration::minutes(1))
        .await
        .unwrap();

    assert!(updated);

    let saved = store
        .find_order_by_client_order_id("run_qmt_live_metadata")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.payload_json["reason"], "ma_cross_buy");
    assert_eq!(saved.payload_json["nested"]["keep"], true);
    assert_eq!(saved.payload_json["qmt_live"]["task_identity"]["task_id"], "task-1");
}
```

- [ ] **Step 2: Run the focused store test and confirm it fails because the metadata types/helpers do not exist**

Run: `cargo test qmt_live_runtime_metadata_update_preserves_unrelated_payload_keys -- --nocapture`

Expected: FAIL with missing `QmtLiveRuntimeMetadata` / `try_update_order_qmt_live_metadata(...)`.

- [ ] **Step 3: Add the typed metadata structs and version-checked payload helpers**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveTaskIdentity {
    pub task_id: String,
    pub client_order_id: String,
    pub local_submission_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveLastQuerySummary {
    pub latest_status: String,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_fill_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broker_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveReconciliationState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveRuntimeMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_identity: Option<QmtLiveTaskIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_query: Option<QmtLiveLastQuerySummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconciliation: Option<QmtLiveReconciliationState>,
}
```

```rust
pub async fn try_update_order_payload_with_version(
    &self,
    order_id: &str,
    expected_version: i64,
    payload_json: serde_json::Value,
    updated_at: DateTime<Utc>,
) -> Result<bool> {
    let result = sqlx::query(
        r#"
UPDATE orders
SET payload_json = ?,
    updated_at = ?,
    version = version + 1
WHERE order_id = ? AND version = ?
"#,
    )
    .bind(serde_json::to_string(&payload_json)?)
    .bind(updated_at.to_rfc3339())
    .bind(order_id)
    .bind(expected_version)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub async fn try_update_order_qmt_live_metadata(
    &self,
    order: &OrderRecord,
    metadata: &QmtLiveRuntimeMetadata,
    updated_at: DateTime<Utc>,
) -> Result<bool> {
    let mut payload_json = order.payload_json.clone();
    payload_json["qmt_live"] = serde_json::to_value(metadata)?;
    self.try_update_order_payload_with_version(
        &order.order_id,
        order.version,
        payload_json,
        updated_at,
    )
    .await
}
```

- [ ] **Step 4: Re-run the focused store test and the full runtime-store suite**

Run: `cargo test qmt_live_runtime_metadata_update_preserves_unrelated_payload_keys -- --nocapture`

Expected: PASS

Run: `cargo test --test execution_runtime_store_test -- --nocapture`

Expected: PASS, including the new metadata regression coverage.

- [ ] **Step 5: Commit the storage foundation**

```bash
git add src/execution/models.rs src/execution/runtime_store/orders.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add qmt live runtime metadata storage helpers"
```

### Task 2: Move Manual `qmt_live` Submit Onto Task Receipt Semantics And Persist A Related Order

**Files:**
- Modify: `src/cli/handlers/execution_handler.rs`
- Modify: `src/execution/qmt_task_submit_service.rs`
- Modify: `src/cli/handlers/tests/strategy_execution.rs`
- Test: `tests/qmt_task_contract_test.rs`

- [ ] **Step 1: Write the failing handler test that proves the manual live path now needs a related order plus `task_id` metadata**

```rust
#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    mock_live_capabilities(&server).await;
    mock_task_execute_accepted(&server).await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-task-id", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-task-id",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap();

    let saved_request = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_request.payload_json["execution_result"]["adapter_order_id"], "task-1");
    assert_eq!(saved_request.payload_json["execution_result"]["order_status"], "pending_submit");

    let saved_order = runtime_store
        .find_order_by_client_order_id(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_order.adapter, "qmt_live");
    assert_eq!(saved_order.status, OrderStatus::PendingSubmit);
    assert_eq!(saved_order.payload_json["qmt_live"]["task_identity"]["task_id"], "task-1");
}
```

- [ ] **Step 2: Run the focused handler test and confirm it fails on the legacy submit path**

Run: `cargo test test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order -- --nocapture`

Expected: FAIL because the handler still uses `/api/v1/broker/qmt/orders`, does not create a related order row, and does not persist `qmt_live.task_identity`.

- [ ] **Step 3: Replace the legacy submit call with `QmtTaskSubmitService` and persist the related order**

```rust
let signal = runtime_store
    .get_signal(&request.signal_id)
    .await?
    .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {}", request.signal_id)))?;

let side = match side.to_ascii_lowercase().as_str() {
    "buy" => OrderSide::Buy,
    "sell" => OrderSide::Sell,
    other => {
        return Err(QuantixError::Other(format!("不支持的 side: {other}")));
    }
};
let order_type = match order_type {
    "market" => OrderType::Market,
    _ => OrderType::Limit,
};
let requested_price = Decimal::from_str(price)
    .map_err(|err| QuantixError::Other(format!("无效价格 {price}: {err}")))?;

let submit_service = QmtTaskSubmitService::new(client.clone(), 1, 30_000)
    .map_err(|err| QuantixError::Other(err.to_string()))?;

let receipt = submit_service
    .submit_order(&AdapterOrderRequest {
        client_order_id: request.request_id.clone(),
        symbol: normalize_symbol_for_bridge(symbol),
        side,
        quantity,
        price: requested_price,
    })
    .await
    .map_err(|err| QuantixError::Other(err.to_string()))?;

let metadata = QmtLiveRuntimeMetadata {
    task_identity: Some(QmtLiveTaskIdentity {
        task_id: receipt.task_id.clone(),
        client_order_id: request.request_id.clone(),
        local_submission_id: receipt.local_submission_id.clone(),
        external_order_id: None,
    }),
    last_query: None,
    reconciliation: Some(QmtLiveReconciliationState {
        last_action: Some("no_action".to_string()),
        last_error: None,
        last_attempt_at: Some(started_at.to_rfc3339()),
    }),
};

let related_order = OrderRecord {
    order_id: request.request_id.clone(),
    client_order_id: request.request_id.clone(),
    run_id: signal.run_id.clone(),
    symbol: symbol.to_string(),
    side,
    order_type,
    requested_quantity: quantity,
    requested_price,
    filled_quantity: 0,
    remaining_quantity: quantity,
    avg_fill_price: None,
    status: OrderStatus::PendingSubmit,
    adapter: "qmt_live".to_string(),
    created_at: started_at,
    updated_at: started_at,
    last_transition_at: started_at,
    version: 0,
    payload_json: serde_json::json!({
        "qmt_live": serde_json::to_value(&metadata)?,
    }),
};

match runtime_store
    .find_order_by_client_order_id(&request.request_id)
    .await?
{
    Some(existing) => {
        let _ = runtime_store
            .try_update_order_qmt_live_metadata(&existing, &metadata, started_at)
            .await?;
    }
    None => {
        runtime_store.insert_order(&related_order).await?;
        runtime_store
            .insert_order_event(&OrderEventRecord {
                event_id: uuid::Uuid::new_v4().to_string(),
                order_id: related_order.order_id.clone(),
                client_order_id: related_order.client_order_id.clone(),
                event_type: "pending_submit".to_string(),
                event_time: started_at,
                details_json: serde_json::json!({
                    "task_id": receipt.task_id,
                    "local_submission_id": receipt.local_submission_id,
                }),
            })
            .await?;
    }
}
```

Key rules for this step:

- keep daemon `qmt_live` rejection behavior unchanged
- do not add a new table
- keep request completion semantics as `request_status=Completed` with `execution_result.order_status=pending_submit`
- replace the legacy post-submit `qmt-query` hint with request/reconciliation guidance; do not invent a new bridge polling command

- [ ] **Step 4: Re-run the focused handler and task-contract tests**

Run: `cargo test test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order -- --nocapture`

Expected: PASS

Run: `cargo test --test qmt_task_contract_test -- --nocapture`

Expected: PASS, proving the handler now matches the existing task contract.

- [ ] **Step 5: Commit the manual-path contract alignment**

```bash
git add src/cli/handlers/execution_handler.rs src/execution/qmt_task_submit_service.rs src/cli/handlers/tests/strategy_execution.rs tests/qmt_task_contract_test.rs
git commit -m "feat: align manual qmt live submit with task receipt contract"
```

### Task 3: Add `qmt_live` Reconciliation By `task_id`

**Files:**
- Modify: `src/execution/reconciliation.rs`
- Modify: `src/execution/qmt_task_submit_service.rs`
- Create: `tests/qmt_live_reconciliation_test.rs`

- [ ] **Step 1: Write failing reconciliation integration tests for the three core state classes**

```rust
fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 3, 9, 30, 0).unwrap()
}

fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        trigger: "once".to_string(),
        status: StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({"short": 5, "long": 20}),
    }
}

fn sample_client(server: &MockServer) -> BridgeHttpClient {
    BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap()
}

async fn mock_task_result_pending(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "pending",
            "bridge_contract_version": "miniqmt.v1",
            "result": null
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_reject(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-qmt-live-reject",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-1",
                "occurred_at": "2026-05-03T09:31:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "reject",
                "external_order_id": null,
                "reason_code": "live_bridge_invalid_result",
                "reason_detail": "price rejected",
                "evidence_ref": "evidence-1"
            }
        })))
        .mount(server)
        .await;
}

fn qmt_live_metadata(task_id: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "qmt_live": {
            "task_identity": {
                "task_id": task_id,
                "client_order_id": "req-qmt-live-1",
                "local_submission_id": "local-1",
                "external_order_id": null
            }
        }
    })
}

async fn seed_qmt_live_order(
    store: &StrategyRuntimeStore,
    run_id: &str,
    client_order_id: &str,
    status: OrderStatus,
    payload_json: serde_json::Value,
) {
    store
        .insert_order(&OrderRecord {
            order_id: client_order_id.to_string(),
            client_order_id: client_order_id.to_string(),
            run_id: run_id.to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            requested_quantity: 100,
            requested_price: dec!(10.50),
            filled_quantity: 0,
            remaining_quantity: 100,
            avg_fill_price: None,
            status,
            adapter: "qmt_live".to_string(),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            last_transition_at: fixed_ts(),
            version: 0,
            payload_json,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn qmt_live_reconciliation_keeps_pending_submit_when_task_result_is_pending() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-1",
        OrderStatus::PendingSubmit,
        qmt_live_metadata(Some("task-1")),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_pending(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::PendingSubmit);
    assert_eq!(saved.payload_json["qmt_live"]["last_query"]["latest_status"], "pending_submit");
    assert_eq!(saved.payload_json["qmt_live"]["reconciliation"]["last_action"], "no_action");
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_rejected_and_persists_reason() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-reject",
        OrderStatus::Accepted,
        qmt_live_metadata(Some("task-1")),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_reject(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-reject")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Rejected);
    assert_eq!(saved.payload_json["qmt_live"]["last_query"]["rejection_reason"], "price rejected");
    assert_eq!(saved.payload_json["qmt_live"]["reconciliation"]["last_action"], "state_updated");
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_manual_intervention_when_task_id_missing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-missing-task",
        OrderStatus::Unknown,
        qmt_live_metadata(None),
    )
    .await;

    let server = MockServer::start().await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-missing-task")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Unknown);
    assert_eq!(saved.payload_json["qmt_live"]["reconciliation"]["last_action"], "manual_intervention");
    assert!(saved.payload_json["qmt_live"]["reconciliation"]["last_error"]
        .as_str()
        .unwrap()
        .contains("task_id"));
}
```

- [ ] **Step 2: Run the new reconciliation test file and confirm it fails because no `qmt_live` branch exists yet**

Run: `cargo test --test qmt_live_reconciliation_test -- --nocapture`

Expected: FAIL because `ReconciliationService` has no task-id-aware `qmt_live` path and no metadata persistence.

- [ ] **Step 3: Add a dedicated `qmt_live` reconciliation branch with atomic state-plus-payload writes**

```rust
pub struct ReconciliationService {
    store: StrategyRuntimeStore,
    scanner: OpenOrderScanner,
    qmt_submit_service: Option<QmtTaskSubmitService>,
}

pub fn with_qmt_live_query_service(
    store: StrategyRuntimeStore,
    qmt_submit_service: QmtTaskSubmitService,
) -> Self {
    let scanner = OpenOrderScanner::new(store.clone());
    Self {
        store,
        scanner,
        qmt_submit_service: Some(qmt_submit_service),
    }
}

async fn reconcile_qmt_live_order(&self, order: &OrderRecord) -> Result<OrderReconciliationResult> {
    let task_id = order
        .payload_json
        .get("qmt_live")
        .and_then(|v| v.get("task_identity"))
        .and_then(|v| v.get("task_id"))
        .and_then(|v| v.as_str());

    let Some(task_id) = task_id else {
        return self.persist_qmt_live_manual_intervention(order, "task-id-based recovery is unavailable").await;
    };

    let service = self
        .qmt_submit_service
        .as_ref()
        .ok_or_else(|| QuantixError::Other("qmt_live reconciliation service missing".to_string()))?;

    match service.query_task_result_by_task_id(task_id).await {
        Ok(result) => self.apply_qmt_live_result(order, result).await,
        Err(err) => self.persist_qmt_live_query_failure(order, err).await,
    }
}
```

Implementation rules for this step:

- eligible statuses: `PendingSubmit`, `Submitted`, `Accepted`, `Unknown`
- preserve state for `PendingSubmit` and other non-terminal query results
- move to `Accepted`, `Rejected`, or `Filled` only on explicit completed broker facts
- keep `PartiallyFilled` out of the automatic recovery set but never hide it from CLI
- use last-writer-wins semantics
- update `payload_json.qmt_live.last_query` and `payload_json.qmt_live.reconciliation` on every attempt
- when both state and payload change, update them together in one version-checked store call rather than in two independent writes

- [ ] **Step 4: Re-run the new reconciliation file and the module-level reconciliation tests**

Run: `cargo test --test qmt_live_reconciliation_test -- --nocapture`

Expected: PASS

Run: `cargo test reconciliation -- --nocapture`

Expected: PASS, including the existing `src/execution/reconciliation/tests.rs` unit coverage.

- [ ] **Step 5: Commit the recovery loop**

```bash
git add src/execution/reconciliation.rs src/execution/qmt_task_submit_service.rs tests/qmt_live_reconciliation_test.rs
git commit -m "feat: add qmt live task-id reconciliation"
```

### Task 4: Expose Persisted Recovery Facts Through Request/Order CLI Surfaces

**Files:**
- Modify: `src/cli/handlers/strategy_handler/requests.rs`
- Modify: `src/cli/handlers/execution_handler.rs`
- Modify: `src/cli/handlers/tests/strategy_requests.rs`
- Modify: `src/cli/handlers/tests/strategy_execution.rs`

- [ ] **Step 1: Write the failing formatter tests for compact and detailed `qmt_live` visibility**

```rust
#[test]
fn test_format_strategy_request_detail_displays_qmt_live_recovery_context_from_related_order() {
    let request = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-qmt-live-detail".to_string(),
        signal_id: "signal-qmt-live-detail".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "client_order_id": "req-qmt-live-detail",
                "adapter_order_id": "task-1",
                "order_status": "pending_submit"
            }
        }),
    };
    let order = crate::execution::models::OrderRecord {
        order_id: "req-qmt-live-detail".to_string(),
        client_order_id: "req-qmt-live-detail".to_string(),
        run_id: "run-1".to_string(),
        symbol: "000001".to_string(),
        side: crate::execution::models::OrderSide::Buy,
        order_type: crate::execution::models::OrderType::Limit,
        requested_quantity: 100,
        requested_price: dec!(10.50),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: crate::execution::models::OrderStatus::Accepted,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: json!({
        "qmt_live": {
            "task_identity": {
                "task_id": "task-1",
                "client_order_id": "req-qmt-live-detail",
                "local_submission_id": "local-1",
                "external_order_id": "broker-1"
            },
            "last_query": {
                "latest_status": "accepted",
                "filled_quantity": 0,
                "avg_fill_price": null,
                "broker_event_type": "Acknowledgement",
                "rejection_reason": null,
                "updated_at": "2026-05-03T09:32:00Z"
            },
            "reconciliation": {
                "last_action": "state_updated",
                "last_error": null,
                "last_attempt_at": "2026-05-03T09:32:00Z"
            }
        }
    }),
    };

    let detail = format_strategy_request_detail_with_related_order(&request, Some(&order), false);

    assert!(detail.contains("=== QMT Live Recovery ==="));
    assert!(detail.contains("task_id: task-1"));
    assert!(detail.contains("latest_status: accepted"));
    assert!(detail.contains("broker_event_type: Acknowledgement"));
    assert!(detail.contains("last_action: state_updated"));
}

#[test]
fn test_format_strategy_request_row_appends_compact_qmt_live_recovery_suffix() {
    let request = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-qmt-live-row".to_string(),
        signal_id: "signal-qmt-live-row".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "client_order_id": "req-qmt-live-row",
                "adapter_order_id": "task-1",
                "order_status": "pending_submit"
            }
        }),
    };
    let order = crate::execution::models::OrderRecord {
        order_id: "req-qmt-live-row".to_string(),
        client_order_id: "req-qmt-live-row".to_string(),
        run_id: "run-1".to_string(),
        symbol: "000001".to_string(),
        side: crate::execution::models::OrderSide::Buy,
        order_type: crate::execution::models::OrderType::Limit,
        requested_quantity: 100,
        requested_price: dec!(10.50),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: crate::execution::models::OrderStatus::PendingSubmit,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: json!({
        "qmt_live": {
            "task_identity": { "task_id": "task-1", "client_order_id": "req-qmt-live-row", "local_submission_id": "local-1", "external_order_id": null },
            "last_query": { "latest_status": "pending_submit", "filled_quantity": 0, "avg_fill_price": null, "broker_event_type": null, "rejection_reason": null, "updated_at": "2026-05-03T09:32:00Z" },
            "reconciliation": { "last_action": "no_action", "last_error": null, "last_attempt_at": "2026-05-03T09:32:00Z" }
        }
    }),
    };

    let line = format_strategy_request_row_with_related_order(&request, Some(&order));

    assert!(line.contains("qmt_task_id=task-1"));
    assert!(line.contains("qmt_latest_status=pending_submit"));
    assert!(line.contains("qmt_last_action=no_action"));
}
```

- [ ] **Step 2: Run the formatter tests and confirm they fail because the request formatters only know request payloads today**

Run: `cargo test qmt_live_recovery -- --nocapture`

Expected: FAIL with missing `format_strategy_request_detail_with_related_order(...)` / `format_strategy_request_row_with_related_order(...)`.

- [ ] **Step 3: Add related-order-aware formatting and update the manual submit guidance**

```rust
pub(crate) fn format_strategy_request_row_with_related_order(
    row: &ExecutionRequestRecord,
    related_order: Option<&OrderRecord>,
) -> String {
    let qmt_suffix = related_order
        .and_then(qmt_live_compact_summary)
        .map(|summary| format!(" {summary}"))
        .unwrap_or_default();

    format!(
        "{} signal={} target={}/{} status={}{}{}{} created_at={}",
        row.request_id,
        row.signal_id,
        row.target_mode,
        row.target_account,
        row.request_status.as_str(),
        compact_semantics_suffix(&row.payload_json, row.request_status),
        compact_diag_suffix(&row.payload_json),
        qmt_suffix,
        row.created_at.format("%Y-%m-%dT%H:%M:%SZ")
    )
}
```

```rust
println!();
println!(
    "查看 request 与后续收敛状态: quantix strategy request show {} --verbose",
    request_id
);
```

Implementation rules for this step:

- request/detail surfaces may join to the related runtime order by `execution_result.client_order_id`
- the formatter must never query the bridge directly
- missing `task_id` must render as automatic reconciliation unavailable
- `PartiallyFilled` must remain visible as a non-terminal attention state
- do not keep the old `qmt-query --order-id` post-submit hint once submit semantics are task-based

- [ ] **Step 4: Re-run the formatter and manual-handler test slices**

Run: `cargo test qmt_live_recovery -- --nocapture`

Expected: PASS

Run: `cargo test execute_execution_bridge_qmt_live -- --nocapture`

Expected: PASS, with updated guidance assertions.

- [ ] **Step 5: Commit the observability layer**

```bash
git add src/cli/handlers/strategy_handler/requests.rs src/cli/handlers/execution_handler.rs src/cli/handlers/tests/strategy_requests.rs src/cli/handlers/tests/strategy_execution.rs
git commit -m "feat: surface qmt live reconciliation context in cli"
```

### Task 5: Close The Gate Loop With Focused Verification

**Files:**
- Modify only if a literal assertion or wording lock requires it: `tests/repo_hygiene_test.rs`
- No new production files

- [ ] **Step 1: Run the focused suites in the same narrow acceptance shape promised by the design**

Run: `cargo test --test qmt_task_contract_test -- --nocapture`

Expected: PASS

Run: `cargo test --test execution_runtime_store_test -- --nocapture`

Expected: PASS

Run: `cargo test --test qmt_live_reconciliation_test -- --nocapture`

Expected: PASS

Run: `cargo test execute_execution_bridge_qmt_live -- --nocapture`

Expected: PASS

Run: `cargo test qmt_live_recovery -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run the broader substring acceptance commands promised in the design**

Run: `cargo test qmt_live -- --nocapture`

Expected: PASS, including adapter, handler, and reconciliation names that intentionally include `qmt_live`.

Run: `cargo test reconciliation -- --nocapture`

Expected: PASS

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: PASS unless a literal wording lock must be updated because the user-facing `qmt-query` guidance changed.

- [ ] **Step 3: Inspect change scope before final commit**

Run: `git diff --stat`

Expected: changes limited to the runtime-store, qmt task submit, reconciliation, and CLI request/handler surfaces listed in this plan.

Run: `git status --short`

Expected: no unrelated files added to the change set.

- [ ] **Step 4: Run the required GitNexus scope check before merge/commit**

Run in-tool: `gitnexus_detect_changes(scope=all)`

Expected: affected symbols centered on `execute_execution_bridge_qmt_live`, `QmtTaskSubmitService`, `ReconciliationService`, runtime-store order helpers, and request formatters; no unexpected process expansion outside execution/reconciliation surfaces.

- [ ] **Step 5: Commit the closed loop**

```bash
git add src/execution/models.rs src/execution/runtime_store/orders.rs src/execution/qmt_task_submit_service.rs src/execution/reconciliation.rs src/cli/handlers/execution_handler.rs src/cli/handlers/strategy_handler/requests.rs tests/execution_runtime_store_test.rs tests/qmt_task_contract_test.rs tests/qmt_live_reconciliation_test.rs src/cli/handlers/tests/strategy_execution.rs src/cli/handlers/tests/strategy_requests.rs
git commit -m "feat: harden qmt live query reconciliation"
```

## Self-Review

- Spec coverage
  - submit-time `task_identity` persistence: Task 2
  - typed payload update path with no schema change: Task 1
  - reconciliation by `task_id`: Task 3
  - conservative failure handling and `manual_intervention`: Task 3
  - CLI/detail observability from persisted runtime facts: Task 4
  - narrow acceptance commands and test-name substring discipline: Task 5
- Placeholder scan
  - no `TODO` / `TBD` placeholders remain
  - every task names concrete files, commands, and the expected assertions
- Type consistency
  - plan uses one stable metadata shape: `QmtLiveRuntimeMetadata -> task_identity / last_query / reconciliation`
  - CLI wording aligns with the design decision to stop telling operators to use legacy `qmt-query` after a task-based submit

## Local Graphiti Fallback

- `quantix_rust_docs` episode queued: `551af23a-39cc-4682-a86e-9768c0269c69`
- `quantix_rust_handoff` episode queued: `8521418e-6d9e-4aba-b149-e29cd957ec87`
- Repeated `get_ingest_status` checks remained in `processing` during this session, so this local note is the equivalent persisted summary for the implementation-plan milestone.
- Task 2 checkpoint local summary:
  - commit `a30d4ba feat: align manual qmt live submit with task receipt contract`
  - manual `execute_execution_bridge_qmt_live(...)` now uses `QmtTaskSubmitService` task receipts, persists related runtime order `payload_json.qmt_live.task_identity`, and stores request `execution_result.adapter_order_id=task_id` with `order_status=pending_submit`
  - legacy post-submit `qmt-query` guidance was replaced with `quantix strategy request show <request_id> --verbose`
  - focused verification passed:
    - `cargo test test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order -- --nocapture`
    - `cargo test --test qmt_task_contract_test -- --nocapture`
    - `cargo test execute_execution_bridge_qmt_live -- --nocapture`
  - Graphiti handoff episode `f8e73957-4608-4c65-93d8-96e9401276c8` still `processing` after repeated polling
  - Graphiti backfill required
- Task 3 checkpoint local summary:
  - commit `573af11 feat: add qmt live task-id reconciliation`
  - `ReconciliationService` now routes `qmt_live` orders in `PendingSubmit`/`Submitted`/`Accepted`/`Unknown` through task-id-based recovery before generic Unknown handling
  - runtime order `payload_json.qmt_live.last_query` and `payload_json.qmt_live.reconciliation` are persisted on every attempt
  - broker-confirmed status transitions now use a new atomic state+payload optimistic update helper in `runtime_store/orders.rs`
  - focused verification passed:
    - `cargo test --test qmt_live_reconciliation_test -- --nocapture`
    - `cargo test reconciliation -- --nocapture`
    - `cargo test --test execution_runtime_store_test -- --nocapture`
  - Graphiti handoff episode `d8e9d431-4fb8-434b-9e44-7f674c9e020b` remained `processing`
  - Graphiti backfill required
- Task 4 and closure gate local summary:
  - commit `518cae4 feat: surface qmt live reconciliation context in cli`
  - request list/show now join related runtime orders by `execution_result.client_order_id`
  - request row adds compact `qmt_live` suffixes such as `qmt_task_id`, `qmt_latest_status`, `qmt_last_action`, or `qmt_recovery=unavailable`
  - request detail adds `=== QMT Live Recovery ===` sourced from persisted runtime order facts instead of direct bridge queries
  - closure gates passed on the current head:
    - `cargo test --test qmt_task_contract_test -- --nocapture`
    - `cargo test --test execution_runtime_store_test -- --nocapture`
    - `cargo test qmt_live_recovery -- --nocapture`
    - `cargo test execute_execution_bridge_qmt_live -- --nocapture`
    - `cargo test qmt_live -- --nocapture`
  - worktree is clean except the intentionally untracked local plan/review docs
  - Graphiti handoff episode `818c6f1c-42fc-4ff8-81be-0aa8b9c0054e` remained `queued`
  - Graphiti backfill required

Graphiti backfill required

## 2026-05-03 Closure Re-Verification Fallback

- Fresh closure verification rerun on branch `feat/qmt-live-query-reconciliation-hardening`:
  - `cargo test qmt_live -- --nocapture`
  - `cargo test reconciliation -- --nocapture`
- Both commands passed on the current `HEAD` `518cae4`.
- Real git range scope remains:
  - `src/cli/handlers/execution_handler.rs`
  - `src/cli/handlers/strategy_handler/requests.rs`
  - `src/cli/handlers/tests/strategy_execution.rs`
  - `src/cli/handlers/tests/strategy_requests.rs`
  - `src/execution/models.rs`
  - `src/execution/reconciliation.rs`
  - `src/execution/runtime_store/orders.rs`
  - `tests/execution_runtime_store_test.rs`
  - `tests/qmt_live_reconciliation_test.rs`
- `git status --short --branch` still shows only the intentionally untracked local plan/review docs.
- Graphiti write attempts for this closure checkpoint:
  - `quantix_rust_handoff` episode `50d526f7-aaf5-43c0-bd79-f97995c507a4`
  - `quantix_rust_review` episode `a85cab82-6f41-49f8-8264-0646a4b6c255`
- Repeated `get_ingest_status` polling remained at `processing`, so the Graphiti closure record is not yet trustworthy.

Graphiti backfill required

## 2026-05-03 Local Master Squash Merge Fallback

- Local `master` squashed branch `feat/qmt-live-query-reconciliation-hardening` into commit `d90a43f feat: harden qmt live query reconciliation`.
- The squash commit included exactly these intended files:
  - `src/cli/handlers/execution_handler.rs`
  - `src/cli/handlers/strategy_handler/requests.rs`
  - `src/cli/handlers/tests/strategy_execution.rs`
  - `src/cli/handlers/tests/strategy_requests.rs`
  - `src/execution/models.rs`
  - `src/execution/reconciliation.rs`
  - `src/execution/runtime_store/orders.rs`
  - `tests/execution_runtime_store_test.rs`
  - `tests/qmt_live_reconciliation_test.rs`
- Pre-commit `gitnexus_detect_changes(scope=staged)` on `/opt/claude/quantix-rust` reported `changed_files=9` and `risk_level=critical`; the critical label was expected because execution and reconciliation mainline surfaces were touched, and the changed scope matched the intended qmt_live hardening slice.
- Fresh verification on local `master` after `git merge --squash` passed:
  - `cargo test qmt_live -- --nocapture`
  - `cargo test reconciliation -- --nocapture`
- Existing unrelated local dirty files on `master` remained uncommitted and outside the squash commit:
  - `.omc/project-memory.json`
  - `.omc/state/hud-state.json`
  - `.omc/state/hud-stdin-cache.json`
  - `.omc/state/idle-notif-cooldown.json`
  - `.omc/state/last-tool-error.json`
  - `CHANGELOG.md`
  - `FUNCTION_TREE.md`
- No push and no PR were performed.
- Graphiti handoff episode `ea623b99-f652-444c-b150-180e7e2a720f` remained `queued` after polling.

Graphiti backfill required

## 2026-05-03 Branch Cleanup Fallback

- After the local squash merge landed in `master` as commit `d90a43f feat: harden qmt live query reconciliation`, the local worktree `/opt/claude/quantix-rust/.worktrees/qmt-live-query-reconciliation-hardening` was force-removed.
- Before forced removal, the only worktree-unique material was the local fallback notes in this plan file, and those notes were copied into the main repository copy first.
- The local branch `feat/qmt-live-query-reconciliation-hardening` was then deleted with `git branch -D`.
- No push and no PR were performed.
- Graphiti handoff episode `0828711c-57e8-49e5-ac1a-b5840be9547f` remained `processing` after polling.

Graphiti backfill required
