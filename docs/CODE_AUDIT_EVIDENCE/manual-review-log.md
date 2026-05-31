# Manual Review Log

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Scope

This log implements the manual review phases from the 2026-05-15 audit execution spec. The audit was evidence-only and did not modify production code.

## Historical Evidence Review

At audit time, the prior `findings.csv` contained 19 rows and one known unresolved finding: `AUDIT-S3-009`. The audit rechecked that finding and recorded it as unresolved because the same command contract mismatch was visible in code. Post-remediation PR #159 fixed the TUI placeholder path and `findings.csv` now marks `AUDIT-S3-009` fixed.

## P0 Review

### CLI Menu / TUI Contract

Evidence at audit time: `src/cli/commands/mod.rs:67` declares `Menu`; `src/cli/commands/mod.rs:69` describes `--tui`; `src/cli/commands/mod.rs:180` dispatches `Commands::Menu { tui }`; `src/cli/handlers/app_shell.rs:244` implements `run_tui_menu`; `src/cli/handlers/app_shell.rs:245` prints an in-progress message; `src/cli/handlers/app_shell.rs:247` returns `Ok(())`; `src/tui/app.rs:8` still contains the implementation note for a ratatui menu. Post-remediation evidence: `src/cli/handlers/app_shell.rs` now delegates to `crate::tui::run_menu` under `cfg(feature = "tui")`, and `src/tui/app.rs` contains the ratatui menu shell.

Conclusion: `AUDIT-S3-009` was valid during the audit and is now fixed by the post-audit TUI remediation.

### Factor Score CLI Gate Failure

Evidence: `cargo test --all-targets` exits 101; failing test is `factor_score_cli_writes_csv_output`; failure location is `tests/factor_pipeline_test.rs:454`; GitNexus context for `run_factor_command` shows CLI dispatch and factor pipeline test coverage.

Conclusion: confirmed S2 gate and user-facing output finding `AUDIT-S2-011`.

### Execution / QMT Live Submit Path

Evidence: `src/execution/qmt_live_adapter.rs:156` gates submit through `ensure_bridge_qmt_live_mode`; `src/execution/qmt_live_adapter.rs:160` delegates to `QmtTaskSubmitService`; `src/execution/qmt_task_submit_service.rs:75` builds `BridgeTaskExecuteRequest`; lines 83-86 map side, quantity, price, and order type.

Conclusion: no new S0/S1 live-to-mock fallback finding was confirmed in this sampled review.

### Strategy Run Path

Evidence: GitNexus context for `run_strategy` shows incoming call from `run_strategy_command` and outgoing calls into execution handler, data loading, trade store, risk store, and summary printing.

Conclusion: no new strategy P0 finding was confirmed.

## P1 Review

### Formatting Gate

Evidence: `cargo fmt --check` exits 1 and points to `src/factor/scoring.rs:1`. Conclusion: `AUDIT-S2-010`.

### Clippy Diagnostics

Evidence: clippy exits 0 and JSON diagnostics report 220 warnings. Higher-volume warnings are recorded in `cargo-gates.md`. Conclusion: residual maintainability risk but no S0/S1/S2 finding from clippy alone.

### Unsafe Blocks

Evidence: pattern scan found 128 `unsafe {` matches; line-level classification found 74 in `#[cfg(test)]` contexts and 54 in test files. Conclusion: no production runtime unsafe block was confirmed.

## P2/P3 Sampled Review

### Function Status Registry Consistency

Evidence: `FUNCTION_TREE.md` identifies itself as the only function panorama and status registry; targeted check found 58 `状态`, 39 `证据`, and 45 `边界` mentions, with no competing-source term hits. Conclusion: audit artifacts defer feature status to `FUNCTION_TREE.md`.

### Release Build Gate

Evidence: `cargo build --release` was started, exceeded the MCP command window, continued linking the main binary, and was terminated after extended monitoring. Conclusion: `AUDIT-S3-010`.

## Manual Review Conclusion

No S0 or S1 issue was confirmed. The audit found two S2 gate/output findings, one S3 release-gate verification gap, and carried forward the existing S3 TUI contract finding.
