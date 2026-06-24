# qmt-live-runtime-readiness Specification

## ADDED Requirements

### Requirement: OpenSpec-Governed qmt_live Runtime Readiness

qmt_live runtime-readiness work SHALL be governed by this active OpenSpec change before implementation starts.

#### Scenario: Starting P0.6 implementation

- **WHEN** work starts on qmt_live environment inventory, read-only runtime smoke, redacted runtime evidence, failure-boundary drills, or readiness decision reporting
- **THEN** the executor SHALL use `openspec/changes/qmt-live-runtime-readiness-p0-6/` as the governing proposal, design, task list, and spec delta

#### Scenario: Deferring unrelated work

- **WHEN** a proposed change concerns live order submission, broker cancellation, manual-intervention resolution, generic live broker execution, bridge protocol changes, response-shape changes, storage-schema migration, `OrderStatus`, `ExecutionAdapter`, paper execution semantics, or `.unwrap()` cleanup
- **THEN** it SHALL be kept out of P0.6 unless a separate approved OpenSpec change brings it into scope

### Requirement: Runtime Environment Inventory

P0.6 SHALL record a redacted inventory for the selected qmt_live runtime before claiming runtime readiness.

#### Scenario: Runtime is available

- **WHEN** a miniQMT/Windows Bridge runtime is selected for P0.6
- **THEN** the evidence SHALL include OS/runtime label, bridge host label, account type label, bridge contract version when available, qmt capability summary, kill-switch status, and Quantix commit hash
- **AND** the evidence SHALL NOT include raw account IDs, credentials, raw broker logs, or host-specific secrets

#### Scenario: Runtime is unavailable

- **WHEN** no suitable miniQMT/Windows Bridge runtime is available
- **THEN** P0.6 SHALL record a blocked or deferred readiness decision instead of fabricating runtime evidence

### Requirement: Read-Only Runtime Smoke

P0.6 SHALL validate existing qmt_live read-only command surfaces before proposing any controlled canary.

#### Scenario: Running read-only smoke

- **WHEN** runtime smoke is executed
- **THEN** it SHALL prefer existing read-only commands for status checklist, preview, query, audit evidence, and manual-intervention reporting
- **AND** it SHALL record summarized, redacted pass/fail evidence
- **AND** it SHALL NOT submit or cancel broker orders

### Requirement: Failure-Boundary Drill

P0.6 SHALL prove operator-visible fail-closed behavior for expected qmt_live runtime failures.

#### Scenario: Expected failures occur

- **WHEN** bridge unavailable, qmt disabled, non-live or ambiguous qmt mode, missing required qmt capability, or enabled kill switch is observed or simulated
- **THEN** the resulting output SHALL be fail-closed and operator-readable
- **AND** read-only inspection SHALL remain available where the existing safety model allows it
- **AND** broker/runtime state SHALL NOT be mutated

### Requirement: Readiness Decision

P0.6 SHALL end with an explicit readiness decision before any later qmt_live canary proposal.

#### Scenario: Closing P0.6

- **WHEN** P0.6 is ready for closeout
- **THEN** the final report SHALL choose exactly one of `ready_for_canary_proposal`, `blocked_by_environment`, `blocked_by_command_gap`, `blocked_by_safety_gap`, or `blocked_by_manual_intervention`
- **AND** it SHALL list evidence files, commands, commit hash, redaction policy, residual risks, and required approval before any live canary
- **AND** it SHALL NOT itself authorize automatic live trading
