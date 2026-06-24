# qmt_live Runtime Readiness P0.6 Tasks

## 0. Baseline And Governance

- [x] 0.1 Confirm work starts from clean `master` after P0.5 archive.
- [x] 0.2 Create a dedicated P0.6 FUNCTION_TREE node before editing planning files.
- [x] 0.3 Run Graphiti reads for `quantix_rust_main` and `quantix_rust_docs`; if Graphiti fails, record `Graphiti backfill required` in the slice report.
- [x] 0.4 Create this OpenSpec change as the governing scope for P0.6.
- [x] 0.5 Run `openspec validate qmt-live-runtime-readiness-p0-6 --strict`.
- [x] 0.6 Run `openspec validate --all --strict`.
- [x] 0.7 Run function-tree gate and validation.
- [x] 0.8 Run GitNexus detect_changes before committing.

## 1. P0.6a Environment Inventory And Prerequisite Check

- [x] 1.1 Identify the operator-owned miniQMT/Windows Bridge runtime to test, or record that no runtime is available.
- [x] 1.2 Capture redacted runtime metadata:
  - OS/runtime label;
  - bridge host label;
  - account type label;
  - bridge contract version if available;
  - Quantix commit hash;
  - qmt capability summary;
  - kill-switch status.
- [x] 1.3 Store environment inventory in a commit-safe evidence file without secrets or raw account identifiers.
- [x] 1.4 If the runtime is unavailable, stop and prepare a blocked readiness report instead of fabricating evidence.

## 2. P0.6b Read-Only Command Smoke

- [ ] 2.1 Run `quantix execution qmt status --checklist` against the selected runtime.
- [ ] 2.2 Run `quantix execution qmt preview` in a no-submit path with a redacted request reference or fixture.
- [ ] 2.3 Run `quantix execution qmt query` only if a safe read-only query target exists.
- [ ] 2.4 Run `quantix execution qmt audit` against local runtime-store records when available.
- [ ] 2.5 Run `quantix execution qmt manual-interventions list/show` against local runtime-store records when available.
- [ ] 2.6 Record outputs as summarized evidence, not raw broker logs.

## 3. P0.6c Redacted Runtime Evidence Package

- [ ] 3.1 Add or update the P0.6 evidence template.
- [ ] 3.2 Include redaction rules for account IDs, account names, credentials, bridge URLs, broker logs, and screenshots.
- [ ] 3.3 Include a checklist proving no submit/cancel or broker-state mutation occurred.
- [ ] 3.4 Add repo hygiene coverage if needed to prevent committing secrets or raw broker evidence.

## 4. P0.6d Failure-Boundary Drill

- [ ] 4.1 Verify bridge-unavailable behavior is fail-closed and operator-readable.
- [ ] 4.2 Verify `qmt.enabled=false` behavior is fail-closed.
- [ ] 4.3 Verify non-live or ambiguous `qmt.mode` behavior is fail-closed.
- [ ] 4.4 Verify missing required qmt capability behavior is fail-closed.
- [ ] 4.5 Verify enabled kill switch blocks mutation while read-only inspection remains available.
- [ ] 4.6 Record which cases were actually executed and which were deferred due to unavailable runtime controls.

## 5. P0.6e Readiness Decision Report

- [ ] 5.1 Produce a final report with one decision:
  - `ready_for_canary_proposal`;
  - `blocked_by_environment`;
  - `blocked_by_command_gap`;
  - `blocked_by_safety_gap`;
  - `blocked_by_manual_intervention`.
- [ ] 5.2 Reference exact evidence files and commands.
- [ ] 5.3 List residual risks and required approval before any controlled canary.
- [ ] 5.4 Update README, CHANGELOG, and FUNCTION_TREE if operator-visible status changes.
- [ ] 5.5 Run final gates for the implemented slice.
- [ ] 5.6 Write Graphiti memory after durable conclusions and verify ingest.

## 6. Closure

- [ ] 6.1 Confirm all completed P0.6 slices have passed their declared gates.
- [ ] 6.2 Confirm OpenSpec task status matches actual evidence.
- [ ] 6.3 Archive this OpenSpec change only after P0.6 is closed or explicitly blocked with documented reason.
