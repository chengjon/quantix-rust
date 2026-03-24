# Phase 27B Live Import Risk Mirror Design

**Date:** 2026-03-24
**Status:** Draft for user file review
**Depends On:** Phase 26 paper-trade baseline, Phase 27A local risk baseline, and Phase 25B stop command closure

> This document is the source of truth for the next risk slice: import normalized real-account trade/cash ledgers, rebuild an isolated live-import mirror account locally, and let `risk` commands read that mirror explicitly without contaminating the existing paper account path.

---

## Goal

Build the smallest useful “real account import” slice so a user can:

1. import normalized trade/cash records from CSV or JSON
2. rebuild a local mirror account state from imported ledger rows
3. inspect `risk status`, `risk position`, and `risk pnl` against that mirror account
4. keep the existing paper-trade workflow untouched
5. prepare a clean base for later volatility rules, industry rules, and auto-deleverage

This slice must not:

- consume broker-native CSV/XLSX directly
- mutate `paper_trade.json`
- allow `trade buy/sell` to operate on imported live data
- auto-place hedge or reduce orders
- require live market connectivity for rebuild correctness

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. import a normalized ledger file for one account
2. run a deterministic full rebuild for that account
3. query the rebuilt mirror from `risk status|pnl|position`
4. compare `paper` and `live_import` sources side by side
5. rerun rebuild safely after dedupe or source-data fixes

### Exact CLI boundary

This slice adds:

```bash
quantix risk import live-trades --account <ID> --input <FILE>
quantix risk rebuild live-account --account <ID>
```

This slice extends:

```bash
quantix risk status --source paper|live_import [--account <ID>]
quantix risk pnl --source paper|live_import [--account <ID>]
quantix risk position --source paper|live_import [--account <ID>]
```

Rules:

- `--source` defaults to `paper`
- `--source live_import` requires `--account`
- imported live data is read-only from the CLI perspective in this slice

### Explicitly deferred

This slice does not include:

- broker-specific raw import adapters
- position snapshots as a required import source
- order-state or pending-order import
- live execution against imported accounts
- auto-deleverage execution requests
- multi-broker reconciliation workflows

## Approaches Considered

### Option A: Import directly into `risk` only

Pros:

- smallest code surface

Cons:

- imported state becomes trapped inside `risk`
- future stop / deleverage / execution reuse gets harder

### Option B: Add an isolated live-import mirror account, then let risk read it

Pros:

- preserves separation from paper-trade
- creates a reusable read-only account substrate for later phases
- keeps current command behavior explicit

Cons:

- requires one more persistence layer

### Option C: Import directly into `paper_trade.json`

Pros:

- superficially simpler

Cons:

- pollutes simulation state with real-account semantics
- breaks existing paper assumptions
- makes future automation boundaries muddy

## Recommendation

Choose **Option B**.

The right next step is to create a read-only local mirror account derived from imported ledgers. That gives `risk` a better data source without breaking `paper` or forcing later `stop` / auto-deleverage work to tunnel through risk internals.

## Architecture

### Preserved split

Keep these responsibilities distinct:

1. import layer
   - parse normalized CSV/JSON
   - validate schema
   - dedupe by stable external key
   - store raw normalized records

2. rebuild layer
   - replay imported ledgers
   - derive mirror cash / positions / realized pnl / fees
   - persist mirror account state

3. risk layer
   - read `paper` or `live_import`
   - apply the same rule evaluation semantics to either source

4. trade layer
   - remains the owner of local paper simulation
   - does not mutate live-import mirrors

### Why the split matters

Import and rebuild need strong replay semantics and operator auditability. Risk should consume a stable derived view, not own ledger replay as a hidden side effect.

## Import Format

### First-slice record types

Only two record types are supported:

- `trade`
- `cash`

### `trade` fields

Required:

- `record_type=trade`
- `account_id`
- `external_id`
- `code`
- `side`
- `price`
- `volume`
- `fee_total`
- `executed_at`

Rules:

- `side` supports only `buy|sell`
- `external_id` must be stable for dedupe
- `fee_total` is kept aggregated in v1

### `cash` fields

Required:

- `record_type=cash`
- `account_id`
- `external_id`
- `business_type`
- `amount`
- `occurred_at`

Rules:

- `business_type` supports only `deposit|withdraw`
- `deposit` should be positive
- `withdraw` should be negative

### CSV shape

Use a single wide table:

```csv
record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at
trade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,
cash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z
```

### JSON shape

Use an array of the same logical records.

### Explicitly deferred format features

Not supported in v1:

- pending orders
- daily statements
- holdings snapshots
- dividends
- interest
- financing / securities lending
- per-fee breakdown columns

## Import Persistence

### Import-side storage

Persist:

- normalized records
- import batch metadata
- dedupe conflicts / parse errors

Suggested entities:

- `live_import_batches`
- `live_import_records`
- `live_import_rebuilds`

### Idempotency key

Deduplicate on:

- `account_id + external_id`

Rules:

- identical duplicate rows are skipped
- conflicting duplicates are stored as errors and reported
- import may partially succeed while still surfacing row-level failures

## Mirror Account Model

### Rebuilt mirror state

Persist at least:

- `account_id`
- `as_of`
- `cash_balance`
- `positions`
- `realized_pnl`
- `total_fees`
- `last_rebuild_at`
- rebuild source count or digest

Each position keeps:

- `code`
- `volume`
- `avg_cost`
- `last_trade_at`

### Why not reuse `paper_trade.json`

Because `paper` is a writable simulation surface, while `live_import` is a replayed read-only mirror. Combining them would make later behavior ambiguous and unsafe.

## Rebuild Semantics

### Rebuild mode

`risk rebuild live-account --account <ID>` always performs a full replay from imported records for that account.

The first slice does not support incremental rebuild.

### Replay ordering

Replay ledgers in deterministic order:

1. by logical event timestamp
2. if tied, by `external_id`

This guarantees repeatable rebuild outputs.

### `trade` replay

Buy:

- increase position volume
- update average cost
- reduce cash by amount plus fee
- accumulate total fees

Sell:

- reduce position volume
- increase cash by proceeds minus fee
- compute realized pnl
- accumulate total fees

If a sell exceeds available position volume:

- rebuild fails
- a rebuild audit error is recorded
- the last successful mirror result remains intact

### `cash` replay

Deposit:

- increase cash balance

Withdraw:

- decrease cash balance

Unsupported or contradictory business semantics:

- rebuild fails with a clear error

### Rebuild result persistence

On success:

- overwrite the previous mirror account state for that account
- overwrite the previous mirror positions for that account
- record a successful rebuild audit row

On failure:

- keep the last successful mirror account state
- record a failed rebuild audit row
- return a CLI failure summary

## Risk Command Source Selection

### Source switch

Extend these commands with:

- `--source paper|live_import`

Commands:

- `risk status`
- `risk pnl`
- `risk position`

Defaults:

- `--source paper`
- `--account` optional for `paper`
- `--account` required for `live_import`

### Why explicit source selection

The system must not silently switch existing commands from paper to imported live data. Explicit source selection keeps operator intent clear and makes side-by-side comparison possible.

## Error Handling

### Import failures

Surface:

- unsupported format extension
- malformed records
- missing required fields
- invalid side / business_type
- invalid timestamp
- invalid numeric values
- conflicting duplicates

### Rebuild failures

Surface:

- unsupported record ordering assumptions
- oversell
- invalid withdrawal semantics
- unknown business type

### Command behavior on missing mirror state

If `risk ... --source live_import --account <ID>` is requested before a successful rebuild:

- return a clear error
- do not silently fall back to `paper`

## Testing Strategy

### Import/storage tests

Add coverage for:

- normalized trade/cash CSV parse
- normalized trade/cash JSON parse
- duplicate skip
- duplicate conflict
- batch summary counts

### Rebuild tests

Add coverage for:

- buy-only replay
- buy/sell replay with realized pnl
- deposit/withdraw replay
- oversell failure
- failed rebuild preserves last successful mirror

### Risk CLI tests

Add parser and handler coverage for:

- `risk import live-trades`
- `risk rebuild live-account`
- `risk status --source live_import --account <ID>`
- `risk pnl --source live_import --account <ID>`
- `risk position --source live_import --account <ID>`

### Regression tests

Preserve:

- existing `paper` risk status semantics
- existing `trade buy/sell` guardrails
- existing stop and execution behavior

## Implementation Order

Recommended chunks:

1. normalized import storage and dedupe
2. full replay rebuild engine
3. risk source switching and live_import views
4. docs and repo hygiene

## Acceptance Criteria

This slice is complete when:

1. users can import normalized trade/cash ledgers for an account
2. users can rebuild a deterministic mirror account state
3. `risk status|pnl|position --source live_import --account <ID>` read that mirror
4. duplicate imports are idempotent and conflicting duplicates are surfaced
5. failed rebuilds do not erase the last successful mirror state
6. current `paper` paths remain behaviorally unchanged
