# qmt_live Runtime Readiness P0.6 Design

## Context

P0.3 and P0.4 hardened qmt_live capability, identity, reconciliation, and diagnostics semantics. P0.5 then closed operator-facing safety surfaces and archived the qmt_live operational safety OpenSpec change.

P0.6 is the first stage after that archive. Its job is to verify that the system can be operated safely in a real or test-owned miniQMT/Windows Bridge environment while staying read-only by default. It is not a trading expansion stage.

## Operating Principle

Runtime readiness is a gate, not an execution feature. A P0.6 pass means the environment, read-only commands, evidence capture, and failure boundaries are understood well enough to propose a later controlled canary. It does not mean automatic live trading is approved.

## Command Surface Strategy

P0.6 should prefer existing commands:

- `quantix execution qmt status --checklist` for readiness status.
- `quantix execution qmt preview` for payload preview without submission.
- `quantix execution qmt query` for read-only broker/task observation.
- `quantix execution qmt audit` for runtime-store evidence.
- `quantix execution qmt manual-interventions list/show` for unresolved intervention cases.
- `quantix safety kill-switch` for mutation blocking state.

New commands should be avoided unless the existing command ownership is clearly insufficient and GitNexus impact is LOW/MEDIUM with explicit approval.

## Slice Order

### P0.6a: Environment Inventory And Prerequisite Check

Record the selected runtime without secrets:

- OS/runtime label;
- bridge base URL host label only;
- miniQMT account type label, not raw account ID;
- bridge contract version if visible;
- qmt capability summary;
- kill-switch status;
- Quantix commit hash;
- command versions and config paths with secrets redacted.

### P0.6b: Read-Only Command Smoke

Run existing read-only qmt_live commands against the selected runtime and capture pass/fail evidence:

- preflight checklist;
- preview dry path;
- read-only query path where safe;
- audit report path against local runtime store records;
- manual-intervention list/show path.

The smoke MUST NOT submit or cancel broker orders.

### P0.6c: Redacted Runtime Evidence Package

Add or update a report/evidence template that can store P0.6 outputs in a commit-safe format.

The artifact must redact:

- raw account IDs;
- account names that identify a real person;
- access tokens;
- bridge URLs with credentials;
- raw broker logs;
- screenshots that expose account or position details.

### P0.6d: Failure-Boundary Drill

Prove operator-visible behavior for expected runtime failures:

- bridge unavailable;
- `qmt.enabled=false`;
- `qmt.mode` is not live or is ambiguous;
- required `order_submit` capability missing;
- kill switch enabled.

The drill should confirm fail-closed output without changing broker/runtime state.

### P0.6e: Readiness Decision Report

Produce a final report that states one of:

- `ready_for_canary_proposal`;
- `blocked_by_environment`;
- `blocked_by_command_gap`;
- `blocked_by_safety_gap`;
- `blocked_by_manual_intervention`.

The report must list the exact evidence files, commands, commit hash, redaction policy, known gaps, and explicit non-approval of live trading unless a later stage approves it.

## Dependency And Risk Decisions

- Real miniQMT availability is an external dependency. If unavailable, P0.6 may close only as blocked or as documentation-prepared; it must not fabricate runtime evidence.
- GitNexus impact remains mandatory before any source-code or handler edits.
- P0.6 must stay separate from later canary execution. A positive readiness decision only authorizes drafting the canary plan, not running it.

## Non-Goals

- No live order submission.
- No broker cancellation.
- No storage schema migration.
- No bridge protocol change.
- No global response-shape rewrite.
- No `ExecutionAdapter` or `OrderStatus` migration.
- No `paper_immediate` or `paper_sim_lifecycle` work.
