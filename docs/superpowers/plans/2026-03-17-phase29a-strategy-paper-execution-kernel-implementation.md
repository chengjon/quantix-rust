# Phase 29A Strategy Paper Execution Kernel Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first real `strategy run --mode paper` path for `ma_cross` by introducing a unified execution kernel, a dedicated runtime SQLite audit store, and a paper execution adapter that reuses the existing trade and risk services.

**Architecture:** Extend `CliRuntime` with a strategy-runtime SQLite path, add a new `execution/` module for shared models, runtime-store persistence, kernel orchestration, and a `PaperExecutionAdapter`, then add a small `strategy::runtime` layer that drives `ma_cross` to a signal and feeds it through a default execution policy. Keep `paper_trade.json` and `risk_state.json` as the authoritative write stores, and let the new `runtime.db` hold run/order/event audit rows plus future recovery checkpoints.

**Tech Stack:** Rust, tokio, clap, sqlx/sqlite, serde/serde_json, rust_decimal, existing ClickHouse client, existing paper-trade/risk JSON stores, GitNexus impact analysis, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-17-phase29a-strategy-paper-execution-kernel-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-17-phase29a-strategy-paper-execution-kernel-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Work in an isolated worktree before touching code.
- Before editing any existing function, method, or other indexed symbol, run `gitnexus_impact` and stop if the result is HIGH or CRITICAL until the blast radius is reviewed.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Use `CARGO_TARGET_DIR=/tmp/quantix-target-phase29a` for all builds/tests in this phase.
- Treat Phase 29A as single-process only for paper execution. Do not try to solve cross-process locking in this slice.

## File Map

- `src/lib.rs`
  - Export the new `execution` module.
- `src/core/runtime.rs`
  - Add `QUANTIX_STRATEGY_RUNTIME_DB_PATH` resolution and include the new path in `CliRuntime`.
- `src/execution/mod.rs`
  - Export execution submodules.
- `src/execution/models.rs`
  - Shared execution models: run status, order status, signal envelope, execution policy, order intent, run/order/event row structs.
- `src/execution/runtime_store.rs`
  - SQLite bootstrap, schema creation, insert/update/query helpers, idempotency lookups.
- `src/execution/adapter.rs`
  - Execution adapter trait and request/response structs.
- `src/execution/paper.rs`
  - Paper adapter that wraps `TradeService`.
- `src/execution/kernel.rs`
  - Kernel orchestration, idempotency, risk bridge, order/event persistence, recovery placeholder.
- `src/strategy/runtime.rs`
  - Single-shot strategy runner and bar-loader abstraction for `ma_cross`.
- `src/strategy/mod.rs`
  - Export `runtime`.
- `src/cli/handlers.rs`
  - Wire `strategy run --mode paper` into the new execution stack while preserving `backtest` and keeping `live` unsupported.
- `src/cli/tests/mod.rs`
  - Register `strategy` parser tests.
- `src/cli/tests/strategy.rs`
  - Parser coverage for strategy modes and flags.
- `tests/execution_runtime_store_test.rs`
  - Focused runtime-store schema and idempotency tests.
- `tests/execution_kernel_test.rs`
  - Translation, adapter, and kernel orchestration tests.
- `tests/strategy_paper_run_test.rs`
  - End-to-end paper-run tests using temp stores and fake market data.
- `README.md`
  - Update the strategy capability summary and Phase 29A boundary.
- `docs/USER_MANUAL.md`
  - Document paper-mode support, runtime DB path, and current limitations.
- `docs/QUICKSTART.md`
  - Fix the outdated strategy command syntax and reflect the current supported path.
- `tests/repo_hygiene_test.rs`
  - Lock the new documentation boundary.

## Chunk 1: Runtime Path And Audit Store Foundation

### Task 1: Add strategy runtime DB path resolution to `CliRuntime`

**Files:**
- Modify: `src/core/runtime.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `CliRuntime`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "CliRuntime", direction: "upstream"})
```

Expected: low/medium infrastructure risk centered on CLI path consumers. If HIGH/CRITICAL, review all current runtime-path callers before changing the struct.

- [ ] **Step 2: Write the failing runtime-path tests**

Add focused tests that assert:
- `CliRuntime::load()` resolves `strategy_runtime_db_path` from `QUANTIX_STRATEGY_RUNTIME_DB_PATH`
- without that env var, the default path is `~/.quantix/strategy/runtime.db`
- without `HOME`, the fallback is `.quantix/strategy/runtime.db`

Add assertions in the existing `src/core/runtime.rs` test module, next to the existing watchlist/trade/risk/monitor path tests.

- [ ] **Step 3: Run the focused runtime tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib core::runtime::tests:: -- --nocapture
```

Expected: FAIL because `CliRuntime` does not yet expose a strategy runtime DB path.

- [ ] **Step 4: Implement the new runtime path**

Add:

```rust
pub const STRATEGY_RUNTIME_DB_PATH_ENV: &str = "QUANTIX_STRATEGY_RUNTIME_DB_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRuntime {
    pub clickhouse: ClickHouseSettings,
    pub watchlist_path: PathBuf,
    pub trade_path: PathBuf,
    pub risk_path: PathBuf,
    pub monitor_db_path: PathBuf,
    pub monitor_config_path: PathBuf,
    pub strategy_runtime_db_path: PathBuf,
}
```

Implement `resolve_strategy_runtime_db_path()` using the same env/`HOME`/relative fallback pattern already used in the file.

- [ ] **Step 5: Re-run the focused runtime tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib core::runtime::tests:: -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/core/runtime.rs` is affected.

Commit:
```bash
git add src/core/runtime.rs
git commit -m "feat: add phase29a strategy runtime path"
```

### Task 2: Create the execution module skeleton and runtime SQLite store

**Files:**
- Modify: `src/lib.rs`
- Create: `src/execution/mod.rs`
- Create: `src/execution/models.rs`
- Create: `src/execution/runtime_store.rs`
- Create: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write the failing runtime-store tests**

Cover:
- bootstrapping an empty runtime DB creates all Phase 29A tables
- inserting a `strategy_runs` row twice with the same `(strategy_name, mode, symbol, timeframe, bar_end)` key is rejected
- inserting two `orders` rows with the same `client_order_id` is rejected
- `runner_checkpoints` supports upsert-style writes for one `(strategy_name, mode, symbol, timeframe)` key

Use a temp directory and file-backed SQLite path, not an in-memory DB, so path creation is covered too.

Suggested helper shape:

```rust
#[tokio::test]
async fn bootstrap_creates_phase29a_schema() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");

    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    assert!(store.has_table("strategy_runs").await.unwrap());
    assert!(store.has_table("orders").await.unwrap());
}
```

- [ ] **Step 2: Run the focused store tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: FAIL because the `execution` module and runtime store do not exist yet.

- [ ] **Step 3: Implement the minimal execution module and shared record models**

Create `src/execution/mod.rs` and export:

```rust
pub mod models;
pub mod runtime_store;
```

In `src/execution/models.rs`, add the persistence-facing enums/records needed by the store:

```rust
pub enum StrategyRunStatus { Running, Success, Failed }
pub enum OrderStatus { PendingSubmit, Submitted, Accepted, PartiallyFilled, Filled, Canceled, Rejected, Unknown }

pub struct StrategyRunRecord { /* run_id, strategy_name, mode, symbol, timeframe, bar_end, status, timestamps */ }
pub struct OrderRecord { /* order_id, client_order_id, run_id, symbol, side, requested_quantity, requested_price, status */ }
```

Export the module from `src/lib.rs` with:

```rust
pub mod execution;
```

- [ ] **Step 4: Implement `StrategyRuntimeStore`**

Use `sqlx::SqlitePool` or `SqliteConnection` with file-backed SQLite and create parent directories on startup.

Add focused helpers such as:

```rust
pub struct StrategyRuntimeStore {
    pool: sqlx::SqlitePool,
}

impl StrategyRuntimeStore {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self>;
    pub async fn insert_run(&self, run: &StrategyRunRecord) -> Result<()>;
    pub async fn insert_order(&self, order: &OrderRecord) -> Result<()>;
    pub async fn insert_order_event(&self, event: &OrderEventRecord) -> Result<()>;
    pub async fn find_order_by_client_order_id(&self, client_order_id: &str) -> Result<Option<OrderRecord>>;
    pub async fn upsert_checkpoint(&self, checkpoint: &RunnerCheckpointRecord) -> Result<()>;
}
```

Keep raw SQL in this file for now; do not introduce migrations tooling in Phase 29A.

- [ ] **Step 5: Re-run the focused store tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/lib.rs`, `src/execution/mod.rs`, `src/execution/models.rs`, `src/execution/runtime_store.rs`, and `tests/execution_runtime_store_test.rs` are affected.

Commit:
```bash
git add src/lib.rs src/execution/mod.rs src/execution/models.rs src/execution/runtime_store.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add phase29a execution runtime store"
```

## Chunk 2: Strategy Runtime And Signal Translation

### Task 3: Add `SignalEnvelope`, `ExecutionPolicy`, and single-shot strategy runtime helpers

**Files:**
- Modify: `src/execution/models.rs`
- Create: `src/strategy/runtime.rs`
- Modify: `src/strategy/mod.rs`
- Create: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write the failing strategy/runtime and translation tests**

Cover:
- `Hold` translates to a no-op
- `Buy` uses a fixed-cash policy and rounds down to an A-share board lot
- `Sell` uses sell-all semantics for the current position
- `StrategyRuntime` can drive `MACrossStrategy` over fixture bars and return a `SignalEnvelope`

Use a fake bar loader trait rather than real ClickHouse in the tests.

Suggested test snippets:

```rust
#[test]
fn hold_signal_produces_no_order_intent() {
    let envelope = SignalEnvelope::new(Signal::Hold);
    let result = translate_signal(&envelope, &policy, None).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn strategy_runtime_returns_latest_signal_for_ma_cross() {
    let loader = FakeBarLoader::with_bars("000001", create_ma_cross_fixture());
    let runtime = StrategyRuntime::new(loader);
    let envelope = runtime.run_ma_cross_once("000001", 5, 20).await.unwrap();
    assert!(matches!(envelope.signal, Signal::Buy | Signal::Sell | Signal::Hold));
}
```

- [ ] **Step 2: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test -- --nocapture
```

Expected: FAIL because the signal envelope, policy, translator, and strategy runtime helpers do not exist yet.

- [ ] **Step 3: Implement the shared strategy/execution primitives**

In `src/execution/models.rs`, add:

```rust
pub struct SignalEnvelope {
    pub signal: Signal,
    pub metadata_json: serde_json::Value,
}

pub struct ExecutionPolicy {
    pub fixed_cash_per_buy: Decimal,
    pub slippage_bps: u32,
}

pub struct OrderIntent {
    pub symbol: String,
    pub side: OrderSide,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub order_type: OrderType,
    pub reason: String,
    pub policy_snapshot_json: serde_json::Value,
}
```

Add pure helper functions for:
- `SignalEnvelope::new(signal)`
- board-lot rounding
- signal-to-intent translation

Do not make the translator reach into ClickHouse or JSON stores directly.

- [ ] **Step 4: Implement `src/strategy/runtime.rs`**

Add a small abstraction:

```rust
#[async_trait]
pub trait StrategyBarLoader: Send + Sync {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>>;
}

pub struct StrategyRuntime<L> {
    loader: L,
}
```

Then implement a single-shot MA-cross helper:

```rust
impl<L> StrategyRuntime<L>
where
    L: StrategyBarLoader,
{
    pub async fn run_ma_cross_once(&self, code: &str, short: usize, long: usize) -> Result<SignalEnvelope>;
}
```

Modify `src/strategy/mod.rs` to export `runtime`.

- [ ] **Step 5: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test -- --nocapture
```

Expected: PASS for the pure translation and fake-loader strategy-runtime cases.

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/execution/models.rs`, `src/strategy/runtime.rs`, `src/strategy/mod.rs`, and `tests/execution_kernel_test.rs` are affected.

Commit:
```bash
git add src/execution/models.rs src/strategy/runtime.rs src/strategy/mod.rs tests/execution_kernel_test.rs
git commit -m "feat: add phase29a strategy signal translation"
```

## Chunk 3: Paper Adapter And Kernel Orchestration

### Task 4: Add the execution adapter contract and paper adapter

**Files:**
- Modify: `src/execution/mod.rs`
- Create: `src/execution/adapter.rs`
- Create: `src/execution/paper.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write the failing paper-adapter tests**

Cover:
- successful buy submission returns a filled-style adapter response and updates the underlying paper account
- successful sell submission returns a filled-style response
- unsupported cancel returns a clear unsupported error
- `query_order` returns a simple not-supported or placeholder response in Phase 29A

Build the adapter tests with a fake `PaperTradeStore` like the existing [trade_service_test.rs](/opt/claude/quantix-rust/tests/trade_service_test.rs).

- [ ] **Step 2: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test paper_adapter -- --nocapture
```

Expected: FAIL because the adapter contract and paper adapter do not exist yet.

- [ ] **Step 3: Implement the adapter contract**

In `src/execution/adapter.rs`, add:

```rust
#[async_trait]
pub trait ExecutionAdapter: Send + Sync {
    async fn submit_order(&self, request: AdapterOrderRequest) -> Result<OrderInitialResponse, AdapterError>;
    async fn query_order(&self, order_id: &str) -> Result<OrderQueryResponse, AdapterError>;
    async fn cancel_order(&self, order_id: &str) -> Result<(), AdapterError>;
}
```

Include request/response structs that can represent:
- adapter order id
- latest known `OrderStatus`
- filled quantity
- average fill price
- rejection reason

- [ ] **Step 4: Implement `PaperExecutionAdapter`**

Wrap the existing `TradeService`:

```rust
pub struct PaperExecutionAdapter<Store> {
    trade_service: TradeService<Store>,
}
```

Rules:
- map buy/sell intents into `TradeOrderRequest`
- call the existing paper trade service
- return an immediate `Filled` response on success
- do not simulate delay, partial fill, or `Unknown`
- let the adapter own the `paper_trade.json` mutation by going through `TradeService`

- [ ] **Step 5: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test paper_adapter -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/execution/mod.rs`, `src/execution/adapter.rs`, `src/execution/paper.rs`, and `tests/execution_kernel_test.rs` are affected.

Commit:
```bash
git add src/execution/mod.rs src/execution/adapter.rs src/execution/paper.rs tests/execution_kernel_test.rs
git commit -m "feat: add phase29a paper execution adapter"
```

### Task 5: Implement `ExecutionKernel` with idempotency, risk evaluation, and audit sequencing

**Files:**
- Modify: `src/execution/mod.rs`
- Create: `src/execution/kernel.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write the failing kernel tests**

Cover:
- successful paper buy run writes:
  - one `strategy_runs` row
  - one `signal_events` row
  - one `orders` row
  - one or more `order_events`
- risk rejection creates a terminal `Rejected` order row and does not call the adapter
- duplicate `client_order_id` returns the stored result instead of resubmitting
- `recover_pending_orders()` exists and returns an empty/not-implemented summary

Suggested test flow:

```rust
#[tokio::test]
async fn kernel_success_path_persists_run_signal_order_and_events() {
    let store = StrategyRuntimeStore::new(temp_path()).await.unwrap();
    let kernel = build_kernel_with_fake_deps(store.clone());

    let result = kernel.execute_once(sample_run_request(), sample_signal_envelope()).await.unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Filled));
    assert_eq!(store.count_runs().await.unwrap(), 1);
    assert_eq!(store.count_orders().await.unwrap(), 1);
}
```

- [ ] **Step 2: Run the focused kernel tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test kernel_ -- --nocapture
```

Expected: FAIL because the kernel orchestration layer does not exist yet.

- [ ] **Step 3: Implement the minimal kernel**

In `src/execution/kernel.rs`, add:

```rust
pub enum RiskDecision {
    Allow(OrderIntent),
    Reject { reason: String },
}

pub struct ExecutionKernel<S, A, R> {
    store: S,
    adapter: A,
    risk: R,
}
```

Implement the Phase 29A sequencing from the spec:
1. create run row
2. persist signal event
3. translate signal
4. generate `client_order_id`
5. check order idempotency
6. evaluate risk
7. insert `PendingSubmit` or `Rejected` order row
8. submit through adapter if allowed
9. append order events
10. update order status
11. sync post-trade risk if filled
12. mark the run `success` or `failed`

For post-trade risk synchronization, load the current paper account state and derive `RiskAccountSnapshot` from it instead of duplicating accounting logic.

- [ ] **Step 4: Re-run the focused kernel tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test kernel_ -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/execution/mod.rs`, `src/execution/kernel.rs`, and `tests/execution_kernel_test.rs` are affected.

Commit:
```bash
git add src/execution/mod.rs src/execution/kernel.rs tests/execution_kernel_test.rs
git commit -m "feat: add phase29a execution kernel"
```

## Chunk 4: CLI Integration And Paper Run UX

### Task 6: Add strategy parser coverage and a structured paper-run integration helper

**Files:**
- Modify: `src/cli/tests/mod.rs`
- Create: `src/cli/tests/strategy.rs`
- Modify: `src/cli/handlers.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the strategy handler symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_strategy_command", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_strategy", direction: "upstream"})
```

Expected: medium CLI risk concentrated in `src/cli/handlers.rs`. If the graph reports HIGH because of the large file, keep edits tightly scoped to strategy-only helper functions and record that rationale.

- [ ] **Step 2: Write the failing parser and handler tests**

Add parser coverage in `src/cli/tests/strategy.rs` for:
- `quantix strategy run -n ma_cross --mode paper -c 000001`
- `quantix strategy run -n ma_cross --mode live -c 000001`
- `quantix strategy run -n ma_cross` still defaults to `backtest`

Add handler-focused tests in `src/cli/handlers.rs` for:
- paper mode without `--code` returns a clear validation error
- paper mode without an initialized paper account returns a clear `trade init` prerequisite error
- live mode remains unsupported

Do not capture stdout directly; add a small structured helper that returns a summary object.

- [ ] **Step 3: Run the focused parser/handler tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::tests::strategy:: -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
```

Expected: FAIL because the parser module is missing and the handler has no paper-mode path.

- [ ] **Step 4: Implement the structured strategy-run helper and CLI wiring**

In `src/cli/handlers.rs`, add a focused helper along these lines:

```rust
struct StrategyRunSummary {
    run_id: String,
    strategy_name: String,
    mode: String,
    symbol: String,
    signal: Signal,
    order_status: Option<OrderStatus>,
    message: String,
}

async fn execute_strategy_run_with_components<L, TS, RS>(
    name: &str,
    mode: &str,
    code: Option<String>,
    loader: L,
    trade_store: TS,
    risk_store: RS,
    runtime_store: &StrategyRuntimeStore,
) -> Result<StrategyRunSummary>
```

Rules:
- keep `backtest` on the existing path
- `paper` requires an explicit `code`
- `paper` requires an initialized paper account; do not auto-init capital
- `live` still returns unsupported
- use `CliRuntime::load()` to wire:
  - ClickHouse settings
  - `JsonPaperTradeStore`
  - `JsonRiskStore`
  - `StrategyRuntimeStore`

Print from the public handler only after the helper returns the structured summary.

- [ ] **Step 5: Re-run the focused parser/handler tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::tests::strategy:: -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/cli/tests/mod.rs`, `src/cli/tests/strategy.rs`, and `src/cli/handlers.rs` are affected.

Commit:
```bash
git add src/cli/tests/mod.rs src/cli/tests/strategy.rs src/cli/handlers.rs
git commit -m "feat: wire phase29a strategy paper mode"
```

### Task 7: Add an end-to-end paper-run test over temp stores and fake bars

**Files:**
- Create: `tests/strategy_paper_run_test.rs`

- [ ] **Step 1: Write the failing end-to-end tests**

Cover:
- a successful paper run writes runtime rows, mutates the paper account, and leaves `live` unsupported
- a second run on the same bar is deduplicated by the run key
- risk rejection keeps the paper account unchanged and records a rejected order attempt

Use:
- temp JSON paths for trade/risk
- temp SQLite path for runtime DB
- fake bar loader with deterministic MA-cross fixtures
- real `TradeService` and `RiskService`

- [ ] **Step 2: Run the focused end-to-end tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test strategy_paper_run_test -- --nocapture
```

Expected: FAIL because the full paper-run path is not yet wired end-to-end.

- [ ] **Step 3: Implement the missing glue to make the end-to-end tests pass**

Typical missing pieces at this point:
- stable run-key generation from `(strategy_name, mode, symbol, timeframe, bar_end)`
- `client_order_id` generation from `<run_id>_<symbol>_<sequence>`
- risk-snapshot derivation from current paper account state
- concise run summary formatting

Keep this step small: fill gaps in the execution stack; do not reopen Phase 29B daemon scope.

- [ ] **Step 4: Re-run the focused end-to-end tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test strategy_paper_run_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `tests/strategy_paper_run_test.rs` plus the minimum supporting files touched in this chunk are affected.

Commit:
```bash
git add tests/strategy_paper_run_test.rs src/cli/handlers.rs src/execution/mod.rs src/execution/models.rs src/execution/adapter.rs src/execution/paper.rs src/execution/runtime_store.rs src/execution/kernel.rs src/strategy/runtime.rs
git commit -m "test: cover phase29a strategy paper run"
```

## Chunk 5: Docs And Final Verification

### Task 8: Update docs, quickstart, and repo hygiene coverage

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `docs/QUICKSTART.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene tests**

Add strategy-boundary assertions for:
- `Phase 29` or equivalent strategy paper wording in `README.md`
- `quantix strategy run -n ma_cross --mode paper -c 000001`
- `live` remaining deferred
- `QUANTIX_STRATEGY_RUNTIME_DB_PATH`
- `~/.quantix/strategy/runtime.db`
- the requirement to initialize the paper account before paper strategy execution

Keep the checks focused on stable user-facing strings, not long prose paragraphs.

- [ ] **Step 2: Run the hygiene tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test repo_hygiene_test -- --nocapture
```

Expected: FAIL because the docs do not yet advertise Phase 29A paper strategy support or the runtime DB path.

- [ ] **Step 3: Update the docs**

Document:
- `paper` is supported for `ma_cross` single-shot runs
- `live` is still in development
- `paper` requires existing trade/risk local setup, starting with `quantix trade init`
- runtime audit path:
  - env: `QUANTIX_STRATEGY_RUNTIME_DB_PATH`
  - default: `~/.quantix/strategy/runtime.db`
- current Phase 29A limitations:
  - single code
  - single-shot only
  - no daemon/service yet
  - no simulated partial fills

Also fix `docs/QUICKSTART.md` so the strategy example uses the current CLI shape, for example:

```bash
cargo run -- strategy run -n ma_cross --code 000001
```

If you choose to showcase paper mode there, make the prerequisite explicit:

```bash
cargo run -- trade init --capital 1000000
cargo run -- strategy run -n ma_cross --mode paper --code 000001
```

- [ ] **Step 4: Re-run the hygiene tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: docs + hygiene test only.

Commit:
```bash
git add README.md docs/USER_MANUAL.md docs/QUICKSTART.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase29a strategy paper guidance"
```

### Task 9: Final verification

**Files:**
- No new file changes expected

- [ ] **Step 1: Run the focused new test suites**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_runtime_store_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test execution_kernel_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --test strategy_paper_run_test -- --nocapture
```

Expected: PASS

- [ ] **Step 2: Run the relevant library test groups**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::tests::strategy:: -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test --lib core::runtime::tests:: -- --nocapture
```

Expected: PASS

- [ ] **Step 3: Run the full automated suite**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo test
```

Expected: PASS

- [ ] **Step 4: Run optional no-risk CLI smoke checks**

Only run if ClickHouse is reachable and has enough daily bars for the chosen code:

```bash
tmp_home="$(mktemp -d)"
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo run -- trade init --capital 1000000
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo run -- strategy run -n ma_cross --mode paper --code 000001
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo run -- trade history --limit 5
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase29a cargo run -- risk status
```

Expected:
- `trade init` succeeds
- paper strategy run either executes or fails only on expected external-data prerequisites
- successful runs leave trade/risk state readable

- [ ] **Step 5: Run change detection and commit the verification pass**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: affected files match the complete Phase 29A scope only.

Commit:
```bash
git add -A
git commit -m "test: verify phase29a strategy paper execution"
```

## Execution Notes

- Do not auto-initialize the paper account inside `strategy run --mode paper`; require the user to run `quantix trade init` first.
- Keep `paper_trade.json` and `risk_state.json` authoritative. The kernel may observe and audit them, but must not duplicate their accounting logic.
- Keep `StrategyRuntime` testable by depending on a bar-loader trait, not directly on `ClickHouseClient`.
- Keep the `ExecutionAdapter` interface live-compatible now, even though the paper adapter immediately fills orders.
- Keep `recover_pending_orders()` present but intentionally minimal; that work starts in Phase 29C.
- Avoid unrelated refactors inside `src/cli/handlers.rs`. Introduce small, strategy-specific helpers instead of broad reshaping.
