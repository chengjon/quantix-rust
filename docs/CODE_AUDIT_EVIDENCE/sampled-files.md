# Sampled Files

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Selection Criteria

- P0 execution and broker bridge paths.
- Gate failures.
- Pattern-scan hotspots.
- Existing open finding continuity.
- Feature-status registry consistency.

## Files Reviewed

| File | Reason | Conclusion |
|---|---|---|
| `src/cli/commands/mod.rs` | CLI `menu --tui` dispatch and advertised option | Supports carried-forward `AUDIT-S3-009`. |
| `src/cli/handlers/app_shell.rs` | TUI handler behavior and CLI menu surface | `run_tui_menu` prints in-progress text and returns success. |
| `src/tui/app.rs` | TUI placeholder implementation | Contains implementation note and simple printed menu. |
| `tests/factor_pipeline_test.rs` | Failing all-target test | Supports `AUDIT-S2-011`. |
| `src/cli/handlers/factor.rs` | Factor score command path | Confirms CLI score output path and GitNexus flow. |
| `src/factor/scoring.rs` | Formatting failure and factor scoring output construction | Supports `AUDIT-S2-010`; relevant to `AUDIT-S2-011`. |
| `src/execution/qmt_live_adapter.rs` | P0 live execution adapter | No new live-to-mock fallback finding confirmed. |
| `src/execution/qmt_task_submit_service.rs` | QMT task payload construction | Side, quantity, price, and order type are mapped into bridge payload. |
| `src/cli/handlers/strategy_handler/catalog.rs` | Strategy execution entry | GitNexus shows integration through execution handler and stores. |
| `src/sync/etl.rs` | `unsafe` hotspot | Unsafe blocks are in `#[cfg(test)]` context. |
| `tests/watchlist_handler_test.rs` | Clippy `await_holding_lock` examples | Warning examples are test-scoped. |
| `FUNCTION_TREE.md` | Feature-status registry consistency | Still identifies itself as the sole feature panorama and status registry. |

## Residual Risk

The sampled review did not exhaustively inspect every P2/P3 module. Unreviewed areas remain residual risk and should be covered by follow-up audit slices if remediation work begins.
