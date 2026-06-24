# qmt_live Runtime Readiness P0.6e Decision Report

Date: 2026-06-25

Status: `blocked_by_environment`

Branch: `feat/p0-6e-readiness-decision`

OpenSpec: `openspec/changes/qmt-live-runtime-readiness-p0-6`

## Decision

P0.6 readiness decision:

```text
blocked_by_environment
```

Quantix must not start a qmt_live canary from the current evidence set.

P0.6a through P0.6d established that the local safety, redaction, and fail-closed evidence framework is in place. The missing item is not another local code or documentation probe. The missing item is an operator-selected miniQMT Windows Bridge runtime with commit-safe environment labels and read-only execution approval.

Until that runtime exists, qmt_live runtime readiness remains blocked by environment and the qmt_live runtime-readiness line is archived as maintenance-only.

## Evidence Used

| Slice | Evidence | Result |
| --- | --- | --- |
| P0.6a environment inventory | `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6A_2026-06-24.md`; `docs/reports/evidence/qmt-live-runtime-readiness-20260624/environment-inventory.json` | Local binary and paths were present, local kill switch was disabled, but no operator-selected bridge endpoint/config/account label or observed qmt/bridge process existed. |
| P0.6b read-only smoke | `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6B_2026-06-24.md`; `docs/reports/evidence/qmt-live-runtime-readiness-20260624/read-only-smoke.json` | Local read-only commands ran, but bridge-backed qmt status/preview/query/audit/manual-interventions smoke was intentionally skipped because no selected runtime existed. |
| P0.6c redacted evidence package | `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6C_2026-06-24.md`; `docs/reports/evidence/qmt-live-runtime-readiness-20260624/README.md`; `docs/reports/evidence/qmt-live-runtime-readiness-20260624/evidence.template.json` | Evidence redaction and no-mutation attestations were standardized for future runtime smoke. |
| P0.6d failure-boundary drill | `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6D_2026-06-24.md`; `docs/reports/evidence/qmt-live-runtime-readiness-20260624/failure-boundary-drill.json` | Bridge-unavailable behavior was executed and fail-closed; other runtime-controlled boundaries were contract-verified and explicitly deferred due to unavailable runtime controls. |
| Graphiti fallback records | `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6B_GRAPHITI_BACKFILL_2026-06-24.md`; `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6C_GRAPHITI_BACKFILL_2026-06-24.md`; `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6D_GRAPHITI_BACKFILL_2026-06-24.md` | Durable local backfill records exist for Graphiti ingest failures or stuck processing states. |

## Commands Actually Executed In P0.6

The following read-only commands were executed during P0.6 evidence collection:

```text
target/debug/quantix --version
target/debug/quantix safety kill-switch status
target/debug/quantix execution qmt status --checklist
```

The `execution qmt status --checklist` probe was executed only in the bridge-unavailable environment. It returned an operator-readable fail-closed state with `ready=false` and `failure_category=bridge_unreachable`.

Focused current-code tests were also run in P0.6d to verify fail-closed contracts:

```text
cargo test --lib test_qmt_live_preflight_report_classifies_fail_closed_categories -- --test-threads=1
cargo test --lib remains_available_when_kill_switch_enabled -- --test-threads=1
cargo test --lib kill_switch_enabled -- --test-threads=1
```

## Commands Not Executed

The following bridge-backed smoke commands were not executed against a real or test miniQMT runtime:

```text
quantix execution qmt status --checklist
quantix execution qmt preview --request-id <redacted>
quantix execution qmt query <redacted>
quantix execution qmt audit <redacted>
quantix execution qmt manual-interventions list/show <redacted>
```

They require an operator-selected runtime and commit-safe labels. Treating the current no-runtime environment as a successful smoke would fabricate readiness evidence, so P0.6 records a blocked decision instead.

## Residual Risks

- No real or test Windows Bridge endpoint/config was selected for P0.6.
- No commit-safe account label was selected for P0.6.
- No running qmt/bridge process was observed for read-only bridge smoke.
- No bridge-backed preview/query/audit/manual-intervention read-only chain was verified.
- No canary readiness can be inferred from local fail-closed behavior alone.

## Canary Position

qmt_live canary is not approved.

Before a canary proposal can be reconsidered, an operator must provide:

- a running, isolated miniQMT Windows Bridge environment;
- a commit-safe bridge host label;
- a test-owned or otherwise approved account label;
- explicit read-only inspection approval;
- safe request/query/runtime-store targets for preview, query, audit, and manual-intervention checks;
- confirmation that no submit, cancel, broker mutation, manual-intervention resolution, or runtime-store write is authorized during the read-only smoke.

After those prerequisites exist, P0.6b can be rerun or a new P0.7 runtime-smoke slice can be opened. The existing P0.6 evidence template and failure-boundary framework should be reused.

## Mainline Priority Handoff

P0.6 is archived as maintenance-only until an operator supplies the missing runtime.

Development priority should move to work that can advance the project goal without external broker-environment dependency:

| Priority | Line | Reason |
| --- | --- | --- |
| Primary | ExecutionCapabilities continuation from the landed P0.3e/P0.3f baseline | Standardizes execution-adapter semantics for paper, mock_live, and qmt_live without requiring miniQMT runtime access. |
| Primary | OpenStock data consumption adaptation | Advances the data to indicators to backtest to local simulation loop, which is the shortest path to a runnable quant workflow. |
| Archived maintenance | P0.6 qmt_live runtime readiness | Blocked by missing operator-selected miniQMT Windows Bridge runtime; no further qmt_live environment-validation slices should be opened before runtime exists. |

## Non-Goals Preserved

P0.6e does not:

- run any new runtime probe;
- execute qmt_live submit;
- execute broker cancel;
- resolve manual-intervention cases;
- write broker or runtime state;
- change bridge protocol, response shapes, storage schema, `OrderStatus`, `ExecutionAdapter`, or paper execution semantics;
- resume `.unwrap()` cleanup.
