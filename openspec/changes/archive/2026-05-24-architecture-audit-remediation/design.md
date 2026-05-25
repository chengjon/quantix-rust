## Context

The reviewed audit set defines the source of truth for architecture remediation:

- `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md` is the primary findings and roadmap document.
- `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md` is the correction baseline and takes precedence for refined architecture recommendations.
- `docs/superpowers/specs/2026-05-23-architecture-audit-design.md` defines the three-document relationship and issue gate.

Issue #63 is closed and represents the completed correction baseline. Issues #64-#72 remain the execution queue.

## Goals / Non-Goals

**Goals:**

- Make OpenSpec the development control plane for all architecture remediation work from the audit.
- Keep each code slice tied to a requirement, a GitHub issue, characterization tests, and validation evidence.
- Preserve current behavior while improving dependency direction, module ownership, and error handling.
- Prevent large-file splits from creating shallow pass-through modules.

**Non-Goals:**

- Do not re-audit the whole repository before each task.
- Do not move the full `Strategy` trait into `core`.
- Do not create new GitHub issues from stale pre-review audit rows.
- Do not remediate safety patterns solely because a pattern count is high.
- Do not perform broad formatting, naming, or cosmetic cleanup while closing a focused task.

## Decisions

- **OpenSpec change boundary:** `architecture-audit-remediation` is the umbrella change for issues #64-#72. Individual tasks remain small and issue-linked.
- **Resolution precedence:** the review report overrides the primary audit when recommendations conflict.
- **Characterization first:** #64 precedes or accompanies each architecture seam change so behavior is captured before refactoring.
- **Signal extraction scope:** #65 extracts only the shared `Signal` value type into `core`; the async `Strategy` interface stays in `strategy`.
- **Domain/storage separation:** #66 keeps risk industry domain types pure and moves SQLite persistence behind storage/adapter boundaries.
- **Calculation/data acquisition separation:** #68 and #69 keep risk volatility and market strength calculations independent from strategy runtime, DB lookup, EastMoney fetching, and anomaly/risk acquisition.
- **Dependency direction cleanup:** #70 removes upward production dependencies from `db/clickhouse` to `market` and from `core/runtime` to `test_support`.
- **CLI boundary cleanup:** #67 thins `cli/handlers/mod.rs` into command registry and context modules without changing command behavior.
- **Safety cleanup:** #72 handles true runtime risks with focused tests and validations. Existing progress: `src/trade/service.rs` initialized-account expects are remediated.
- **Large-file splits last:** #71 starts only after relevant seams and characterization tests are stable.

## Phase 3 CLI Boundary Audit

Current evidence for #67 was captured on 2026-05-23 before starting CLI handler extraction:

| Current location | Evidence | Extraction target |
| --- | --- | --- |
| `src/cli/handlers/mod.rs:1-110` | Root-level prelude imports command enums plus domain types from analysis, bridge, data, DB, execution, fundamentals, market, monitor, risk, sources, stop, strategy, tasks, trade, and watchlist. | Move domain-specific imports into the concrete handler or existing support module that consumes them. Keep only command entrypoint signature imports that are required by the root. |
| `src/cli/handlers/mod.rs:112-258` | Module declarations, re-exports, and handler-module helper imports are mixed together. | Keep the root as module declaration plus stable re-export glue; move private helper imports into their owning modules where possible. |
| `src/cli/handlers/mod.rs:260-409` | `create_clickhouse_client` and `run_strategy_command` own strategy command dispatch in the root. | Move strategy dispatch and ClickHouse client construction into `strategy_handler` or a strategy-handler support submodule, then re-export the stable `run_strategy_command` entrypoint. |
| `src/cli/handlers/mod.rs:437-507` | `run_execution_command` owns execution command dispatch in the root. | Move execution dispatch into `execution_handler` or an execution-handler support submodule, then re-export the stable `run_execution_command` entrypoint. |
| `src/cli/handlers/mod.rs:423-435` and `510-604` | `StrategyRiskBridge` and `StrategyFillDeltaBridge` bridge strategy execution with risk/trade behavior in the root. | Move bridge glue beside the strategy/execution handler code that uses it; avoid introducing context types until the extracted ownership is clear. |
| `src/cli/handlers/mod.rs:606-703` | Shared trade/risk helpers (`create_trade_store`, `create_risk_store`, `sync_risk_from_trade_store`, `load_initialized_trade_account`, `load_trade_quote_prices`, `build_risk_account_snapshot`, `build_projected_buy_impact`) live in the root. | Move shared helper code into `shared_support` or narrower handler-owned support modules, based on actual call sites. |

`HelperContext`, `TradeContext`, and `RiskContext` are not existing symbols in the current code. They must not be treated as pre-existing extraction targets. If context structs are introduced later, their names and ownership must come from the extracted responsibilities above.

## Risks / Trade-offs

- Several target modules participate in execution and risk flows. GitNexus impact must be run before symbol edits and HIGH/CRITICAL results must pause for explicit risk handling.
- Some safety hotspots are test-only or macro-generated. Each remediation must reclassify production risk before changing code.
- File splitting can make navigation worse if it creates pass-through modules. Splits must deepen ownership and preserve public interfaces.
- The worktree may contain unrelated dirty changes. Each slice must use path-scoped diff checks and must not revert user changes.
