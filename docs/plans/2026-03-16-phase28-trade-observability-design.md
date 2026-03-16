# Phase 28A Trade Observability Design

**Date:** 2026-03-16
**Status:** Drafted from in-session design approval
**Depends On:** Phase 27A green baseline (`master` @ `b1a0502`)

> This document is the source of truth for the next approved paper-trade slice: read-side observability on top of the existing Phase 26A/27A write path.

---

## Goal

Build the smallest useful read-side layer on top of paper trade so a user can answer four questions without reading raw JSON:

1. What trades have been executed recently?
2. How much has been paid in fees?
3. What does the paper account look like at an account-summary level?
4. What are current positions worth right now, if live prices are available?

This phase should remain read-mostly. It must not change trade accounting rules, storage format, or risk-rule semantics.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. List recent trade executions, optionally filtered to one code
2. List recent fee rows, optionally filtered to one code
3. View a concise account overview from persisted paper-trade state
4. Recalculate current position value from best-effort live quotes when explicitly requested
5. Keep all commands usable even when quote lookup is partially unavailable

### Exact CLI boundary

Only implement:

```bash
quantix trade history [--code <CODE>] [--limit <N>]
quantix trade fees [--code <CODE>] [--limit <N>]
quantix trade overview [--current]
quantix trade position [--current]
```

Rules:

- `history` and `fees` default to the most recent 20 rows
- `history` and `fees` may filter by `--code`
- `overview` without `--current` uses only persisted paper-trade bookkeeping data
- `overview --current` and `position --current` attempt best-effort live quote lookup
- quote lookup failures do not fail the command; they downgrade the live-price fields
- `position` without `--current` must preserve the existing Phase 26A behavior

### Explicitly deferred

Phase 28A does not include:

- order history search beyond `code` and `limit`
- pagination
- CSV/JSON export
- realized-vs-unrealized tax-lot accounting
- daily equity curves
- benchmark comparison
- cached quote snapshots
- quote-driven persistence back into paper-trade storage
- real-account reconciliation

## Approaches Considered

### Option A: Keep extending `TradeService`

Pros:

- least file count
- no new read-side module

Cons:

- grows `TradeService` into a mixed write/reporting object
- mixes accounting mutations with presentation-oriented aggregation
- makes later quote-aware read paths harder to isolate

### Option B: Add a thin `trade::reporting` read-side module

Pros:

- keeps `TradeService` focused on `init/reset/buy/sell`
- gives all new read-only commands one aggregation surface
- easy to test with existing `PaperTradeState` fixtures
- supports quote-aware overlays without coupling trade storage to quote lookup

Cons:

- introduces one more module
- requires a few new output structs

### Option C: Add a persisted reporting snapshot layer

Pros:

- cheap repeated reads
- could support future charts/history faster

Cons:

- overbuilds Phase 28A
- adds schema/persistence decisions before they are needed
- duplicates derivable state

## Recommendation

Choose **Option B**.

Add a thin `trade::reporting` module that reads `PaperTradeState` and produces reporting rows. Keep live quote lookup outside the trade domain and inject it from the CLI layer only for `--current` views.

This is the smallest design that keeps the write path stable while creating a reusable read-side foundation for future trade/risk views.

## Architecture

### File boundaries

- `src/cli/mod.rs`
  - extend `TradeCommands`
  - add `History`, `Fees`, `Overview`
  - change `Position` from unit variant to `Position { current: bool }`

- `src/trade/models.rs`
  - keep write-side structs as-is
  - add read-side output structs:
    - `TradeHistoryRow`
    - `TradeFeeRow`
    - `TradeOverview`
    - `TradePositionCurrentRow`
    - `TradeQuoteStatus`

- `src/trade/reporting.rs`
  - new read-only aggregation module
  - no mutation logic
  - no direct quote provider dependency

- `src/trade/mod.rs`
  - export reporting types/module

- `src/cli/handlers.rs`
  - add the new read-only command handlers
  - define a small quote-lookup helper for `--current`
  - adapt `trade position` printing to support current-value columns

- `tests/trade_reporting_test.rs`
  - new read-side unit tests

- existing handler/parser/doc tests
  - extend only as needed

### Data flow

#### `trade history`

1. Load `PaperTradeState`
2. Require initialized account if consistent with existing trade read commands
3. Read `trade_records`
4. Filter by optional `code`
5. Sort newest first
6. Apply `limit`
7. Map into `TradeHistoryRow`

#### `trade fees`

1. Load `PaperTradeState`
2. Read `trade_records`
3. Filter by optional `code`
4. Sort newest first
5. Apply `limit`
6. Map into `TradeFeeRow`

#### `trade overview`

Without `--current`:

1. Load `PaperTradeState`
2. Reuse persisted account cash and positions
3. Compute totals from `trade_records` and persisted positions

With `--current`:

1. Build the same bookkeeping overview
2. Attempt batch quote lookup for held codes
3. If all quotes resolve, compute live position value and live total assets
4. If some quotes are missing, leave live aggregate fields empty and report coverage

#### `trade position --current`

1. Load positions
2. If `--current` is absent, keep the existing view
3. If `--current` is present, batch lookup quotes
4. For each row:
   - if quote exists, compute current price, current market value, unrealized PnL, unrealized PnL%
   - if quote is missing, leave current-value fields empty and mark status

## Read Models

```rust
pub struct TradeHistoryRow {
    pub executed_at: DateTime<Utc>,
    pub code: String,
    pub side: TradeSide,
    pub price: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub total_fee: Decimal,
    pub net_cash_impact: Decimal,
}

pub struct TradeFeeRow {
    pub executed_at: DateTime<Utc>,
    pub code: String,
    pub side: TradeSide,
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
}

pub struct TradeOverview {
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub booked_position_value: Decimal,
    pub booked_total_assets: Decimal,
    pub trade_count: usize,
    pub holding_count: usize,
    pub total_buy_amount: Decimal,
    pub total_sell_amount: Decimal,
    pub total_fee: Decimal,
    pub live_position_value: Option<Decimal>,
    pub live_total_assets: Option<Decimal>,
    pub quote_coverage: Option<(usize, usize)>,
}

pub struct TradePositionCurrentRow {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_price: Decimal,
    pub current_price: Option<Decimal>,
    pub current_market_value: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub unrealized_pnl_pct: Option<Decimal>,
    pub quote_status: TradeQuoteStatus,
}

pub enum TradeQuoteStatus {
    BookOnly,
    Live,
    Missing,
}
```

Notes:

- `net_cash_impact` is negative for buys and positive for sells after fees
- `booked_*` fields always exist
- `live_*` fields only exist when `--current` is requested and quote coverage is complete

## Quote Lookup Strategy

Phase 28A should **not** push quote lookup into the trade domain.

Instead:

- keep trade reporting purely bookkeeping-based
- let the CLI layer request quotes only when `--current` is present
- reuse existing best-effort quote-loading behavior already used elsewhere in the CLI

This preserves a clean boundary:

- `trade::reporting` = deterministic aggregation from persisted state
- CLI handler = optional live overlay

### Degradation behavior

- quote provider returns all prices:
  - current fields are populated
- quote provider returns partial prices:
  - per-position missing rows show `quote_status = Missing`
  - overview live totals are withheld
  - command still succeeds
- quote provider fails entirely:
  - current fields are empty
  - booked view still prints
  - command still succeeds

## Output expectations

### `trade history`

Show a compact table:

- time
- code
- side
- price
- volume
- amount
- fee
- net cash impact

### `trade fees`

Show:

- time
- code
- side
- commission
- stamp duty
- transfer fee
- total fee

### `trade overview`

Always show:

- initial capital
- available cash
- booked position value
- booked total assets
- trade count
- holding count
- total buy amount
- total sell amount
- total fee

When `--current` is present:

- if quote coverage is complete, also show live position value and live total assets
- if quote coverage is partial/incomplete, show a readable coverage note instead of fake totals

### `trade position --current`

Extend the existing table with:

- current price
- current market value
- unrealized PnL
- unrealized PnL%
- quote status

Without `--current`, do not change the current UX.

## Error handling

Hard errors:

- uninitialized paper-trade account
- invalid CLI arguments
- JSON read/deserialize failure

Soft errors:

- quote lookup failure
- partial quote coverage

Soft errors must not abort the command. They only affect live-value fields.

## Testing strategy

1. Parser tests
   - new subcommands and `--current`
2. Reporting unit tests
   - sorting, filtering, totals, empty-state behavior
3. Handler tests
   - `--current` happy path
   - partial quote coverage
   - quote failure degradation
4. Docs/hygiene tests
   - README and manual reflect the new commands

## Non-Goals

Phase 28A is not:

- a performance analytics dashboard
- a tax engine
- a portfolio optimization layer
- a real-time streaming mark-to-market monitor
- a replacement for the Phase 27A risk domain
