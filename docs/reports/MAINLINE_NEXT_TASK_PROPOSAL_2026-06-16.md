# Mainline Next Task Proposal - 2026-06-16

## Context

This proposal applies the user-provided mainline alignment methodology from:

`/mnt/c/Users/John Cheng/Documents/Obsidian Vault/软件开发/项目开发方法/项目主线对齐标准化开发方法论.md`

Current governance checks:

- `FUNCTION_TREE.md` exists and remains the feature/state source of truth.
- Function-tree helper status reports one program: `project-governance`.
- Active gates: `0`.
- `.governance/programs/project-governance/nodes.json` contains 7 historical governance nodes, all `closed`.
- The prior `.unwrap()` cleanup line remains closed and is not part of this proposal.

## Function Tree Evidence

`FUNCTION_TREE.md` status registry currently contains:

- 62 primary registry rows.
- 27 `[已实现]` rows.
- 35 `[部分实现]` rows.
- 16 rows in the `已设计/待实现节点` table.

Important governance rule from `FUNCTION_TREE.md`:

- The status registry table is the state source of truth.
- Evidence trees are only evidence excerpts and do not independently declare usable capability.

## Candidate Assessment

The next task must be a runnable business capability, not documentation cleanup, lint cleanup, metadata refresh, or repository hygiene.

### Recommended Candidate

**Paper order query/cancel runnable closure**

Capability wording:

> Paper order query/cancel can run through the existing local paper-trading flow without returning placeholder `Unsupported` for the paper adapter path.

Why this is the preferred next mainline slice:

- It maps to `FUNCTION_TREE.md` designed/unavailable node `Paper query/cancel`.
- It is a real business-flow closure in the execution/trade domain.
- It avoids external broker, Windows Bridge, ClickHouse, miniQMT registry, and third-party provider dependencies.
- Existing code already has nearby concepts:
  - `src/cli/commands/trade.rs` exposes order-id fields for query/cancel-oriented command variants.
  - `src/cli/handlers/trade_handler.rs` owns local paper trade command dispatch.
  - `src/execution/paper.rs` currently returns explicit unsupported errors for `query_order` and `cancel_order`.
- GitNexus pre-change impact is LOW for the likely production symbols:
  - `src/execution/paper.rs::query_order`: LOW, 1 direct affected symbol, 0 affected processes.
  - `src/execution/paper.rs::cancel_order`: LOW, 1 direct affected symbol, 0 affected processes.
  - `src/cli/handlers/trade_handler.rs::run_trade_command`: LOW, 6 affected symbols, 1 affected process.

### Deferred Candidates

These are not recommended as the immediate next slice:

- `miniQMT artifact -> ClickHouse shadow import`: real capability, but crosses ClickHouse persistence and controlled import policy; higher integration risk.
- `资金流向 / 分红基本面`: real data capability, but depends on external provider/data mapping and broader parser/runtime behavior.
- `AKShare/TDX StockInfo/Kline 拉取`: real data-source capability, but overlaps external source integration and already has separate `tdx-api` bridge progress.
- `多股票并行策略守护`: business capability, but broadens daemon/runtime semantics.
- `AI 多轮对话`, `AI 技能注册`, `Brave / SearXNG 新闻提供者`, `POV / Iceberg`: useful features, but better treated as later P2/P3 capability tracks.
- Documentation, lint, metadata, `.unwrap()` cleanup, and repo hygiene tasks: explicitly not mainline candidates under the current methodology.

## Proposed Authorization Boundary

### Allowed Scope

Only changes needed to make the selected paper query/cancel capability runnable:

- Local paper execution adapter query/cancel behavior.
- Local paper trade service/store reads or status mapping if required.
- CLI dispatch only if an existing paper query/cancel command path is present but not wired.
- Focused tests for the paper query/cancel flow.
- Minimal `FUNCTION_TREE.md` status update only after the capability is verified.

### Non-Goals

- No `.unwrap()` cleanup.
- No repository hygiene cleanup.
- No lint-only or format-only work.
- No QMT live behavior changes.
- No Windows Bridge behavior changes.
- No ClickHouse or miniQMT persistence changes.
- No external broker integration.
- No multi-account or multi-stock daemon expansion.
- No broad CLI manual refresh unless a user-visible command contract changes and verification requires it.

## Proposed Gates

Before source edits:

- Create or authorize a function-tree governance node for this capability.
- Run GitNexus impact on each production symbol before editing it.

Implementation gates:

- Targeted tests for paper query/cancel behavior.
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- GitNexus `detect_changes(scope=all)`

Closeout gates:

- Confirm no unrelated cleanup entered the diff.
- Update `FUNCTION_TREE.md` only if the runnable capability actually closes.
- Record remaining non-goals as backlog/technical debt, not as active work.

## Recommendation

Proceed with `Paper order query/cancel runnable closure` as the next mainline task, subject to explicit authorization to create the function-tree gate and begin implementation.
