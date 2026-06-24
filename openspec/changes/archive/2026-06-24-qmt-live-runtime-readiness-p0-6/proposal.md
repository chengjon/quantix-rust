# qmt_live Runtime Readiness P0.6

## Why

P0.5 closed the operator-facing qmt_live safety surface: preflight checklist, canary runbook, kill-switch acceptance, audit evidence, and manual-intervention reporting. The next risk is not another local abstraction change; it is proving that those surfaces are usable against a real or test-owned miniQMT/Windows Bridge runtime without committing secrets, raw account identifiers, or broker logs.

P0.6 establishes a formal runtime-readiness gate before any additional canary or production trading expansion. It keeps the first follow-up stage read-only by default and separates environmental verification from broker mutation.

## What Changes

- Add an active OpenSpec change under `openspec/changes/qmt-live-runtime-readiness-p0-6/`.
- Govern P0.6 as a sequence of small runtime-readiness slices:
  - P0.6a environment inventory and prerequisite check.
  - P0.6b read-only qmt_live command smoke against the selected runtime.
  - P0.6c redacted runtime evidence package.
  - P0.6d failure-boundary drill for bridge unavailable, qmt disabled, non-live mode, missing capability, and kill switch enabled.
  - P0.6e readiness decision report for whether a later controlled canary may start.
- Reuse existing read-only commands first:
  - `quantix execution qmt status --checklist`
  - `quantix execution qmt preview`
  - `quantix execution qmt query`
  - `quantix execution qmt audit`
  - `quantix execution qmt manual-interventions list/show`
  - `quantix safety kill-switch`
- Keep submit/cancel or broker-state mutation out of P0.6 unless a later task explicitly receives separate approval.

## Capabilities

### New Capabilities

- Runtime-readiness evidence format for qmt_live environment verification.
- Operator acceptance checklist for deciding whether a controlled qmt_live canary can be proposed later.

### Modified Capabilities

- None in this planning slice.

## Impact

- Adds OpenSpec proposal, design, task list, and spec delta files.
- Adds FUNCTION_TREE registration for active P0.6 planning.
- May later touch documentation, operations runbooks, CLI tests, or read-only qmt_live command surfaces after fresh FUNCTION_TREE authorization and GitNexus impact.
- Does not change runtime code by itself.
- Requires an operator-owned miniQMT/Windows Bridge environment for final runtime-readiness evidence. Secrets, raw account IDs, raw broker logs, credentials, host-specific tokens, and unredacted screenshots MUST NOT be committed.

## Non-Goals

- Do not submit live orders.
- Do not cancel broker orders.
- Do not mark manual-intervention cases resolved.
- Do not alter bridge protocol, response shapes, storage schema, `OrderStatus`, `ExecutionAdapter`, or paper execution semantics.
- Do not implement generic live broker execution.
- Do not resume `.unwrap()` cleanup or other unrelated technical-debt work.
