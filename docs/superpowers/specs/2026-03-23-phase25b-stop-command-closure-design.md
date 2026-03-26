# Phase 25B Stop Command Closure Design

**Date:** 2026-03-23
**Status:** Draft for user file review
**Depends On:** Phase 24 monitor snapshot evaluation, Phase 25A stop rule baseline, and Phase 26 paper-trade account baseline

> This document is the source of truth for the next `stop` slice: complete the current stop-rule command family with `status`, `history`, `update`, and percent-based thresholds, while keeping the subsystem local, auditable, and compatible with existing monitor/watchlist evaluation.

---

## Goal

Build the smallest useful closure for the current `stop` subsystem so a user can:

1. define stop rules with fixed-price or percent-based thresholds
2. update existing rules without rewriting the entire rule shape
3. inspect current stop status separately from raw rule storage
4. inspect rule-change and trigger history through an audit view
5. keep the existing `monitor watchlist --once` stop evaluation flow

This slice must not:

- auto-sell or place execution orders after stop triggers
- collapse `stop` into `risk`
- require a live broker or real-time account feed
- add a generalized anchor-policy system in v1

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. set a stop rule with fixed price or percent thresholds
2. patch an existing stop rule with `update`
3. inspect current evaluated state via `stop status`
4. inspect audit history via `stop history`
5. continue using `stop list` as a low-level stored-rule view

### Exact CLI boundary

This slice keeps:

```bash
quantix stop set <CODE> [--loss <PRICE>] [--profit <PRICE>] [--trailing <PCT>]
quantix stop list
quantix stop remove <CODE>
```

This slice adds:

```bash
quantix stop set <CODE> [--loss-pct <PCT>] [--profit-pct <PCT>]
quantix stop update <CODE> [--loss <PRICE>] [--profit <PRICE>] [--trailing <PCT>] [--loss-pct <PCT>] [--profit-pct <PCT>] [--clear-loss] [--clear-profit] [--clear-trailing] [--clear-loss-pct] [--clear-profit-pct]
quantix stop status [--code <CODE>]
quantix stop history [--code <CODE>] [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]
```

Rules:

- `set` remains the create/full-overwrite path
- `update` is a partial patch path
- `list` remains a stored-rule view
- `status` is an evaluated view
- `history` is an audit view

### Explicitly deferred

This slice does not include:

- automatic order placement after stop trigger
- event-driven daemon execution on stop trigger
- cross-account stop policies
- explicit `--anchor cost|fixed` selection
- multi-account or real-broker anchor sources

## Approaches Considered

### Option A: Expand the local `stop` subsystem

Extend `stop_rules`, add `stop_history`, and keep `stop` responsible for its own storage, evaluation, and audit surface. Read paper-trade account state only to compute percent-threshold anchors.

Pros:

- smallest focused diff
- preserves current stop/monitor architecture
- easiest to roll out incrementally

Cons:

- introduces a small read-only dependency on paper-trade account state

### Option B: Merge stop audit into monitor events

Use monitor event storage for rule changes and trigger history.

Pros:

- one fewer audit table

Cons:

- blurs stop and monitor responsibilities
- makes rule-change audit harder to query cleanly

### Option C: Fold stop into risk

Treat stop rules as risk rules and store everything in risk state.

Pros:

- conceptually unified position-protection surface

Cons:

- much larger scope
- crosses subsystem boundaries that are not currently coupled

## Recommendation

Choose **Option A**.

The current code already has a viable stop-rule subsystem:

- `src/stop/models.rs`
- `src/stop/service.rs`
- `src/stop/storage.rs`
- CLI wiring in `src/cli/mod.rs` and `src/cli/handlers.rs`
- stop evaluation reuse in `monitor watchlist --once`

The right next step is to finish that subsystem, not to merge it into `monitor` or `risk`.

## Architecture

### Preserved split

Keep the following responsibilities distinct:

1. `stop`
   - owns stop-rule storage
   - owns stop-rule validation
   - owns stop-specific history
   - owns stop status view

2. `trade`
   - remains the authority for current local paper positions and average cost

3. `monitor`
   - continues to load watchlist quotes
   - continues to call stop evaluation during snapshot scans

### Why the split matters

The stop subsystem needs paper-trade account state only as an anchor source for percent thresholds. It does not need to own trade execution, account mutation, or risk-rule lifecycle.

## Rule Model

### Public stop rule fields

Extend `StopRule` with:

- `stop_loss_pct: Option<f64>`
- `take_profit_pct: Option<f64>`
- `reference_price: Option<f64>`

The resulting rule model becomes:

- fixed loss threshold
  - `stop_loss_price`
- fixed profit threshold
  - `take_profit_price`
- percent loss threshold
  - `stop_loss_pct`
- percent profit threshold
  - `take_profit_pct`
- trailing stop threshold
  - `trailing_pct`
- trailing runtime state
  - `highest_price`
- percent-threshold fallback anchor
  - `reference_price`

### Validation rules

Per side, allow only one threshold expression:

- `loss` and `loss_pct` are mutually exclusive
- `profit` and `profit_pct` are mutually exclusive

Trailing-loss remains mutually exclusive with all loss-side fixed/percent thresholds:

- `trailing` cannot coexist with `loss`
- `trailing` cannot coexist with `loss_pct`

But trailing-loss can still coexist with profit-side thresholds:

- `trailing + profit`
- `trailing + profit_pct`

If an `update` results in no active threshold remaining, reject it and require the user to use `remove`.

## Anchor Semantics

### Default percent-threshold anchor

When evaluating `--loss-pct` or `--profit-pct`, choose the anchor in this order:

1. current local paper-trade average cost for the code, if a position exists
2. otherwise the rule's stored `reference_price`
3. otherwise no anchor is available

The first slice does not add explicit user-selected anchor modes.

### Why this order

This matches the most natural trading semantics:

- if the user holds the stock locally, percent stop logic should follow position cost
- if the user does not hold it, the rule can still function as a watchlist-oriented price monitor anchored to the setup-time reference price

### Anchor source labels

Expose the evaluated anchor source as one of:

- `position_cost`
- `reference_price`
- `anchor_missing`

## Evaluation Semantics

### Threshold derivation

Derived thresholds are:

- `loss_pct = 5`
  - `loss_threshold = anchor_price * (1 - 0.05)`
- `profit_pct = 10`
  - `profit_threshold = anchor_price * (1 + 0.10)`

Trailing-loss continues to use:

- `highest_price`
- `threshold = highest_price * (1 - trailing_pct / 100.0)`

### Missing anchor behavior

If a percent threshold is configured but no anchor can be resolved:

- evaluation does not fail the command
- the rule remains stored
- `status` reports an `anchor_missing` state
- monitor snapshot evaluation simply skips percent-trigger derivation for that rule in that iteration

### Missing quote behavior

If the current quote is unavailable:

- evaluation does not fail the command
- `status` reports `quote_missing`
- no trigger is produced in that iteration

## Status View

### `stop list`

`stop list` remains a low-level storage view. It should show the stored rule values directly and preserve existing lightweight semantics.

### `stop status`

`stop status` is the evaluated operator view and should show:

- `code`
- `last_price`
- `anchor_price`
- `anchor_source`
- `loss_threshold`
- `profit_threshold`
- `trailing_pct`
- `highest_price`
- `last_triggered_at`
- `eval_state`

Recommended `eval_state` values:

- `armed`
- `triggered`
- `anchor_missing`
- `quote_missing`

This command is designed to answer:

- what is active right now
- what reference was used
- why a percent rule may not currently be evaluable

## History Model

### History storage

Add a new `stop_history` table. First-slice schema:

- `id`
- `code`
- `event_type`
  - `set`
  - `update`
  - `remove`
  - `trigger`
- `trigger_type`
  - `loss`
  - `profit`
  - `trailing`
- `trigger_price`
- `anchor_price`
- `anchor_source`
  - `position_cost`
  - `reference_price`
- `snapshot_json`
- `created_at`

### Why no `rule_id` in v1

The current `stop_rules` schema uses `code` as the primary key. Introducing stable `rule_id` semantics would force a heavier schema migration than this slice needs. For the first slice, `code + snapshot_json` is sufficient for auditability.

### History event rules

Record:

- `set` on create/full overwrite
- `update` on patch
- `remove` on delete
- `trigger` when a stop condition fires

For `remove`, persist the removed rule snapshot in `snapshot_json`.

For `trigger`, persist:

- trigger kind
- current trigger price
- resolved anchor price
- resolved anchor source
- full rule snapshot

## Update Semantics

### `stop update`

`update` is a partial patch command:

- only explicitly provided fields change
- omitted fields remain unchanged
- explicit clear flags remove one field at a time

Suggested clear flags:

- `--clear-loss`
- `--clear-profit`
- `--clear-trailing`
- `--clear-loss-pct`
- `--clear-profit-pct`

### Constraint behavior after patch

The merged rule must still satisfy all normal validation:

- no conflicting threshold combinations
- at least one threshold remains active

### Why patch semantics are preferred

Users should be able to:

- add profit protection to an existing trailing-loss rule
- swap fixed loss to percent loss
- clear only one side of a rule

without having to reconstruct the entire rule from scratch.

## Data Flow

### Set/update flow

1. CLI parses command
2. handler validates watchlist membership
3. handler optionally resolves setup-time `reference_price`
4. stop service validates/merges rule
5. store upserts `stop_rules`
6. store appends `stop_history`
7. CLI prints result

### Status flow

1. CLI loads current rules
2. CLI loads best-effort quote snapshot
3. CLI loads local paper-trade account state
4. stop service derives evaluated status rows
5. CLI prints status rows

### Monitor trigger flow

1. monitor snapshot loads quote rows
2. stop service evaluates all rules
3. updated rule runtime state is persisted when needed
4. trigger events are appended to `stop_history`
5. monitor output continues to show triggered stops

## Error Handling

### User-facing validation errors

Surface clear errors for:

- no threshold provided
- conflicting threshold combinations
- invalid percent values
- invalid price values
- `update` on missing rule
- `update` that clears all thresholds
- code not present in local watchlist

### Storage compatibility

Schema migration must preserve existing `stop_rules` rows and default new fields to `NULL`.

## Testing Strategy

### Model/store tests

Add coverage for:

- schema migration from old stop_rules layout
- stop rule round-trip with percent fields and reference price
- history row insert/list round-trip

### Service tests

Add coverage for:

- `loss_pct` anchored to paper-trade average cost
- fallback to `reference_price` when no position exists
- `anchor_missing` behavior when no anchor exists
- trailing plus profit percent coexistence
- conflicting combinations rejected
- `update` patch behavior
- clear flags
- trigger writes history with anchor source and snapshot

### CLI tests

Add parser and handler coverage for:

- `stop set --loss-pct`
- `stop set --profit-pct`
- `stop update`
- `stop status`
- `stop history`

### Integration tests

Add at least:

- monitor evaluation still updates trailing state
- monitor evaluation writes trigger history
- status view degrades cleanly when quotes are missing

## Implementation Order

Recommended implementation chunks:

1. extend stop models and SQLite storage
2. add stop history persistence
3. implement percent-anchor evaluation and status rows
4. add `update`, `status`, and `history` CLI
5. update README / USER_MANUAL / repo hygiene

## Acceptance Criteria

This slice is complete when:

1. users can define percent-based stop thresholds
2. users can patch rules via `stop update`
3. `stop status` shows evaluated thresholds and anchor source
4. `stop history` shows rule-change and trigger audit entries
5. monitor stop evaluation remains compatible
6. docs and hygiene tests reflect the new command surface
