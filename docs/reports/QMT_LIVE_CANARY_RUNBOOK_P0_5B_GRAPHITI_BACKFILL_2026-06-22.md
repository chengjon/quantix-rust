# qmt_live Canary Runbook P0.5b Graphiti Backfill

Date: 2026-06-22

Status: Graphiti ingest failed; local backfill record committed

Graphiti backfill required

## Scope

This report records the mandatory local fallback for the P0.5b qmt_live canary runbook closeout memory.

P0.5b itself was completed and merged through PR #270 as merge commit `5c576748aff234f0c46a40506d8ce721da6cdc2b`.

## Intended Graphiti Memory

Group: `quantix_rust_main`

Episode name: `P0.5b qmt_live canary runbook closed`

Episode UUID: `52b90280-9be2-48b3-b699-b9e46686d29e`

Intended content:

- P0.5b qmt_live canary runbook and evidence artifact closed on 2026-06-22.
- Added `docs/operations/QMT_LIVE_CANARY_RUNBOOK_2026-06-22.md`.
- Added `docs/reports/evidence/qmt-live-canary-20260622/README.md`.
- Added `docs/reports/evidence/qmt-live-canary-20260622/evidence.template.json`.
- Added `docs/reports/QMT_LIVE_CANARY_RUNBOOK_P0_5B_2026-06-22.md`.
- Added repo hygiene coverage in `tests/repo_hygiene_test.rs`.
- The runbook fixes the operator sequence:
  - start Windows Bridge;
  - confirm miniQMT login;
  - run qmt_live preflight;
  - run preview;
  - verify preview payload;
  - confirm kill switch;
  - record operator confirmation;
  - submit with explicit `--yes`;
  - query;
  - reconciliation verification;
  - manual-intervention status.
- The evidence template is redacted and includes commit hash, commands, readiness summary, preview payload hash, operator confirmation, submission/query/reconciliation summaries, manual-intervention status, and redaction checklist.
- Preserved boundaries:
  - no qmt_live runtime behavior change;
  - no submit/cancel/query logic change;
  - no bridge protocol change;
  - no response shape change;
  - no storage schema change;
  - no `OrderStatus` change;
  - no `ExecutionAdapter` change;
  - no `paper_immediate` change;
  - no `paper_sim_lifecycle` change;
  - no `.unwrap()` cleanup resumed.
- Verification:
  - focused hygiene RED/GREEN;
  - `cargo test --test repo_hygiene_test`;
  - `git diff --check`;
  - `cargo fmt --check`;
  - `cargo clippy -- -D warnings`;
  - `cargo test`;
  - `npx openspec validate qmt-live-operational-safety-p0-5 --strict`;
  - Function Tree P0.5b closed with active gates none;
  - GitNexus `detect_changes` LOW risk and 0 affected processes.
- PR #270 was squash-merged as `5c576748aff234f0c46a40506d8ce721da6cdc2b`.
- Master CI run `27954991844` passed Documentation, Lint, and Test.

## Failure Evidence

Graphiti `add_memory` queued successfully:

```text
episode_uuid: 52b90280-9be2-48b3-b699-b9e46686d29e
group_id: quantix_rust_main
queue_position: 1
```

Graphiti ingest status later failed:

```text
state: failed
last_error: Request timed out.
last_error_code: apitimeouterror
attempt_count: 1
```

Graphiti search for this backfill context was also attempted against `quantix_rust_main` and `quantix_rust_docs`; it timed out with `Error searching nodes: Request timed out.`.

## Backfill Instruction

When Graphiti is available again, add the intended memory above to `quantix_rust_main`, then run `get_ingest_status` until the episode reaches `completed`.

After successful ingest, this local file remains the audit trail for the original timeout and should not be treated as a replacement for the Graphiti memory.
