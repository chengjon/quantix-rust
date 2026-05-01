# Project Cleanup Mainline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clean up the repository structure, documentation surface, code hotspots, and CI ownership without breaking the current strategy-to-execution mainline.

**Architecture:** The cleanup is split into two layers. First, stabilize the repository contract by defining canonical entry documents, archive rules, and hygiene checks. Then clean the codebase along the active product mainline from CLI entrypoints into strategy and execution, followed by adjacent operator domains and finally the broader discovery/data plane.

**Tech Stack:** Rust, Cargo, Markdown, GitHub Actions, GitNexus impact/detect_changes, Graphiti MCP

---

## Cleanup Ordering Confirmation Gate

Before implementing Tasks 3-6, confirm the cleanup ordering with the user.

**Recommended ordering**

1. Repository entrypoints and docs hygiene
2. `cli -> strategy -> execution -> runtime_store`
3. `risk -> stop -> trade -> monitor -> monitoring`
4. `watchlist -> screener -> market -> analysis`
5. `sources -> data -> db -> sync -> io -> tasks`
6. Scripts, workflows, and archive follow-up

**Why this is recommended**

- It matches the current roadmap mainline in [ROADMAP.md](/opt/claude/quantix-rust/ROADMAP.md).
- It reduces the risk of cleaning peripheral modules while the execution path still has oversized hotspots.
- It lets repository hygiene and docs cleanup happen once, before any feature-slice code moves.

**Alternative ordering**

1. `core -> data -> sources`
2. `analysis -> strategy`
3. `execution -> account -> risk`
4. Operator/user slices
5. Docs/workflows/scripts

Only use the alternative if the user explicitly wants a dependency-bottom-up cleanup instead of a product-mainline cleanup.

### Task 1: Canonical Entry Documents And Retention Policy

**Files:**
- Create: `docs/archive/README.md`
- Modify: `README.md`
- Modify: `ROADMAP.md`
- Modify: `docs/FUNCTION_MAP.md`
- Modify: `tests/repo_hygiene_test.rs`
- Move or mark legacy: `docs/DEVELOPMENT_ROADMAP.md`
- Move or mark legacy: `docs/ROADMAP_REVIEW.md`

- [ ] **Step 1: Write the failing hygiene assertions for canonical entrypoints**

Run: `cargo test repo_hygiene_test -- --test-threads=1`
Expected: FAIL after tightening the test to require a single canonical roadmap entrypoint and an archive index.

- [ ] **Step 2: Make the top-level navigation unambiguous**

Update `README.md` so it points to one canonical roadmap, one function map, one user manual, and one archive index.

- [ ] **Step 3: Retire duplicate roadmap entrypoints**

Either move `docs/DEVELOPMENT_ROADMAP.md` and `docs/ROADMAP_REVIEW.md` under `docs/archive/`, or keep them in place with an explicit legacy banner and a link back to `ROADMAP.md`.

- [ ] **Step 4: Add archive retention rules**

Create `docs/archive/README.md` that defines what belongs in archive, what stays canonical, and how new historical phase material should be filed.

- [ ] **Step 5: Re-run the hygiene test**

Run: `cargo test repo_hygiene_test -- --test-threads=1`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add README.md ROADMAP.md docs/FUNCTION_MAP.md docs/archive/README.md docs/DEVELOPMENT_ROADMAP.md docs/ROADMAP_REVIEW.md tests/repo_hygiene_test.rs
git commit -m "docs: define canonical repo entrypoints and archive policy"
```

### Task 2: Archive Historical Plans And Reduce Docs Noise

**Files:**
- Create: `docs/archive/plans/README.md`
- Create: `docs/archive/reports/README.md`
- Create: `docs/archive/ad-hoc/README.md`
- Move: `docs/plans/*.md`
- Move: `docs/reports/*.md`
- Move or classify: `docs/AUDIT_OPTIMIZATION_PLAN_2026-03-29.md`
- Move or classify: `docs/INDICATOR_PIPELINE_MVP_PLAN.md`
- Move or classify: `docs/INDICATOR_PIPELINE_OPTIMIZATION_PLAN.md`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`

- [ ] **Step 1: Inventory the docs buckets before moving files**

Run: `find docs -maxdepth 2 -type f | sort`
Expected: the inventory identifies canonical docs, active plans, archived reports, and ad-hoc notes.

- [ ] **Step 2: Create archive bucket READMEs**

Document what goes into `archive/plans`, `archive/reports`, and `archive/ad-hoc`.

- [ ] **Step 3: Move completed historical material out of the primary docs surface**

Use `git mv` so history is preserved and the active docs surface becomes smaller and easier to navigate.

- [ ] **Step 4: Repair links after the moves**

Update `README.md` and `docs/USER_MANUAL.md` only where they still point at moved files.

- [ ] **Step 5: Verify that the canonical docs still read cleanly**

Run: `cargo test repo_hygiene_test -- --test-threads=1`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add docs README.md
git commit -m "docs: archive historical plans and reports"
```

### Task 3: Clean The Mainline CLI Slice (`strategy` / `execution`)

**Files:**
- Modify: `src/cli/handlers/mod.rs`
- Create: `src/cli/handlers/strategy.rs`
- Create: `src/cli/handlers/execution.rs`
- Create: `src/cli/handlers/output.rs`
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/tests/strategy.rs`
- Modify: `src/cli/tests/execution.rs`
- Modify: `src/cli/handlers/tests/mod.rs`

- [ ] **Step 1: Run GitNexus impact before editing CLI entry symbols**

Run impact for `run_strategy_command` and `run_execution_command`.
Expected: capture direct callers, affected processes, and risk level before any edit.

- [ ] **Step 2: Write or extend failing tests around the CLI boundary**

Run: `cargo test cli::tests::strategy cli::tests::execution --all-features`
Expected: FAIL once tests begin asserting the new module boundaries or exported behavior.

- [ ] **Step 3: Move strategy-specific handlers out of `mod.rs`**

Create `src/cli/handlers/strategy.rs` for strategy config, daemon, signal, request, and service subcommands.

- [ ] **Step 4: Move execution-specific handlers out of `mod.rs`**

Create `src/cli/handlers/execution.rs` for execution config, daemon, bridge, and preview/live query helpers.

- [ ] **Step 5: Move shared formatting helpers into `output.rs`**

Only move shared output helpers that are used by both strategy and execution paths. Leave unrelated domains in `mod.rs` for later tasks.

- [ ] **Step 6: Slim the handler test support file**

Move strategy/execution-specific test scaffolding out of `src/cli/handlers/tests/mod.rs` if the file still grows after the split.

- [ ] **Step 7: Re-run focused tests**

Run: `cargo test strategy --all-features`
Run: `cargo test execution --all-features`
Expected: PASS

- [ ] **Step 8: Run GitNexus change detection**

Run: `gitnexus detect-changes --scope all`
Expected: only CLI strategy/execution symbols and their expected tests/processes are affected.

- [ ] **Step 9: Commit**

```bash
git add src/cli/handlers src/cli/mod.rs src/cli/tests
git commit -m "refactor(cli): split strategy and execution handlers"
```

### Task 4: Clean The Runtime Store Hotspot

**Files:**
- Modify: `src/execution/mod.rs`
- Delete: `src/execution/runtime_store.rs`
- Create: `src/execution/runtime_store/mod.rs`
- Create: `src/execution/runtime_store/schema.rs`
- Create: `src/execution/runtime_store/signals.rs`
- Create: `src/execution/runtime_store/requests.rs`
- Create: `src/execution/runtime_store/orders.rs`
- Create: `src/execution/runtime_store/checkpoints.rs`
- Modify: `tests/execution_runtime_store_test.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Run GitNexus impact for `StrategyRuntimeStore`**

Expected: identify all direct callers and affected execution flows before splitting storage responsibilities.

- [ ] **Step 2: Write failing store regression tests**

Run: `cargo test execution_runtime_store_test --all-features`
Expected: FAIL when new module boundaries are introduced before logic is moved.

- [ ] **Step 3: Move schema SQL into `schema.rs`**

Keep SQL constants and migration/bootstrap helpers together.

- [ ] **Step 4: Split store responsibilities by data domain**

Use `signals.rs`, `requests.rs`, `orders.rs`, and `checkpoints.rs` to separate read/write flows while preserving the public `StrategyRuntimeStore` API from `mod.rs`.

- [ ] **Step 5: Re-run execution runtime tests**

Run: `cargo test execution_runtime_store_test execution_kernel_test --all-features`
Expected: PASS

- [ ] **Step 6: Run GitNexus change detection**

Run: `gitnexus detect-changes --scope all`
Expected: only execution runtime storage flows and directly related tests are affected.

- [ ] **Step 7: Commit**

```bash
git add src/execution tests/execution_runtime_store_test.rs tests/execution_kernel_test.rs
git commit -m "refactor(execution): split runtime store by domain"
```

### Task 5: Clean Adjacent Operator Domains (`risk` / `stop` / `trade` / `monitor`)

**Files:**
- Modify: `src/risk/service.rs`
- Modify: `src/stop/service.rs`
- Modify: `src/trade/service.rs`
- Modify: `src/monitor/service.rs`
- Modify: `src/monitoring/notification.rs`
- Modify: `tests/risk_service_test.rs`
- Modify: `tests/stop_service_test.rs`
- Modify: `tests/trade_service_test.rs`
- Modify: `tests/monitor_service_test.rs`

- [ ] **Step 1: Confirm the user wants this slice immediately after the execution mainline**

Do not start this task until the user confirms the mainline ordering.

- [ ] **Step 2: Run GitNexus impact for each touched service symbol**

Capture risk for `RiskService`, `StopService`, `TradeService`, and `MonitorService` before editing.

- [ ] **Step 3: Clean one domain at a time**

Reduce file size, remove duplicate formatting logic, and push CLI-only presentation concerns out of domain services.

- [ ] **Step 4: Re-run the domain tests after each sub-slice**

Run the smallest relevant test target after each edit instead of batching all domains together.

- [ ] **Step 5: Commit per domain**

Use one commit per domain so the cleanup stays reversible.

### Task 6: Clean Discovery And Data Plane (`watchlist` / `screener` / `market` / `analysis` / `sources`)

**Files:**
- Modify: `src/watchlist/*`
- Modify: `src/screener/*`
- Modify: `src/market/*`
- Modify: `src/analysis/*`
- Modify: `src/sources/*`
- Modify: related tests in `tests/`

- [ ] **Step 1: Confirm this slice stays behind the strategy/execution mainline**

Do not pull discovery/data cleanup ahead of the mainline unless the user changes the ordering.

- [ ] **Step 2: Triage by hotspot size and coupling**

Start with large or high-change files, not with the smallest modules.

- [ ] **Step 3: Preserve behavior-first boundaries**

Avoid renames or moves that only change aesthetics without shrinking maintenance cost.

- [ ] **Step 4: Re-run focused tests per module**

Use module-specific test targets rather than one full-suite batch after every small cleanup.

- [ ] **Step 5: Commit per module family**

Keep watchlist, screener, market, analysis, and sources in separate commits.

### Task 7: Consolidate Workflows, Scripts, And Repo Hygiene

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/audit.yml`
- Modify: `.github/workflows/cleanup.yml`
- Modify: `.github/workflows/docker.yml`
- Modify: `tests/ci_workflow_structure_test.rs`
- Modify: `scripts/verify_features.sh`
- Modify: `scripts/health-check.sh`
- Modify: `README.md`

- [ ] **Step 1: Write failing CI structure assertions first**

Run: `cargo test ci_workflow_structure_test -- --test-threads=1`
Expected: FAIL once the workflow ownership model is tightened.

- [ ] **Step 2: Remove duplicate workflow responsibility**

Keep `ci.yml` focused on fast and layered CI, and avoid duplicating scheduled audit behavior that already belongs in `audit.yml`.

- [ ] **Step 3: Classify scripts as active, operator-only, or legacy**

If a script is no longer part of the documented operator path, either archive it or mark it legacy before deleting it.

- [ ] **Step 4: Re-run workflow structure tests**

Run: `cargo test ci_workflow_structure_test repo_hygiene_test -- --test-threads=1`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .github/workflows tests scripts README.md
git commit -m "chore(ci): align workflow and script ownership"
```

## Verification Gate Before Declaring Cleanup Complete

- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run the focused tests touched by each cleanup commit
- [ ] Run `gitnexus detect-changes --scope all` before the final commit of each cleanup slice
- [ ] Re-check that `README.md`, `ROADMAP.md`, and `docs/FUNCTION_MAP.md` still agree on the active mainline

## Notes For Execution

- Do not start Tasks 3-7 until the user confirms the function-tree cleanup ordering.
- Prefer small reversible commits over one repo-wide cleanup commit.
- Treat `docs/plans/`, `docs/reports/`, and `docs/superpowers/` as different retention classes; do not merge them blindly.
- If any cleanup step requires a symbol rename, use GitNexus rename preview first instead of text search.
