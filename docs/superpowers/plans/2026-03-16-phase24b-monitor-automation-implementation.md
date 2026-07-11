# Phase 24B Monitor Automation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Phase 24B monitor automation on top of the current monitor/stop baseline: foreground repeat mode, daemon mode, `systemd --user` service management, persisted monitor config, and deduplicated business event history.

**Architecture:** Keep one shared monitor loop and expose it through two entrypoints: `monitor watchlist --repeat` for foreground use and `monitor daemon run` for background execution. Store runtime settings in a JSON config file, extend the existing monitor SQLite database for event history and dedupe state, and keep `systemd --user` integration as a thin wrapper around that loop instead of mixing service logic into business evaluation.

**Tech Stack:** Rust, clap, tokio, serde/serde_json, sqlx SQLite, chrono, existing `src/monitor/*` service code, existing `src/stop/*` rule evaluation, `systemctl --user`, GitNexus impact analysis, repo hygiene tests.

---

## Preflight

- Read the approved spec section in [docs/superpowers/plans/2026-03-15-phase24-monitor-implementation.md](/opt/claude/quantix-rust/docs/superpowers/plans/2026-03-15-phase24-monitor-implementation.md#L362).
- Use `@superpowers/test-driven-development` for each implementation task and `@superpowers/verification-before-completion` before claiming the phase is done.
- GitNexus is stale in this repo right now. Run `gitnexus analyze` before the first impact-analysis call if the index is still reported as behind `HEAD`.
- Before editing any existing symbol, run `gitnexus_impact` against that exact symbol name and record the risk in the task notes before touching the file.
- Before each commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})` and confirm the diff only affects the intended files and flows.
- In this environment, prefer `CARGO_TARGET_DIR=/tmp/quantix-target` for all `cargo` commands unless `target/` ownership has been fixed.

## File Map

- `src/cli/mod.rs`
  - Extend `MonitorCommands` with repeat/config/daemon/service/event CLI surface.
- `src/cli/tests/monitor.rs`
  - Parser coverage for all new monitor subcommands and invalid combinations.
- `src/core/runtime.rs`
  - Add `QUANTIX_MONITOR_CONFIG_PATH` and expose `CliRuntime.monitor_config_path`.
- `src/monitor/config.rs`
  - Persisted monitor config model plus JSON load/save helpers.
- `src/monitor/models.rs`
  - Add event history and run-mode/read-model types without mixing them into alert storage internals.
- `src/monitor/storage.rs`
  - Extend the SQLite store with monitor event history and trigger-state dedupe persistence.
- `src/monitor/runner.rs`
  - Shared one-iteration and looping monitor execution that composes watchlist snapshot, alerts, stop rules, event persistence, and render/log payloads.
- `src/monitor/systemd.rs`
  - Render the user unit file and wrap `systemctl --user` calls.
- `src/monitor/mod.rs`
  - Export the new config/runner/systemd modules and types.
- `src/cli/handlers.rs`
  - Wire CLI commands to config storage, runner, event listing, and systemd helpers.
- `tests/monitor_config_test.rs`
  - Focused config-file persistence tests.
- `tests/monitor_event_storage_test.rs`
  - Event history/dedupe persistence tests.
- `tests/monitor_runner_test.rs`
  - Shared-loop behavior tests.
- `tests/monitor_systemd_test.rs`
  - Unit rendering and systemctl wrapper tests.
- `README.md`
  - Advertise the new Phase 24B monitor commands and boundary.
- `docs/USER_MANUAL.md`
  - Document the new monitor commands, config path, and deferred capabilities.
- `tests/repo_hygiene_test.rs`
  - Lock the doc boundary for Phase 24B.

## Chunk 1: CLI Surface, Runtime, And Config Core

### Task 1: Extend the `monitor` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/tests/monitor.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the existing monitor CLI symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "MonitorCommands", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_monitor_command", direction: "upstream"})
```

Expected: Low/medium CLI-only blast radius. If GitNexus reports unrelated high-risk flows, stop and narrow the plan before editing.

- [ ] **Step 2: Write the failing parser tests**

Add parser coverage for:
- `quantix monitor watchlist --repeat`
- `quantix monitor watchlist --once --repeat` rejects conflicting flags
- `quantix monitor config show`
- `quantix monitor config set --interval-seconds 15`
- `quantix monitor config set --group core`
- `quantix monitor config set --persist-events false`
- `quantix monitor config set --group core --persist-events true` rejects multiple fields
- `quantix monitor config clear-group`
- `quantix monitor daemon run`
- `quantix monitor service install`
- `quantix monitor service status`
- `quantix monitor event list --limit 10 --code 000001 --type price-alert`

- [ ] **Step 3: Run the parser tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --lib cli::tests::monitor:: -- --nocapture
```

Expected: FAIL because the new monitor subcommands and validations do not exist yet.

- [ ] **Step 4: Implement the minimal CLI definitions**

Add new enums and args in `src/cli/mod.rs`:

```rust
pub enum MonitorCommands {
    Watchlist { once: bool, repeat: bool },
    Alert(MonitorAlertCommands),
    Config(MonitorConfigCommands),
    Daemon(MonitorDaemonCommands),
    Service(MonitorServiceCommands),
    Event(MonitorEventCommands),
}
```

Use clap validation to enforce:
- `watchlist` requires exactly one of `--once` or `--repeat`
- `config set` accepts exactly one mutable field per invocation
- `event list` defaults `limit` to `20`

- [ ] **Step 5: Re-run the parser tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --lib cli::tests::monitor:: -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/cli/mod.rs` and `src/cli/tests/monitor.rs` are affected.

Commit:
```bash
git add src/cli/mod.rs src/cli/tests/monitor.rs
git commit -m "feat: add phase24b monitor cli surface"
```

### Task 2: Add persisted monitor config and runtime path support

**Files:**
- Create: `src/monitor/config.rs`
- Modify: `src/monitor/mod.rs`
- Modify: `src/core/runtime.rs`
- Test: `tests/monitor_config_test.rs`
- Test: `src/core/runtime.rs`

- [ ] **Step 1: Run GitNexus impact analysis for runtime symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "CliRuntime", direction: "upstream"})
```

Expected: Low/medium risk with CLI handlers and runtime tests only.

- [ ] **Step 2: Write the failing runtime and config tests**

Add tests covering:
- `QUANTIX_MONITOR_CONFIG_PATH` override
- default fallback path `~/.quantix/monitor/config.json`
- missing config file creates defaults
- saved config round-trips
- malformed JSON returns a hard error

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test monitor_config -- --nocapture
```

Expected: FAIL because the config module and runtime path do not exist.

- [ ] **Step 4: Implement the monitor config module**

Create `src/monitor/config.rs` with a focused API:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub interval_seconds: u64,
    pub watchlist_group: Option<String>,
    pub persist_events: bool,
    pub max_event_history: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            watchlist_group: None,
            persist_events: true,
            max_event_history: 1000,
        }
    }
}

pub struct JsonMonitorConfigStore {
    path: PathBuf,
}

impl JsonMonitorConfigStore {
    pub fn load_or_create(&self) -> Result<MonitorConfig> {
        if !self.path.exists() {
            let default = MonitorConfig::default();
            self.save(&default)?;
            return Ok(default);
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self, config: &MonitorConfig) -> Result<()> {
        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, serde_json::to_string_pretty(config)?)?;
        std::fs::rename(tmp_path, &self.path)?;
        Ok(())
    }
}
```

- [ ] **Step 5: Extend runtime path resolution**

Add:
- `MONITOR_CONFIG_PATH_ENV`
- `CliRuntime.monitor_config_path`
- `resolve_monitor_config_path()`

Mirror the existing watchlist/trade/risk path style.

- [ ] **Step 6: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test monitor_config -- --nocapture
```

Expected: PASS

- [ ] **Step 7: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only runtime/config files plus focused tests are affected.

Commit:
```bash
git add src/monitor/config.rs src/monitor/mod.rs src/core/runtime.rs tests/monitor_config_test.rs
git commit -m "feat: add phase24b monitor config storage"
```

## Chunk 2: Event Storage And Dedupe Persistence

### Task 3: Extend monitor models and SQLite storage for business events

**Files:**
- Modify: `src/monitor/models.rs`
- Modify: `src/monitor/storage.rs`
- Modify: `src/monitor/mod.rs`
- Test: `tests/monitor_event_storage_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the SQLite monitor store**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "SqliteMonitorAlertStore", direction: "upstream"})
```

Expected: Medium risk limited to monitor CLI and storage tests.

- [ ] **Step 2: Write the failing event-storage tests**

Cover:
- event table auto-creates with the existing alert table
- event history survives reopen
- repeated active trigger writes only one row
- clearing a trigger then re-triggering writes a second row
- `list_events` respects `limit`, `code`, and `type`
- trimming respects `max_event_history`

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test monitor_event_storage -- --nocapture
```

Expected: FAIL because event models and storage APIs do not exist.

- [ ] **Step 4: Add explicit event read models**

Extend `src/monitor/models.rs` with:

```rust
pub enum MonitorEventType { PriceAlert, StopLoss, StopProfit, TrailingStop }
pub enum MonitorRunMode { Foreground, Daemon }

pub struct MonitorEventRow {
    pub id: i64,
    pub event_time: DateTime<Utc>,
    pub event_type: MonitorEventType,
    pub code: String,
    pub price: Option<f64>,
    pub message: String,
    pub source_type: String,
    pub source_key: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub run_mode: MonitorRunMode,
}

pub struct MonitorEventFilter {
    pub limit: usize,
    pub code: Option<String>,
    pub event_type: Option<MonitorEventType>,
}
```

- [ ] **Step 5: Extend `SqliteMonitorAlertStore` with event and trigger-state APIs**

Add focused methods like:

```rust
pub async fn record_event_edge(
    &self,
    source_type: &str,
    source_key: &str,
    is_triggered: bool,
    new_event: Option<NewMonitorEvent>,
    max_event_history: usize,
) -> Result<bool>;

pub async fn list_events(&self, filter: &MonitorEventFilter) -> Result<Vec<MonitorEventRow>>;
```

Implementation rules:
- write a new event only when `is_triggered` transitions from `false` to `true`
- clear the trigger state when the condition becomes false
- keep non-business lifecycle logs out of SQLite

- [ ] **Step 6: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test monitor_event_storage -- --nocapture
```

Expected: PASS

- [ ] **Step 7: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/monitor/models.rs`, `src/monitor/storage.rs`, `src/monitor/mod.rs`, and the new storage test are affected.

Commit:
```bash
git add src/monitor/models.rs src/monitor/storage.rs src/monitor/mod.rs tests/monitor_event_storage_test.rs
git commit -m "feat: add phase24b monitor event storage"
```

## Chunk 3: Shared Runner And CLI Handler Wiring

### Task 4: Build the shared monitor runner

**Files:**
- Create: `src/monitor/runner.rs`
- Modify: `src/monitor/mod.rs`
- Modify: `src/cli/handlers.rs`
- Test: `tests/monitor_runner_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the current monitor handler entrypoint**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "execute_monitor_command_with_stop_store", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "MonitorService", direction: "upstream"})
```

Expected: Medium risk within monitor CLI flows only.

- [ ] **Step 2: Write the failing runner tests**

Cover:
- empty watchlist returns a readable no-data iteration
- partial quote coverage does not abort the iteration
- triggered price alerts persist one business event on first activation
- a still-active trigger does not duplicate event history
- a cleared trigger can fire again later
- stop-rule triggers also produce business events with the right event type

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test monitor_runner_test -- --nocapture
```

Expected: FAIL because `src/monitor/runner.rs` does not exist yet.

- [ ] **Step 4: Implement `src/monitor/runner.rs`**

Use a narrow result model:

```rust
pub struct MonitorIterationOutput {
    pub snapshot: MonitorWatchlistSnapshot,
    pub triggered_stops: Vec<TriggeredStop>,
    pub new_events: Vec<MonitorEventRow>,
}

pub struct MonitorRunner<RW, RQ, AS, SS> {
    monitor_service: MonitorService<RW, RQ, AS>,
    stop_store: SS,
    event_store: SqliteMonitorAlertStore,
}

impl<RW, RQ, AS, SS> MonitorRunner<RW, RQ, AS, SS> {
    pub async fn run_once(
        &self,
        config: &MonitorConfig,
        run_mode: MonitorRunMode,
        now: DateTime<Utc>,
    ) -> Result<MonitorIterationOutput>
}
```

Implementation details:
- reuse `MonitorService::load_watchlist_snapshot()`
- reuse the existing stop-rule evaluation path, but move shared pieces out of handler-local code if needed
- push all edge-trigger writes through the SQLite store extension from Task 3
- keep loop-independent business logic out of `src/cli/handlers.rs`

- [ ] **Step 5: Re-run the runner tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test monitor_runner_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: runner/module files, focused tests, and any small handler extraction only.

Commit:
```bash
git add src/monitor/runner.rs src/monitor/mod.rs src/cli/handlers.rs tests/monitor_runner_test.rs
git commit -m "feat: add phase24b monitor runner"
```

### Task 5: Wire config, event list, repeat mode, and daemon mode into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `run_monitor_command`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_monitor_command", direction: "upstream"})
```

Expected: Monitor command routing only. If unrelated CLI flows appear, keep the changes narrower.

- [ ] **Step 2: Write the failing handler tests**

Add focused tests for:
- `monitor config show` returns defaults or persisted values
- `monitor config set --interval-seconds` updates the saved config
- `monitor config set` rejects multi-field updates
- `monitor event list` returns filtered rows newest-first
- `monitor watchlist --repeat` delegates to the shared runner path
- `monitor daemon run` uses daemon run mode and does not require `systemd` to be present in unit tests

Use handler helpers instead of stdout capture whenever possible.

- [ ] **Step 3: Run the focused handler tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test cli::handlers::tests::test_execute_monitor_ -- --nocapture
```

Expected: FAIL because the new monitor commands are not wired.

- [ ] **Step 4: Implement minimal handler wiring**

Add handler helpers along these lines:

```rust
async fn execute_monitor_config_command(
    cmd: MonitorConfigCommands,
    runtime: &CliRuntime,
) -> Result<MonitorConfigCommandOutput>

async fn execute_monitor_event_command(
    cmd: MonitorEventCommands,
    store: &SqliteMonitorAlertStore,
) -> Result<MonitorEventCommandOutput>

async fn execute_monitor_repeat_command(
    runtime: &CliRuntime,
    runner: &ConfiguredMonitorRunner,
) -> Result<()>

async fn execute_monitor_daemon_command(
    runtime: &CliRuntime,
    runner: &ConfiguredMonitorRunner,
) -> Result<()>
```

Behavior rules:
- `watchlist --repeat` loops until interrupted and prints iteration output in foreground mode
- `daemon run` uses the same runner in daemon mode and writes logs without the foreground snapshot banner noise
- `event list` delegates to the SQLite event query API from Task 3
- config commands load/update/save via `JsonMonitorConfigStore`

- [ ] **Step 5: Re-run the focused handler tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test cli::handlers::tests::test_execute_monitor_ -- --nocapture
```

Expected: PASS, including the existing Phase 24A tests.

- [ ] **Step 6: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only monitor handler/routing files and tests are affected.

Commit:
```bash
git add src/cli/handlers.rs
git commit -m "feat: wire phase24b monitor commands"
```

## Chunk 4: `systemd --user`, Docs, And Final Verification

### Task 6: Add `systemd --user` unit rendering and service wrappers

**Files:**
- Create: `src/monitor/systemd.rs`
- Modify: `src/monitor/mod.rs`
- Modify: `src/cli/handlers.rs`
- Test: `tests/monitor_systemd_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the monitor CLI handler again**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_monitor_command", direction: "upstream"})
```

Expected: Medium risk confined to monitor command dispatch.

- [ ] **Step 2: Write the failing systemd tests**

Cover:
- rendered unit uses `std::env::current_exe()`
- rendered unit executes `monitor daemon run`
- unit rendering includes `Restart=on-failure`
- install/uninstall wrappers target `systemctl --user`
- install does not imply `start` or `enable`
- non-default runtime paths are emitted as `Environment=` lines when needed

- [ ] **Step 3: Run the focused systemd tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test monitor_systemd_test -- --nocapture
```

Expected: FAIL because the systemd helper module does not exist.

- [ ] **Step 4: Implement `src/monitor/systemd.rs`**

Provide:

```rust
pub struct MonitorUserServiceInstaller {
    runtime: CliRuntime,
    executable_path: PathBuf,
}

impl MonitorUserServiceInstaller {
    pub fn render_unit(&self) -> String
    pub fn install(&self) -> Result<()>
    pub fn uninstall(&self) -> Result<()>
    pub fn start(&self) -> Result<()>
    pub fn stop(&self) -> Result<()>
    pub fn status(&self) -> Result<String>
    pub fn enable(&self) -> Result<()>
    pub fn disable(&self) -> Result<()>
}
```

Keep direct `systemctl --user` process execution in this one module, not scattered across handlers.

- [ ] **Step 5: Re-run the focused systemd tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test monitor_systemd_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: systemd helper/module/handler files and focused tests only.

Commit:
```bash
git add src/monitor/systemd.rs src/monitor/mod.rs src/cli/handlers.rs tests/monitor_systemd_test.rs
git commit -m "feat: add phase24b monitor systemd integration"
```

### Task 7: Update docs and hygiene coverage for Phase 24B

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene tests**

Update the Phase 24 expectations so they document:
- `quantix monitor watchlist --repeat`
- `quantix monitor config show`
- `quantix monitor daemon run`
- `quantix monitor service install`
- `quantix monitor event list`
- `QUANTIX_MONITOR_CONFIG_PATH`
- `~/.quantix/monitor/config.json`
- deferred abilities now exclude `--repeat` but still defer desktop notifications

- [ ] **Step 2: Run the hygiene tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test repo_hygiene_test -- --nocapture
```

Expected: FAIL because README and the user manual still describe the old Phase 24A boundary.

- [ ] **Step 3: Update README and the user manual**

Keep the docs aligned with Phase 24B only:
- foreground repeat mode is now supported
- daemon/service management exists for WSL2 `systemd --user`
- runtime config lives in `config.json`
- event history is business-only
- desktop notifications remain deferred

- [ ] **Step 4: Re-run the hygiene tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run GitNexus change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only docs and hygiene tests are affected.

Commit:
```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase24b monitor automation usage"
```

### Task 8: Final regression verification

**Files:**
- No code changes expected

- [ ] **Step 1: Run the full automated test suite**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo test
```

Expected: PASS

- [ ] **Step 2: Run non-destructive CLI smoke checks**

Run:
```bash
tmp_home="$(mktemp -d)"
HOME="$tmp_home" CARGO_TARGET_DIR=/tmp/quantix-target cargo run -- monitor config show
HOME="$tmp_home" CARGO_TARGET_DIR=/tmp/quantix-target cargo run -- monitor event list
HOME="$tmp_home" CARGO_TARGET_DIR=/tmp/quantix-target timeout 3 cargo run -- monitor watchlist --repeat
```

Expected:
- `config show` prints default config
- `event list` prints an empty/readable table on a fresh DB
- `watchlist --repeat` performs at least one readable iteration and exits via `timeout` without panicking

- [ ] **Step 3: Run optional live WSL2 `systemd --user` smoke checks**

Run only if you are comfortable touching your actual user service state:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target cargo run -- monitor service install
systemctl --user cat quantix-monitor.service
CARGO_TARGET_DIR=/tmp/quantix-target cargo run -- monitor service status
CARGO_TARGET_DIR=/tmp/quantix-target cargo run -- monitor service uninstall
```

Expected:
- install writes a valid user unit and reloads the user daemon
- `systemctl --user cat` shows `monitor daemon run`
- uninstall removes the unit cleanly

- [ ] **Step 4: Run GitNexus change detection one last time**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: affected files match the complete Phase 24B monitor automation scope only.

- [ ] **Step 5: Commit the verification pass**

```bash
git add -A
git commit -m "test: verify phase24b monitor automation"
```

## Execution Notes

- Keep the Phase 24B loop logic in `src/monitor/runner.rs`; do not grow `src/cli/handlers.rs` into the business engine.
- Keep `src/monitor/systemd.rs` focused on unit rendering and `systemctl --user` process calls. It should not evaluate quotes, alerts, or stop rules.
- Do not move this work into the experimental `task` subsystem.
- Do not persist daemon lifecycle noise into the business event table.
- Preserve existing `monitor watchlist --once` behavior and existing alert/stop storage paths.
- If a file starts growing too large during implementation, split by responsibility before proceeding to later tasks.
