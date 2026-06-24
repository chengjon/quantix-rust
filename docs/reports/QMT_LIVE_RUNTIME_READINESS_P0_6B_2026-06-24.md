# qmt_live Runtime Readiness P0.6b

Date: 2026-06-24

Status: read-only smoke blocked by environment selection

Branch: `feat/p0-6b-qmt-live-read-only-smoke`

Base commit: `5d853d96afb64fd54f52a447f43c29be44b57c68`

Evidence: `docs/reports/evidence/qmt-live-runtime-readiness-20260624/read-only-smoke.json`

## Summary

P0.6b was opened to run qmt_live read-only command smoke after P0.6a. A fresh pre-smoke probe found the same blocking condition as P0.6a:

| Item | Status |
| --- | --- |
| Canonical Windows Bridge path | present |
| miniQMT workspace path | present |
| `target/debug/quantix` | present |
| `target/debug/quantix --version` | `quantix 0.1.0` |
| Local kill switch | `enabled=false` |
| qmt/bridge process observed | no |
| selected qmt bridge endpoint/config | no |
| selected account label | no |

Because no operator-selected runtime was detected, P0.6b did not run qmt_live read-only commands against a bridge. This avoids a misleading smoke result and preserves the P0.6 rule that runtime evidence must not be fabricated.

## Commands Executed

Only safe local read-only commands were executed:

```text
target/debug/quantix --version
target/debug/quantix safety kill-switch status
```

Both completed successfully. The kill-switch status was `enabled=false`.

## Commands Intentionally Skipped

The following commands were intentionally not executed because they require an operator-selected qmt_live runtime and commit-safe labels:

| Command | Reason |
| --- | --- |
| `quantix execution qmt status --checklist` | no selected bridge endpoint/config |
| `quantix execution qmt preview --request-id <redacted>` | no selected request/runtime |
| `quantix execution qmt query <redacted>` | no safe query target |
| `quantix execution qmt audit <redacted>` | no selected runtime record target |
| `quantix execution qmt manual-interventions list/show` | no selected runtime record target |

This is a deliberate blocked smoke decision, not a successful smoke.

## Decision

P0.6b result:

```text
blocked_by_environment_selection
```

P0.6c should not proceed as if runtime smoke passed. P0.6b must be rerun when an operator supplies:

- a running Windows Bridge process;
- a commit-safe bridge host label;
- a test-owned miniQMT account label;
- confirmation that read-only inspection is allowed;
- safe request/query/runtime-store targets for preview, query, audit, and manual-intervention checks.

## Mutation Guard

No mutation path was executed:

- no `qmt_live` submit;
- no broker cancel;
- no manual-intervention resolution;
- no runtime-store write;
- no broker-state mutation.

## Redaction

The evidence does not include raw account identifiers, credentials, raw bridge URLs, raw broker logs, screenshots, `.env` contents, or process command lines.

## Boundaries Preserved

P0.6b did not change runtime code, tests, bridge protocol, storage schema, response shapes, `OrderStatus`, `ExecutionAdapter`, qmt_live submit/cancel behavior, paper execution semantics, or `.unwrap()` technical-debt items.
