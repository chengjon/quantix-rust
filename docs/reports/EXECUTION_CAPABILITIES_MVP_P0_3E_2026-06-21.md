# ExecutionCapabilities MVP P0.3e

Date: 2026-06-21

## Summary

P0.3e adds the first static `ExecutionCapabilities` API to the execution adapter boundary.
This is an additive capability declaration only. It does not migrate upper-layer mode checks,
change order lifecycle behavior, or alter any broker, bridge, storage, CLI, or response-shape
contracts.

## New Adapter Contract

`ExecutionAdapter` now exposes:

```rust
fn capabilities(&self) -> ExecutionCapabilities;
```

`ExecutionCapabilities` records the stable static behavior of an adapter:

- `channel`
- `status_source`
- `fill_source`
- `relies_on_broker_api`
- `supports_pending_order_lifecycle`
- `supports_partial_fill`
- `cancel_semantics`

The supporting enums are:

- `ExecutionChannel`
- `ExecutionStatusSource`
- `ExecutionFillSource`
- `ExecutionCancelSemantics`

## Static Capability Matrix

| Adapter | Channel | Status source | Fill source | Broker API | Pending lifecycle | Partial fill | Cancel semantics |
|---|---|---|---|---:|---:|---:|---|
| `PaperExecutionAdapter` | `PaperImmediate` | `LocalImmediateAccounting` | `LocalImmediateAccounting` | false | false | false | `AlreadyFilledOnly` |
| `MockLiveExecutionAdapter` | `MockLive` | `LocalSimulatedLifecycle` | `LocalSimulatedMatcher` | false | true | true | `LocalLifecycle` |
| `QmtLiveExecutionAdapter` | `QmtLive` | `Broker` | `Broker` | true | true | true | `Broker` |

## Explicit Non-Goals

- No replacement of upper-layer `mode == ...` checks.
- No `OrderStatus` changes.
- No order query response-shape changes.
- No storage schema changes.
- No bridge protocol or bridge payload changes.
- No miniQMT runtime capability probing.
- No startup self-check.
- No CLI handler or `request_diagnostics` changes.
- No `.unwrap()` cleanup or unrelated trading architecture changes.

## Verification

The implementation followed RED/GREEN TDD:

- RED: `cargo test --test execution_adapter_capabilities_test` failed because the capability enums and `capabilities()` method did not exist.
- GREEN: the same test passed after the minimal static capability implementation.

Additional local gates:

- `cargo test --test execution_adapter_capabilities_test`
- `cargo test --test execution_kernel_test`
- `cargo test --test qmt_live_adapter_test`
- `cargo test --test mock_live_adapter_test`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
- GitNexus `detect_changes`: LOW risk, no affected processes
