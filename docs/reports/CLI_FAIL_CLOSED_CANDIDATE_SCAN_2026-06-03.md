# CLI Fail-Closed Candidate Scan - 2026-06-03

> Scope: candidate scan only. No production behavior was changed in this slice.
>
> Repository state during scan: `master` was clean except for the in-progress documentation update to `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md`.
>
> Closure status: closed by `docs/reports/CLI_FAIL_CLOSED_SCAN_CLOSURE_2026-06-04.md`; the CLI fail-closed implementation line is paused under that report's stop rule.

## Purpose

Continue the fail-closed CLI hardening line after PR #190, PR #191, and PR #192 by identifying the next small, testable validation gaps.

This report intentionally does not claim that every candidate is a bug. It ranks user-facing CLI paths where unsupported or invalid user input is still represented as `QuantixError::Other`, appears to rely on fallback branches, or may do store/output work before returning the validation error. Each candidate needs a focused red test before any behavior change.

## Scan Method

1. Scanned Rust files under:
   - `src/cli/handlers/`
   - `src/cli/commands/`
2. Looked for function-level candidates containing:
   - `QuantixError::Other`
   - unsupported/invalid wording such as `不支持`, `无效`, `未知`, `只支持`, `unsupported`, `invalid`, `unknown`
   - fallback/default branches such as `_ =>` around user-facing option parsing
3. Ranked candidates higher when:
   - the value looks user-supplied (`--type`, `--source`, `--level`, `--status`, `--algo-type`, etc.)
   - the function touches store/runtime/provider state before the `Other` return
   - no nearby `QuantixError::Unsupported` is present
   - the command has clear CLI testability
4. Cross-checked for existing text/documentation references to estimate whether a hygiene guard likely already exists.

## Summary

- Files scanned: 72 CLI handler/command Rust files
- Function-level candidates found: 40
- Best next target cluster: `quantix algo create --algo-type <unsupported>` / `quantix algo plan --algo-type <unsupported>`
- Best single-command follow-up: `quantix account group set-strategy --strategy <unsupported>`

The highest-signal cluster is `src/cli/handlers/algo.rs`, but the actionable CLI boundary is on commands that actually accept `--algo-type`: `algo create` and `algo plan`. The earlier lifecycle-command read was too broad: `algo start`, `algo pause`, `algo resume`, and `algo cancel` take `--algo-id`, not `--algo-type`, so they are not valid repro commands for unsupported algo-type handling.

## Candidate Table

| Priority | Command | Function | Current Signal | Pre-Error Work Signal | Suggested Test |
|---|---|---|---|---|---|
| P0 | `quantix algo create --algo-type <TYPE>` | `src/cli/handlers/algo.rs::run_algo_create` | `Other`; message contains `不支持的算法类型` | no store/output before error detected | `algo_create_rejects_unsupported_algo_type_as_unsupported` |
| P1 | `quantix algo plan --algo-type <TYPE>` | `src/cli/handlers/algo.rs::run_algo_plan` | `Other`; message contains `不支持的算法类型` | no store/output before preview detected | `algo_plan_rejects_unsupported_algo_type_as_unsupported` |
| P1 | `quantix account group set-strategy --strategy <STRATEGY>` | `src/cli/handlers/account.rs::parse_allocation_strategy` | `Other`; message contains `无效的分配策略` | output-related formatting nearby; no store work before parser error detected | `account_group_set_strategy_rejects_unsupported_strategy_as_unsupported` |
| P1 | `quantix notify send --level <LEVEL>` | `src/cli/handlers/notify.rs::run_notify_send` | `Other`; message contains `无效的通知级别` | no store/output before error detected | `notify_send_rejects_unsupported_level_as_unsupported` |
| P1 | `quantix risk status --source <SOURCE>` | `src/cli/handlers/risk.rs::parse_risk_source` | `Other`; message contains `risk --source 不支持` | no store/output before error detected | `risk_status_rejects_unsupported_source_as_unsupported` |
| P1 | `quantix risk import live-trades --input <FILE>` | `src/cli/handlers/risk.rs::parse_live_import_by_path` | `Other`; message contains `risk import 暂不支持的文件扩展` | no store/output before error detected | `risk_import_live_trades_rejects_unsupported_extension_as_unsupported` |
| P2 | `quantix monitor event list --type <TYPE>` | `src/cli/handlers/monitor_handler.rs::parse_monitor_event_type` | `Other`; message contains `monitor event list 不支持的事件类型` | no store/output before error detected | `monitor_event_list_rejects_unsupported_type_as_unsupported` |
| P2 | `quantix stop history --type <TYPE>` | `src/cli/handlers/shared_support.rs::parse_stop_history_event_type` | `Other`; message contains `未知 stop history event_type` | no store/output before error detected | `stop_history_rejects_unsupported_event_type_as_unsupported` |
| P2 | `quantix strategy request list --status <STATUS>` | `src/cli/handlers/strategy_handler/requests/execution_requests.rs::parse_execution_request_status` | `Other`; message contains `未知 request_status` | request/store work appears in function context | `strategy_request_list_rejects_unsupported_status_as_unsupported` |

## Recommended Next Slice

Start with one P0 algo lifecycle command, preferably:

```text
quantix algo create --code 600519.SH --side buy --quantity 1000 --algo-type iceberg --duration 10
```

Rationale:

- It is user-facing and enum-like.
- Unsupported `--algo-type` currently maps to `QuantixError::Other` on the actionable command path, while the recent hardening line uses explicit `Unsupported` for unsupported option values.
- The actionable boundary is still clear even without prior store mutation: unsupported enum-like CLI input should fail as `Unsupported` before task creation output or context initialization.
- `algo` already has recent validation precedent from `algo plan` hardening.

Suggested TDD shape:

1. Create an isolated HOME/temp runtime fixture.
2. Register or create the minimum valid algo task needed to reach `start`.
3. Run `quantix algo create --code 600519.SH --side buy --quantity 1000 --algo-type iceberg --duration 10`.
4. Assert:
   - command fails
   - stderr contains `Unsupported`
   - stderr contains `不支持的算法类型`
   - stdout does not contain a start/success line such as `算法已启动`
5. Only after red failure, return `QuantixError::Unsupported` from the unsupported algo-type parse branch before any task output or context initialization.

Do not use lifecycle commands as unsupported `--algo-type` repros unless the CLI surface later grows that option; today they only accept `--algo-id`.

## Non-Goals

- Do not bulk-convert every `QuantixError::Other` in CLI handlers.
- Do not clean unrelated Rust warnings.
- Do not change docs until a concrete candidate has a red/green behavior test.
- Do not touch `.mcp.json` or `var/`.
- Do not merge external `/opt/claude/GitNexus/...` review paths into this repository.

## Closure Checks For This Scan

- The scan was read-only for source code.
- This report is the only intended new artifact for the candidate-scan slice.
- Next implementation slice still requires GitNexus impact on the selected symbol before editing.
