# qmt_live Operational Safety P0.5 Tasks

## 0. Baseline And Governance

- [x] 0.1 Confirm the implementation branch starts from a clean `master` and record the commit hash in the slice report.
- [x] 0.2 Confirm this OpenSpec change is the governing scope before editing qmt_live operational safety code.
- [x] 0.3 Create or update a FUNCTION_TREE node for the specific implementation slice before source edits.
- [x] 0.4 Run Graphiti reads for `quantix_rust_main` and `quantix_rust_docs`; if Graphiti times out, record `Graphiti backfill required` in the slice report.
- [x] 0.5 Run GitNexus impact for every function, method, class, or handler selected for editing.
- [x] 0.6 Stop and request approval before editing any HIGH or CRITICAL impact target.
- [x] 0.7 Keep each implementation PR single-purpose; do not combine P0.5a-P0.5e in one runtime PR.

## 1. P0.5a Preflight Doctor

- [x] 1.1 Identify the current owner for qmt_live readiness output:
  - Prefer `quantix execution qmt status --checklist`.
  - Add `quantix execution qmt doctor` only if GitNexus impact and local command ownership show it is cleaner.
- [x] 1.2 Write RED tests for ready qmt_live preflight:
  - bridge capability endpoint reachable;
  - `qmt.enabled=true`;
  - `qmt.mode=live`;
  - `qmt.supports` contains `order_submit`;
  - qmt_live adapter capability source is broker-owned;
  - kill switch state is visible.
- [x] 1.3 Write RED tests for fail-closed preflight categories:
  - `bridge_unreachable`;
  - `qmt_capability_missing`;
  - `qmt_disabled`;
  - `qmt_mode_not_live`;
  - `qmt_order_submit_missing`;
  - `qmt_live_capability_mismatch`;
  - `kill_switch_enabled`.
- [x] 1.4 Implement the minimum read-only preflight model and formatter.
- [x] 1.5 Ensure preflight does not submit orders, cancel orders, mutate runtime store state, or write broker state.
- [x] 1.6 Run focused tests for the preflight command/model.
- [x] 1.7 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
- [x] 1.8 Run GitNexus detect_changes and confirm affected scope is limited to qmt_live preflight/readiness surfaces.
- [x] 1.9 Update FUNCTION_TREE and PR documentation with P0.5a behavior and boundaries.

## 2. P0.5b Canary Runbook And Evidence Artifact

- [x] 2.1 Add `docs/operations/QMT_LIVE_CANARY_RUNBOOK_2026-06-22.md` or a same-date successor if implemented later.
- [x] 2.2 Document the exact canary sequence:
  - start Windows Bridge;
  - confirm miniQMT login;
  - run qmt_live preflight;
  - run `quantix execution qmt preview --request-id <ID>`;
  - verify preview payload;
  - confirm kill switch status;
  - submit only with explicit operator confirmation;
  - run `quantix execution qmt query <ID>`;
  - run reconciliation verification;
  - record manual-intervention status.
- [x] 2.3 Define an evidence artifact path under `docs/reports/evidence/qmt-live-canary-<YYYYMMDD>/`.
- [x] 2.4 Define a redacted evidence JSON shape with commit hash, command lines, readiness summary, preview hash, submission summary, query summary, reconciliation summary, and operator confirmation timestamp.
- [x] 2.5 Add a repo hygiene or documentation test if a stable local guard already exists for operations docs.
- [x] 2.6 Run documentation-focused validation and full project gates required by the touched files.
- [x] 2.7 Run GitNexus detect_changes before commit.

## 3. P0.5c Kill Switch Acceptance

- [x] 3.1 Confirm existing `quantix safety kill-switch` behavior and qmt_live submit-path enforcement.
- [x] 3.2 Add missing tests only if current coverage does not prove:
  - kill switch blocks `qmt_live` approval/submission;
  - kill switch blocks `mock_live`;
  - kill switch allows `paper`;
  - kill switch allows qmt_live status/checklist/preflight;
  - kill switch allows qmt_live preview and read-only query;
  - kill switch block payload includes reason, enabled timestamp, blocked timestamp, and target mode.
- [x] 3.3 Update operator documentation to make kill switch enablement a required pre-canary and incident-response step.
- [x] 3.4 Preserve current safety storage path and payload semantics unless a separate OpenSpec change approves a schema change.
- [x] 3.5 Run focused safety and qmt_live tests.
- [x] 3.6 Run full gates and GitNexus detect_changes.

## 4. P0.5d Audit Evidence Closure

- [ ] 4.1 Inventory existing runtime store payload fields for qmt_live submit, query, and reconciliation.
- [ ] 4.2 Run GitNexus impact before editing runtime store, qmt task submit, reconciliation, or CLI report handlers.
- [ ] 4.3 Prefer an audit report assembled from existing payloads over a new storage schema.
- [ ] 4.4 Add tests that the audit view includes:
  - request ID;
  - redacted account label;
  - symbol;
  - side;
  - quantity;
  - local submission ID;
  - client order ID;
  - task ID;
  - external order ID when present;
  - bridge contract version;
  - qmt_live error category when present;
  - reconciliation decision;
  - manual-intervention marker when present.
- [ ] 4.5 Ensure audit output redacts secrets and raw account identifiers.
- [ ] 4.6 Run focused audit/report tests.
- [ ] 4.7 Run full gates and GitNexus detect_changes.

## 5. P0.5e Manual-Intervention Report

- [ ] 5.1 Identify current persisted signals for qmt_live manual intervention:
  - identity mismatch;
  - broker unknown state;
  - missing external order ID after bridge task completion;
  - reconciliation preserved local state;
  - bridge failure requiring operator review.
- [ ] 5.2 Add a read-only list/show report for unresolved qmt_live manual-intervention cases.
- [ ] 5.3 The first report MUST NOT mark cases resolved or mutate runtime state.
- [ ] 5.4 Add tests for listing and showing each supported intervention category.
- [ ] 5.5 Add operator guidance text for each category:
  - inspect miniQMT same-day orders;
  - compare task ID, client order ID, local submission ID, and external order ID;
  - avoid resubmission until the ambiguous state is resolved.
- [ ] 5.6 Run focused report tests.
- [ ] 5.7 Run full gates and GitNexus detect_changes.

## 6. Release And Closure Gates

- [ ] 6.1 Confirm P0.5a-P0.5e are either complete or explicitly deferred in `tasks.md`.
- [ ] 6.2 Confirm `FUNCTION_TREE.md` reflects the final P0.5 status and boundaries.
- [ ] 6.3 Confirm README/CHANGELOG are updated if user-facing commands or operator workflows changed.
- [ ] 6.4 Run final gates:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

- [ ] 6.5 Run function-tree gate and validation.
- [ ] 6.6 Run GitNexus detect_changes and confirm only expected symbols and execution flows are affected.
- [ ] 6.7 Create compact Graphiti memory for the completed P0.5 result; if ingest fails, add a local Graphiti backfill report containing `Graphiti backfill required`.
- [ ] 6.8 Archive the OpenSpec change only after the implementation, evidence, documentation, and CI gates are complete.
