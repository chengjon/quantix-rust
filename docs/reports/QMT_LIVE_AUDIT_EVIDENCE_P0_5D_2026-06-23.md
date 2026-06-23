# QMT Live Audit Evidence P0.5d

Date: 2026-06-23

Branch: `feat/p0-5d-qmt-live-audit-evidence`

OpenSpec change: `qmt-live-operational-safety-p0-5`

FUNCTION_TREE node: `project-governance/P0.5d` (closed; final active gates: none)

## Scope

P0.5d adds a read-only qmt_live audit evidence view assembled from already persisted runtime records. It does not change qmt_live submit, query, cancel, reconciliation, bridge protocol, response shapes, storage schema, `OrderStatus`, `ExecutionAdapter`, paper execution, or paper simulation behavior.

User-facing CLI entry points:

```bash
quantix execution qmt audit --request-id <REQUEST_ID>
quantix execution qmt audit --task-id <TASK_ID>
quantix execution qmt audit --local-submission-id <LOCAL_SUBMISSION_ID>

quantix execution bridge qmt-audit --request-id <REQUEST_ID>
quantix execution bridge qmt-audit --task-id <TASK_ID>
quantix execution bridge qmt-audit --local-submission-id <LOCAL_SUBMISSION_ID>
```

The `execution bridge qmt-audit` form is a compatibility alias for the same local audit view.

## Data Sources

The audit builder reads these existing records only:

- `ExecutionRequestRecord`
  - `request_id`
  - `target_mode`
  - `target_account`
  - `payload_json.execution_snapshot`
  - `payload_json.execution_result`
  - `payload_json.execution_diagnostics.qmt_live_failure_category`
- `OrderRecord`
  - `client_order_id`
  - `symbol`
  - `side`
  - `requested_quantity`
  - `requested_price`
  - `order_type`
  - `payload_json.qmt_live.bridge_contract_version`
  - `payload_json.qmt_live.task_identity`
  - `payload_json.qmt_live.reconciliation`

To support task/local-submission lookup without schema changes, `StrategyRuntimeStore::list_orders()` was added as a read-only SELECT using the existing `orders` table and existing `row_to_order` mapper.

## Output Shape

The audit JSON is grouped by lookup, request, order, and qmt_live evidence:

```json
{
  "lookup": {
    "type": "task_id",
    "value": "task-audit-1"
  },
  "request": {
    "request_id": "...",
    "target_mode": "qmt_live",
    "redacted_account_label": "**************4567",
    "target_account_raw": null
  },
  "order": {
    "symbol": "000001",
    "side": "buy",
    "quantity": 800,
    "order_type": "limit",
    "price_intent": "12.34"
  },
  "qmt_live": {
    "local_submission_id": "local-audit-1",
    "client_order_id": "...",
    "task_id": "task-audit-1",
    "external_order_id": "broker-audit-1",
    "bridge_contract_version": "miniqmt.v1",
    "qmt_live_error_category": "broker_unknown_state",
    "reconciliation_decision": "manual_intervention",
    "manual_intervention_marker": true
  }
}
```

Missing optional fields are emitted as JSON `null`. The audit view does not call miniQMT or the bridge; it only reports local runtime evidence already persisted by prior qmt_live flows.

## Redaction

Raw-looking numeric account identifiers are never emitted. The audit view emits:

- `request.redacted_account_label`: masked account label, preserving only the last four digits and capping the mask width.
- `request.target_account_raw`: always `null`.

Non-raw account aliases remain visible as labels so operators can distinguish configured account aliases without exposing raw account identifiers.

## GitNexus Impact

Pre-edit impact analysis:

- `ExecutionBridgeCommands` in `src/cli/commands/trade.rs`: LOW, direct callers 0, affected processes 0.
- `ExecutionQmtCommands` in `src/cli/commands/trade.rs`: LOW, direct callers 0, affected processes 0.
- `execute_execution_command` in `src/cli/handlers/execution_handler.rs`: LOW, direct callers 1, affected processes 1.
- `StrategyRuntimeStore.list_open_orders` in `src/execution/runtime_store/orders.rs`: LOW, direct callers 0, affected processes 0.

The GitNexus index reported the known stale-index warning while also reporting `fresh_for_staged_diff: true`.

Post-edit `detect_changes`:

- Changed files: 10.
- Changed symbols: 11.
- Affected processes: 7.
- Reported risk: HIGH.
- Reason: all affected processes route through `execute_execution_command`, the CLI command dispatcher.

Manual review of the high-risk classification confirmed the affected processes are CLI dispatch flows. The implementation adds a new read-only `qmt audit` / `bridge qmt-audit` branch and does not alter qmt_live submit, query, cancel, reconciliation, bridge protocol, storage schema, order status semantics, or paper execution behavior.

## Verification

RED/GREEN and focused verification:

```bash
cargo test --lib qmt_audit -- --test-threads=1
cargo test --lib parses_execution_config_and_daemon_commands -- --test-threads=1
```

Full closure gates:

```bash
git diff --check
cargo fmt --check
cargo clippy -- -D warnings
cargo test
openspec validate qmt-live-operational-safety-p0-5 --strict
node /root/.codex/skills/myskills/skills/function-tree/scripts/ft-governance.cjs gate --verbose
node /root/.codex/skills/myskills/skills/function-tree/scripts/ft-governance.cjs validate
node /root/.codex/skills/myskills/skills/function-tree/scripts/ft-governance.cjs scope-check
```

All commands above completed with exit code 0. After closeout, FUNCTION_TREE reported `active gates: none` and governance validation passed. GitNexus `detect_changes(scope=all)` completed and reported the CLI-dispatch HIGH classification described above.

## Graphiti

Graphiti pre-read was attempted before implementation against the project memory groups, but the request timed out. Work proceeded using the local OpenSpec, FUNCTION_TREE, GitNexus, and repository evidence. A final Graphiti write or local backfill record is still required after merge-time conclusions converge.
