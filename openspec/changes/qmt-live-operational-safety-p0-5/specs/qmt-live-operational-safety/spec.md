# qmt-live-operational-safety Specification

## ADDED Requirements

### Requirement: OpenSpec-Governed qmt_live Operational Safety

qmt_live real-money operational safety work SHALL be governed by this active OpenSpec change before implementation starts.

#### Scenario: Starting P0.5 implementation

- **WHEN** work starts on qmt_live preflight, canary evidence, kill-switch acceptance, audit evidence, or manual-intervention reporting
- **THEN** the executor SHALL use `openspec/changes/qmt-live-operational-safety-p0-5/` as the governing proposal, task list, design, and spec delta

#### Scenario: Deferring unrelated work

- **WHEN** a proposed change concerns generic broker execution, `paper_sim_lifecycle`, global `ExecutionAdapter` migration, `OrderStatus` changes, bridge protocol changes, response-shape changes, storage-schema migration, background daemons, or `.unwrap()` cleanup
- **THEN** it SHALL be kept out of P0.5 unless a separate approved OpenSpec change brings it into scope

### Requirement: qmt_live Preflight Doctor

The qmt_live preflight check SHALL be read-only and SHALL produce a deterministic readiness verdict before operator canary or real-money submission.

#### Scenario: qmt_live is ready

- **WHEN** the bridge capability endpoint is reachable, qmt capability metadata exists, `qmt.enabled=true`, `qmt.mode=live`, `qmt.supports` contains `order_submit`, qmt_live execution capabilities identify broker-owned status/fill/cancel sources, and the kill switch state is visible
- **THEN** the preflight output SHALL mark qmt_live readiness as ready
- **AND** the output SHALL include the bridge contract version or an explicit unknown marker
- **AND** the output SHALL include the capability source used for the verdict

#### Scenario: bridge is unreachable

- **WHEN** the preflight cannot reach the bridge capability endpoint
- **THEN** the preflight output SHALL mark qmt_live readiness as not ready
- **AND** the failure category SHALL be `bridge_unreachable`
- **AND** no order submission, order cancellation, broker-state mutation, or runtime-store mutation SHALL occur

#### Scenario: qmt capability metadata is not usable

- **WHEN** qmt capability metadata is missing, disabled, non-live, or lacks `order_submit`
- **THEN** the preflight output SHALL mark qmt_live readiness as not ready
- **AND** the failure category SHALL be one of `qmt_capability_missing`, `qmt_disabled`, `qmt_mode_not_live`, or `qmt_order_submit_missing`

#### Scenario: local qmt_live capability semantics are inconsistent

- **WHEN** bridge capability metadata is live-ready but local qmt_live execution capabilities do not identify broker-owned status, fill, and cancel semantics
- **THEN** the preflight output SHALL mark qmt_live readiness as not ready
- **AND** the failure category SHALL be `qmt_live_capability_mismatch`

#### Scenario: kill switch is enabled

- **WHEN** the safety kill switch is enabled for live-capable target modes
- **THEN** the preflight output SHALL include kill-switch state
- **AND** the preflight output SHALL mark real-money submission readiness as blocked by `kill_switch_enabled`
- **AND** read-only status, checklist, preview, query, and preflight commands SHALL remain available

### Requirement: Canary Runbook And Evidence Artifact

qmt_live canary execution SHALL have a documented operator runbook and redacted evidence artifact before broader real-money usage is claimed.

#### Scenario: Preparing a qmt_live canary

- **WHEN** an operator prepares a qmt_live canary
- **THEN** the runbook SHALL require bridge startup, miniQMT login confirmation, preflight success, preview payload review, kill-switch status review, explicit operator confirmation, post-submit query, reconciliation verification, and manual-intervention review

#### Scenario: Saving canary evidence

- **WHEN** a canary run completes, fails, or is blocked
- **THEN** evidence SHALL be saved under `docs/reports/evidence/qmt-live-canary-<YYYYMMDD>/`
- **AND** the evidence SHALL include commit hash, command lines, redacted environment labels, readiness summary, preview payload hash, operator confirmation timestamp, submission summary, query summary, reconciliation summary, and manual-intervention status
- **AND** secrets, raw tokens, full account identifiers, and raw broker logs SHALL NOT be committed

### Requirement: qmt_live Kill-Switch Acceptance

The existing safety kill switch SHALL be treated as a required qmt_live operational safety gate.

#### Scenario: kill switch blocks real-money-capable submission

- **WHEN** the kill switch is enabled
- **THEN** qmt_live approval and submission SHALL be blocked
- **AND** mock_live submission SHALL be blocked
- **AND** paper execution SHALL remain available

#### Scenario: kill switch preserves read-only qmt_live operations

- **WHEN** the kill switch is enabled
- **THEN** qmt_live status/checklist/preflight SHALL remain available
- **AND** qmt_live preview SHALL remain available
- **AND** qmt_live read-only query SHALL remain available

#### Scenario: kill switch block is recorded

- **WHEN** a qmt_live operation is blocked by the kill switch
- **THEN** the error or persisted payload SHALL include target mode, reason, enabled timestamp, and blocked timestamp

### Requirement: qmt_live Audit Evidence

qmt_live operational use SHALL produce enough redacted audit evidence to support post-incident review without treating local state as broker truth.

#### Scenario: Building an audit view

- **WHEN** an operator requests qmt_live audit evidence for a request, task, or local submission
- **THEN** the audit view SHALL include request ID, redacted account label, symbol, side, quantity, order type or price intent, local submission ID, client order ID, task ID, external order ID when present, bridge contract version, qmt_live error category when present, reconciliation decision, and manual-intervention marker when present

#### Scenario: Protecting sensitive data

- **WHEN** audit evidence is printed or saved
- **THEN** secrets, raw tokens, full account identifiers, and raw broker logs SHALL be omitted or redacted

### Requirement: Manual-Intervention Report

qmt_live ambiguity requiring operator review SHALL be discoverable through a read-only report before any state-mutating resolution workflow is added.

#### Scenario: Listing manual-intervention cases

- **WHEN** qmt_live persisted state contains identity mismatch, broker unknown state, missing external order ID after bridge task completion, preserved local state after ambiguous reconciliation, or bridge failure requiring review
- **THEN** the manual-intervention report SHALL list the case with enough identifiers for operator lookup
- **AND** the report SHALL include task ID, client order ID, local submission ID, external order ID when available, failure category, and suggested human action

#### Scenario: Keeping first release read-only

- **WHEN** the first manual-intervention report is implemented
- **THEN** it MUST NOT mark cases resolved
- **AND** it MUST NOT mutate runtime store state

### Requirement: Release Closure Gate

P0.5 SHALL close only after implementation evidence, documentation, and repository gates are complete.

#### Scenario: Closing P0.5

- **WHEN** P0.5 is ready for closure
- **THEN** `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` SHALL have captured exit 0 evidence
- **AND** function-tree gate and validation SHALL pass
- **AND** GitNexus detect_changes SHALL show only expected symbols and execution flows
- **AND** README, CHANGELOG, FUNCTION_TREE, and operations docs SHALL reflect any user-facing command or workflow changes
