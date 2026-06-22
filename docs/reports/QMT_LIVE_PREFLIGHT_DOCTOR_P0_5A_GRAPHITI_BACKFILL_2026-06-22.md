# qmt_live Preflight Doctor P0.5a Graphiti Backfill

Date: 2026-06-22

Status: Graphiti ingest failed

Group: `quantix_rust_main`

Episode UUID: `319affab-a8bc-466a-8c69-a57a103335e3`

Error: `Request timed out.`

Graphiti backfill required.

## Summary To Backfill

P0.5a qmt_live preflight doctor closed. The slice added a read-only qmt_live preflight report to `quantix execution qmt status --checklist` in `src/cli/handlers/execution_handler.rs`, without adding a new `doctor` command.

The report classifies readiness across:

- bridge reachability;
- `qmt.enabled`;
- `qmt.mode=live`;
- `qmt.supports` containing `order_submit`;
- local qmt_live broker-owned capability semantics;
- execution kill-switch state.

The fail-closed categories are:

- `bridge_unreachable`
- `qmt_capability_missing`
- `qmt_disabled`
- `qmt_mode_not_live`
- `qmt_order_submit_missing`
- `qmt_live_capability_mismatch`
- `kill_switch_enabled`

Preserved boundaries:

- no submit/cancel/query mutation;
- no runtime store or broker writes;
- no bridge protocol or response shape changes;
- no storage schema changes;
- no `OrderStatus` changes;
- no `ExecutionAdapter` changes;
- no `paper_immediate` or `paper_sim_lifecycle` behavior changes;
- no `.unwrap()` cleanup resumed.

Verification passed:

- TDD RED/GREEN for preflight tests in `src/cli/handlers/tests/strategy_bridge.rs`;
- `cargo fmt --check`;
- `cargo clippy -- -D warnings`;
- `cargo test`;
- `git diff --check`;
- `npx openspec validate qmt-live-operational-safety-p0-5 --strict`;
- Function Tree scope-check/gate/validate;
- GitNexus staged `detect_changes`: MEDIUM, 9 changed files, 4 affected processes, expected qmt_live handler scope;
- PR #268 CI success;
- squash merge commit `7fb19c487d424e89e5c5facf44e73d2be0d0d9f7`;
- master CI run `27950180753` success.

P0.5b-P0.5e remain pending.
