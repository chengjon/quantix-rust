# Phase 27B Live Import Risk Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Import normalized live-account trade/cash ledgers, rebuild a read-only local mirror account, and let risk commands read that mirror explicitly via `--source live_import` without affecting `paper_trade.json`.

**Architecture:** Add a dedicated live-import persistence layer separate from `paper_trade.json` and `risk_state.json`, then build a deterministic full-replay rebuild engine that produces one mirror account view per imported account. `risk` commands remain the only consumer in this slice, and source selection stays explicit (`paper` vs `live_import`) so existing behavior does not drift silently.

**Tech Stack:** Rust, clap, tokio, serde/serde_json, csv, sqlx/sqlite, chrono, existing `src/risk/*`, existing `src/trade/*`, existing CLI handler patterns, GitNexus impact analysis, Graphiti MCP workflow, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-24-phase27b-live-import-risk-mirror-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-24-phase27b-live-import-risk-mirror-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})`; if the result is HIGH/CRITICAL, review the blast radius before proceeding.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Graphiti is mandatory for design/review/debug/handoff memory. If ingest fails or hangs, keep an equivalent local summary and mark `Graphiti backfill required`.
- The repository already contains unrelated dirty files. Stage only files in the active task and never revert unrelated user changes.

## File Map

- `src/core/runtime.rs`
  - Add runtime paths for live-import risk mirror storage if separate files are used.
- `src/risk/models.rs`
  - Add normalized import record models, rebuild result models, source enum, and mirror account view models.
- `src/risk/mod.rs`
  - Re-export new risk import / mirror types.
- `src/risk/import_store.rs`
  - New persistence layer for import batches, normalized records, rebuild audit rows, and mirror account state.
- `src/risk/importer.rs`
  - New CSV/JSON import parser and batch writer.
- `src/risk/rebuild.rs`
  - New deterministic replay engine that rebuilds mirror cash, positions, realized pnl, and fees.
- `src/risk/service.rs`
  - Extend risk service so `status`, `pnl`, and `position` can read `paper` or `live_import` explicitly.
- `src/cli/mod.rs`
  - Add `risk import live-trades`, `risk rebuild live-account`, and `--source/--account` extensions on risk read commands.
- `src/cli/handlers/risk.rs`
  - Add import/rebuild handlers and explicit source switching.
- `src/cli/tests/risk.rs`
  - Add parser coverage for new risk command surface.
- `tests/risk_import_test.rs`
  - New tests for normalized CSV/JSON import, dedupe, and conflict handling.
- `tests/risk_rebuild_test.rs`
  - New tests for full replay rebuild behavior.
- `tests/risk_service_test.rs`
  - Extend service tests for source switching.
- `README.md`
  - Document live-import mirror workflow and explicit `--source` semantics.
- `docs/USER_MANUAL.md`
  - Document import format, rebuild flow, and risk source selection.
- `tests/repo_hygiene_test.rs`
  - Lock new docs wording and CLI examples.

## Implementation Assumptions To Preserve

The following design constraints must remain true:

1. import consumes only normalized project-defined CSV/JSON
2. import dedupe key is `account_id + external_id`
3. rebuild is always a full replay, not incremental in v1
4. failed rebuilds preserve the last successful mirror account state
5. `risk ... --source live_import` requires `--account`
6. imported live data is read-only; it never mutates `paper_trade.json`

## Chunk 1: Import Persistence And Normalized Record Parsing

### Task 1: Add normalized import models, storage, and CSV/JSON parsing

**Files:**
- Modify: `src/risk/models.rs`
- Modify: `src/risk/mod.rs`
- Create: `src/risk/import_store.rs`
- Create: `src/risk/importer.rs`
- Create: `tests/risk_import_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for risk model/service entry points**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "RiskState", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "RiskService", direction: "upstream"})
```

Expected:
- medium blast radius centered on current risk commands/tests. If HIGH/CRITICAL, review direct risk CLI callers before editing.

- [ ] **Step 2: Write failing import tests**

Add import-layer coverage for:
- normalized CSV parse for `trade`
- normalized CSV parse for `cash`
- normalized JSON parse for mixed trade/cash rows
- duplicate skip when `account_id + external_id` repeats identically
- conflict error when same key repeats with different content
- batch summary counts for inserted / skipped / conflicted rows

Suggested assertions:

```rust
assert_eq!(summary.inserted, 2);
assert_eq!(summary.skipped_duplicates, 1);
assert_eq!(summary.conflicts, 1);
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test risk_import_test -- --nocapture
```

Expected:
- FAIL because normalized import models, storage, and parser do not exist yet.

- [ ] **Step 4: Implement normalized import storage and parser**

Add models for:
- import batch metadata
- normalized ledger records
- import summary
- import conflict rows
- source enum for `paper|live_import`

Implement:
- CSV/JSON parsing limited to `trade` and `cash`
- SQLite persistence for:
  - batches
  - normalized records
  - conflicts/errors
- unique constraint on `(account_id, external_id)`

Keep format rules strict:
- unknown `record_type` fails
- unknown `side` or `business_type` fails
- malformed numeric/time fields fail

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --test risk_import_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- focused diff stays inside the new import models/store/parser/tests.

Commit:
```bash
git add src/risk/models.rs src/risk/mod.rs src/risk/import_store.rs src/risk/importer.rs tests/risk_import_test.rs
git commit -m "feat: add live import ledger foundation"
```

## Chunk 2: Full-Replay Mirror Rebuild Engine

### Task 2: Rebuild mirror account state deterministically from imported ledgers

**Files:**
- Create: `src/risk/rebuild.rs`
- Modify: `src/risk/import_store.rs`
- Create: `tests/risk_rebuild_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for rebuild consumers**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "RiskService", direction: "upstream"})
```

Expected:
- low/medium risk because rebuild is new but later read by risk commands.

- [ ] **Step 2: Write failing rebuild tests**

Add rebuild coverage for:
- buy-only replay creates cash reduction and one open position
- buy + sell replay computes realized pnl and remaining position
- deposit + withdraw replay adjusts cash balance
- oversell fails rebuild
- failed rebuild preserves the last successful mirror state
- same import set rebuilt twice produces identical results

Suggested assertions:

```rust
assert_eq!(mirror.cash_balance, dec!(...));
assert_eq!(mirror.positions["000001"].avg_cost, dec!(...));
assert_eq!(mirror.realized_pnl, dec!(...));
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test risk_rebuild_test -- --nocapture
```

Expected:
- FAIL because the rebuild engine and mirror storage do not exist yet.

- [ ] **Step 4: Implement full replay rebuild**

Implement:
- deterministic ordering by logical event time then `external_id`
- trade replay:
  - buy increases position and reduces cash by amount plus fee
  - sell decreases position, increases cash by proceeds minus fee, and books realized pnl
- cash replay:
  - deposit adds cash
  - withdraw subtracts cash
- rebuild audit rows for success/failure
- persistence of mirror account summary + positions

Critical behavior:
- oversell is a hard rebuild failure
- invalid cash semantics are a hard rebuild failure
- failed rebuild never deletes the prior successful mirror snapshot

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --test risk_rebuild_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/risk/rebuild.rs src/risk/import_store.rs tests/risk_rebuild_test.rs
git commit -m "feat: add live import mirror rebuild engine"
```

## Chunk 3: Risk Source Switching And CLI Handlers

### Task 3: Add `risk import`, `risk rebuild`, and `--source live_import`

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/handlers/risk.rs`
- Modify: `src/risk/service.rs`
- Modify: `src/cli/tests/risk.rs`
- Modify: `tests/risk_service_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for risk CLI surface**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_risk_command", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "print_risk_rules", direction: "upstream"})
```

Expected:
- CLI-facing risk should show medium risk, mainly in risk handler/parser/tests.

- [ ] **Step 2: Write failing parser and source-switch tests**

Add parser coverage for:
- `risk import live-trades --account live-001 --input /tmp/live.csv`
- `risk rebuild live-account --account live-001`
- `risk status --source live_import --account live-001`
- `risk pnl --source live_import --account live-001`
- `risk position --source live_import --account live-001`

Add service/handler coverage for:
- `--source paper` keeps current behavior
- `--source live_import` reads rebuilt mirror state
- `--source live_import` without `--account` fails clearly
- `risk import` returns batch summary
- `risk rebuild` returns rebuild summary

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --lib cli::tests::risk:: -- --nocapture
cargo test --test risk_service_test -- --nocapture
```

Expected:
- FAIL because the new risk command surface and source switching do not exist yet.

- [ ] **Step 4: Implement risk CLI source selection**

Extend CLI:
- add `risk import` subcommand tree
- add `risk rebuild` subcommand tree
- add `--source paper|live_import`
- add optional `--account`

Implement handlers:
- `risk import live-trades`
- `risk rebuild live-account`
- `risk status/pnl/position` source selection

Keep behavior explicit:
- `paper` remains the default
- `live_import` requires account id
- no silent fallback from `live_import` to `paper`

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --lib cli::tests::risk:: -- --nocapture
cargo test --test risk_service_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/cli/mod.rs src/cli/handlers/risk.rs src/risk/service.rs src/cli/tests/risk.rs tests/risk_service_test.rs
git commit -m "feat: add live import risk source switching"
```

## Chunk 4: Docs, Hygiene, And Final Verification

### Task 4: Document live-import mirror workflow and lock it with tests

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write failing doc/hygiene expectations**

Add assertions requiring docs to mention:
- `risk import live-trades`
- `risk rebuild live-account`
- `risk status --source live_import --account`
- `risk pnl --source live_import --account`
- `risk position --source live_import --account`
- normalized CSV/JSON import
- explicit `paper|live_import` source switching
- mirror state is separate from `paper_trade.json`
- failed rebuild preserves last successful mirror

- [ ] **Step 2: Run focused doc tests to verify RED**

Run:
```bash
cargo test --test repo_hygiene_test risk_ -- --nocapture
```

Expected:
- FAIL because docs still describe only the Phase 27A local paper-risk baseline.

- [ ] **Step 3: Update README and USER_MANUAL**

Document:
- normalized import format scope
- two-step import/rebuild flow
- explicit source selection
- non-goals:
  - no broker-native raw import
  - no auto-deleverage execution
  - no paper account mutation

- [ ] **Step 4: Re-run focused doc tests to verify GREEN**

Run:
```bash
cargo test --test repo_hygiene_test risk_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Run full verification**

Run:
```bash
cargo test --test risk_import_test --test risk_rebuild_test --test risk_service_test --test repo_hygiene_test -- --nocapture
cargo test --lib cli::tests::risk:: -- --nocapture
cargo test --lib cli::handlers::tests::test_risk_ -- --nocapture
```

Then run broader regression:

```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test strategy_daemon_test --test execution_daemon_test -- --nocapture
```

Expected:
- live-import risk mirror passes without regressing current paper / strategy / execution paths.

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- focused diff stays within risk import/mirror, CLI, docs, and tests.

Commit:
```bash
git add src/risk/models.rs src/risk/mod.rs src/risk/import_store.rs src/risk/importer.rs src/risk/rebuild.rs src/risk/service.rs src/cli/mod.rs src/cli/handlers/risk.rs src/cli/tests/risk.rs tests/risk_import_test.rs tests/risk_rebuild_test.rs tests/risk_service_test.rs README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "feat: add live import risk mirror"
```

## Final Memory

- [ ] **Step 1: Record Graphiti outcome**

Write a conclusion-oriented Graphiti memory for the design and implementation outcome. If ingest fails, preserve an equivalent local summary and mark:

```text
Graphiti backfill required
```

- [ ] **Step 2: Write local completion summary if Graphiti is unavailable**

If Graphiti ingest fails or hangs, append a local completion note documenting:
- commands delivered
- import/rebuild storage added
- mirror-source semantics
- fresh verification commands and results

- [ ] **Step 3: Verify acceptance criteria before declaring the phase complete**

Confirm all acceptance criteria from the design spec are satisfied:
1. [ ] users can import normalized trade/cash ledgers for an account
2. [ ] users can rebuild a deterministic mirror account state
3. [ ] `risk status|pnl|position --source live_import --account <ID>` read that mirror
4. [ ] duplicate imports are idempotent and conflicting duplicates are surfaced
5. [ ] failed rebuilds do not erase the last successful mirror state
6. [ ] current `paper` paths remain behaviorally unchanged
