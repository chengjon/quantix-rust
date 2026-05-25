## ADDED Requirements

### Requirement: OpenSpec-Governed Architecture Remediation
All architecture remediation work from the reviewed audit SHALL be executed through an active OpenSpec change before code changes are made.

#### Scenario: Starting an audit remediation task
- **WHEN** work starts on any GitHub issue from #64 through #72
- **THEN** the active OpenSpec change SHALL identify the requirement, design decision, task item, validation gate, and related GitHub issue

#### Scenario: Conflicting audit documents
- **WHEN** the primary audit and review report disagree about remediation scope
- **THEN** the review report SHALL take precedence

### Requirement: Characterization Before Refactor
Architecture seam changes SHALL have characterization coverage before behavior-preserving refactors are applied.

#### Scenario: Refactoring a dependency seam
- **WHEN** a task changes a seam between `strategy`, `execution`, `risk`, `market`, `db`, `core`, or `cli`
- **THEN** tests SHALL first capture the current observable behavior for that seam

#### Scenario: Editing a symbol
- **WHEN** a function, method, class, or refactor target is edited
- **THEN** GitNexus impact analysis SHALL be run before the edit and GitNexus detect_changes SHALL be run before closure

### Requirement: Stable Shared Signal Type
The shared `Signal` value type SHALL live in a low-level core module without moving the full async `Strategy` interface into core.

#### Scenario: Updating Signal consumers
- **WHEN** execution, monitoring, runtime codec, analysis, or strategy code needs only the `Signal` value type
- **THEN** it SHALL import `Signal` from the shared core signal module

#### Scenario: Preserving Strategy ownership
- **WHEN** the `Strategy` trait depends on strategy behavior or `Kline`
- **THEN** it SHALL remain owned by the strategy layer

### Requirement: Domain Logic Independent From Adapters
Domain calculations and domain types SHALL not depend directly on storage, runtime, network, or test-support adapters.

#### Scenario: Risk industry refactor
- **WHEN** risk industry classification is refactored
- **THEN** domain records, normalization, and classification SHALL be separated from SQLite persistence

#### Scenario: Risk volatility refactor
- **WHEN** risk volatility is refactored
- **THEN** volatility calculation SHALL operate over risk-owned plain inputs rather than strategy runtime defaults

#### Scenario: Market strength refactor
- **WHEN** market strength is refactored
- **THEN** calculation SHALL operate over deterministic input rows while DB, fundamental, EastMoney, risk, and anomaly acquisition remain adapter concerns

### Requirement: Focused Safety and Hygiene Remediation
Safety cleanup SHALL target behavior risks and SHALL avoid unrelated formatting or cosmetic churn.

#### Scenario: Replacing unwrap, expect, panic, or print macros
- **WHEN** a safety hotspot is remediated
- **THEN** the task SHALL classify whether it is production, test-only, or macro-generated and SHALL add or preserve focused validation for the behavior risk

#### Scenario: Safety pattern count without behavior risk
- **WHEN** a pattern count is high but no runtime behavior risk is identified
- **THEN** the code SHALL NOT be changed solely to reduce the count

### Requirement: Large File Splits After Seams Stabilize
Large implementations SHALL be split only after relevant dependency seams are corrected and covered by tests.

#### Scenario: Splitting a large implementation
- **WHEN** a file from issue #71 is split
- **THEN** the new modules SHALL have meaningful ownership, focused tests, and no pass-through-only boundaries

#### Scenario: Preserving execution interfaces
- **WHEN** execution implementation files are split
- **THEN** existing execution interfaces such as `ExecutionAdapter`, `RiskEvaluator`, and `FillDeltaApplier` SHALL be preserved unless a separate approved spec changes them
