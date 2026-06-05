# Phase29A Branch Closure - 2026-06-05

## Decision

Close the live remote `phase29a-*` branch set as covered by the current
`master` implementation.

This is a capability-coverage closure, not a byte-for-byte merge of the old
branch tips. Several old `phase29a` commits remain patch-unique versus
`master`, but current `master` already carries the evolved strategy paper
execution/runtime architecture and the corresponding docs/tests. Pulling the
old branch tips directly would reintroduce stale baseline drift.

## Branches Closed

| Remote branch | Tip | Tip subject |
| --- | --- | --- |
| `origin/phase29a-guidance` | `daf5b83aeb86` | `docs: add phase29a strategy paper guidance` |
| `origin/phase29a-guidance-refresh` | `82898fa8f750` | `docs: add phase29a strategy paper guidance` |
| `origin/phase29a-guidance-refresh2` | `c9a9f5578bf8` | `docs: add phase29a strategy paper guidance` |
| `origin/phase29a-runtime-path` | `b6268b333e77` | `feat: add strategy runtime config paths` |
| `origin/phase29a-runtime-store` | `21ec10a905b7` | `feat: add phase29a execution runtime store` |
| `origin/phase29a-signal-translation` | `f45b3023b824` | `feat: add phase29a strategy signal translation` |
| `origin/phase29a-paper-kernel` | `c262b67cf2c6` | `feat: add phase29a paper execution kernel` |
| `origin/phase29a-paper-mode` | `e17258a234ae` | `feat: wire phase29a strategy paper mode` |

The implementation stack was cumulative:

```text
b6268b333e77 feat: add strategy runtime config paths
21ec10a905b7 feat: add phase29a execution runtime store
f45b3023b824 feat: add phase29a strategy signal translation
c262b67cf2c6 feat: add phase29a paper execution kernel
e17258a234ae feat: wire phase29a strategy paper mode
```

The containment graph confirmed:

```text
phase29a-runtime-path -> phase29a-runtime-store
phase29a-runtime-store -> phase29a-signal-translation
phase29a-signal-translation -> phase29a-paper-kernel
phase29a-paper-kernel -> phase29a-paper-mode
```

The guidance branches were single-commit documentation lines. The
`phase29a-guidance-refresh` and `phase29a-guidance-refresh2` tips had the same
stable patch-id and were therefore equivalent guidance refreshes.

## Coverage Matrix

| Old branch capability | Current `master` coverage |
| --- | --- |
| Strategy runtime config paths | Covered by `CliRuntime` loading in `src/core/runtime/init.rs`, `src/core/runtime/settings.rs`, and strategy/runtime path consumers such as `src/cli/handlers/strategy_handler.rs`. |
| Execution runtime store | Covered by `src/execution/runtime_store/mod.rs` and related split modules. The old branch's store naming evolved; current master exposes `StrategyRuntimeStore` rather than the old flat store shape. |
| Strategy signal translation | Covered by `translate_signal`, `StrategySignalRecord`, `ExecutionRequestRecord`, and signal/request tests in current execution modules. |
| Paper execution kernel | Covered by `src/execution/kernel.rs`, paper adapter code, and `tests/execution_kernel_test.rs`. |
| Strategy paper mode CLI wiring | Covered by `execute_strategy_run_with_risk_service_and_kill_switch` and strategy handler tests. |
| Phase29A guidance docs | Covered by current `README.md`, `docs/USER_MANUAL.md`, `docs/QUICKSTART.md`, and repo hygiene tests. The meaningful added guidance lines from the old guidance commits are present in current docs. |

## Verification

The following target gates were run on current `master` before closure:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test execution_runtime_store_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test execution_kernel_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::handlers::tests::strategy_execution::test_strategy_paper_allows_execution_when_kill_switch_enabled
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::handlers::tests::strategy_requests::test_execute_strategy_signal_approve_allows_paper_when_kill_switch_enabled
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test execution_kernel_test strategy_runtime_returns_latest_signal_for_ma_cross
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test execution_kernel_test request_prepared_execution_supports_paper_mode
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test readme_documents_phase29_strategy_paper_boundary
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test user_manual_documents_phase29_strategy_paper_commands
```

Results:

```text
execution_runtime_store_test: 23 passed, 0 failed
execution_kernel_test: 23 passed, 0 failed
test_strategy_paper_allows_execution_when_kill_switch_enabled: 1 passed, 0 failed
test_execute_strategy_signal_approve_allows_paper_when_kill_switch_enabled: 1 passed, 0 failed
strategy_runtime_returns_latest_signal_for_ma_cross: 1 passed, 0 failed
request_prepared_execution_supports_paper_mode: 1 passed, 0 failed
readme_documents_phase29_strategy_paper_boundary: 1 passed, 0 failed
user_manual_documents_phase29_strategy_paper_commands: 1 passed, 0 failed
```

GitNexus scope check before this report:

```text
detect_changes(scope=all): changed_files=0, affected_count=0, risk_level=none, stale=false
```

## Archive Tags

The old tips are preserved under archive tags:

| Archive tag | Tip |
| --- | --- |
| `archive/phase29a-guidance-20260605` | `daf5b83aeb86` |
| `archive/phase29a-guidance-refresh-20260605` | `82898fa8f750` |
| `archive/phase29a-guidance-refresh2-20260605` | `c9a9f5578bf8` |
| `archive/phase29a-runtime-path-20260605` | `b6268b333e77` |
| `archive/phase29a-runtime-store-20260605` | `21ec10a905b7` |
| `archive/phase29a-signal-translation-20260605` | `f45b3023b824` |
| `archive/phase29a-paper-kernel-20260605` | `c262b67cf2c6` |
| `archive/phase29a-paper-mode-20260605` | `e17258a234ae` |

## Closure Boundary

This closure removes stale remote branch-board noise for `phase29a`.

It does not reopen `phase29b`, `phase29c`, or `phase27d`, and it does not
perform cosmetic cleanup. After this closure, the remaining live branch-board
work should be the `phase29b` stack only.
