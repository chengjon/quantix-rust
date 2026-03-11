# Phase 26A Paper Trade Design

**Date:** 2026-03-11
**Status:** Approved in-session
**Depends On:** Phase 25A green baseline (`phase25-stop`)

> This document is the Phase 26A source of truth in this branch. It intentionally supersedes the broader ideas in `docs/plans/PHASE26_PAPER_TRADE_DESIGN.md` where those ideas exceed the approved MVP boundary or do not fit the current repository structure.

---

## Goal

Build the smallest useful paper-trade loop:

- initialize or reset one local paper-trade account
- execute fully-filled limit buys and sells at the user-provided price
- persist positions, cash, fee config, and trade records locally
- inspect current positions and cash snapshot from the CLI

This phase does not attempt to become a quote-driven broker simulator or an order-management system.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Initialize a default paper-trade account with capital and fee config
2. Reset that account when the user wants a clean slate
3. Buy a stock with a limit price and explicit volume
4. Sell a held stock with a limit price and explicit volume
5. View current positions
6. View current cash plus a deterministic asset snapshot

Anything beyond those jobs is out of scope for Phase 26A.

### Exact CLI boundary

Only implement:

```bash
quantix trade init [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]
quantix trade reset [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]
quantix trade buy <CODE> --price <PRICE> --volume <N>
quantix trade sell <CODE> --price <PRICE> --volume <N>
quantix trade position
quantix trade cash
```

Rules:

- MVP only supports one account: `account_id = "default"`
- `trade init` creates the account if no state exists yet
- `trade reset` overwrites the full local trade state
- `buy` and `sell` are immediately and fully filled at the provided price
- every successful buy/sell appends a persisted trade record
- `cash` shows available cash and an execution-price-based asset snapshot

Explicitly defer:

- `trade history`
- `trade account`
- `trade overview`
- `trade fees`
- `trade position --code`
- `--current`
- market orders
- partial fills
- slippage simulation
- pending orders / cancel
- multi-account support
- real-time mark-to-market valuation
- automatic linkage from Phase 25 stops into trade execution

## Approaches Considered

### Option A: Reuse `analysis::Portfolio` directly

Pros:

- existing buy/sell position mechanics already exist
- lower raw code volume at first glance

Cons:

- current portfolio code only models a single commission rate
- no persisted trade-record log
- data model is backtest-oriented rather than CLI paper-trade oriented
- would still need adapter layers for fee config, storage, and CLI output

### Option B: Dedicated `trade` domain with local JSON storage

Pros:

- matches the current CLI-first architecture used by watchlist and stop features
- easy to persist and inspect locally
- keeps fee calculation and paper-trade semantics explicit
- avoids leaking backtest assumptions into user-facing trade state

Cons:

- introduces a new domain module
- duplicates a small amount of position accounting logic

### Option C: SQLite-backed order engine

Pros:

- stronger future queryability
- closer to a later broker/order lifecycle model

Cons:

- too much scope for Phase 26A
- pushes the phase toward order management instead of minimal trade simulation
- not required for the approved user jobs

## Recommendation

Choose **Option B**.

Phase 26A should add a dedicated `trade` domain with local JSON persistence. That is the smallest path that still satisfies the user’s real jobs: initialize, trade, inspect positions, inspect cash, and retain records for later replay.

## Architecture

### Domain split

- `src/trade/*` owns paper-trade models, fee calculation, service logic, and JSON persistence
- `src/cli/mod.rs` owns `trade` command parsing
- `src/cli/handlers.rs` owns command execution and terminal output
- `src/core/runtime.rs` owns trade-state path resolution

### Storage choice

Use a dedicated local JSON file pointed to by `QUANTIX_TRADE_PATH`.

Default path:

```text
~/.quantix/trade/paper_trade.json
```

Reason:

- one user, one account, one local state file is enough for MVP
- JSON matches the current repo patterns for lightweight local state
- trade records are append-only and do not need SQL query support yet
- Phase 26A explicitly does not ship a `history` query command

## Minimal Data Model

```rust
pub struct PaperTradeState {
    pub version: u32,
    pub account: Option<PaperTradeAccount>,
    pub trade_records: Vec<TradeRecord>,
}

pub struct PaperTradeAccount {
    pub account_id: String,
    pub initial_capital: Decimal,
    pub available_cash: Decimal,
    pub fee_config: FeeConfig,
    pub positions: BTreeMap<String, TradePosition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TradePosition {
    pub code: String,
    pub volume: i64,
    pub avg_cost: Decimal,
    pub last_trade_price: Decimal,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TradeRecord {
    pub id: String,
    pub code: String,
    pub side: TradeSide,
    pub price: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub commission: Decimal,
    pub stamp_duty: Decimal,
    pub transfer_fee: Decimal,
    pub total_fee: Decimal,
    pub executed_at: DateTime<Utc>,
}

pub struct FeeConfig {
    pub commission_rate: Decimal,
    pub commission_min: Decimal,
    pub stamp_duty_rate: Decimal,
    pub transfer_fee_rate: Decimal,
}
```

Notes:

- `account` is optional so the CLI can detect “not initialized yet”
- no `frozen_cash` field because Phase 26A has no pending orders
- no `available_volume` field because there is no settlement-delay simulation
- no `name`, `current_price`, `profit`, `profit_pct` fields because Phase 26A is not quote-driven
- `trade_records` are persisted even though there is no `trade history` command yet

## Valuation Model

Phase 26A does not load live prices.

So `trade cash` uses a deterministic bookkeeping snapshot:

- `estimated_position_value = sum(position.volume * position.last_trade_price)`
- `estimated_total_assets = available_cash + estimated_position_value`

This is intentionally not a real-time market valuation. It is only a stable local snapshot based on the latest executed price per code.

## Trade Semantics

### Buy

Input:

- code
- price
- volume

Checks:

- account exists
- price is finite and positive
- volume is positive
- `amount + total_fee <= available_cash`

Effects:

- reduce `available_cash` by `amount + total_fee`
- create or increase the position
- update `avg_cost` using buy-side fees in the basis
- set `last_trade_price = price`
- append one `TradeRecord`

### Sell

Input:

- code
- price
- volume

Checks:

- account exists
- price is finite and positive
- volume is positive
- position exists
- position volume is sufficient

Effects:

- reduce or remove the position
- increase `available_cash` by `amount - total_fee`
- update remaining position `last_trade_price = price` when a remainder exists
- append one `TradeRecord`

### Immediate execution rule

Phase 26A does not maintain pending orders. A limit order entered through the CLI is treated as immediately and fully executed at the given price.

This matches the approved MVP boundary and avoids fake exchange-state complexity.

## Fee Model

### Defaults

- commission rate: `0.00025`
- commission minimum: `5`
- stamp duty rate: `0.001` on sells only
- transfer fee rate: `0.00001` on Shanghai stocks only (`60*`, `68*`)

### Calculation

Buy:

- `commission = max(amount * commission_rate, commission_min)`
- `stamp_duty = 0`
- `transfer_fee = amount * transfer_fee_rate` for Shanghai codes, otherwise `0`
- `total_fee = commission + transfer_fee`

Sell:

- `commission = max(amount * commission_rate, commission_min)`
- `stamp_duty = amount * stamp_duty_rate`
- `transfer_fee = amount * transfer_fee_rate` for Shanghai codes, otherwise `0`
- `total_fee = commission + stamp_duty + transfer_fee`

### Config surface

Fee rates are configurable only via `trade init` and `trade reset` in Phase 26A.

That satisfies the user’s configurability requirement without adding a standalone `fees` command too early.

## Account Lifecycle

### Init

- if no state exists, create `default` account
- if an account already exists, return a user-facing error asking the user to use `trade reset`

### Reset

- replace the full stored trade state
- clear all positions
- clear all trade records
- create a fresh `default` account with the supplied or default config

## Terminal Output

### `trade init` / `trade reset`

Show:

- account id
- initial capital
- fee config summary

### `trade buy` / `trade sell`

Show:

- side
- code
- price
- volume
- amount
- fee breakdown
- remaining available cash

### `trade position`

Print a compact table:

- code
- volume
- avg cost
- last trade price
- estimated position value

### `trade cash`

Print:

- initial capital
- available cash
- estimated position value
- estimated total assets

## Error Handling

- running trade commands before `trade init` returns a user-facing error
- invalid capital, rates, prices, or volumes return CLI-facing validation errors
- insufficient cash returns a clear user-facing error
- selling a non-held code returns a clear user-facing error
- selling more than held volume returns a clear user-facing error
- malformed stored JSON should surface as storage/data-parse errors rather than silent resets

## Testing Strategy

Required tests:

- CLI parser tests for `trade init/reset/buy/sell/position/cash`
- trade service tests for:
  - init default account
  - init custom fee config
  - reset overwrites existing state
  - buy opens a position
  - second buy updates weighted cost
  - buy rejects insufficient cash
  - sell reduces or closes a position
  - sell rejects missing position / insufficient volume
  - cash snapshot uses execution-price-based estimated assets
- runtime tests for `QUANTIX_TRADE_PATH`
- JSON storage tests for create/load/save/reopen behavior
- handler tests for user-facing trade command flows
- doc/repo-hygiene tests for Phase 26A command boundary

## Explicit Non-Goals

Phase 26A is not:

- a broker adapter
- a market-data consumer
- a settlement simulator
- a multi-account ledger
- a stop-order execution engine
- a trade-history query/reporting system
