# tdx-api-import-e2e Specification

## ADDED Requirements

### Requirement: OpenSpec-Governed tdx-api Import E2E

tdx-api import release-readiness work SHALL be governed by this active OpenSpec change before implementation or E2E closure work is started.

#### Scenario: Starting tdx-api import hardening

- **WHEN** work starts on tdx-api ClickHouse K-line import or TDengine tick import acceptance
- **THEN** the executor SHALL use `openspec/changes/tdx-api-import-e2e-hardening/` as the governing proposal, task list, and spec delta

#### Scenario: Deferring unrelated follow-up work

- **WHEN** a follow-up item concerns broad clippy cleanup, logging hygiene, data-source abstraction, WebSocket streaming, progress UI, notification, systemd packaging, factor computation, or backtest auto-preparation
- **THEN** it SHALL be kept out of this change unless a separate approved OpenSpec change brings it into scope

### Requirement: External Dependency Preflight

E2E gates SHALL verify their real external dependencies before import commands are executed.

#### Scenario: All dependencies are available

- **WHEN** tdx-api, ClickHouse, and TDengine connectivity checks all succeed through the same configuration paths used by the import commands
- **THEN** the E2E gate SHALL proceed and record the checked endpoints or database targets with secrets redacted

#### Scenario: A dependency is unavailable

- **WHEN** tdx-api, ClickHouse, or TDengine connectivity fails before an import gate
- **THEN** the gate SHALL stop with status BLOCKED and SHALL record the missing dependency, command, endpoint or target, and error summary

### Requirement: ClickHouse K-line Import E2E

The ClickHouse K-line gate SHALL exercise the tdx-api `import-klines --all` code path against a real tdx-api and ClickHouse environment.

#### Scenario: Running the K-line import gate

- **WHEN** the K-line E2E gate runs
- **THEN** it SHALL execute `quantix data tdx-api import-klines --all --type day` or the same command with an explicitly recorded `--exchange <market>` bound

#### Scenario: Validating K-line import output

- **WHEN** the K-line import command exits successfully
- **THEN** the evidence SHALL include command exit code, selected market scope, imported count, skipped count, failed count, elapsed time, ClickHouse row count, minimum date, maximum date, at least one imported symbol, and source marker for THS front-adjusted data

#### Scenario: Checking K-line data sanity

- **WHEN** imported K-line rows are queried from ClickHouse
- **THEN** validation SHALL confirm non-zero rows, `high >= low`, open and close within high and low when those fields are present, non-negative volume, and ordered dates per sampled symbol

#### Scenario: Checking incremental rerun behavior

- **WHEN** the same K-line import command is rerun without `--force`
- **THEN** the rerun SHALL either import zero new rows or only rows newer than the previous maximum imported date

### Requirement: TDengine Tick Import E2E

The TDengine tick gate SHALL exercise the tdx-api `import-ticks` path against a real tdx-api and TDengine environment.

#### Scenario: Running the tick import gate

- **WHEN** the tick E2E gate runs
- **THEN** it SHALL execute `quantix data tdx-api import-ticks --code <code> --date <YYYYMMDD>` with the selected code/date recorded in evidence

#### Scenario: Validating tick import output

- **WHEN** the tick import command exits successfully
- **THEN** the evidence SHALL include command exit code, code, date, imported tick count, elapsed time, TDengine row count, minimum timestamp, maximum timestamp, and target table or stable name

#### Scenario: Checking tick data sanity

- **WHEN** imported tick rows are queried from TDengine
- **THEN** validation SHALL confirm non-zero rows, valid timestamps, positive prices for trade rows, and non-negative volumes

#### Scenario: Selected code/date has no tick data

- **WHEN** tdx-api returns no tick rows for the selected code/date
- **THEN** the executor SHALL retry with a documented alternate trading day before marking the tick gate BLOCKED

### Requirement: Evidence Artifacts

E2E evidence SHALL be stored as compact, reviewable repository artifacts without secrets or large raw logs.

#### Scenario: Saving E2E evidence

- **WHEN** an E2E gate completes, fails, or is blocked
- **THEN** evidence SHALL be saved under `docs/reports/evidence/tdx-api-import-e2e-<YYYYMMDD>/` with command lines, exit codes, dependency targets, row counts, validation summaries, and failure diagnostics

#### Scenario: Handling secrets and raw logs

- **WHEN** evidence includes endpoints, credentials, tokens, or large command output
- **THEN** secrets SHALL be redacted and raw large logs SHALL remain outside the repository with only compact summaries committed

### Requirement: Release Closure Gate

This change SHALL not be closed as release-ready until test and release-build evidence is captured after the E2E hardening work.

#### Scenario: Closing the change

- **WHEN** the E2E gates are ready for closure
- **THEN** `cargo test -p quantix-cli --quiet` and `cargo build --release --quiet` SHALL both have captured exit 0 evidence

#### Scenario: Rust code changes are made

- **WHEN** implementation changes Rust functions, methods, classes, or refactor targets
- **THEN** GitNexus impact SHALL run before editing, task-relevant tests and clippy SHALL run before closure, and GitNexus `detect_changes` SHALL confirm the affected scope before the change is closed
