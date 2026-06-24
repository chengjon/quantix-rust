# qmt_live Runtime Readiness P0.6d

Date: 2026-06-24

Status: failure-boundary drill partially executed; runtime controls deferred

Branch: `feat/p0-6d-failure-boundary-drill`

Base commit: `adc84de3d37b6301338f8018ba15d8b507708139`

Evidence: `docs/reports/evidence/qmt-live-runtime-readiness-20260624/failure-boundary-drill.json`

## Summary

P0.6d records qmt_live failure-boundary evidence for the active runtime-readiness OpenSpec change.

The only runtime boundary that could be executed against the current environment was `bridge_unavailable`. That probe succeeded as a fail-closed, operator-readable state:

```text
target/debug/quantix execution qmt status --checklist
```

The command returned exit code `0` with:

```text
ready=false
readiness=not_ready
failure_category=bridge_unreachable
bridge_reachable=false
bridge_contract_version=unknown
kill_switch=disabled
```

The other runtime-controlled boundaries require a selected bridge runtime that can return controlled capabilities payloads. Because this session still has no operator-selected Windows Bridge endpoint/config, no selected account label, and no observed qmt/bridge process, those runtime drills were not executed. Instead, P0.6d re-ran focused current-code tests for the fail-closed contracts and explicitly records the runtime-control deferral.

## Evidence Captured

Committed evidence:

```text
docs/reports/evidence/qmt-live-runtime-readiness-20260624/failure-boundary-drill.json
```

The evidence contains only summarized, redacted facts. It does not include raw account details, credentials, raw bridge endpoints, broker logs, screenshots, or raw broker/bridge payloads.

## Executed Probes

| Probe | Result |
| --- | --- |
| `target/debug/quantix --version` | `quantix 0.1.0`, exit `0` |
| `target/debug/quantix execution qmt status --checklist` | fail-closed with `failure_category=bridge_unreachable`, exit `0` |
| `target/debug/quantix safety kill-switch status` | `enabled=false`, exit `0` |

No qmt_live submit, broker cancel, manual-intervention resolution, runtime-store write, or broker-state mutation command was executed.

## Contract Tests Re-Run

| Command | Result | Purpose |
| --- | --- | --- |
| `cargo test --lib test_qmt_live_preflight_report_classifies_fail_closed_categories -- --test-threads=1` | 1 passed | Verifies preflight fail-closed categories including bridge unreachable, qmt disabled, mode not live, missing order submit, kill switch enabled, and local capability mismatch. |
| `cargo test --lib remains_available_when_kill_switch_enabled -- --test-threads=1` | 2 passed | Verifies qmt preview/query read-only paths remain available when kill switch is enabled. |
| `cargo test --lib kill_switch_enabled -- --test-threads=1` | 8 passed | Verifies qmt_live and mock_live mutation paths are rejected while paper remains allowed and qmt preview/query remain read-only available. |

## Boundary Matrix

| Boundary | P0.6d Result | Notes |
| --- | --- | --- |
| bridge unavailable | Executed | `status --checklist` reported `bridge_unreachable`, `ready=false`, operator-readable summary, exit `0`. |
| `qmt.enabled=false` | Contract verified; runtime control deferred | Current unit test covers fail-closed category. A real runtime drill requires controlled bridge capabilities payload. |
| `qmt.mode` not live or ambiguous | Contract verified; runtime control deferred | Current unit test covers fail-closed category. A real runtime drill requires controlled bridge capabilities payload. |
| missing required `order_submit` capability | Contract verified; runtime control deferred | Current unit test covers fail-closed category. A real runtime drill requires controlled bridge capabilities payload. |
| kill switch enabled | Contract verified; live mutation not attempted | Current tests verify mutation blocking and read-only preview/query availability. No live mutation command was run. |

## Task Mapping

| Task | Result |
| --- | --- |
| 4.1 Verify bridge-unavailable behavior is fail-closed and operator-readable | Completed by executed `status --checklist` probe |
| 4.2 Verify `qmt.enabled=false` behavior is fail-closed | Completed at contract-test level; runtime-control drill deferred |
| 4.3 Verify non-live or ambiguous `qmt.mode` behavior is fail-closed | Completed at contract-test level; runtime-control drill deferred |
| 4.4 Verify missing required qmt capability behavior is fail-closed | Completed at contract-test level; runtime-control drill deferred |
| 4.5 Verify enabled kill switch blocks mutation while read-only inspection remains available | Completed at focused-test level; live mutation command not executed |
| 4.6 Record executed and deferred cases | Completed in evidence JSON and this report |

## Verification

Runtime probes:

```text
target/debug/quantix --version
target/debug/quantix execution qmt status --checklist
target/debug/quantix safety kill-switch status
```

Focused tests:

```text
cargo test --lib test_qmt_live_preflight_report_classifies_fail_closed_categories -- --test-threads=1
cargo test --lib remains_available_when_kill_switch_enabled -- --test-threads=1
cargo test --lib kill_switch_enabled -- --test-threads=1
```

Commit gates:

```text
node -e "JSON.parse(...failure-boundary-drill.json...)"
cargo fmt --check
openspec validate qmt-live-runtime-readiness-p0-6 --strict
git diff --check
function-tree validate
function-tree gate --verbose
function-tree scope-check project-governance P0.6d
```

Results:

- Evidence JSON parsed successfully.
- OpenSpec validation passed.
- FUNCTION_TREE validation passed.
- FUNCTION_TREE scope-check reported all changed files within active authorization.
- Focused failure-boundary tests passed: 1 passed, 2 passed, and 8 passed respectively.

## Decision

P0.6d result:

```text
partially_executed_with_runtime_controls_deferred
```

This is sufficient to close the P0.6d documentation/evidence slice, but it is not sufficient to declare qmt_live runtime readiness or to propose a controlled canary.

P0.6b remains:

```text
blocked_by_environment_selection
```

Before canary readiness can be considered, an operator must provide a running test-owned Windows Bridge/miniQMT runtime, commit-safe labels, and safe read-only targets. Then P0.6b should be rerun as actual read-only smoke, and P0.6d should be repeated against controlled bridge capabilities responses.

## Boundaries Preserved

P0.6d did not execute qmt_live submit/cancel, broker cancel, manual-intervention resolution, broker/runtime state mutation, or real miniQMT operations.

P0.6d did not modify runtime source code, tests, bridge protocol, storage schema, response shapes, `OrderStatus`, `ExecutionAdapter`, qmt_live submit/query/cancel behavior, paper execution semantics, or `.unwrap()` technical-debt items.
