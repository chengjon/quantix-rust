# Mock Policy Github Rebuild Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the validated mock-policy behavior directly on top of `github/master` with minimal semantic changes and no replay of unrelated local history.

**Architecture:** Keep the old `github/master` file layout intact and patch only the existing strategy handler, execution daemon, risk status path, and notification log sender. Use focused regression tests to drive each behavior change before modifying production code.

**Tech Stack:** Rust, Tokio async IO, existing CLI handler layer, existing execution daemon/runtime store, existing risk service and models, cargo test.

---

### Task 1: Lock `strategy request execute` away from `qmt_live`

**Files:**
- Modify: `src/cli/handlers/mod.rs`
- Test: `src/cli/handlers/tests/mod.rs`
- Test: `src/cli/tests/strategy.rs`

- [ ] Add a failing regression test that proves direct `strategy request execute` currently allows or routes `qmt_live` requests through the generic execution path.
- [ ] Run the focused test command and verify the failure is caused by missing `qmt_live` guard behavior, not by test setup.
- [ ] Implement the minimal guard in the strategy request execution path so `qmt_live` requests return a clear manual-bridge error instead of executing.
- [ ] Re-run the focused handler and CLI parsing tests until they pass.

### Task 2: Reject pending `qmt_live` in daemon auto-consume

**Files:**
- Modify: `src/execution/daemon.rs`
- Test: `tests/execution_daemon_test.rs`
- Test: `src/cli/handlers/tests/mod.rs`

- [ ] Add a failing regression test for daemon execution that proves pending `qmt_live` requests are auto-consumed today when they should be rejected with manual-bridge guidance.
- [ ] Run the focused daemon test and verify it fails for the expected behavior gap.
- [ ] Implement the minimal daemon-side rejection branch for `qmt_live`, keeping existing `paper` and `mock_live` flows unchanged.
- [ ] Re-run the focused daemon tests and any touched handler tests until they pass.

### Task 3: Preserve persisted semantics for `risk --source live_import status`

**Files:**
- Modify: `src/cli/handlers/risk.rs`
- Modify: `src/risk/service.rs`
- Modify: `src/risk/models.rs` if required
- Test: `src/cli/tests/risk.rs`
- Test: `src/cli/handlers/tests/mod.rs` if helper coverage is needed

- [ ] Add a failing regression test that proves `live_import` status currently drops persisted lock semantics such as `manual_release_active`, lock source, or trigger metadata.
- [ ] Run the focused risk test and verify the failure comes from status reconstruction semantics.
- [ ] Implement the smallest change set needed to reuse persisted risk state semantics for `live_import` status while keeping the current storage model intact.
- [ ] Re-run the focused risk tests until they pass.

### Task 4: Harden notification log sender IO semantics

**Files:**
- Modify: `src/monitoring/notification.rs`
- Test: `src/monitoring/notification.rs` existing test module, or a nearby focused notification test file if needed

- [ ] Add failing regression coverage for the current log sender behavior: missing parent directories, swallowed open/write errors, and no explicit flush before success.
- [ ] Run the focused notification tests and confirm the expected failures.
- [ ] Implement the minimum IO hardening: create parent dirs, propagate open/write errors, and flush explicitly before returning `Ok(())`.
- [ ] Re-run the focused notification tests until they pass.

### Task 5: Verification And Baseline Accounting

**Files:**
- Modify: `CHANGELOG.md` only if this branch ends up intentionally documenting completed behavior
- No required doc edits for baseline hygiene failures in this scoped rebuild

- [ ] Run the targeted suites covering strategy request, daemon, risk, and notification rebuild behavior.
- [ ] Run `cargo test -q` and record which failures, if any, remain.
- [ ] Distinguish any remaining failures between newly introduced regressions and the known baseline `repo_hygiene_test` failures from `origin/master`.
- [ ] Capture the final branch state and verification results for handoff memory.
