---
status: active
trigger: "Investigate reported `risk rule set` timeout 124 under PM2 and timeout-based harnesses"
created: 2026-04-17T00:00:00Z
updated: 2026-04-17T09:42:29Z
---

## Current Focus
Determine whether `risk rule set` timeouts are caused by risk-rule execution logic, persistence, or by the cargo/build path used by the test harness.

## Resolution
root_cause: The reproduced `124` timeouts are caused by the harness execution path, not by `RiskService::set_rule`, rule-type/value handling, or risk JSON persistence.

current_conclusion:
1. Direct execution of an already-built `quantix` binary completes successfully within 5 seconds, persists the rule, and appends a `rule_set` event.
2. `cargo run` under `timeout 5` can time out before the command executes because it spends the timeout budget in cargo startup, dependency compilation, crate compilation, or cargo lock waits.
3. Earlier cargo-path confusion was amplified by a separate compile blocker in backtest CLI wiring; that blocker has now been removed and isolated `cargo build --bin quantix --offline` succeeds.
4. Even after compilation was recovered, the direct-binary path still succeeds while both warm and cold `cargo run` paths still time out under 5 seconds on the current worktree.

impact:
- Do not treat `timeout 5 cargo run -- risk rule set ...` as evidence of a risk-module defect.
- Use an already-built binary for validating risk command behavior.
- If cargo-path behavior itself is under test, label it explicitly as cargo startup/build-path verification rather than risk-rule logic verification.

## Local Memory Backfill Notes

### 2026-04-17 risk timeout checkpoint

Graphiti writes were queued during this investigation, but multiple new episodes remained in `processing` during the session instead of reaching `completed`:
- `3d0159da-f062-4ff3-b361-70c1656cccb7`
- `af707947-4e54-4029-bc12-8292fe28c4fd`
- `3642d5f5-b2e3-4ce4-97b6-5b4d6a9b8263`
- `0a3cd0af-971a-4852-b90f-7304681f064d`

This file is the local source-of-truth checkpoint for the session conclusions until Graphiti ingest is confirmed completed.

Graphiti backfill required

## Verification

Successful direct binary verification:
- `timeout 5 env QUANTIX_RISK_PATH=/tmp/quantix-risk-direct-current.json QUANTIX_TRADE_PATH=/tmp/quantix-trade-direct-current.json /tmp/quantix-target-verify/debug/quantix risk rule set --type daily-loss-limit --value 50000`
- Result: success message returned immediately
- Persisted store: `/tmp/quantix-risk-direct-current.json`
- Persisted event: `rule_set` with detail `daily-loss-limit = 50000`

Successful isolated compile verification:
- `env -C /opt/claude/quantix-rust CARGO_TARGET_DIR=/tmp/quantix-target-verify cargo build --bin quantix --offline`
- Result: exited `0`
- Duration: about 5m50s

Timed-out cargo-path verification on the current worktree:
- Warm target:
  - `env -C /opt/claude/quantix-rust timeout 5 env CARGO_TARGET_DIR=/tmp/quantix-target-verify QUANTIX_RISK_PATH=/tmp/quantix-risk-cargo-warm-current.json QUANTIX_TRADE_PATH=/tmp/quantix-trade-cargo-warm-current.json cargo run --offline -- risk rule set --type volatility-limit --value 4%`
  - Result: exit code `124`
  - Last observed output: `Blocking waiting for file lock on package cache`, then `Compiling quantix-cli v0.1.0`
- Fresh target:
  - `env -C /opt/claude/quantix-rust timeout 5 env CARGO_TARGET_DIR=/tmp/quantix-target-fresh-current QUANTIX_RISK_PATH=/tmp/quantix-risk-cargo-cold-current.json QUANTIX_TRADE_PATH=/tmp/quantix-trade-cargo-cold-current.json cargo run --offline -- risk rule set --type position-limit --value 20%`
  - Result: exit code `124`
  - Last observed output: dependency compilation from scratch

## Evidence

- timestamp: 2026-04-17T08:33:00Z
  checked: direct `target/debug/quantix risk rule set` runs for multiple rule types
  found: `daily-loss-limit`, `volatility-limit`, and `position-limit` all completed and persisted
  implication: risk-rule execution and JSON persistence are functional when command execution is actually reached

- timestamp: 2026-04-17T08:47:00Z
  checked: PM2 reconstruction with binary path and cold cargo path
  found: binary PM2 task exited `0`; cold cargo PM2 task exited `124`
  implication: PM2 itself is not the trigger; the execution path matters

- timestamp: 2026-04-17T09:00:00Z
  checked: cargo build and cargo run diagnostics after continuing investigation
  found: cargo output showed `Blocking waiting for file lock on package cache` and `Blocking waiting for file lock on build directory`
  implication: lock contention alone can exhaust a 5 second timeout before command execution

- timestamp: 2026-04-17T09:20:00Z
  checked: current worktree compile status
  found: compile errors traced to backtest CLI wiring, including `BacktestCommands` import path and an invalid `.await` on `show_backtest_report`
  implication: cargo-path harnesses were also sensitive to unrelated compile health during the investigation

- timestamp: 2026-04-17T09:37:00Z
  checked: isolated build with `CARGO_TARGET_DIR=/tmp/quantix-target-verify`
  found: `cargo build --bin quantix --offline` completed successfully
  implication: repo compile failure is no longer an active confounder for current conclusions

- timestamp: 2026-04-17T09:41:00Z
  checked: current-worktree binary vs warm cargo vs cold cargo comparison
  found: direct binary succeeded and persisted; warm cargo timed out at cargo lock/quantix compile stage; cold cargo timed out during dependency compilation
  implication: even after compile recovery, timeout reproduction still points to cargo startup/build-path behavior rather than risk logic

## Files Changed During Investigation
- `src/account/registry.rs`
- `src/account/router.rs`
- `src/cli/handlers/account.rs`
- `src/cli/handlers/mod.rs`

## Follow-up
1. Update any PM2 or shell-based verification workflow to prefer a prebuilt binary for risk command checks.
2. If a timeout-based cargo harness must remain, raise timeout thresholds substantially or separate compile/build warmup from command execution timing.
3. Optionally backfill this checkpoint into a long-lived implementation plan or review doc once Graphiti ingest completes.
