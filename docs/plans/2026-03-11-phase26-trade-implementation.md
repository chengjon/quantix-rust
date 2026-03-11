# Phase 26A Paper Trade Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the smallest paper-trade loop on top of the Phase 25A green baseline: initialize/reset one account, execute immediate limit buys/sells, persist trade state, and inspect positions/cash.

**Architecture:** Introduce a dedicated `trade` domain with explicit fee calculation and JSON persistence. Keep CLI parsing in `src/cli/mod.rs`, command execution in `src/cli/handlers.rs`, and local path resolution in `src/core/runtime.rs`.

**Tech Stack:** Rust, clap, chrono, rust_decimal, serde JSON, async-trait, existing CLI handler pattern, Markdown docs.

---

## P0 Boundary

Only implement:

- `quantix trade init [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]`
- `quantix trade reset [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]`
- `quantix trade buy <CODE> --price <PRICE> --volume <N>`
- `quantix trade sell <CODE> --price <PRICE> --volume <N>`
- `quantix trade position`
- `quantix trade cash`
- JSON persistence at `QUANTIX_TRADE_PATH`

Explicitly exclude:

- `trade history`
- `trade account`
- `trade overview`
- `trade fees`
- `trade position --code`
- `--current`
- market orders
- partial fills
- slippage
- pending orders / cancel
- multi-account support
- automatic linkage from stop triggers into trade execution

### Task 1: Add top-level `trade` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

**Step 1: Write the failing test**

Add parser tests for:

- `quantix trade init`
- `quantix trade init --capital 1500000 --commission-rate 0.0003`
- `quantix trade reset --capital 500000`
- `quantix trade buy 000001 --price 15.0 --volume 1000`
- `quantix trade sell 000001 --price 16.0 --volume 500`
- `quantix trade position`
- `quantix trade cash`

Also assert parser rejection for:

- missing `--price`
- missing `--volume`

**Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::parses_trade -- --nocapture`

Expected: FAIL because `Commands::Trade` does not exist yet.

**Step 3: Write minimal implementation**

Add:

- `Commands::Trade(TradeCommands)`
- `TradeCommands::Init`
- `TradeCommands::Reset`
- `TradeCommands::Buy`
- `TradeCommands::Sell`
- `TradeCommands::Position`
- `TradeCommands::Cash`

Do not add `history`, `overview`, or `fees`.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::tests::parses_trade -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add phase26 trade cli surface"
```

### Task 2: Add trade models, fee calculator, and core service

**Files:**
- Create: `src/trade/mod.rs`
- Create: `src/trade/models.rs`
- Create: `src/trade/fees.rs`
- Create: `src/trade/service.rs`
- Modify: `src/lib.rs`
- Test: `tests/trade_service_test.rs`

**Step 1: Write the failing test**

Add service tests covering:

- `init_account` creates the default account with `1000000`
- `init_account` stores custom fee config
- `reset_account` overwrites an existing account and clears old trades
- `buy` opens a new position and reduces cash by amount plus fees
- second `buy` updates weighted average cost including buy-side fees
- `buy` rejects insufficient cash
- `sell` reduces a position and increases cash by amount minus fees
- `sell` removes the position when volume reaches zero
- `sell` rejects missing positions
- `sell` rejects insufficient position volume
- `cash_snapshot` uses `last_trade_price` to compute estimated assets
- invalid capital / rate / price / volume inputs are rejected
- fee calculation for Shanghai buy/sell applies the expected commission, stamp duty, and transfer fee

**Step 2: Run test to verify it fails**

Run: `cargo test --test trade_service_test -v`

Expected: FAIL because the `trade` module does not exist yet.

**Step 3: Write minimal implementation**

Create core types:

- `PaperTradeState`
- `PaperTradeAccount`
- `TradePosition`
- `TradeRecord`
- `TradeSide`
- `FeeConfig`
- `FeeBreakdown`
- `CashSnapshot`

Create traits:

- `PaperTradeStore`

Create `TradeService<Store>` methods:

- `init_account(...)`
- `reset_account(...)`
- `buy(...)`
- `sell(...)`
- `positions()`
- `cash_snapshot()`

Rules:

- one account only: `"default"`
- `init_account` fails if already initialized
- `reset_account` replaces the full state
- trades always fill immediately at the input price
- average cost includes buy-side fees
- `cash_snapshot` uses execution-price-based estimated position value

**Step 4: Run test to verify it passes**

Run: `cargo test --test trade_service_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/trade/mod.rs src/trade/models.rs src/trade/fees.rs src/trade/service.rs src/lib.rs tests/trade_service_test.rs
git commit -m "feat: add phase26 trade service core"
```

### Task 3: Add trade runtime path support and JSON storage

**Files:**
- Modify: `src/core/runtime.rs`
- Create: `src/trade/storage.rs`
- Test: `tests/trade_storage_test.rs`
- Test: `src/core/runtime.rs`

**Step 1: Write the failing test**

Add runtime tests for:

- `QUANTIX_TRADE_PATH` override
- default path fallback to `~/.quantix/trade/paper_trade.json`

Add storage tests for:

- creating the store when the file does not exist
- save/load round-trip for initialized state
- persisted trades survive reopen
- reset-state persistence overwrites previous content cleanly

**Step 2: Run test to verify it fails**

Run: `cargo test trade_path -- --nocapture`

Run: `cargo test --test trade_storage_test -v`

Expected: FAIL because trade runtime path and storage do not exist yet.

**Step 3: Write minimal implementation**

In `src/core/runtime.rs` add:

- `TRADE_PATH_ENV`
- `CliRuntime.trade_path`
- default resolver for `~/.quantix/trade/paper_trade.json`

In `src/trade/storage.rs` implement:

- `JsonPaperTradeStore`
- file create-on-save behavior
- `load_state()`
- `save_state()`

Reuse the same local-state approach as watchlist storage.

**Step 4: Run test to verify it passes**

Run: `cargo test trade_path -- --nocapture`

Run: `cargo test --test trade_storage_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/core/runtime.rs src/trade/storage.rs tests/trade_storage_test.rs
git commit -m "feat: add phase26 trade json storage"
```

### Task 4: Wire trade commands into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/cli/mod.rs`
- Test: `src/cli/handlers.rs`

**Step 1: Write the failing test**

Add handler-level tests with fake trade collaborators covering:

- `trade init` succeeds and returns account summary
- `trade reset` succeeds and clears previous state
- `trade buy` succeeds and returns trade summary
- `trade sell` succeeds and returns trade summary
- `trade position` returns current positions
- `trade cash` returns the current cash snapshot
- buy/sell before init return user-facing errors
- buy rejects invalid price or volume
- sell rejects unheld code or excess volume

Prefer testing a helper such as `execute_trade_command_with_service(...)` instead of stdout capture.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::test_execute_trade -- --nocapture`

Expected: FAIL because `run_trade_command` does not exist.

**Step 3: Write minimal implementation**

Add:

- `run_trade_command`
- request builders / validators for init/reset/buy/sell
- trade print helpers
- `create_trade_store()`

Reuse:

- `CliRuntime::load().trade_path`
- current CLI output style from watchlist / stop commands

Do not add `trade history`, `trade fees`, or any quote-loading path.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::test_execute_trade -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs src/cli/mod.rs
git commit -m "feat: wire phase26 trade commands"
```

### Task 5: Document Phase 26A and extend repo hygiene coverage

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

**Step 1: Write the failing test**

Extend repo hygiene coverage so docs must mention:

- `quantix trade init`
- `quantix trade buy`
- `quantix trade sell`
- `quantix trade position`
- `quantix trade cash`
- `QUANTIX_TRADE_PATH`
- deferred `trade history / trade overview / trade fees / --current`

**Step 2: Run test to verify it fails**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: FAIL because docs do not mention the new trade commands yet.

**Step 3: Write minimal implementation**

Update docs to describe only Phase 26A:

- single-account local paper trading
- immediate limit fills
- fee config via init/reset only
- JSON persistence path
- explicit deferred features

Do not document unimplemented history, quote-driven valuation, or stop-execution linkage.

**Step 4: Run test to verify it passes**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase26 trade command usage"
```

### Task 6: Final regression

**Files:**
- Verify only

**Step 1: Run focused trade regression**

Run:

- `cargo test cli::tests::parses_trade -- --nocapture`
- `cargo test cli::handlers::tests::test_execute_trade -- --nocapture`
- `cargo test --test trade_service_test -v`
- `cargo test --test trade_storage_test -v`
- `cargo test --test repo_hygiene_test -- --nocapture`

Expected: PASS

**Step 2: Run full regression**

Run: `cargo test`

Expected: PASS on the full suite.

**Step 3: Verify clean worktree**

Run: `git status --short`

Expected: empty output
