# Phase 29B Strategy Signal Daemon Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a signal-first strategy daemon that monitors one code with multiple strategy instances, writes durable signals into `runtime.db`, supports manual signal approval/rejection, and creates `execution_request` rows without automatically trading.

**Architecture:** Extend the current Phase 29A runtime database instead of replacing it. Add strategy-daemon configuration and service configuration stores, a strategy registry plus daemon runner, new `signals` and `execution_requests` tables in the runtime SQLite store, and a new CLI path for config, daemon, signal, request, and service management. Keep `strategy run --mode paper` unchanged as the direct execution path while Phase 29B adds a parallel signal-oriented path that is approval-gated and non-executing.

**Tech Stack:** Rust, tokio, clap, sqlx/sqlite, serde/serde_json, chrono/chrono-tz, tracing, existing ClickHouse client, existing runtime SQLite store, existing `monitor` config/service/systemd patterns, GitNexus impact analysis, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-18-phase29b-strategy-signal-daemon-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-18-phase29b-strategy-signal-daemon-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})` and stop if the risk is HIGH or CRITICAL until the blast radius is reviewed.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Use `CARGO_TARGET_DIR=/tmp/quantix-target-phase29b` for all builds/tests in this phase.
- Treat the strategy daemon as the only writer to Phase 29B signal/request tables. Do not add cross-process locking beyond SQLite transaction boundaries in this slice.
- The repository already contains unrelated dirty files. When executing this plan, stage only the files listed in the active task and do not revert unrelated user changes.

## File Map

- `src/core/runtime.rs`
  - Add strategy-daemon config path resolution and any runtime path plumbing needed by the daemon/service path.
- `src/strategy/mod.rs`
  - Export new strategy-daemon modules.
- `src/strategy/config.rs`
  - JSON config model and store for daemon configuration, including bootstrap policy and configured strategy instances.
- `src/strategy/service_config.rs`
  - JSON config model and store for strategy-daemon service configuration.
- `src/strategy/registry.rs`
  - Strategy registry, configured evaluator trait, and `ma_cross` instance factory.
- `src/strategy/daemon.rs`
  - Daemon runner, hot-reload loop, bootstrap handling, new-bar detection, signal generation orchestration, and graceful loop control.
- `src/strategy/systemd.rs`
  - Wrapper and unit rendering plus `systemctl --user` integration for the strategy daemon.
- `src/execution/models.rs`
  - Add Phase 29B signal/request/checkpoint record models and any enums needed by the new tables.
- `src/execution/runtime_store.rs`
  - Extend schema creation and add signal/request/checkpoint transaction helpers.
- `src/cli/mod.rs`
  - Add `strategy config`, `strategy daemon`, `strategy signal`, `strategy request`, `strategy service`, and `strategy service-config` subcommands.
- `src/cli/handlers.rs`
  - Add handler entry points for new strategy-daemon commands and wire them into runtime/config/store helpers.
- `src/cli/tests/mod.rs`
  - Register new parser test modules if split out from the current `strategy` parser tests.
- `src/cli/tests/strategy.rs`
  - Parser coverage for the new strategy-daemon command tree.
- `tests/execution_runtime_store_test.rs`
  - Extend runtime-store tests for signals, execution requests, and daemon checkpoints.
- `tests/strategy_daemon_test.rs`
  - Daemon integration tests using fake bar loaders, temp config, and temp runtime DB.
- `tests/strategy_systemd_test.rs`
  - Strategy-daemon systemd rendering and service-config tests.
- `README.md`
  - Update the strategy capability summary and Phase 29B boundary.
- `docs/USER_MANUAL.md`
  - Document strategy-daemon config, signal approval, request listing, and systemd usage.
- `docs/QUICKSTART.md`
  - Add a minimal signal-daemon quickstart once the CLI exists.
- `tests/repo_hygiene_test.rs`
  - Lock the new docs/CLI examples if repo hygiene tests currently cover these capability summaries.

## Chunk 1: Runtime Paths And JSON Config Stores

### Task 1: Add strategy-daemon runtime paths

**Files:**
- Modify: `src/core/runtime.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `CliRuntime`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "CliRuntime", direction: "upstream"})
```

Expected: low/medium infrastructure risk centered on CLI runtime path consumers. If HIGH/CRITICAL, review all path consumers before editing the struct.

- [ ] **Step 2: Write failing runtime-path tests**

Add focused tests in the existing `src/core/runtime.rs` test module for:
- `QUANTIX_STRATEGY_CONFIG_PATH`
- default `~/.quantix/strategy/config.json`
- relative fallback `.quantix/strategy/config.json` without `HOME`

If the design requires a runtime path for a strategy service env file, add matching tests now.

- [ ] **Step 3: Run the focused runtime tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib core::runtime::tests:: -- --nocapture
```

Expected: FAIL because the new strategy-daemon runtime paths do not exist yet.

- [ ] **Step 4: Implement the runtime path changes**

Add env constants and `CliRuntime` fields using the same resolution pattern already used for watchlist/trade/risk/monitor paths.

Expected additions:

```rust
pub const STRATEGY_CONFIG_PATH_ENV: &str = "QUANTIX_STRATEGY_CONFIG_PATH";
```

and a `strategy_config_path: PathBuf` field on `CliRuntime`.

- [ ] **Step 5: Re-run the focused runtime tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib core::runtime::tests:: -- --nocapture
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
git commit -m "feat: add phase29b strategy config runtime path"
```

### Task 2: Add strategy-daemon config and service-config stores

**Files:**
- Create: `src/strategy/config.rs`
- Create: `src/strategy/service_config.rs`
- Modify: `src/strategy/mod.rs`
- Test: `tests/strategy_systemd_test.rs`
- Test: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Write the failing config-store tests**

Add tests that cover:
- `strategy config init` storage shape defaults to `bootstrap_policy=latest_only`
- JSON store `load_or_create()` creates parent directories and default config
- service-config store validates absolute executable `quantix_bin_path`
- optional env-file path is preserved when set

Suggested default config fixture:

```json
{
  "check_interval_secs": 60,
  "bootstrap_policy": "latest_only",
  "stocks": [
    {
      "code": "000001",
      "enabled": true,
      "strategies": [
        {
          "id": "ma_fast_5_slow_20",
          "name": "ma_cross",
          "enabled": true,
          "params": { "fast": 5, "slow": 20 }
        }
      ]
    }
  ]
}
```

- [ ] **Step 2: Run the focused config tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_systemd_test -- --nocapture
```

Expected: FAIL because the strategy config/service-config stores do not exist yet.

- [ ] **Step 3: Implement the config stores**

Create:
- `StrategyDaemonConfig`
- `ConfiguredStock`
- `ConfiguredStrategyInstance`
- `BootstrapPolicy`
- `JsonStrategyConfigStore`
- `StrategyServiceConfig`
- `JsonStrategyServiceConfigStore`

Follow the repository’s existing JSON store pattern from `src/monitor/config.rs` and `src/monitor/service_config.rs`.

- [ ] **Step 4: Re-run the focused config tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_systemd_test -- --nocapture
```

Expected: PASS for the new store tests.

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only the new config/store modules and their focused tests are affected.

Commit:
```bash
git add src/strategy/config.rs src/strategy/service_config.rs src/strategy/mod.rs tests/strategy_systemd_test.rs
git commit -m "feat: add phase29b strategy daemon config stores"
```

## Chunk 2: Runtime DB Schema And Transaction Helpers

### Task 3: Extend runtime models for signals, requests, and daemon checkpoints

**Files:**
- Modify: `src/execution/models.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `StrategyRuntimeStore` and `OrderStatus`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "StrategyRuntimeStore", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "OrderStatus", direction: "upstream"})
```

Expected: low/medium risk concentrated in execution/runtime-store callers. If HIGH/CRITICAL, stop and review the direct callers first.

- [ ] **Step 2: Write failing runtime-model tests**

Extend `tests/execution_runtime_store_test.rs` with new compile-targeted expectations for:
- `SignalStatus::{New, Superseded, Expired}`
- `ApprovalStatus::{Pending, Approved, Rejected}`
- `ExecutionRequestStatus::{Pending, Completed, Failed, Canceled}`
- `StrategySignalRecord`
- `ExecutionRequestRecord`
- `StrategyDaemonCheckpointRecord`

- [ ] **Step 3: Run the focused runtime-store tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: FAIL because the new records and enums do not exist yet.

- [ ] **Step 4: Implement the new runtime models**

Add string-backed enums and row structs consistent with the existing Phase 29A model style:

```rust
pub enum SignalStatus { New, Superseded, Expired }
pub enum ApprovalStatus { Pending, Approved, Rejected }
pub enum ExecutionRequestStatus { Pending, Completed, Failed, Canceled }
pub struct StrategySignalRecord { ... }
pub struct ExecutionRequestRecord { ... }
pub struct StrategyDaemonCheckpointRecord { ... }
```

Use `TEXT` IDs, not integer IDs, to stay consistent with existing runtime records.

- [ ] **Step 5: Re-run the focused runtime-store tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: PASS for model-related tests.

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/execution/models.rs` and `tests/execution_runtime_store_test.rs` are affected.

Commit:
```bash
git add src/execution/models.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add phase29b signal runtime models"
```

### Task 4: Add schema and transaction helpers to `StrategyRuntimeStore`

**Files:**
- Modify: `src/execution/runtime_store.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write the failing store tests**

Add tests for:
- schema bootstrap creates `signals`, `execution_requests`, and `strategy_daemon_checkpoints`
- `signals` rejects duplicate `(strategy_instance_id, symbol, timeframe, bar_end)`
- approve transaction changes signal approval state and inserts exactly one execution request
- reject transaction only changes approval state
- superseding a stream cancels only pending execution requests
- daemon checkpoint upserts by `(strategy_instance_id, symbol, timeframe)`

Include a transaction-oriented helper expectation like:

```rust
let outcome = store
    .approve_signal_and_create_request(&signal_id, "paper", "default", Some("cli-user"))
    .await
    .unwrap();
assert_eq!(outcome.request_status, ExecutionRequestStatus::Pending);
```

- [ ] **Step 2: Run the focused store tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: FAIL because the new tables and helpers do not exist yet.

- [ ] **Step 3: Implement the schema changes**

Extend `ensure_schema()` with:
- `signals`
- `execution_requests`
- `strategy_daemon_checkpoints`

and their unique constraints/indexes exactly as approved in the spec.

- [ ] **Step 4: Implement the transaction helpers**

Add focused helpers such as:

```rust
pub async fn insert_signal(&self, record: &StrategySignalRecord) -> Result<()>;
pub async fn list_signals(&self, filter: SignalListFilter) -> Result<Vec<StrategySignalRecord>>;
pub async fn approve_signal_and_create_request(...) -> Result<ExecutionRequestRecord>;
pub async fn reject_signal(...) -> Result<()>;
pub async fn supersede_previous_signals_and_cancel_pending_requests(...) -> Result<usize>;
pub async fn upsert_daemon_checkpoint(...) -> Result<()>;
pub async fn find_daemon_checkpoint(...) -> Result<Option<StrategyDaemonCheckpointRecord>>;
pub async fn list_execution_requests(...) -> Result<Vec<ExecutionRequestRecord>>;
```

Implementation rules:
- keep ClickHouse and strategy evaluation outside these helpers
- use SQLite transactions for approve/reject and supersede/cancel flows
- use conditional updates instead of read-then-write approval logic

- [ ] **Step 5: Re-run the focused store tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test execution_runtime_store_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/execution/runtime_store.rs`, `src/execution/models.rs`, and `tests/execution_runtime_store_test.rs` are affected.

Commit:
```bash
git add src/execution/runtime_store.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add phase29b signal runtime store"
```

## Chunk 3: Strategy Registry And Daemon Engine

### Task 5: Add strategy registry and configured `ma_cross` evaluators

**Files:**
- Create: `src/strategy/registry.rs`
- Modify: `src/strategy/mod.rs`
- Test: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Write the failing registry tests**

Add tests for:
- registry resolves multiple configured `ma_cross` instances with different params
- evaluator reports the required lookback window
- evaluator returns `SignalEnvelope` for the latest bar
- unknown strategy names fail with a user-facing error

Suggested interface shape:

```rust
trait ConfiguredStrategyEvaluator {
    fn lookback_required(&self) -> usize;
    fn evaluate(&self, klines: &[Kline]) -> Result<SignalEnvelope>;
}
```

- [ ] **Step 2: Run the focused daemon tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_daemon_test -- --nocapture
```

Expected: FAIL because the registry does not exist yet.

- [ ] **Step 3: Implement the registry**

Add:
- `StrategyRegistry`
- `ConfiguredStrategyEvaluator`
- `ConfiguredStrategyInstanceRef` or equivalent metadata carrier

Phase 29B implementation rules:
- fully support `ma_cross`
- parse `fast` and `slow` from `params`
- return clean configuration errors for missing/invalid params
- keep the daemon orchestration agnostic once an evaluator is built

- [ ] **Step 4: Re-run the focused daemon tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_daemon_test -- --nocapture
```

Expected: PASS for the registry-related tests.

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only the registry module, exports, and focused tests are affected.

Commit:
```bash
git add src/strategy/registry.rs src/strategy/mod.rs tests/strategy_daemon_test.rs
git commit -m "feat: add phase29b strategy registry"
```

### Task 6: Implement the daemon loop, bootstrap, and hot reload

**Files:**
- Create: `src/strategy/daemon.rs`
- Modify: `src/strategy/mod.rs`
- Modify: `src/execution/runtime_store.rs`
- Test: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Write the failing daemon integration tests**

Cover:
- first run with no checkpoint bootstraps to latest bar and writes no signal
- no duplicate signal when no new bar appears
- a new bar inserts `strategy_run`, `signal`, and checkpoint rows
- older active signals on the same stream become `superseded`
- pending execution requests for superseded signals become `canceled`
- config hot reload activates/deactivates strategy instances between loops

Use:
- fake bar loader
- temp config path
- temp runtime DB path
- deterministic timestamps or fake clock helpers

- [ ] **Step 2: Run the focused daemon tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_daemon_test daemon_ -- --nocapture
```

Expected: FAIL because the daemon engine does not exist yet.

- [ ] **Step 3: Implement the daemon runner**

Add a focused runner that can:
- load config
- detect config `mtime` changes
- rebuild active streams
- fetch bars for one configured code
- compute normalized `bar_end` using `Asia/Shanghai 15:00:00` converted to UTC
- bootstrap with `latest_only`
- write runtime data only when a new bar is detected

Keep the loop testable by separating:
- one-loop evaluation
- config reload detection
- long-running sleep/wait orchestration

- [ ] **Step 4: Re-run the focused daemon tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_daemon_test daemon_ -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only daemon/registry/runtime-store related files and tests are affected.

Commit:
```bash
git add src/strategy/daemon.rs tests/strategy_daemon_test.rs
git commit -m "feat: add phase29b strategy daemon loop"
```

## Chunk 4: CLI Commands For Config, Daemon, Signal, And Request

### Task 7: Add parser support for Phase 29B strategy commands

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/tests/mod.rs`
- Modify: `src/cli/tests/strategy.rs`

- [ ] **Step 1: Write the failing parser tests**

Add clap parser tests for:
- `strategy config init`
- `strategy config show`
- `strategy daemon run`
- `strategy daemon run --once`
- `strategy signal list --approval-status pending`
- `strategy signal approve --signal-id <ID> --target-mode paper --target-account default`
- `strategy signal reject --signal-id <ID> --reason "manual reject"`
- `strategy request list --status pending`
- `strategy service install`
- `strategy service-config set --quantix-bin /opt/quantix/bin/quantix --env-file /tmp/strategy.env`

- [ ] **Step 2: Run the focused parser tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib cli::tests::strategy:: -- --nocapture
```

Expected: FAIL because the new subcommands do not exist yet.

- [ ] **Step 3: Implement the parser changes**

Add nested strategy subcommands that match the approved CLI tree exactly. Keep `strategy run/list/show` intact.

- [ ] **Step 4: Re-run the focused parser tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib cli::tests::strategy:: -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only CLI parser files and tests are affected.

Commit:
```bash
git add src/cli/mod.rs src/cli/tests/mod.rs src/cli/tests/strategy.rs
git commit -m "feat: add phase29b strategy daemon cli parsing"
```

### Task 8: Implement CLI handlers for config, daemon, signal, and request commands

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/core/runtime.rs`
- Modify: `src/strategy/config.rs`
- Modify: `src/strategy/daemon.rs`
- Test: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `run_strategy_command`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_strategy_command", direction: "upstream"})
```

Expected: medium CLI routing risk. If HIGH/CRITICAL, stop and review all command entry points before editing.

- [ ] **Step 2: Write the failing handler tests**

Cover:
- `strategy config init` creates the default config file
- `strategy config show` prints the saved config
- `strategy daemon run --once` bootstraps or emits one signal as appropriate
- `strategy signal list` shows pending signals
- `strategy signal approve` creates exactly one execution request
- `strategy signal reject` changes approval state only
- `strategy request list` prints the request rows

Prefer focused tempdir-based handler tests over end-to-end process spawning.

- [ ] **Step 3: Run the focused handler tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib cli::handlers::tests::test_execute_strategy_ -- --nocapture
```

Expected: FAIL because the handlers do not exist yet.

- [ ] **Step 4: Implement the handlers**

Add small handler helpers rather than putting all logic into `run_strategy_command`.

Suggested split:
- `execute_strategy_config_init(...)`
- `execute_strategy_config_show(...)`
- `execute_strategy_daemon_run(...)`
- `execute_strategy_signal_list(...)`
- `execute_strategy_signal_approve(...)`
- `execute_strategy_signal_reject(...)`
- `execute_strategy_request_list(...)`

Implementation rules:
- `strategy daemon run --once` should run one loop and exit
- foreground `strategy daemon run` should keep looping until interrupted
- approval and rejection should delegate transaction semantics to `StrategyRuntimeStore`

- [ ] **Step 5: Re-run the focused handler tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --lib cli::handlers::tests::test_execute_strategy_ -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only the new strategy-daemon CLI plumbing and related tests are affected.

Commit:
```bash
git add src/cli/handlers.rs src/core/runtime.rs src/strategy/config.rs src/strategy/daemon.rs tests/strategy_daemon_test.rs
git commit -m "feat: add phase29b strategy daemon handlers"
```

## Chunk 5: Systemd Service Layer, Docs, And Final Verification

### Task 9: Add strategy-daemon service and systemd integration

**Files:**
- Create: `src/strategy/systemd.rs`
- Modify: `src/strategy/service_config.rs`
- Modify: `src/strategy/mod.rs`
- Modify: `src/cli/handlers.rs`
- Test: `tests/strategy_systemd_test.rs`

- [ ] **Step 1: Write the failing systemd tests**

Cover:
- wrapper script runs `quantix strategy daemon run`
- unit path points at `~/.config/systemd/user/quantix-strategy.service`
- wrapper path points at `~/.local/bin/quantix-strategy-run`
- unit renders `Environment=QUANTIX_STRATEGY_CONFIG_PATH=...`
- unit renders `Environment=QUANTIX_STRATEGY_RUNTIME_DB_PATH=...`
- unit renders `EnvironmentFile=-...` when configured
- service status summary returns structured fields

- [ ] **Step 2: Run the focused systemd tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_systemd_test -- --nocapture
```

Expected: FAIL because strategy-daemon systemd support does not exist yet.

- [ ] **Step 3: Implement the service layer**

Follow the monitor-service pattern:
- validate stable binary path
- render wrapper script
- render user unit
- call `systemctl --user`
- surface `status` as a structured summary plus optional raw status text

Wire new CLI handlers for:
- `strategy service ...`
- `strategy service-config ...`

- [ ] **Step 4: Re-run the focused systemd tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test strategy_systemd_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only strategy service/systemd files and tests are affected.

Commit:
```bash
git add src/strategy/systemd.rs src/strategy/service_config.rs src/strategy/mod.rs src/cli/handlers.rs tests/strategy_systemd_test.rs
git commit -m "feat: add phase29b strategy daemon service support"
```

### Task 10: Update docs and run final verification

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `docs/QUICKSTART.md`
- Modify: `tests/repo_hygiene_test.rs` (only if needed to lock docs/examples)

- [ ] **Step 1: Write the failing docs/repo-hygiene checks**

If repo-hygiene tests already pin CLI/docs examples, extend them first so they fail until docs are updated.

- [ ] **Step 2: Update documentation**

Document:
- Phase 29B boundary
- `strategy config init/show`
- `strategy daemon run` and `--once`
- signal listing and approval/rejection
- request listing
- strategy-daemon `systemd --user` workflow
- explicit note that approval creates an `execution_request` but does not trade

- [ ] **Step 3: Run focused docs/repo-hygiene checks**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 4: Run the full Phase 29B verification suite**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase29b cargo test
```

Expected: PASS with `0 failed`

- [ ] **Step 5: Optional manual smoke on WSL2 + systemd**

If the environment supports it, verify:

```bash
cargo run -- strategy config init
cargo run -- strategy daemon run --once
cargo run -- strategy signal list
cargo run -- strategy service-config set --quantix-bin "$(pwd)/target/debug/quantix"
cargo run -- strategy service install
cargo run -- strategy service start
cargo run -- strategy service status
```

Expected:
- config file created
- daemon bootstrap or signal generation visible
- systemd service installs and reports structured status

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: docs and any intentional repo-hygiene updates only.

Commit:
```bash
git add README.md docs/USER_MANUAL.md docs/QUICKSTART.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase29b strategy daemon guidance"
```

## Final Gate

- Re-read the Phase 29B spec and confirm each delivery item maps to a passing test or an implemented CLI/service path.
- Verify `strategy run --mode paper` still works unchanged.
- Verify approved signals create `execution_request` rows but no order/trade rows.
- Verify first bootstrap creates checkpoints without backfilling historical signals.
- Verify no unrelated files were staged in any commit.

## Expected Commit Sequence

1. `feat: add phase29b strategy config runtime path`
2. `feat: add phase29b strategy daemon config stores`
3. `feat: add phase29b signal runtime models`
4. `feat: add phase29b signal runtime store`
5. `feat: add phase29b strategy registry`
6. `feat: add phase29b strategy daemon loop`
7. `feat: add phase29b strategy daemon cli parsing`
8. `feat: add phase29b strategy daemon handlers`
9. `feat: add phase29b strategy daemon service support`
10. `docs: add phase29b strategy daemon guidance`
