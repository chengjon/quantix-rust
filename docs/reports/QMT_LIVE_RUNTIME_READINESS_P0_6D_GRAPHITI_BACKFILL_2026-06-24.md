# qmt_live Runtime Readiness P0.6d Graphiti Backfill

Date: 2026-06-24

Status: local Graphiti backfill record required

Branch: `feat/p0-6d-graphiti-backfill`

Base commit: `d3951354f2e0f447b9069a1a1746728a0e940522`

Related PR: `#286`

Related master CI: `28092646658`

## Summary

Graphiti backfill required

After P0.6d was merged and master CI passed, the required Graphiti closeout memory was queued but could not be verified as completed.

Episode:

```text
4fe294c3-712e-49b8-b730-d627bc6d7fe5
```

Group:

```text
quantix_rust_main
```

Observed ingest state after repeated polling:

```text
state=processing
queue_depth=0
attempt_count=1
processed_at=null
last_error=null
last_error_code=null
queued_at=2026-06-24T10:43:15.893561+00:00
started_at=2026-06-24T10:43:15.913370+00:00
```

Because ingest completion could not be verified, this report records the equivalent durable memory locally for later Graphiti backfill.

## Equivalent Memory Summary

P0.6d qmt_live runtime readiness failure-boundary drill closed on 2026-06-24.

PR #286 merged to master as:

```text
d3951354f2e0f447b9069a1a1746728a0e940522
```

P0.6d added:

- `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6D_2026-06-24.md`;
- `docs/reports/evidence/qmt-live-runtime-readiness-20260624/failure-boundary-drill.json`;
- OpenSpec task 4 completion in `openspec/changes/qmt-live-runtime-readiness-p0-6/tasks.md`;
- FUNCTION_TREE and governance closeout for P0.6d.

P0.6d executed only local read-only probes:

- `target/debug/quantix --version`;
- `target/debug/quantix execution qmt status --checklist`;
- `target/debug/quantix safety kill-switch status`.

The qmt status checklist probe returned a fail-closed bridge-unavailable state:

```text
ready=false
failure_category=bridge_unreachable
bridge_reachable=false
readiness=not_ready
```

P0.6d also re-ran focused current-code tests:

- `cargo test --lib test_qmt_live_preflight_report_classifies_fail_closed_categories -- --test-threads=1`;
- `cargo test --lib remains_available_when_kill_switch_enabled -- --test-threads=1`;
- `cargo test --lib kill_switch_enabled -- --test-threads=1`.

These tests passed and verify the current fail-closed contracts for:

- bridge unavailable;
- `qmt.enabled=false`;
- non-live or ambiguous `qmt.mode`;
- missing required `order_submit` capability;
- kill switch enabled mutation blocking;
- qmt preview/query read-only availability while kill switch is enabled.

The `qmt.enabled=false`, non-live or ambiguous `qmt.mode`, missing `order_submit`, and kill-switch runtime drills remain runtime-control deferred because no operator-selected Windows Bridge/miniQMT runtime and no controlled bridge capabilities payload were available.

P0.6d does not claim qmt_live runtime readiness. P0.6b remains:

```text
blocked_by_environment_selection
```

## Boundaries Preserved

P0.6d and this backfill record did not execute or modify:

- qmt_live submit;
- qmt_live cancel;
- broker cancel;
- manual-intervention resolution;
- runtime-store write;
- broker/runtime state mutation;
- runtime source code;
- bridge protocol;
- storage schema;
- response shape;
- `OrderStatus`;
- `ExecutionAdapter`;
- paper execution semantics;
- `.unwrap()` cleanup.

## Verification Already Completed For P0.6d

Local gates:

```text
node JSON parse for failure-boundary-drill.json
cargo fmt --check
openspec validate qmt-live-runtime-readiness-p0-6 --strict
git diff --check
function-tree validate
function-tree gate --verbose
function-tree scope-check project-governance P0.6d
GitNexus detect_changes: LOW / 0 affected processes
```

Remote gates:

```text
PR #286 CI passed.
master CI run 28092646658 passed.
```

## Backfill Requirement

When Graphiti ingest is healthy and verifiable, backfill this summary into:

```text
group_id=quantix_rust_main
```

The existing unverified episode UUID should be checked first:

```text
4fe294c3-712e-49b8-b730-d627bc6d7fe5
```

If it remains stuck, failed, or unsearchable, add a fresh compact memory using the equivalent summary above and verify `get_ingest_status` reaches `completed`.

