# Phase 28A Trade Observability Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the smallest read-side observability layer for paper trade: history, fee rows, account overview, and optional current valuation views.

**Architecture:** Keep trade accounting unchanged. Add a thin `trade::reporting` read-side module for deterministic aggregation from `PaperTradeState`, then let CLI handlers optionally overlay best-effort live prices only when `--current` is requested.

**Tech Stack:** Rust, clap, chrono, rust_decimal, serde JSON, existing trade storage/service, existing best-effort quote lookup path, Markdown docs.

---

## File Map

- Create: `src/trade/reporting.rs`
- Create: `tests/trade_reporting_test.rs`
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/handlers.rs`
- Modify: `src/trade/mod.rs`
- Modify: `src/trade/models.rs`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`
- Modify: `src/cli/tests/trade.rs`

Do not change:

- `src/trade/storage.rs` JSON schema
- `src/trade/service.rs` write semantics for `init/reset/buy/sell`
- `src/risk/*` behavior

## Chunk 1: CLI Surface

### Task 1: Extend `TradeCommands`

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/tests/trade.rs`

- [ ] **Step 1: Write the failing parser tests**

Add parser coverage for:

- `quantix trade history`
- `quantix trade history --code 000001 --limit 5`
- `quantix trade fees`
- `quantix trade fees --code 600000 --limit 10`
- `quantix trade overview`
- `quantix trade overview --current`
- `quantix trade position --current`

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test cli::tests::trade -- --nocapture
```

Expected: FAIL because the new `TradeCommands` variants do not exist yet.

- [ ] **Step 3: Add the minimal CLI surface**

Update `src/cli/mod.rs`:

- add `History { code: Option<String>, limit: Option<usize> }`
- add `Fees { code: Option<String>, limit: Option<usize> }`
- add `Overview { current: bool }`
- change `Position` from unit variant to `Position { current: bool }`

Use `#[arg(long)] current: bool` for both `overview` and `position`.

- [ ] **Step 4: Re-run parser tests**

Run:

```bash
cargo test cli::tests::trade -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli/mod.rs src/cli/tests/trade.rs
git commit -m "feat: add phase28 trade observability cli surface"
```

## Chunk 2: Read-Side Reporting Core

### Task 2: Add reporting rows and aggregate logic

**Files:**
- Create: `src/trade/reporting.rs`
- Modify: `src/trade/mod.rs`
- Modify: `src/trade/models.rs`
- Test: `tests/trade_reporting_test.rs`

- [ ] **Step 1: Write the failing reporting tests**

Cover:

- history rows sort newest first
- history rows respect optional code filter
- history rows respect optional limit
- fee rows expose commission/stamp-duty/transfer-fee correctly
- overview computes:
  - booked position value
  - booked total assets
  - trade count
  - holding count
  - total buy amount
  - total sell amount
  - total fee
- position-current rows compute unrealized PnL correctly when live prices are supplied
- empty state behavior for no trades and no positions

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test trade_reporting_test -- --nocapture
```

Expected: FAIL because `trade::reporting` does not exist yet.

- [ ] **Step 3: Add the minimal reporting module**

In `src/trade/models.rs`, add:

- `TradeHistoryRow`
- `TradeFeeRow`
- `TradeOverview`
- `TradePositionCurrentRow`
- `TradeQuoteStatus`

In `src/trade/reporting.rs`, add a read-only service with methods like:

- `history_rows(state, code_filter, limit)`
- `fee_rows(state, code_filter, limit)`
- `overview(state)`
- `position_rows(state)`
- `position_rows_with_quotes(state, quote_map)`
- `overview_with_quotes(state, quote_map, coverage_mode)`

Do not add persistence or mutation here.

- [ ] **Step 4: Re-run reporting tests**

Run:

```bash
cargo test --test trade_reporting_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/trade/mod.rs src/trade/models.rs src/trade/reporting.rs tests/trade_reporting_test.rs
git commit -m "feat: add phase28 trade reporting core"
```

## Chunk 3: Handler Wiring and Current Valuation

### Task 3: Wire `history`, `fees`, and `overview`

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

- [ ] **Step 1: Write failing handler tests for read-only commands**

Add handler-level tests for:

- `trade history` returns newest-first rows
- `trade history --code` filters correctly
- `trade fees` returns fee rows
- `trade overview` returns booked summary without live lookup
- uninitialized account returns the same readable error already used by `trade cash/position`

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test execute_trade_ -- --nocapture
```

Expected: FAIL on the new history/fees/overview test cases.

- [ ] **Step 3: Implement minimal handler wiring**

In `src/cli/handlers.rs`:

- extend `TradeCommandOutput`
- map new `TradeCommands` variants to reporting methods
- add printer functions for:
  - history table
  - fee table
  - booked overview block

Keep `trade overview` separate from `trade cash`; do not overload existing output.

- [ ] **Step 4: Re-run handler tests**

Run:

```bash
cargo test execute_trade_ -- --nocapture
```

Expected: PASS for the new read-only cases.

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers.rs
git commit -m "feat: add phase28 trade history fees and overview handlers"
```

### Task 4: Add `--current` quote overlay for overview and position

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

- [ ] **Step 1: Write failing tests for quote-aware views**

Add tests covering:

- `trade position --current` with full quote coverage
- `trade position --current` with partial quote coverage
- `trade overview --current` with full quote coverage
- `trade overview --current` with partial quote coverage
- quote lookup failure degrades gracefully without failing the command

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test current -- --nocapture
```

Expected: FAIL because `--current` still behaves like the booked-only path.

- [ ] **Step 3: Add quote overlay logic**

In `src/cli/handlers.rs`:

- add a small quote lookup helper local to the CLI layer
- reuse existing best-effort quote loading patterns already used by watchlist/monitor
- for missing quotes:
  - per-position live fields stay `None`
  - `quote_status` becomes `Missing`
  - command still succeeds
- for `overview --current`:
  - only set `live_position_value` / `live_total_assets` when all held codes have quotes
  - otherwise print a readable coverage note

- [ ] **Step 4: Re-run the targeted tests**

Run:

```bash
cargo test current -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers.rs
git commit -m "feat: add phase28 current valuation views"
```

## Chunk 4: Docs and Final Verification

### Task 5: Update user-facing docs

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Add failing doc assertions**

Extend hygiene tests so they require:

- `trade history`
- `trade fees`
- `trade overview`
- `trade position --current`
- the best-effort `--current` degradation note

- [ ] **Step 2: Run doc hygiene tests to verify failure**

Run:

```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Expected: FAIL because docs do not mention the new commands yet.

- [ ] **Step 3: Update docs**

In `README.md` and `docs/USER_MANUAL.md`:

- move Phase 26A boundary forward to include the new read-only commands
- document:
  - default limits
  - `--code` filtering
  - `--current` quote lookup behavior
  - graceful degradation when live prices are unavailable

- [ ] **Step 4: Re-run hygiene tests**

Run:

```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase28 trade observability usage"
```

### Task 6: Run the full verification set

**Files:**
- No new files

- [ ] **Step 1: Run targeted trade tests**

Run:

```bash
cargo test --test trade_reporting_test -- --nocapture
cargo test --test trade_service_test -- --nocapture
cargo test --test trade_storage_test -- --nocapture
```

Expected: PASS

- [ ] **Step 2: Run parser and hygiene coverage**

Run:

```bash
cargo test cli::tests::trade -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 3: Run full suite**

Run:

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Smoke the CLI**

Run:

```bash
cargo run -- trade --help
cargo run -- trade overview
```

Expected:

- help text lists `history`, `fees`, `overview`
- `trade overview` succeeds when an account exists, otherwise returns the standard init error

- [ ] **Step 5: Final commit (if needed)**

If verification changes required small fixes:

```bash
git add -A
git commit -m "test: finish phase28 trade observability verification"
```

Otherwise skip this commit.

## Execution Notes

- Keep `src/trade/service.rs` write semantics unchanged unless a test proves a bug in existing behavior
- Do not add a new storage file or snapshot cache for Phase 28A
- Do not push quote lookup into `trade::reporting`
- Do not refactor `src/cli/handlers.rs` into many submodules as part of this phase; stay focused on the observability slice
