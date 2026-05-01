# Phase 4 Validation

## Phase

Phase 4: Risk rule enhancement

## Goal-Backward Validation

`RSK-01` says the risk enhancement phase must cover:

1. live-import-based risk visibility
2. volatility rules
3. industry rules
4. auto-reduce

Validation should therefore work backward from those four outcomes instead of only checking whether code files changed.

## Current Coverage Assessment

### Live Import

Current code already supports:

- standardized live-trade import
- mirror-account rebuild
- `risk status|pnl|position --source live_import --account <ID>`

Validation question:

- does the planned work preserve the isolation and read-only contract of `live_import`?

### Volatility Rules

Current code already supports:

- `volatility-limit` rule parsing
- ATR-based buy rejection
- fail-closed handling for missing or insufficient bars

Validation question:

- does the planned work keep these semantics stable while extending Phase 4?

### Industry Rules

Current code partially supports:

- runtime industry resolution
- `industry-blocklist`
- placeholder-only `industry-limit`

Validation question:

- does the phase end with industry rules being more complete than the current placeholder state?

### Auto-Reduce

Current code partially supports:

- rule type parsing
- trigger decision helper
- event-type scaffolding

Validation question:

- does the phase end with a real operator-visible contract for `auto-reduce`, rather than only latent types and helpers?

## Required End-State

Phase 4 should only be considered complete when:

- live import remains functional and explicitly bounded
- volatility rules remain regression-backed
- industry rules include at least one real concentration or enforcement path beyond blocklist-only behavior
- auto-reduce has explicit and test-backed semantics

## Validation Dimensions

### 1. Behavior Correctness

- `live_import` remains read-only and separate from `paper`
- `volatility-limit` still rejects buys when the configured threshold is exceeded
- `industry-limit` no longer behaves as a no-op placeholder
- `auto-reduce` behavior is explicit, bounded, and testable

### 2. Operator Correctness

- CLI output tells operators what happened and why
- docs distinguish implemented behavior from deferred behavior
- no command implies automatic selling unless that behavior is actually delivered

### 3. Safety Correctness

- risk extensions do not silently widen execution authority
- placeholder features are not documented as complete
- live-import views remain observational rather than mutating execution state

## Verification Loop

For each Phase 4 plan, verify:

1. requirement slice addressed
2. primary behavior tests pass
3. CLI/operator regressions pass
4. docs/hygiene reflect the true delivered boundary

## Suggested Exit Checks

- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_service_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_volatility_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_rebuild_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib risk`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test`

## Failure Conditions

Phase 4 is not complete if any of the following remain true:

- `industry-limit` is still only a placeholder
- `auto-reduce` still has no operator-visible contract
- docs continue to describe major delivered behavior as deferred
- risk enhancements mutate the wrong account boundary or break execution-mainline safety
