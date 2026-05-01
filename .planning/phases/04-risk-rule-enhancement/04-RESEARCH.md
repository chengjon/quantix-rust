# Phase 4 Research

## Phase

Phase 4: Risk rule enhancement

## Current State

Phase 4 is not a greenfield risk phase. The repository already contains meaningful risk infrastructure and several delivered slices that map into `RSK-01`:

- `src/risk/importer.rs`, `src/risk/import_store.rs`, and `src/risk/rebuild.rs` already support standardized `CSV/JSON` live-trade import and mirror-account rebuild.
- `src/risk/volatility.rs` already implements `volatility-limit` as an ATR-based buy gate.
- `src/risk/industry.rs`, `src/risk/industry_store.rs`, and `src/risk/service.rs` already support runtime industry resolution plus `industry-blocklist`.
- `src/cli/handlers/risk.rs` already exposes the risk import / rebuild / status / pnl / position CLI surfaces.
- README, USER_MANUAL, and repo hygiene tests already document a large part of the current risk boundary.

That means Phase 4 should not re-deliver the already-implemented Phase 27 risk slices. Instead, it should formalize the current baseline and close the remaining real gaps against `RSK-01`.

## Gaps Against RSK-01

### 1. `industry-limit` exists in the model, but not in runtime behavior

`RiskRuleType::IndustryLimit` is accepted by parsing and CLI surfaces, but `check_industry_limit(...)` in `src/risk/service.rs` is explicitly a placeholder:

- it validates the rule type
- it logs debug output
- it does not actually block buys or compute sector concentration

This is the clearest Phase 4 implementation gap.

### 2. `auto-reduce` exists as a trigger decision, but not as an execution workflow

`check_auto_reduce_trigger(...)` returns an `AutoReduceDecision`, but there is no end-to-end operator or execution path that:

- evaluates the rule in a runtime workflow
- records the trigger as a meaningful risk event
- expresses what "reduce 50%" means operationally
- distinguishes decision-only behavior from execution or recommendation output

Current docs also say `auto-reduce` is still deferred, which confirms the feature is not yet truly shipped despite the model/type support.

### 3. Existing risk slices need phase-level closure rather than ad hoc documentation

`live_import`, `volatility-limit`, and `industry-blocklist` are already implemented, but Phase 4 still needs a phase-level artifact that says:

- which parts of `RSK-01` are already satisfied by current code
- which parts are still missing
- which docs and tests lock the final boundary

Without this, the roadmap still shows Phase 4 as entirely pending even though part of the requirement surface already exists in production code and docs.

## Likely Code Surfaces

Primary implementation files:

- `src/risk/service.rs`
- `src/risk/models.rs`
- `src/cli/handlers/risk.rs`
- `src/cli/commands/risk.rs`
- `src/risk/rebuild.rs`

Primary regression files:

- `tests/risk_service_test.rs`
- `tests/risk_volatility_test.rs`
- `tests/risk_rebuild_test.rs`
- `src/cli/tests/risk.rs`
- `tests/repo_hygiene_test.rs`

Primary docs:

- `README.md`
- `docs/USER_MANUAL.md`

## Recommended Plan Shape

### Plan 01: Phase 4 baseline and live-import / current-rule contract lock

Goal:

- formalize which `RSK-01` surfaces are already real today

Why first:

- it prevents Phase 4 from re-implementing shipped behavior
- it creates a stable baseline before deeper implementation work
- it aligns planning state with current code reality

Likely work:

- lock the current live-import / volatility / industry-blocklist contract in tests/docs/planning
- verify the CLI/operator wording matches what code actually does today
- explicitly mark `industry-limit` and `auto-reduce` as the remaining implementation gaps

### Plan 02: Implement `industry-limit` as a real concentration gate

Goal:

- replace the current placeholder with real industry concentration enforcement

Why second:

- it is the largest obvious gap in current runtime logic
- it can reuse the existing industry resolution and risk snapshot machinery
- it closes the "industry rules" part of `RSK-01` more completely

Likely work:

- compute current and projected industry exposure
- reject buys when projected concentration exceeds the configured threshold
- add targeted service/CLI/operator regressions

### Plan 03: Turn `auto-reduce` into an operator-visible workflow

Goal:

- turn the current decision helper into a real, test-backed workflow with explicit semantics

Why third:

- it is the last major missing slice within `RSK-01`
- it depends on the phase baseline being explicit
- it needs careful boundary decisions to avoid silently auto-trading without operator clarity

Likely work:

- decide whether v1 is recommendation-only, audit-only, or explicit execution-assisted
- surface trigger diagnostics and event logging
- document and lock the chosen semantics in docs/tests

## Verification Strategy

Phase 4 should validate at three layers:

### 1. Current-slice contract coverage

Use:

- `tests/risk_service_test.rs`
- `tests/risk_volatility_test.rs`
- `tests/risk_rebuild_test.rs`
- `src/cli/tests/risk.rs`

to prove:

- live import and rebuild stay intact
- volatility and industry-blocklist keep their current fail-closed semantics
- CLI surfaces remain aligned with the runtime behavior

### 2. Remaining-gap implementation coverage

Use:

- service-level tests for `industry-limit`
- risk CLI tests for operator-visible behavior
- rebuild/import tests where live-import-backed status or position views matter

to prove:

- `industry-limit` stops being a placeholder
- `auto-reduce` has a defined and test-backed behavior contract

### 3. Documentation and hygiene coverage

Use:

- `tests/repo_hygiene_test.rs`

to prove:

- docs match the delivered risk boundary
- docs stop claiming deferred behavior after it is implemented
- docs do not overstate automation if `auto-reduce` remains operator-mediated

## Risks And Constraints

- Do not break the already-stable execution mainline while extending risk behavior.
- Do not silently enable fully automatic trading behavior under the name `auto-reduce` without explicit operator-facing semantics.
- Reuse the existing live-import mirror-account boundary rather than coupling it back into `paper_trade.json`.
- Prefer incremental closure of current placeholder seams over broad redesign of the entire risk subsystem.

## Candidate Verification Commands

- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_service_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_volatility_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test risk_rebuild_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib risk`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test`

## Planning Recommendation

Proceed with a three-plan phase:

1. lock the current live-import / volatility / industry-blocklist baseline and explicitly scope the remaining gaps
2. implement real `industry-limit` enforcement
3. define and ship the minimal `auto-reduce` workflow with clear operator semantics

This is the smallest shape that closes `RSK-01` without pretending the currently deferred `auto-reduce` and placeholder `industry-limit` behavior are already complete.
