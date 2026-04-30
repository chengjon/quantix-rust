# miniQMT Task Contract Alignment Design

## 1. Goal

Align `quantix-rust` with the external miniQMT v1 contract by introducing a contract-first QMT live submission path based on:

- `POST /api/v1/task/execute`
- `GET /api/v1/task/result/{task_id}`
- `Authorization: Bearer <token>`
- `X-Bridge-Contract-Version`

The alignment must preserve the existing guarded `qmt_live` safety gate and the existing `qmt-preview` path, while changing live submission semantics from direct broker-style submit to receipt-plus-result semantics.

## 2. Current Baseline

The current Rust-side QMT integration is still broker-style:

- `src/bridge/client.rs`
  - calls `/api/v1/broker/qmt/orders*`
  - uses `X-Quantix-Api-Key`
- `src/execution/qmt_bridge.rs`
  - keeps preview on `/api/v1/broker/qmt/orders/preview`
- `src/execution/qmt_live_adapter.rs`
  - treats submit response as initial order state
- `src/cli/handlers/execution_handler.rs`
  - manual `execution bridge qmt-live` marks `execution_request` complete or failed immediately after submit
- `src/execution/daemon.rs`
  - `qmt_live` currently flows through the same immediate-completion execution kernel shape used by adapters that do not require deferred polling

This drifts from the external miniQMT v1 plan, where:

- `task/execute` only returns bridge receipt
- `task/result` is the canonical result lookup
- bridge failure and broker-facing result must be distinguished explicitly
- stable identity echo must rely on `client_order_id` and `local_submission_id`

## 3. Chosen Approach

Use a compatibility-first layered migration.

### 3.1 Preserve

- `qmt-preview` preview path
- `capabilities()` preflight checks
- existing QMT query, cancel, account, positions, and asset compatibility endpoints
- existing guarded `qmt_live` mode requirement:
  - `qmt.enabled = true`
  - `qmt.mode = live`
  - `qmt.supports` contains `order_submit`

### 3.2 Add

- task contract models
- Bearer token and contract-version support
- task-contract-aware bridge client methods
- QMT live submit/result orchestration service
- deferred `execution_request` completion semantics

### 3.3 Avoid

- removing current broker-style helper endpoints in this phase
- changing preview semantics
- re-enabling generic `target_mode=live`
- changing runtime store database schema in this phase

## 4. Architecture Boundaries

### 4.1 Preview Boundary

`src/execution/qmt_bridge.rs` remains preview-only and continues to use `/api/v1/broker/qmt/orders/preview`.

Preview is intentionally excluded from the new task contract.

### 4.2 Live Submit Boundary

All real QMT submission paths are realigned to the task contract:

- manual CLI path: `execution bridge qmt-live`
- adapter path: `QmtLiveExecutionAdapter`
- daemon `qmt_live` execution path

These paths must:

1. run `ensure_bridge_qmt_live_mode(...)`
2. call `task/execute`
3. receive bridge receipt
4. poll `task/result`
5. only write broker-facing final state after result resolution

### 4.3 Compatibility Boundary

Existing broker-style endpoints remain available for:

- preview
- query
- cancel
- account status
- positions
- asset

Only submit semantics move to task contract in this phase.

## 5. Runtime Configuration

Extend `src/core/runtime.rs` so `BridgeRuntimeSettings` becomes contract-aware while keeping current settings valid.

### 5.1 Existing Fields Kept

- `base_url`
- `api_key` compatibility input

### 5.2 New Fields

- `bearer_token: Option<String>`
- `api_key_fallback: Option<String>`
- `contract_version: String`
- `timeout_ms: u64`
- `poll_interval_ms: u64`
- `poll_timeout_ms: u64`

### 5.3 New Environment Variables

- `QUANTIX_BRIDGE_BEARER_TOKEN`
- `QUANTIX_BRIDGE_CONTRACT_VERSION`
- `QUANTIX_BRIDGE_TIMEOUT_MS`
- `QUANTIX_BRIDGE_POLL_INTERVAL_MS`
- `QUANTIX_BRIDGE_POLL_TIMEOUT_MS`

### 5.4 Rules

- task contract requests prefer `Authorization: Bearer`
- `QUANTIX_BRIDGE_API_KEY` remains fallback-only
- if neither Bearer nor fallback API key is present, task contract calls fail with configuration error

## 6. Bridge Error Model

Expand `src/bridge/error.rs` from generic HTTP/config failures into explicit bridge-contract failures.

### 6.1 Required Error Kinds

- `Config`
- `Timeout`
- `Unavailable`
- `Unauthorized`
- `UnsupportedContractVersion`
- `UnsupportedMethod`
- `InvalidResult`
- `Protocol`
- `Http`

### 6.2 Mapping Rules

- `401/403` -> `Unauthorized`
- HTTP `400` with `reason_code=live_bridge_unsupported_contract_version` -> `UnsupportedContractVersion`
- HTTP `400` with `reason_code=live_bridge_unsupported_method` -> `UnsupportedMethod`
- connect error / `5xx` -> `Unavailable`
- polling deadline exceeded -> `Timeout`
- missing required identity echo or malformed result shape -> `InvalidResult` or `Protocol`

These error kinds are the Rust-side source of truth for downstream CLI output and `execution_request.payload_json` failure evidence.

## 7. Bridge Contract Models

Add task contract models to `src/bridge/models.rs` alongside the existing broker-style preview/live models.

### 7.1 Execute Request

`BridgeTaskExecuteRequest`

- `provider: "qmt"`
- `method: "submit_order"`
- `params: BridgeTaskExecuteParams`

`BridgeTaskExecuteParams` must include:

- `request_id`
- `client_order_id`
- `local_submission_id`
- `symbol`
- `side`
- `quantity`
- `price`
- `order_type`
- `strategy_name: Option<String>`
- `order_remark: Option<String>`
- `snapshot_metadata: Option<serde_json::Value>`

### 7.2 Execute Receipt

`BridgeTaskExecuteReceipt` must include:

- `task_id`
- `status`
- `receipt_timestamp`
- `bridge_contract_version`
- `source_name`

`status` is expected to be `bridge_task_accepted` for the accepted receipt path.

### 7.3 Task Result

`BridgeTaskResultResponse` must expose:

- `task_id`
- `status`
- `bridge_contract_version`
- `result`

`result` must contain:

- `client_order_id`
- `local_submission_id`
- `account_scope`
- `event_id`
- `occurred_at`
- `source_name`
- `broker_event_type: Option<BridgeBrokerEventType>`
- `external_order_id: Option<String>`
- `reason_code: Option<BridgeFailureCode>`
- `reason_detail: Option<String>`
- `evidence_ref: Option<String>`

### 7.4 Enums

Required enums:

- `BridgeTaskLifecycleStatus`
- `BridgeImmediateOutcomeStatus`
- `BridgeFailureCode`
- `BridgeBrokerEventType`

`BridgeFailureCode` must at minimum support:

- `live_bridge_timeout`
- `live_bridge_unavailable`
- `live_bridge_auth_failed`
- `live_bridge_unsupported_contract_version`
- `live_bridge_unsupported_method`
- `live_bridge_invalid_result`
- `live_bridge_identity_mismatch`

## 8. Bridge Client Changes

`src/bridge/client.rs` becomes contract-aware without breaking current capability and preview usage.

### 8.1 Preserve

- `capabilities()`
- `fetch_tdx_quotes(...)`
- `fetch_tdx_kline(...)`
- `qmt_preview_order(...)`
- broker-style query/cancel/account/positions/asset methods

### 8.2 Add

- contract-aware header injection
- `task_execute_qmt_submit(...)`
- `task_result(...)`

### 8.3 Header Rules

For task contract requests:

- include `Authorization: Bearer <token>` when `bearer_token` exists
- include `X-Bridge-Contract-Version`
- optionally include fallback API-key header only for compatibility if Bearer is absent

For legacy compatibility methods:

- preserve existing behavior
- future migration is allowed but is not part of this design

## 9. QMT Live Submit Orchestration

Introduce a dedicated execution-layer service for task-contract-driven real submission, for example:

- `QmtTaskSubmitService`

Responsibilities:

1. generate `local_submission_id`
2. create `BridgeTaskExecuteRequest`
3. call `task/execute`
4. validate receipt shape
5. poll `task/result`
6. validate identity echo
7. translate task result into Rust-side order semantics

### 9.1 Identity Rules

The service must treat the following as mandatory identity anchors:

- `client_order_id`
- `local_submission_id`

`external_order_id` is optional and must only appear when the bridge provides real broker-facing evidence.

### 9.2 Polling Rules

- polling interval uses runtime config
- polling deadline uses runtime config
- `pending` remains pending until deadline
- deadline exceeded becomes bridge `Timeout`

## 10. Adapter Semantics

Keep the existing `ExecutionAdapter` trait unchanged in `src/execution/adapter.rs`.

### 10.1 `submit_order(...)`

`QmtLiveExecutionAdapter.submit_order(...)` changes meaning:

- before submit: still run `ensure_bridge_qmt_live_mode(...)`
- submit path: call `task/execute`
- returned `OrderInitialResponse` represents receipt, not broker acknowledgement

Receipt mapping:

- `adapter_order_id = task_id`
- `latest_status = PendingSubmit`
- `filled_quantity = 0`
- `avg_fill_price = None`
- `fill_details = None`
- `rejection_reason = None`

`local_submission_id` is preserved in request payload and evidence, but `adapter_order_id` uses `task_id` because adapter query semantics require a pollable remote handle.

### 10.2 `query_order(task_id)`

`QmtLiveExecutionAdapter.query_order(...)` uses `task/result/{task_id}` and maps:

- `pending` -> `PendingSubmit`
- `broker_event_type=acknowledgement` -> `Submitted` or `Accepted`
- `broker_event_type=reject` -> `Rejected`
- `broker_event_type=execution` -> `PartiallyFilled` or `Filled`
- bridge failure -> `AdapterError::Execution(...)`

### 10.3 `cancel_order(...)`

Cancellation remains on existing compatibility endpoint in this phase.

## 11. Manual CLI Flow

Realign `src/cli/handlers/execution_handler.rs` for `execute_execution_bridge_qmt_live(...)`.

### 11.1 Keep

- explicit `YES` confirmation semantics
- `request target_mode = qmt_live` validation
- live capability gate

### 11.2 Change

Current immediate-complete behavior is removed.

New flow:

1. claim request with `try_start_execution_request(...)`
2. execute live submission through shared task submit service
3. write receipt data into `payload_json`
4. poll `task/result`
5. only after result resolution:
   - `try_complete_execution_request(...)` for broker-facing terminal result
   - `try_fail_execution_request(...)` for bridge failure

### 11.3 CLI Output

After receipt:

- print `task_id`
- print `local_submission_id`
- print `bridge_contract_version`
- print `source_name`

After result:

- print `broker_event_type` or `reason_code`
- print `external_order_id` only when present

Do not print broker order query guidance before broker-facing identity exists.

## 12. Daemon Flow

Do not keep `qmt_live` on the current immediate-completion execution-kernel path.

### 12.1 Preserve Current Kernel Use For

- `paper`
- `mock_live`

### 12.2 Introduce Dedicated `qmt_live` Path

`src/execution/daemon.rs` should route `qmt_live` through a dedicated task-contract flow, such as:

- `execute_qmt_live_request_with_task_contract(...)`

This dedicated path must:

1. claim `Pending -> InProgress`
2. create receipt
3. persist receipt evidence
4. poll `task/result`
5. complete or fail only after result resolution

### 12.3 Final State Rules

- broker-facing `acknowledgement`, `reject`, `execution` -> `Completed`
- bridge `Timeout`, `Unauthorized`, `UnsupportedContractVersion`, `Unavailable`, `InvalidResult` -> `Failed`

Broker reject is a completed business result, not a transport failure.

## 13. `execution_request.payload_json` Extensions

Do not change the runtime DB schema in this phase. Store alignment facts in `payload_json`.

### 13.1 `bridge_task`

- `task_id`
- `local_submission_id`
- `receipt_status`
- `receipt_timestamp`
- `bridge_contract_version`
- `poll_started_at`
- `poll_deadline_at`

### 13.2 `bridge_result`

- `status`
- `source_name`
- `broker_event_type`
- `external_order_id`
- `reason_code`
- `reason_detail`
- `evidence_ref`
- `occurred_at`

These sections are additive and do not replace existing `execution_result` or `execution_error` immediately; they become the factual substrate from which those compatibility views can be derived.

## 14. Test Strategy

### 14.1 Existing Tests To Preserve

- `tests/qmt_bridge_preview_test.rs`
- `tests/qmt_live_gate_test.rs`
- `tests/bridge_client_test.rs`
- `tests/qmt_live_adapter_test.rs`
- `src/cli/handlers/tests/strategy_execution.rs`
- `tests/execution_daemon_test.rs`
- `tests/repo_hygiene_test.rs`

### 14.2 New Test Surface

Add:

- `tests/qmt_task_contract_test.rs`

This test file should cover:

- Bearer + contract-version header behavior
- `task/execute` request shape
- receipt parsing
- `task/result` pending parsing
- broker event parsing
- bridge failure parsing

### 14.3 Adapter Tests

Update `tests/qmt_live_adapter_test.rs` so:

- successful submit returns `PendingSubmit`
- `adapter_order_id` equals `task_id`
- preview-only mode still rejects
- missing `order_submit` still rejects

### 14.4 CLI Tests

Update `src/cli/handlers/tests/strategy_execution.rs` so:

- receipt does not mark request `Completed`
- `bridge_task.task_id` and `bridge_task.local_submission_id` are persisted
- broker-facing result later completes request
- bridge failure later fails request

### 14.5 Daemon Tests

Update `tests/execution_daemon_test.rs` so:

- `qmt_live` no longer completes immediately on receipt
- timeout and auth/version failures become request failure
- broker reject remains completed terminal result

### 14.6 Hygiene Tests

Update `tests/repo_hygiene_test.rs` and related docs assertions so repository docs reflect:

- `qmt-preview` remains preview-only
- `qmt-live` now means receipt-plus-result
- `execution qmt` remains preferred entrypoint
- `execution bridge` remains compatibility entrypoint

## 15. Acceptance Criteria

This design is considered implemented only when all of the following are true:

1. Rust can submit a valid QMT live task via `task/execute`.
2. Rust can poll `task/result/{task_id}` and distinguish:
   - pending
   - bridge failure
   - broker-facing result
3. manual CLI `qmt-live` no longer writes request completion at receipt time.
4. daemon `qmt_live` no longer writes request completion at receipt time.
5. preview path remains unchanged.
6. live capability gate remains unchanged.
7. `execution_request.payload_json` records both receipt evidence and result evidence.
8. documentation matches the new receipt/result semantics.

## 16. Non-Goals

This phase does not:

- remove current broker-style compatibility endpoints
- redesign miniQMT Python implementation
- add generic `target_mode=live`
- redesign runtime DB schema
- redefine preview semantics
- finalize callback-driven broker lifecycle mapping beyond task result polling

## 17. Risks and Mitigations

### 17.1 Risk: Kernel Assumes Immediate Initial Result

Mitigation:

- keep `ExecutionAdapter` trait stable
- isolate `qmt_live` daemon flow into dedicated task-contract orchestration instead of reusing the immediate-completion kernel path

### 17.2 Risk: Identity Drift

Mitigation:

- require `client_order_id` and `local_submission_id` in result identity validation
- never invent `external_order_id`

### 17.3 Risk: Preview Regression

Mitigation:

- explicitly keep preview on existing endpoint and preserve preview tests unchanged

### 17.4 Risk: Documentation Drift

Mitigation:

- keep repo hygiene assertions updated with receipt/result semantics

## 18. Final Recommendation

Implement the miniQMT alignment in four low-risk slices:

1. runtime config, bridge errors, bridge models, bridge client
2. QMT task submit service and live adapter receipt/result semantics
3. manual `execution bridge qmt-live` lifecycle changes
4. daemon `qmt_live` lifecycle changes plus docs/hygiene alignment

This preserves current guarded `qmt_live` safety guarantees, keeps preview stable, and moves the real submission path onto the miniQMT v1 contract without overreaching into unrelated workflow redesign.
