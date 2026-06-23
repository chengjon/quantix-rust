# qmt_live Kill Switch Acceptance P0.5c

Date: 2026-06-22

Status: implementation and local gates complete; PR CI and master CI pending

Branch: `feat/p0-5c-qmt-live-kill-switch`

Base commit: `603b83b31a81a500d709719c80a5b95361eb2702`

## Summary

P0.5c validates the existing safety kill switch as a qmt_live operational safety gate.

This slice intentionally avoids shared kill-switch helper changes. GitNexus classified `build_kill_switch_payload` as HIGH risk and `load_blocking_kill_switch_state` as CRITICAL risk, while the current qmt_live submit helper was LOW risk. The existing runtime behavior already blocks live-capable mutation paths and preserves read-only qmt_live investigation paths, so this slice adds explicit acceptance coverage and operator documentation instead of changing production flow.

## Confirmed Behavior

- Kill switch blocks `qmt_live` live submission through `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`.
- Kill switch blocks `mock_live` strategy execution.
- Kill switch allows `paper` strategy execution.
- Kill switch block payload includes:
  - reason;
  - enabled timestamp;
  - blocked timestamp;
  - target mode.
- qmt_live status/checklist/preflight remains read-only and available when kill switch state is visible.
- qmt_live preview remains available when kill switch is enabled.
- qmt_live read-only query remains available when kill switch is enabled.

## Delivered Artifacts

- `src/cli/handlers/tests/strategy_execution.rs`
  - Adds explicit acceptance tests for qmt_live preview/query remaining available when kill switch is enabled.
- `docs/operations/QMT_LIVE_CANARY_RUNBOOK_2026-06-22.md`
  - Adds kill switch operating rule, pre-canary visibility/readiness requirement, live-submit disabled-state requirement, and incident-response enablement guidance.

## GitNexus Impact

Pre-edit impact:

- `src/safety/kill_switch.rs::build_kill_switch_payload`
  - risk: HIGH
  - direct callers: 2
  - affected processes: 3
  - decision: do not modify in P0.5c.
- `src/safety/kill_switch.rs::load_blocking_kill_switch_state`
  - risk: CRITICAL
  - direct callers: 4
  - affected processes: 4
  - decision: do not modify in P0.5c.
- `src/cli/handlers/execution_handler.rs::execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`
  - risk: LOW
  - direct callers: 2
  - affected processes: 1
  - decision: no production change required because acceptance tests confirm existing behavior.

Final GitNexus `detect_changes` result:

- risk: LOW
- changed symbols: 7
- changed files: 7
- affected processes: 0
- changed file classes: config(2), documentation(4), governance(1), test(1)
- note: stale index warning remained present, but GitNexus reported `fresh_for_staged_diff: true`.

## Test Evidence

Acceptance characterization:

```text
cargo test --lib remains_available_when_kill_switch_enabled -- --test-threads=1
```

Result:

```text
2 passed; 0 failed
```

The tests passed immediately because the existing qmt_live preview/query paths do not consult the kill switch and remain read-only. No production code change was needed.

Focused safety/qmt_live verification:

```text
cargo test --lib kill_switch_enabled -- --test-threads=1
cargo test --lib qmt_live -- --test-threads=1
```

Results:

```text
kill_switch_enabled: 8 passed; 0 failed
qmt_live: 27 passed; 0 failed
```

## Verification

Completed before edits:

- baseline `cargo test`
  - the first context-mode call hit the RPC timeout while the cargo process continued;
  - the cached rerun completed successfully.

Local closeout gates completed:

```text
git diff --check
cargo fmt --check
cargo clippy -- -D warnings
cargo test
npx openspec validate qmt-live-operational-safety-p0-5 --strict
Function Tree scope-check/gate/validate
GitNexus detect_changes
```

## Preserved Boundaries

- No shared kill switch helper changes.
- No safety storage path change.
- No kill switch payload schema change.
- No qmt_live submit/cancel/query runtime behavior change.
- No bridge protocol or bridge response shape change.
- No storage schema change.
- No `OrderStatus` change.
- No `ExecutionAdapter` change.
- No `paper_immediate` or `paper_sim_lifecycle` change.
- No `.unwrap()` cleanup resumed.

## Graphiti Status

Graphiti pre-read for P0.5c kill switch acceptance context was attempted against `quantix_rust_main` and `quantix_rust_docs` and timed out with `Request timed out.`.

Graphiti backfill required if final P0.5c memory ingest also fails.

## Remaining Closeout Gates

- Close Function Tree node.
- Commit implementation.
- PR CI and master CI, or documented failure.
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`.
