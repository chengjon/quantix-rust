# Phase 27A Risk Design

**Date:** 2026-03-11
**Status:** Approved in-session
**Depends On:** Phase 26A green baseline (`phase26-trade`)

> This document is the Phase 27A source of truth in this branch. It intentionally supersedes the broader ideas in `docs/archive/plans/PHASE27_RISK_MANAGEMENT_DESIGN.md` where those ideas exceed the approved MVP boundary or do not fit the current repository structure.

---

## Goal

Build the smallest useful risk-management loop on top of paper trade:

- configure paper-trade risk rules locally
- block new paper-trade buys when a position-limit rule would be exceeded
- lock new paper-trade buys for the rest of the trading day after a daily-loss rule triggers
- allow sells even when buy lock is active
- inspect current risk status from the CLI

This phase does not attempt to become a full broker risk engine, a real-account risk monitor, or a multi-channel alerting system.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Set a single-stock maximum position rule for the default paper-trade account
2. Set a daily-loss rule for the default paper-trade account
3. See which rules are configured and whether they are enabled
4. Prevent a `trade buy` that would exceed the configured position limit
5. Prevent any further `trade buy` after the daily-loss rule has triggered for the trading day
6. View the current paper-trade risk status

Anything beyond those jobs is out of scope for Phase 27A.

### Exact CLI boundary

Only implement:

```bash
quantix risk rule set --type position-limit --value 20%
quantix risk rule set --type daily-loss-limit --value 50000
quantix risk rule set --type daily-loss-limit --value 5%
quantix risk rule list
quantix risk rule enable --type position-limit
quantix risk rule disable --type daily-loss-limit
quantix risk status
```

Rules:

- Phase 27A only covers the default paper-trade account
- `risk rule` commands work even before `trade init`
- `risk status` requires an initialized paper-trade account
- `trade buy` must consult risk rules before execution
- `trade sell` is never blocked by risk buy lock
- position-limit checks are evaluated against the projected post-buy position ratio
- daily-loss lock persists for the trading day and auto-resets when the trading date changes
- Phase 27A reminders are limited to CLI-visible blocking errors and persisted lock/status fields

Explicitly defer:

- external push, popup, or sound alerts
- real-account import, monitoring, or reporting
- `risk pnl`
- `risk position`
- `risk log`
- `risk lock status`
- `risk lock release`
- `risk trigger-history`
- volatility, sector, or board exposure rules
- auto-reduction / auto-liquidation
- live quote mark-to-market valuation
- automatic linkage from stop triggers into risk-triggered selling

## Key Constraint From Phase 26A

Phase 26A paper trade is not quote-driven.

So Phase 27A cannot calculate true intraday mark-to-market drawdown yet. The only stable source of truth is the paper-trade bookkeeping snapshot:

- `estimated_position_value = sum(position.volume * position.last_trade_price)`
- `estimated_total_assets = available_cash + estimated_position_value`

This means the Phase 27A daily-loss rule is based on execution-price bookkeeping equity, not live market prices. That is an intentional limitation inherited from the approved Phase 26A boundary.

## Approaches Considered

### Option A: Put risk rules directly into `trade` state

Pros:

- one file
- fewer top-level modules at first glance

Cons:

- mixes account bookkeeping with independent risk policy
- makes future risk iteration harder
- couples Phase 27 changes tightly to Phase 26 storage schema

### Option B: Add a dedicated `risk` domain with its own JSON store and let CLI trade handlers consult it

Pros:

- smallest change that still preserves clear boundaries
- matches the repository’s existing lightweight local-state pattern
- avoids changing trade accounting semantics
- keeps sell-bypass and buy-guard rules explicit in one place

Cons:

- introduces one more local JSON file
- requires the trade CLI handler to coordinate trade state and risk state

### Option C: Build a SQLite-backed risk event engine

Pros:

- stronger queryability
- easier future trigger history reporting

Cons:

- too much scope for Phase 27A
- adds schema and migration overhead before it is needed
- solves reporting problems that the current MVP does not expose yet

## Recommendation

Choose **Option B**.

Phase 27A should add a dedicated `risk` domain with local JSON persistence, and the `trade` CLI handler should call into that domain before buy execution and after every trade mutation. That is the smallest path that still satisfies the user’s real jobs without overbuilding a broker-style risk system.

## Architecture

### Domain split

- `src/risk/*` owns risk models, rule parsing, evaluation, and JSON persistence
- `src/cli/mod.rs` owns `risk` command parsing
- `src/cli/handlers/risk.rs` owns `risk` command execution and terminal output
- `src/cli/handlers/trade.rs` owns buy pre-check and post-trade risk-state sync
- `src/core/runtime.rs` owns risk-state path resolution

### Storage choice

Use a dedicated local JSON file pointed to by `QUANTIX_RISK_PATH`.

Default path:

```text
~/.quantix/risk/risk_state.json
```

Reason:

- keeps risk policy independent from paper-trade accounting
- mirrors the repo’s watchlist/trade local-state pattern
- avoids a database dependency for a two-rule MVP

## Minimal Data Model

```rust
pub struct RiskState {
    pub version: u32,
    pub account_id: String,
    pub rules: Vec<RiskRule>,
    pub daily_baseline: Option<DailyRiskBaseline>,
    pub buy_lock: BuyLockState,
}

pub struct RiskRule {
    pub rule_type: RiskRuleType,
    pub value: RuleValue,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum RiskRuleType {
    PositionLimit,
    DailyLossLimit,
}

pub enum RuleValue {
    Percentage(Decimal),
    Amount(Decimal),
}

pub struct DailyRiskBaseline {
    pub trading_date: NaiveDate,
    pub starting_total_assets: Decimal,
}

pub struct BuyLockState {
    pub locked: bool,
    pub reason: Option<String>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub trading_date: Option<NaiveDate>,
}

pub struct RiskStatus {
    pub account_id: String,
    pub trading_date: NaiveDate,
    pub starting_total_assets: Decimal,
    pub current_total_assets: Decimal,
    pub daily_pnl: Decimal,
    pub daily_pnl_pct: Decimal,
    pub buy_locked: bool,
    pub lock_reason: Option<String>,
    pub position_ratios: Vec<PositionRiskRow>,
    pub rules: Vec<RiskRuleSnapshot>,
}
```

Notes:

- rule configuration and derived lock/baseline state live together
- there is no trigger-history list in Phase 27A
- there is no real-account model in Phase 27A
- there is no quote-driven market-value field in Phase 27A

## Evaluation Model

### Position limit

When `trade buy` is requested:

- load current paper-trade account snapshot
- compute projected post-buy position value for the target code using the execution price
- compute projected total assets using the same bookkeeping valuation model
- reject the buy if `projected_position_ratio > configured_limit`

This is a per-order validation, not a day-long lock.

### Daily loss

When risk state is synchronized for a trading day:

- if there is no baseline for `today`, create one from current total assets and clear any previous buy lock
- compute `daily_pnl = current_total_assets - starting_total_assets`
- compute `daily_pnl_pct = daily_pnl / starting_total_assets`
- if the configured daily-loss rule is exceeded, set `buy_lock = true` for the trading day

Once daily-loss lock is triggered, it stays active until the trading date rolls over or the account is reset.

## Command Semantics

### `risk rule set`

- upserts one rule by type
- preserves the other rule if it already exists
- parses `--value` as:
  - percentage when it ends with `%`
  - amount otherwise
- `position-limit` only accepts percentage values
- `daily-loss-limit` accepts either amount or percentage values

### `risk rule list`

- returns all configured rules with enabled/disabled state
- does not require an initialized trade account

### `risk rule enable` / `risk rule disable`

- toggles the target rule if it exists
- returns a user-facing error when the rule has not been configured yet

### `risk status`

- requires an initialized trade account
- reads current paper-trade cash snapshot
- refreshes day-rollover and daily-loss evaluation if needed
- prints:
  - current total assets
  - current daily PnL and daily PnL percentage
  - whether buy is locked
  - current lock reason
  - enabled/disabled rules
  - current per-position ratios

### `trade init` / `trade reset`

- preserve configured rules
- clear buy lock
- set or refresh the daily baseline from the reset account snapshot

### `trade buy`

Execution order:

1. refresh daily baseline / lock state
2. if daily-loss lock is active, reject the buy
3. if position-limit rule would be exceeded, reject the buy
4. execute the buy through the existing trade service
5. refresh risk state from the post-buy account snapshot

### `trade sell`

Execution order:

1. execute the sell through the existing trade service
2. refresh risk state from the post-sell account snapshot

Sell is always allowed, even when buy lock is active.

## Testing Strategy

Add tests for:

- CLI parser coverage for `risk rule set/list/enable/disable` and `risk status`
- risk rule-value parsing and validation
- JSON storage round-trip for rules, baseline, and lock state
- risk service evaluation for:
  - position-limit rejection
  - daily-loss trigger by amount
  - daily-loss trigger by percentage
  - day rollover clearing lock and resetting baseline
- handler-level `risk` command flows
- handler-level `trade buy` rejection when risk blocks the order
- handler-level `trade sell` success while buy lock is active

## Non-Goals

Phase 27A is not:

- a real-account risk dashboard
- a push-notification system
- a live quote risk engine
- a historical trigger reporting feature
- an auto-liquidation engine
- a replacement for single-stock stop rules
