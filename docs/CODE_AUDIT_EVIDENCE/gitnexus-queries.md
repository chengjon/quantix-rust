# GitNexus Query Evidence

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Index Status

| Check | Result |
|---|---|
| Repository | `quantix-rust` |
| Index state | `ready` |
| Indexed commit | `b30de31` |
| `gitnexus analyze` | exit 0, `Already up to date` |

## Queries

| Query purpose | Query terms | Result summary | Follow-up |
|---|---|---|---|
| P0 execution and mock/live boundary | `execution live mock adapter bridge risk strategy submit order gate` | Found `Run_execution_command`, `Run_strategy`, and QMT live adapter definitions. | Manually inspected execution adapter and task submit service. |
| Factor gate failure | `factor score cli writes csv output factor_pipeline_test scoring CSV` | Found `run_factor_command` flow and `tests/factor_pipeline_test.rs`. | Added `AUDIT-S2-011`. |
| Carried TUI finding | `menu tui app CLI advertised TUI development in progress` | Found `run_tui_menu` and CLI `Menu { tui }` dispatch. | Fixed by PR #159; `AUDIT-S3-009` is now closed in `findings.csv`. |
| Unsafe hotspot review | `unsafe sync etl pointer parquet dataframe production unsafe blocks` | Found `src/sync/etl.rs` and factor definitions. | Line-level scan classified unsafe blocks as test or `#[cfg(test)]`. |

## Symbol Context Checks

| Symbol | File | Key evidence |
|---|---|---|
| `run_factor_command` | `src/cli/handlers/factor.rs` | Incoming calls include CLI dispatch and factor pipeline tests; outgoing calls include dataset validation, factor catalog, and score/export functions. |
| `run_tui_menu` | `src/cli/handlers/app_shell.rs` | Incoming call from CLI `run`; post-remediation implementation delegates to `crate::tui::run_menu` under `cfg(feature = "tui")` and provides a default-build feature-gating fallback. |
| `run_strategy` | `src/cli/handlers/strategy_handler/catalog.rs` | Participates in multiple `Run_strategy` processes and routes through execution handler, stores, and data loading. |
| `submit_order` | `src/execution/qmt_task_submit_service.rs` | Builds `BridgeTaskExecuteRequest` and submits through bridge task API. |

## Graph Caveat

GitNexus indexes committed files. This audit worktree contains uncommitted and untracked files, so graph evidence was combined with direct file and gate evidence.
