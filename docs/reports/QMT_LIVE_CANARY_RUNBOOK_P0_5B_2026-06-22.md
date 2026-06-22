# qmt_live Canary Runbook And Evidence P0.5b

Date: 2026-06-22

Status: local closeout complete; PR pending

Branch: `docs/p0-5b-qmt-live-canary-runbook`

Base commit: `acb5a359e5809052f1e51fdd637732c6d5a15a7b`

## Summary

P0.5b adds the operator runbook and redacted evidence artifact shape required before broader qmt_live real-money usage can be claimed.

The slice is intentionally docs-first. It does not change qmt_live runtime behavior, bridge protocol, response shape, storage schema, `OrderStatus`, `ExecutionAdapter`, `paper_immediate`, or `paper_sim_lifecycle`.

## Delivered Artifacts

- `docs/operations/QMT_LIVE_CANARY_RUNBOOK_2026-06-22.md`
- `docs/reports/QMT_LIVE_CANARY_RUNBOOK_P0_5B_2026-06-22.md`
- `docs/reports/evidence/qmt-live-canary-20260622/README.md`
- `docs/reports/evidence/qmt-live-canary-20260622/evidence.template.json`
- `tests/repo_hygiene_test.rs`

The runbook documents the required canary sequence:

- start Windows Bridge;
- confirm miniQMT login;
- run `quantix execution qmt status --checklist`;
- run `quantix execution qmt preview --request-id <ID>`;
- verify preview payload;
- confirm kill switch status;
- record explicit operator confirmation;
- submit with `quantix execution qmt live --request-id <ID> --yes`;
- run `quantix execution qmt query <ID>`;
- run reconciliation verification;
- record manual-intervention status.

The evidence template defines the redacted JSON shape for:

- commit hash;
- command lines;
- redacted environment labels;
- readiness summary;
- preview payload hash;
- operator confirmation timestamp;
- submission summary;
- query summary;
- reconciliation summary;
- manual-intervention status;
- redaction checklist.

## Preserved Boundaries

- No production Rust source changes.
- No qmt_live submit/cancel/query behavior change.
- No runtime store mutation change.
- No broker write behavior change.
- No bridge protocol or bridge response shape change.
- No storage schema change.
- No `OrderStatus` change.
- No `ExecutionAdapter` change.
- No paper execution behavior change.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Pre-edit impact:

- Production symbols: not applicable; P0.5b selected no production function, method, class, handler, trait, enum, storage schema, or bridge protocol symbol for editing.
- Hygiene test insertion neighbor: `legacy_docs_are_archived_without_moving_current_reports`
  - risk: LOW
  - direct callers: 0
  - affected processes: 0

Final GitNexus `detect_changes` result:

- risk: LOW
- changed files: 6
- affected processes: 0
- rationale: no changed symbols participate in indexed processes
- changed file classes: documentation, governance, test, config

The GitNexus index reported a stale-index warning because the indexed commit differs from the current commit. The diff still resolved against the active worktree and reported `fresh_for_staged_diff: true`.

## TDD Evidence

RED:

```text
cargo test --test repo_hygiene_test qmt_live_canary_runbook_and_evidence_template_are_present -- --test-threads=1
```

The new hygiene test failed before documentation was added:

```text
expected docs/operations/QMT_LIVE_CANARY_RUNBOOK_2026-06-22.md to be readable
```

GREEN:

```text
cargo test --test repo_hygiene_test qmt_live_canary_runbook_and_evidence_template_are_present -- --test-threads=1
```

Result:

```text
1 passed; 0 failed; 90 filtered out
```

## Verification

Completed before edits:

- baseline `cargo test`
- focused repo hygiene test:
  `cargo test --test repo_hygiene_test qmt_live_canary_runbook_and_evidence_template_are_present -- --test-threads=1`

Completed local gates:

- `cargo test --test repo_hygiene_test`
  - result: 91 passed; 0 failed
- `git diff --check`
  - result: passed
- `cargo fmt --check`
  - result: passed
- `cargo clippy -- -D warnings`
  - result: passed
- `cargo test`
  - result: passed
- `npx openspec validate qmt-live-operational-safety-p0-5 --strict`
  - result: valid
- Function Tree scope-check/gate/validate
  - result: passed
- Function Tree closeout
  - result: P0.5b closed; active gates none
- GitNexus `detect_changes`
  - result: LOW risk; 0 affected processes

## Graphiti Status

Graphiti pre-read for P0.5b qmt_live canary runbook/evidence context was attempted against `quantix_rust_main` and `quantix_rust_docs` and timed out with `Request timed out.`.

Graphiti backfill required if final P0.5b memory ingest also fails.

## Remaining Closeout Gates

- Commit implementation.
- PR CI and master CI, or documented failure.
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`.
