# qmt_live Preflight Doctor P0.5a

Date: 2026-06-22

Status: implementation slice

Branch: `feat/p0-5a-qmt-live-preflight-doctor`

Base commit: `966e2a09cb050cf7ceacaa9162cfce6820276849`

## Summary

P0.5a adds a read-only qmt_live preflight report to the existing `quantix execution qmt status --checklist` path.

The slice is intentionally narrow. It keeps the existing submit, cancel, query, runtime-store, bridge protocol, response shape, storage schema, `OrderStatus`, `ExecutionAdapter`, `paper_immediate`, and `paper_sim_lifecycle` behavior unchanged. It does not add a separate `doctor` command because the existing checklist command is the current readiness-output owner.

## Implemented Contract

The preflight report is ready only when all of these inputs are true:

- bridge capabilities endpoint is reachable;
- `qmt.enabled=true`;
- `qmt.mode=live`;
- `qmt.supports` contains `order_submit`;
- the local qmt_live adapter declares broker-owned status, fill, and cancel semantics;
- the execution kill switch is disabled.

The report records these fail-closed categories:

- `bridge_unreachable`
- `qmt_capability_missing`
- `qmt_disabled`
- `qmt_mode_not_live`
- `qmt_order_submit_missing`
- `qmt_live_capability_mismatch`
- `kill_switch_enabled`

When `--checklist` is used, the CLI prints the existing bridge capability JSON plus a `qmt_live_preflight` block and a compact text summary:

```text
QMT live preflight
readiness=ready|not_ready
failure_category=<category|none>
bridge_reachable=<true|false>
bridge_contract_version=unknown
capability_source=bridge:/api/v1/capabilities
kill_switch=enabled|disabled
```

If the bridge is unreachable and `--checklist` is present, the command still prints a read-only preflight report instead of submitting, canceling, or mutating runtime state. Ordinary `quantix execution qmt status` without `--checklist` preserves the existing error-return behavior.

## Preserved Boundaries

- No order submission.
- No order cancellation.
- No runtime store mutation.
- No broker state write.
- No bridge protocol or bridge response shape change.
- No storage schema change.
- No `OrderStatus` change.
- No `ExecutionAdapter` trait change.
- No paper execution behavior change.
- No `paper_sim_lifecycle` implementation.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Pre-edit impact:

| Symbol | Risk | Direct callers | Affected processes |
|---|---:|---:|---:|
| `execute_execution_bridge_status` | LOW | 1 | 2 |
| `format_qmt_promotion_checklist` | LOW | 1 | 0 |

No HIGH or CRITICAL impact target was edited.

Final GitNexus `detect_changes` result:

- risk: MEDIUM
- changed files: 9
- affected processes: 4
- rationale: expected qmt_live status/preflight handler file participation; GitNexus also mapped nearby qmt_live submit-path symbols because the new preflight helpers live in the same CLI handler file
- stale-index note: GitNexus reported the known `current_commit_differs_from_indexed_commit` warning

## TDD Evidence

RED:

```text
cargo test --lib cli::handlers::tests::strategy_bridge::test_qmt_live_preflight_report -- --test-threads=1
```

The new tests failed to compile before implementation because the preflight API did not exist:

```text
unresolved imports build_qmt_live_preflight_report, format_qmt_live_preflight_report, QmtLivePreflightFailureCategory
```

GREEN:

```text
cargo test --lib cli::handlers::tests::strategy_bridge::test_qmt_live_preflight_report -- --test-threads=1
cargo test --lib cli::handlers::tests::strategy_bridge -- --test-threads=1
```

The focused tests now cover the ready case, kill-switch visibility, and all seven fail-closed categories.

## Verification

- `cargo test --lib cli::handlers::tests::strategy_bridge::test_qmt_live_preflight_report -- --test-threads=1`
- `cargo test --lib cli::handlers::tests::strategy_bridge -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `git diff --check`
- `cargo test`
- `npx openspec validate qmt-live-operational-safety-p0-5 --strict`
- `function-tree scope-check`: changed files within active authorization
- `function-tree gate --verbose`: active gates none after closeout
- `function-tree validate`: passed
- GitNexus `detect_changes`: MEDIUM risk, 4 affected processes, expected qmt_live status/preflight handler scope

## Graphiti Status

Graphiti pre-read for P0.5a qmt_live preflight context was attempted against `quantix_rust_main` and `quantix_rust_docs` and timed out with `Request timed out.`.

Graphiti backfill required if final P0.5a memory ingest also fails.

## Remaining Closeout Gates

- Close FUNCTION_TREE node.
- Commit implementation.
- PR CI and master CI, or documented failure.
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`.
