# QMT Live Manual-Intervention Report P0.5e

Date: 2026-06-24

Branch: `feat/p0-5e-qmt-live-manual-intervention-report`

Base commit: `c747b7999958083cfbb474decf6871095769cf4e`

FUNCTION_TREE node: `project-governance/P0.5e`

## Summary

P0.5e adds a read-only qmt_live manual-intervention report for unresolved broker/local ambiguity already persisted in the runtime store. It introduces:

- `quantix execution qmt manual-interventions list`
- `quantix execution qmt manual-interventions show --request-id <ID>`
- `quantix execution qmt manual-interventions show --task-id <TASK_ID>`
- `quantix execution qmt manual-interventions show --local-submission-id <LOCAL_SUBMISSION_ID>`

The report is assembled from existing `ExecutionRequestRecord.payload_json` and `OrderRecord.payload_json.qmt_live` data. It does not call miniQMT, does not call the bridge, does not submit orders, does not cancel orders, does not mark cases resolved, and does not mutate runtime state.

## Persisted Signal Mapping

The first report recognizes these existing persisted signals:

| Category | Persisted source |
| --- | --- |
| `identity_mismatch` | `execution_diagnostics.qmt_live_failure_category=manual_intervention_required` plus reconciliation error text containing identity evidence |
| `broker_unknown_state` | `execution_diagnostics.qmt_live_failure_category=broker_unknown_state` |
| `missing_external_order_id_after_bridge_task_completion` | persisted qmt task identity has a task ID / adapter order ID but no `external_order_id`, with manual-intervention or external-order-id evidence |
| `reconciliation_preserved_local_state` | `qmt_live.reconciliation.last_action=preserved_local_state` or preserved-local reconciliation text |
| `bridge_failure_requires_operator_review` | `execution_diagnostics.qmt_live_failure_category=bridge_failure` or bridge-failure reconciliation text |

Every listed case is emitted with:

- `status=unresolved`
- redacted account label only
- `target_account_raw=null`
- task ID, client order ID, local submission ID, and external order ID when present
- qmt_live failure category and reconciliation evidence when present
- operator guidance

## Operator Guidance

Each case includes the same baseline operator guidance:

- Inspect miniQMT same-day orders before taking action.
- Compare task ID, client order ID, local submission ID, and external order ID.
- avoid resubmission until the ambiguous state is resolved.

This is intentionally guidance only. P0.5e does not implement a resolution workflow.

## Read-Only Guarantee

The list report uses:

- `StrategyRuntimeStore::list_execution_requests(None)`
- `StrategyRuntimeStore::list_orders()`

The show report reuses the same read-only list report and then embeds the existing P0.5d qmt_live audit output for the selected request/task/local-submission lookup.

Regression coverage snapshots execution requests and orders before list/show generation and asserts the records are unchanged after report generation.

## TDD Evidence

RED:

- `cargo test --lib qmt_manual_intervention -- --test-threads=1`
  - failed because `build_execution_bridge_qmt_manual_interventions_list_output`, `build_execution_bridge_qmt_manual_intervention_show_output`, and the CLI command surface were missing.

GREEN:

- `cargo test --lib qmt_manual_intervention -- --test-threads=1`
  - `1 passed; 0 failed`
- `cargo test --lib parses_execution_config_and_daemon_commands -- --test-threads=1`
  - `1 passed; 0 failed`

Full gates:

- `git diff --check`
  - passed
- `cargo fmt --check`
  - passed
- `cargo clippy -- -D warnings`
  - passed
- `cargo test`
  - passed
  - main lib test group included `702 passed; 0 failed`
- `openspec validate qmt-live-operational-safety-p0-5 --strict`
  - passed
- `ft-governance validate`
  - passed
- `ft-governance scope-check`
  - passed; changed files were within active authorization

## GitNexus

Pre-edit impact was LOW for the production command surfaces selected for this slice:

- `ExecutionQmtCommands` in `src/cli/commands/trade.rs`
  - risk: LOW
  - direct callers: 0
  - affected processes: 0
  - affected modules: 0
- `execute_execution_command` in `src/cli/handlers/execution_handler.rs`
  - risk: LOW
  - direct callers: 1
  - affected processes: 1
  - affected modules: 2

`build_execution_bridge_qmt_audit_output` was not found by GitNexus because the project index is behind the recently merged P0.5d symbols. P0.5e reuses that function for show output but does not modify it.

Post-implementation `detect_changes`:

- changed count: 13
- affected count: 0
- changed files: 9
- risk level: LOW
- affected processes: none

The GitNexus index still reported the known stale warning because the indexed commit differs from the current master commit, but the detect_changes metadata confirmed worktree diff resolution against this slice.

## Boundaries

P0.5e does not change:

- qmt_live submit/query/cancel/reconciliation behavior
- bridge protocol or response shapes
- miniQMT runtime probing
- storage schema
- `OrderStatus`
- `ExecutionAdapter`
- paper or paper_sim behavior
- any `.unwrap()` debt

## Closeout Status

Implementation gates and local validation are complete. Remaining project lifecycle work after commit is PR creation, CI verification, merge, final FUNCTION_TREE closeout, and Graphiti closeout memory or local backfill if Graphiti ingest is unavailable.

README/CHANGELOG updates are intentionally left for the P0.5 release-closure authorization because those files are not in the active P0.5e allowed path set.
