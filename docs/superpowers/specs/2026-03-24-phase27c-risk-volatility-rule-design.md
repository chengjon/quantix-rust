# Phase 27C Risk Volatility Rule Design

**Date:** 2026-03-24
**Status:** Draft for user file review
**Depends On:** Phase 27A local risk baseline and Phase 27B live import risk mirror

> This document defines the next risk-rule slice: add a stock-level volatility gate for new buys, using recent daily bars and ATR-based volatility, without introducing a new account lock or mutating existing risk-state semantics.

---

## Goal

Build the smallest useful volatility rule so a user can:

1. configure a volatility threshold with existing `risk rule` commands
2. reject new buy orders for symbols whose recent volatility is too high
3. reuse the same rule across `trade buy`, `strategy run --mode paper`, and `strategy run --mode mock_live`
4. keep current `daily-loss-limit` buy-lock semantics unchanged
5. prepare a clean extension point for later industry rules and auto-deleverage

This slice must not:

- create a new account-level buy lock
- affect sell orders
- introduce a second risk-rule command surface
- silently bypass the rule when price history is unavailable
- add live-import-specific volatility evaluation in v1

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. set a volatility threshold with the existing `risk rule set` command
2. enable or disable the rule with existing toggles
3. see the rule in `risk rule list` and `risk status`
4. have buy orders rejected when recent volatility exceeds the threshold
5. receive a hard error when volatility could not be evaluated

### Exact CLI boundary

This slice does not add new commands. It extends the existing rule type domain:

```bash
quantix risk rule set --type volatility-limit --value 4%
quantix risk rule enable --type volatility-limit
quantix risk rule disable --type volatility-limit
quantix risk rule list
quantix risk status
```

Rules:

- `volatility-limit` accepts percentage values only
- command surface stays identical to `position-limit` and `daily-loss-limit`
- the rule is visible in both `paper` and `live_import` rule listings
- only buy-evaluation paths execute this rule in v1

### Explicitly deferred

This slice does not include:

- account-level volatility buy locks
- volatility-triggered risk log events
- configurable ATR period / timeframe / formula
- industry rules
- automatic deleveraging
- volatility evaluation for `risk status|pnl|position` read paths
- live-import-only execution paths

## Approaches Considered

### Option A: ATR ratio rule on buy evaluation

Definition:

- load recent daily bars
- compute `ATR / latest_close * 100`
- reject the buy when the value exceeds the threshold

Pros:

- reuses existing ATR implementation
- integrates naturally with current `check_buy()` entry point
- easy to explain to a trading user

Cons:

- depends on historical bars being available at buy-evaluation time

### Option B: Return standard deviation rule

Definition:

- compute rolling standard deviation of recent daily returns
- reject the buy above threshold

Pros:

- closer to classic statistical volatility

Cons:

- adds more new implementation than needed for v1
- less aligned with already-available project indicators

### Option C: Account-level volatility lock

Definition:

- if volatility exceeds threshold, lock all new buys

Pros:

- superficially similar to `daily-loss-limit`

Cons:

- semantically different from stock selection risk
- unnecessarily broad for the first slice
- introduces a new persistent lock source too early

## Recommendation

Choose **Option A**.

The first volatility slice should remain a stock-level pre-buy gate, not an account-level state machine. The project already has an ATR indicator and a daily-bar loading boundary, so the lowest-risk path is:

- add `volatility-limit`
- evaluate it only during buy checks
- reject when ATR-derived volatility is above the configured threshold
- fail closed when volatility cannot be computed

## Rule Semantics

### Rule name

Add a new `RiskRuleType`:

- `VolatilityLimit`

CLI string:

- `volatility-limit`

### Rule value

Supported value type:

- percentage only

Examples:

- `3%`
- `4.5%`

Rejected examples:

- `50000`
- `14`

### Fixed first-version formula

The v1 formula is fixed:

```text
volatility_pct = ATR(14) / latest_close * 100
```

Constants:

- `period = 14`
- `timeframe = daily`
- `price_anchor = latest_close`

These parameters are intentionally not exposed to the CLI in v1.

### Evaluation target

The rule evaluates the target symbol of a pending buy only.

It does not:

- inspect the entire account
- evaluate current holdings continuously
- mutate lock state

## Trigger Timing And Data Source

### Trigger points

The rule executes only on new buy evaluation paths:

1. `trade buy`
2. strategy risk evaluation during `strategy run --mode paper`
3. strategy risk evaluation during `strategy run --mode mock_live`
4. later execution-daemon buy evaluation paths that already reuse the same risk bridge

The rule does not execute on:

- sell orders
- `risk status`
- `risk pnl`
- `risk position`
- `risk log`

### Data-source boundary

Volatility computation should live inside the risk subsystem, not in CLI handlers or strategy bridges.

Add a thin daily-bar loader boundary for risk evaluation so `check_buy()` can:

1. load the latest `period + 1` daily bars for the symbol
2. compute ATR from existing analysis utilities
3. use the latest close as the denominator
4. compare the result against the configured rule

This keeps all buy-evaluation callers on one rule implementation and avoids duplicating ATR logic across CLI and strategy surfaces.

### Reuse strategy bar-loading infrastructure

For v1, the risk-side loader should reuse the same daily-bar loading capability already present in the repository, especially the existing fallback path that can read TDX day files when the primary loader is unavailable.

The goal is to reuse the current bar-loading boundary, not to invent a new market-data protocol for risk.

## Failure Semantics

The volatility rule is fail-closed.

### Cases

1. rule not configured

- no volatility check runs
- existing buy behavior stays unchanged

2. rule configured but daily bars are missing, insufficient, or unreadable

- reject the buy
- return a hard error
- do not write a risk event

3. rule configured and computed successfully, but the symbol exceeds threshold

- reject the buy
- return a hard error
- do not write a risk event

### Why fail-closed

If the user explicitly enables `volatility-limit`, a missing-data path must not silently turn the rule into a no-op. Hard rejection is safer and keeps the contract explicit.

## CLI And Output Semantics

### Rule list and status

`risk rule list` and the `[规则]` section of `risk status` should display the new rule exactly like existing rules:

```text
volatility-limit    4%          enabled
```

### No new log event type

This slice does not add dedicated volatility-triggered log events.

Rationale:

- current `risk log` is for durable state transitions
- volatility rejection is a point-in-time evaluation outcome
- logging every rejection would create noisy operator output for strategy and daemon paths

Only rule-management events are recorded:

- `rule-set`
- `rule-enabled`
- `rule-disabled`

### Error messages

Recommended over-threshold message:

```text
risk rule volatility-limit 已超限: code=000001 threshold=4% actual=5.37% period=14
```

Recommended evaluation-failed message:

```text
risk rule volatility-limit 检查失败: code=000001 原因=可用日线不足，至少需要 15 条
```

## Architecture Boundary

### Risk service remains the orchestrator

`RiskService::check_buy()` remains the single orchestration entry point for buy checks.

Its evaluation order becomes:

1. refresh risk state
2. enforce account buy lock from `daily-loss-limit`
3. enforce `position-limit`
4. enforce `volatility-limit`

### Keep indicator I/O outside core rule state

To avoid turning `RiskService` into a monolithic state-and-I/O object, add a focused helper boundary for volatility evaluation, for example:

- a `RiskBarLoader` trait
- a `evaluate_volatility_limit(...)` helper

That helper owns:

- bar loading
- minimum-bar validation
- ATR calculation
- threshold comparison
- formatted error construction

This same pattern can later host industry-rule evaluation without bloating `check_buy()`.

### No new persistent risk-state fields

This slice does not require:

- new fields on `RiskState`
- new lock-state sources
- new replay/recovery semantics

The rule is configuration-only. Its runtime effect is transient buy rejection.

## Testing Scope

The minimum test matrix is:

1. `risk rule set --type volatility-limit --value 4%` succeeds
2. `risk rule set --type volatility-limit --value 50000` fails
3. existing buy behavior stays unchanged when the rule is absent
4. a buy is allowed when computed volatility is below threshold
5. a buy is rejected when computed volatility is above threshold
6. a buy is rejected when fewer than 15 daily bars are available
7. sell paths remain unaffected
8. `risk rule list` shows `volatility-limit`
9. `risk status` shows `volatility-limit`
10. `strategy run --mode paper` receives the same rejection reason
11. `strategy run --mode mock_live` receives the same rejection reason

## File Impact Preview

Expected primary touch points:

- `src/risk/models.rs`
- `src/risk/service.rs`
- `src/risk/mod.rs`
- `src/cli/mod.rs`
- `src/cli/tests/risk.rs`
- `tests/risk_service_test.rs`
- `README.md`
- `docs/USER_MANUAL.md`
- `tests/repo_hygiene_test.rs`

Additional file(s) may be introduced for the risk-side daily-bar loading boundary if that keeps `risk/service.rs` focused.

## Acceptance Criteria

This slice is complete when:

1. users can configure `volatility-limit` through the existing `risk rule` CLI
2. buy orders are rejected when ATR-derived volatility exceeds the configured threshold
3. missing or insufficient bars reject the buy instead of silently bypassing the rule
4. sell orders and account-lock semantics remain unchanged
5. strategy buy-evaluation paths and direct trade buys return the same rule reason
6. docs and hygiene tests reflect the new supported rule type

## Memory Sync Note

- Graphiti write was attempted for `quantix_rust_main` on 2026-03-24.
- The ingest hit a rate-limit retry path before completion.
- Graphiti backfill required
