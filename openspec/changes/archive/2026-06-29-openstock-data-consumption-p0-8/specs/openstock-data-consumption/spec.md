# openstock-data-consumption Specification Delta

## ADDED Requirements

### Requirement: OpenStock Consumption Planning Boundary

The system SHALL define OpenStock data consumption as a broker-independent market-data line before implementing provider code.

#### Scenario: Planning does not require broker runtime

- **WHEN** the OpenStock data-consumption plan is created
- **THEN** it SHALL NOT require qmt_live, miniQMT, broker credentials, or Windows Bridge runtime availability.

#### Scenario: Planning preserves existing source boundaries

- **WHEN** OpenStock slices are planned
- **THEN** they SHALL preserve existing `tdx_api`, `bridge_tdx`, `eastmoney`, and miniQMT market-manifest behavior unless a later slice explicitly authorizes a change.

### Requirement: Fixture-Owned Development

OpenStock parser and normalization work SHALL start from committed fixtures or local artifacts that are safe for CI.

#### Scenario: Tests avoid live network calls

- **WHEN** CI runs OpenStock-related tests
- **THEN** default tests SHALL NOT call live OpenStock endpoints.

#### Scenario: Fixture validation is read-only

- **WHEN** a local OpenStock fixture or artifact is validated
- **THEN** validation SHALL NOT write ClickHouse, broker state, runtime storage, or external systems.

### Requirement: Read-Only Before Persistence

The system SHALL prove read-only parsing, normalization, and downstream consumption before adding any persistence path.

#### Scenario: Persistence requires separate approval

- **WHEN** a slice proposes ClickHouse writes or other persistent storage changes
- **THEN** it SHALL include schema, deduplication, rollback, dry-run, and GitNexus impact evidence before implementation approval.

### Requirement: Downstream Quant Loop Alignment

OpenStock consumption SHALL be sequenced toward a local quant loop that can run without real broker execution.

#### Scenario: First runnable loop is local

- **WHEN** OpenStock data is connected to downstream processing
- **THEN** the first runnable loop SHOULD use indicators, backtest, or paper/mock execution without qmt_live submit/query/cancel behavior.
