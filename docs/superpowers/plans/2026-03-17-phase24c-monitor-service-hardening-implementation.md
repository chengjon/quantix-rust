# Phase 24C Monitor Service Hardening Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the existing Phase 24B WSL2 `systemd --user` integration so installed services use a stable wrapper script and dedicated service config instead of transient build paths.

**Architecture:** Add a dedicated `service.json` for the stable `quantix` binary path, make `monitor service install` validate that path, generate a wrapper script under `~/.local/bin`, and render the user unit to point at that wrapper. Keep monitor runtime settings in the existing `config.json`; only service-specific binary-path concerns move into the new config layer.

**Tech Stack:** Rust, clap, serde/serde_json, std::fs, std::process::Command, existing monitor/systemd code, `systemctl --user`, GitNexus impact analysis, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-17-phase24c-monitor-service-hardening-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-17-phase24c-monitor-service-hardening-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the task is done or deleting any temporary service files.
- Work in an isolated worktree before touching code.
- Before editing an existing symbol, run `gitnexus_impact` for that symbol and record whether the risk is acceptable.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- In this repo, prefer `CARGO_TARGET_DIR=/tmp/quantix-target-phase24c` for test/build commands.

## File Map

- `src/cli/mod.rs`
  - Extend the monitor CLI with `service-config`.
- `src/cli/tests/monitor.rs`
  - Parser coverage for `monitor service-config show` and `set`.
- `src/monitor/service_config.rs`
  - New service-config model, persistence, and path validation.
- `src/monitor/systemd.rs`
  - Extend the existing installer with wrapper-script rendering, unit rendering against the wrapper, stricter install/uninstall behavior, and structured status summaries.
- `src/monitor/mod.rs`
  - Export `service_config`.
- `src/cli/handlers.rs`
  - Wire `service-config` commands and adapt service install/uninstall/status output.
- `tests/monitor_service_config_test.rs`
  - Focused persistence and validation tests for `service.json`.
- `tests/monitor_systemd_test.rs`
  - Wrapper/unit rendering, uninstall safety, and status-summary tests.
- `README.md`
  - Document the hardened service flow and `service-config`.
- `docs/USER_MANUAL.md`
  - Document the new commands and stable service setup.
- `tests/repo_hygiene_test.rs`
  - Lock the doc boundary for Phase 24C.

## Chunk 1: CLI Surface And Service Config Core

### Task 1: Add the `monitor service-config` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/tests/monitor.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the monitor CLI symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "MonitorCommands", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "MonitorServiceCommands", direction: "upstream"})
```

Expected: Low/medium CLI-only risk. If the blast radius includes unrelated command families, stop and narrow the planned edits.

- [ ] **Step 2: Write the failing parser tests**

Add parser coverage for:
- `quantix monitor service-config show`
- `quantix monitor service-config set --quantix-bin /abs/path/to/quantix`
- `quantix monitor service-config set` rejects missing `--quantix-bin`
- `quantix monitor service-config set --quantix-bin relative/path` still parses; path validation belongs in handler/service-config tests

- [ ] **Step 3: Run the parser tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --lib cli::tests::monitor:: -- --nocapture
```

Expected: FAIL because `service-config` commands do not exist yet.

- [ ] **Step 4: Implement the minimal CLI definitions**

Add:

```rust
pub enum MonitorCommands {
    Watchlist { once: bool, repeat: bool },
    Alert(MonitorAlertCommands),
    Config(MonitorConfigCommands),
    Daemon(MonitorDaemonCommands),
    Service(MonitorServiceCommands),
    Event(MonitorEventCommands),
    ServiceConfig(MonitorServiceConfigCommands),
}

pub enum MonitorServiceConfigCommands {
    Show,
    Set { quantix_bin: String },
}
```

Keep argument validation minimal here. Path validation belongs in the service-config module.

- [ ] **Step 5: Re-run the parser tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --lib cli::tests::monitor:: -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/cli/mod.rs` and `src/cli/tests/monitor.rs` are affected.

Commit:
```bash
git add src/cli/mod.rs src/cli/tests/monitor.rs
git commit -m "feat: add phase24c monitor service-config cli"
```

### Task 2: Add dedicated service config persistence and validation

**Files:**
- Create: `src/monitor/service_config.rs`
- Modify: `src/monitor/mod.rs`
- Create: `tests/monitor_service_config_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the current systemd module**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "MonitorUserServiceInstaller", direction: "upstream"})
```

Expected: Low/medium risk centered on monitor service code.

- [ ] **Step 2: Write the failing service-config tests**

Cover:
- loading a missing `service.json` returns a readable “not configured” error
- saving and reloading an absolute binary path round-trips
- validation rejects a relative path
- validation rejects a missing binary path
- validation rejects a non-executable file

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test monitor_service_config_test -- --nocapture
```

Expected: FAIL because `service_config.rs` does not exist.

- [ ] **Step 4: Implement `src/monitor/service_config.rs`**

Add a focused model:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorServiceConfig {
    pub quantix_bin_path: PathBuf,
}

pub struct JsonMonitorServiceConfigStore {
    path: PathBuf,
}

impl JsonMonitorServiceConfigStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self
    pub fn load(&self) -> Result<MonitorServiceConfig>
    pub fn save(&self, config: &MonitorServiceConfig) -> Result<()>
    pub fn validate(config: &MonitorServiceConfig) -> Result<()>
}
```

Keep the default path logic in this module, not in the general monitor runtime config.

- [ ] **Step 5: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test monitor_service_config_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only the new service-config files plus `src/monitor/mod.rs` are affected.

Commit:
```bash
git add src/monitor/service_config.rs src/monitor/mod.rs tests/monitor_service_config_test.rs
git commit -m "feat: add phase24c monitor service config"
```

## Chunk 2: Wrapper Script And Systemd Hardening

### Task 3: Harden `src/monitor/systemd.rs` around wrapper scripts and strict install semantics

**Files:**
- Modify: `src/monitor/systemd.rs`
- Modify: `tests/monitor_systemd_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for `MonitorUserServiceInstaller`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "MonitorUserServiceInstaller", direction: "upstream"})
```

Expected: Low/medium risk. If it comes back HIGH/CRITICAL, explicitly review all callers before editing.

- [ ] **Step 2: Write the failing systemd hardening tests**

Cover:
- `render_unit()` points to the wrapper script, not the binary directly
- wrapper script content runs the configured `quantix` binary with `monitor daemon run`
- install fails if the configured binary is relative, missing, or non-executable
- uninstall fails when the service is still active
- `status` returns a structured summary object/string containing installed/enabled/active/unit_path/wrapper_path/quantix_bin_path
- install still issues `systemctl --user daemon-reload`

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test monitor_systemd_test -- --nocapture
```

Expected: FAIL because the current module still uses direct `current_exe()` semantics and lacks uninstall safety/summary status.

- [ ] **Step 4: Refactor `MonitorUserServiceInstaller`**

Update the installer so it accepts:
- `CliRuntime`
- `MonitorServiceConfig`
- wrapper path
- unit path

Add focused helpers:

```rust
pub fn wrapper_path(&self) -> PathBuf
pub fn render_wrapper_script(&self) -> String
pub fn render_unit(&self) -> String
pub fn status_summary(&self) -> Result<MonitorServiceStatusSummary>
```

Install flow:
1. validate service config
2. write wrapper script
3. write unit
4. `daemon-reload`

Uninstall flow:
1. check active state
2. fail if active
3. remove unit + wrapper
4. `daemon-reload`

- [ ] **Step 5: Re-run the focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test monitor_systemd_test -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/monitor/systemd.rs` and `tests/monitor_systemd_test.rs` are affected.

Commit:
```bash
git add src/monitor/systemd.rs tests/monitor_systemd_test.rs
git commit -m "feat: harden phase24c monitor systemd integration"
```

## Chunk 3: Handler Wiring And User-Facing Output

### Task 4: Wire `service-config` and hardened service behavior into the monitor handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the monitor handler entrypoint**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "run_monitor_command", direction: "upstream"})
```

Expected: Low/medium monitor-only risk. If the large-file graph returns HIGH due to file-level granularity, keep changes restricted to monitor-only helper functions and record that rationale.

- [ ] **Step 2: Write the failing handler tests**

Add focused tests for:
- `service-config show` returns the saved binary path
- `service-config set --quantix-bin` persists the path
- `service-config set` rejects invalid paths via the new validation layer
- `service install` uses service config + installer
- `service uninstall` surfaces the “stop first” error when active
- `service status` prints/returns the structured summary

Prefer dedicated helper tests instead of shelling out or capturing stdout directly.

- [ ] **Step 3: Run the focused handler tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test cli::handlers::tests::test_execute_monitor_ -- --nocapture
```

Expected: FAIL because handlers do not yet know about `service-config` or hardened install semantics.

- [ ] **Step 4: Implement minimal handler wiring**

Add helpers along these lines:

```rust
fn execute_monitor_service_config_command_with_store(
    cmd: MonitorServiceConfigCommands,
    store: &JsonMonitorServiceConfigStore,
) -> Result<MonitorCommandOutput>

fn execute_monitor_service_command(
    cmd: MonitorServiceCommands,
    installer: &MonitorUserServiceInstaller,
) -> Result<MonitorCommandOutput>

fn print_monitor_service_status_summary(
    summary: &MonitorServiceStatusSummary,
)
```

Rules:
- `service-config set` persists only `quantix_bin_path`
- `service install` loads and validates `service.json` before any file writes
- `service status` outputs structured summary first
- do not change monitor loop behavior in this task

- [ ] **Step 5: Re-run the focused handler tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test cli::handlers::tests::test_execute_monitor_ -- --nocapture
```

Expected: PASS, including existing Phase 24B handler coverage.

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: only `src/cli/handlers.rs` should be affected at this step.

Commit:
```bash
git add src/cli/handlers.rs
git commit -m "feat: wire phase24c monitor service hardening"
```

## Chunk 4: Docs And Final Verification

### Task 5: Update docs and hygiene coverage for Phase 24C

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene tests**

Update Phase 24 expectations to include:
- `monitor service-config show`
- `monitor service-config set --quantix-bin`
- stable wrapper-script language
- dedicated `service.json`
- uninstall requiring the service to be stopped first

- [ ] **Step 2: Run the hygiene tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test repo_hygiene_test -- --nocapture
```

Expected: FAIL because docs still describe the older 24B service behavior.

- [ ] **Step 3: Update README and user manual**

Document:
- `service.json`
- wrapper script path
- stable binary-path requirement
- strict install/uninstall semantics
- `systemd --user` scope remains WSL2/Linux only

- [ ] **Step 4: Re-run the hygiene tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: docs + hygiene tests only.

Commit:
```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase24c monitor service guidance"
```

### Task 6: Final verification

**Files:**
- No code changes expected

- [ ] **Step 1: Run the full automated suite**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo test
```

Expected: PASS

- [ ] **Step 2: Run no-risk CLI smoke checks**

Run:
```bash
tmp_home="$(mktemp -d)"
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service-config show
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service-config set --quantix-bin /bin/echo
HOME="$tmp_home" RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service status
```

Expected:
- show reports “not configured” before set
- set accepts `/bin/echo`
- status reports installed/enabled/active summary even when service is not installed

- [ ] **Step 3: Run optional live WSL2 `systemd --user` smoke checks**

Run only if it is acceptable to touch the actual user service state:
```bash
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service-config set --quantix-bin /absolute/path/to/quantix
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service install
systemctl --user cat quantix-monitor.service
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service start
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service status
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service stop
RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo CARGO_TARGET_DIR=/tmp/quantix-target-phase24c cargo run -- monitor service uninstall
```

Expected:
- unit points to wrapper script
- service starts successfully
- status shows installed/enabled/active summary
- uninstall succeeds only after stop

- [ ] **Step 4: Run change detection and commit the verification pass**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected: affected files match the complete Phase 24C hardening scope only.

Commit:
```bash
git add -A
git commit -m "test: verify phase24c monitor service hardening"
```

## Execution Notes

- Keep service-specific binary-path concerns in `service_config.rs`; do not re-open `config.json` scope.
- Keep wrapper-script generation in `systemd.rs`; do not spread shell-template logic into handlers.
- Keep uninstall safety strict. This phase should refuse bad states instead of trying to guess the user’s intent.
- Avoid unrelated refactors inside `src/cli/handlers.rs`; that file is already large and GitNexus tends to over-report blast radius there.
