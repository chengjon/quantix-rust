---
status: active
created: 2026-04-17T00:00:00Z
updated: 2026-04-17T11:15:00Z
---

## Scope

This handoff summarizes the 2026-04-17 investigation into reported `risk rule set` timeout `124` failures and the follow-up workflow changes made to prevent future misclassification.

## Final Conclusion

- `risk rule set` business logic is not the reproduced timeout root cause.
- Direct execution of a built `quantix` binary succeeds, persists rules, and appends `rule_set` events.
- `timeout 5 cargo run -- ...` can time out before command execution because the timeout budget is consumed by cargo lock waits, dependency compilation, or crate compilation.
- This remained true after the worktree was restored to a compilable state.

## Verified Outcomes

- Direct binary success was confirmed with:
  - `/tmp/quantix-target-verify/debug/quantix risk rule set --type daily-loss-limit --value 50000`
- Rule persistence and event logging were confirmed in:
  - `/tmp/quantix-risk-direct-current.json`
- Isolated compile recovery was confirmed with:
  - `env -C /opt/claude/quantix-rust CARGO_TARGET_DIR=/tmp/quantix-target-verify cargo build --bin quantix --offline`
- Warm and cold cargo paths still timed out under `timeout 5` on the current worktree.

## Code And Workflow Changes

- Account validation fixes:
  - reject non-positive account capital
  - exclude non-positive-capital accounts from order splitting
- CLI/build cleanup:
  - recover backtest CLI wiring so the worktree compiles again
- Verification workflow changes:
  - `scripts/verify_features.sh` now builds `quantix` once and runs smoke checks through the built binary
  - smoke checks are split into local binary checks and external dependency checks
  - `tests/verify_features_script_test.rs` was updated and passed
- Documentation updates:
  - `docs/MANUAL_TESTING_GUIDE.md`
  - `docs/USER_MANUAL.md`
  - both now state that command behavior should be verified with the built binary, not timeout-constrained `cargo run`

## Primary Local References

- Detailed debug checkpoint:
  - `.planning/debug/2026-04-17-risk-rule-timeout-investigation.md`
- This handoff:
  - `.planning/debug/2026-04-17-risk-timeout-handoff.md`

## Recommended Next Step

- If more validation is needed, use the built binary for risk command behavior checks.
- Treat any future `timeout 5 cargo run -- risk ...` failures as cargo-path diagnostics unless a binary-path reproduction also fails.

Graphiti backfill required
