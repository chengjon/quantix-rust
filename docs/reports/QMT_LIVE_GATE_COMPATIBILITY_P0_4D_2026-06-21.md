# qmt_live Gate Compatibility P0.4d

Date: 2026-06-21

Status: implementation slice

Branch: `feat/p0-4d-qmt-live-gate-compatibility`

## Summary

P0.4d adds a narrow, qmt_live-local structured classification surface for ambiguous bridge `qmt.mode` values in the existing gate path.

The runtime gate still rejects anything other than `bridge qmt.mode=live` before qmt_live submit can proceed. This slice does not change the adapter submit/query/cancel main flow, bridge protocol, response shapes, storage schema, request diagnostics, CLI output wording, `OrderStatus`, or `ExecutionAdapter`.

## Implemented Contract

`src/execution/qmt_live_gate.rs` now exposes `QmtLiveModeFailureKind`:

- `NonLive`
- `Ambiguous`

`QmtLiveGateFailure::mode_failure_kind()` provides structured classification for existing `ModeNotLive` failures:

- `unknown`, `unsupported`, `unavailable`, empty mode, and whitespace-mutated mode strings classify as `Ambiguous`.
- Other non-live mode strings, such as `preview_only`, classify as `NonLive`.
- Non-mode failures return `None`.

The gate continues to return `QmtLiveGateFailure::ModeNotLive { observed_mode }` for non-live bridge modes. This deliberately avoids adding a new failure enum variant, because doing so would force diagnostics-handler changes outside the approved P0.4d boundary.

## Preserved Boundaries

- No request diagnostics formatting rewrite.
- No CLI output wording or response shape change.
- No bridge protocol change.
- No bridge `/capabilities` response schema change.
- No storage schema change.
- No identity metadata change.
- No `OrderStatus` change.
- No `ExecutionAdapter` trait change.
- No qmt_live submit/query/cancel main-flow change outside the existing gate.
- No miniQMT runtime probe or startup self-check implementation.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Pre-edit GitNexus impact was rerun for the modified production symbol:

| Symbol | Risk | Direct callers | Affected processes | Affected modules |
|---|---:|---:|---:|---:|
| `check_bridge_qmt_live_mode` | HIGH | 2 | 2 | 3 |

Affected processes recorded by GitNexus:

- `execute_execution_command`
- `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`

The HIGH risk was expected from P0.4a and explicitly approved by the user before source edits. The GitNexus index reported a stale warning, but the selected symbol and current worktree diff resolved successfully.

## TDD Evidence

RED:

```text
cargo test --test qmt_live_adapter_test qmt_live_gate_classifies_unknown_mode_as_structured_failure
```

The first RED failed because the test asked for a structured ambiguous-mode gate surface that did not exist yet:

```text
no variant named `AmbiguousMode` found for enum `QmtLiveGateFailure`
```

That first candidate exposed an important boundary issue: adding a new `QmtLiveGateFailure` variant would require touching the diagnostics handler match in `src/cli/handlers/execution_handler.rs`, which was outside the P0.4d authorization and non-goal. The final design kept `ModeNotLive` stable and added `QmtLiveModeFailureKind` plus `mode_failure_kind()`.

GREEN:

```text
cargo test --test qmt_live_adapter_test qmt_live_gate_classifies_unknown_mode_as_structured_failure
```

The targeted test passed after `qmt.mode=unknown` remained fail-closed as `ModeNotLive` and was classified as `QmtLiveModeFailureKind::Ambiguous`.

## Verification

- `cargo test --test qmt_live_adapter_test qmt_live_gate_classifies_unknown_mode_as_structured_failure`
- `cargo test --test qmt_live_adapter_test`
- `cargo fmt --check`
- `cargo test qmt_live`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
- `function-tree scope-check`: 9 changed files within active authorization
- `function-tree gate --verbose`: P0.4d implementation-ready, no blocker
- `function-tree validate`: passed
- GitNexus `detect_changes`: LOW risk, 0 affected execution processes

## CI Follow-up

PR #257 first Lint run failed on GitHub stable clippy with `clippy::let_and_return` in the test helper `sample_bridge_client` inside `tests/qmt_live_adapter_test.rs`.

The follow-up fix returned the `BridgeHttpClient::new_with_contract(...).unwrap()` expression directly. It did not change production code or qmt_live behavior.

Post-fix local verification:

- `cargo fmt --check`
- `cargo clippy --test qmt_live_adapter_test -- -D warnings`
- `cargo test --test qmt_live_adapter_test`

Remaining closeout gates after this report:

- FUNCTION_TREE closeout transition to `closed`
- PR CI and master CI, or documented failure
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`
