# Phase 27D Risk Industry Blocklist Design

**Date:** 2026-03-24
**Status:** Draft for user file review
**Depends On:** Phase 27A local risk baseline, Phase 27B live import risk mirror, and Phase 27C risk volatility rule

> This document defines the next risk-rule slice: add an industry-level blocklist rule for new buys, using the existing industry market-data path as the primary source and a monthly static code-to-industry snapshot as fallback for resilience and historical traceability.

---

## Goal

Build the smallest useful industry rule so a user can:

1. configure an industry blocklist with the existing `risk rule` command family
2. reject new buy orders when the target symbol belongs to a blocked industry
3. reuse the same rule across `trade buy`, `strategy run --mode paper`, and `strategy run --mode mock_live`
4. preserve an auditable monthly code-to-industry snapshot for fallback and historical replay
5. keep current buy-lock, stop, and live-import semantics unchanged

This slice must not:

- introduce account-level industry exposure limits
- introduce an industry whitelist in v1
- add a new command family for industry maintenance
- require precise holiday-aware “first trading day of month” scheduling
- silently bypass the rule when industry resolution fails everywhere

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. set a blocked industry list with the existing `risk rule set` command
2. enable or disable that rule with the existing toggles
3. see the rule in `risk rule list` and `risk status`
4. have new buys rejected when the symbol’s industry is blocked
5. rely on a monthly static snapshot when the latest industry lookup is missing or unavailable

### Exact CLI boundary

This slice does not add new commands. It extends the existing rule-type domain:

```bash
quantix risk rule set --type industry-blocklist --value 银行
quantix risk rule set --type industry-blocklist --value 银行,地产
quantix risk rule enable --type industry-blocklist
quantix risk rule disable --type industry-blocklist
quantix risk rule list
quantix risk status
```

Rules:

- `industry-blocklist` accepts a comma-separated industry-name list
- values are parsed as exact industry names, not fuzzy patterns
- the rule is visible in both `paper` and `live_import` rule listings
- only buy-evaluation paths execute this rule in v1

### Explicitly deferred

This slice does not include:

- industry whitelist rules
- industry exposure caps or portfolio-level sector limits
- manual snapshot refresh commands
- exact first-trading-day-of-month scheduling
- alias normalization or fuzzy industry matching
- per-rejection `risk log` events
- auto-deleverage
- live-import-specific industry exposure analytics

## Approaches Considered

### Option A: Only use the latest market industry mapping

Pros:

- smallest implementation
- no fallback data to maintain

Cons:

- fails hard whenever current industry data is unavailable
- no historical monthly trace
- poor fit for long-span replay or future audit use

### Option B: Only use a static code-to-industry mapping table

Pros:

- stable and predictable at runtime
- independent from current market-data availability

Cons:

- drifts away from the project’s current board/industry data source
- weak freshness unless a separate refresh workflow is built

### Option C: Use latest market industry data first, then monthly static snapshots as fallback

Pros:

- keeps one authoritative live-ish source
- provides fallback when the primary source is missing
- preserves month-level historical industry mapping
- supports later audit and replay use cases

Cons:

- adds a small persistence layer for monthly snapshots
- requires a clear precedence rule

## Recommendation

Choose **Option C**.

The rule should resolve industry membership in this order:

1. latest `sector_daily / industry` mapping
2. the snapshot for the query month
3. the most recent available snapshot

Only when all three fail should the buy be rejected as an industry-resolution error.

## Rule Semantics

### Rule name

Add a new `RiskRuleType`:

- `IndustryBlocklist`

CLI string:

- `industry-blocklist`

### Rule value

Supported value type:

- comma-separated exact industry names

Examples:

- `银行`
- `银行,地产`
- `银行, 地产, 煤炭`

First-version parsing rules:

- split on commas
- trim surrounding whitespace
- drop empty segments
- preserve user-specified order
- compare by exact industry-name match

This rule should use a structured value form such as:

- `RuleValue::TextList(Vec<String>)`

and not a JSON-encoded string blob.

### Evaluation target

The rule evaluates the target symbol of a pending buy only.

It does not:

- inspect total industry exposure of the whole account
- affect sell paths
- mutate any account lock state

## Industry Resolution And Monthly Snapshots

### Resolution order

For a given `code` and query date, industry resolution follows this fixed order:

1. latest primary source
   - use the existing market-data path backed by `sector_daily` with `sector_type = industry`

2. query-month snapshot
   - use the static snapshot for the month matching the query date, for example `2026-03`

3. nearest historical fallback snapshot
   - if the query month is missing, use the most recent available snapshot

4. total failure
   - if all three are unavailable, reject the buy with a hard error

### Why monthly snapshots are required

The snapshot layer serves two goals:

1. resilience
   - latest industry lookup may be unavailable or incomplete

2. traceability
   - industry classifications can drift over time, especially on longer time horizons

The snapshot is not a new primary truth source. It is a fallback and audit substrate.

### Monthly snapshot update rule

The repository does not yet have a complete holiday-aware trading-calendar implementation, so v1 should not attempt to schedule updates on the exact “first trading day of the month.”

Instead:

- the first successful latest-source resolution for a code in a given month freezes that month’s snapshot row
- if that `(snapshot_month, code)` row already exists, do not overwrite it in v1

This yields deterministic month-level snapshots without depending on currently incomplete holiday data.

### Snapshot storage

Use a lightweight risk-side SQLite table rather than a JSON file.

Suggested table:

- `risk_industry_snapshots`

Suggested columns:

- `snapshot_month` (for example `2026-03`)
- `code`
- `industry_name`
- `source`
- `captured_at`

Unique key:

- `(snapshot_month, code)`

### Source field

The snapshot row should record where the mapping came from when it was frozen, for example:

- `market_latest`

That keeps the snapshot auditable without adding much complexity.

## Failure Semantics

The industry blocklist is fail-closed when all resolution tiers fail.

### Cases

1. rule not configured

- no industry check runs
- existing buy behavior stays unchanged

2. primary latest-source lookup succeeds

- evaluate against the blocklist
- if this is the first successful resolution for the month, write the month snapshot

3. latest-source lookup fails, but the query-month snapshot exists

- use the query-month snapshot
- do not error

4. query-month snapshot is missing, but an older snapshot exists

- use the most recent available snapshot
- do not error

5. all three tiers fail

- reject the buy
- return a hard error
- do not write a `risk log` event

### Recommended error message

```text
risk rule industry-blocklist 检查失败: code=000001 原因=未找到行业归属（latest/monthly/fallback 均为空）
```

### Recommended block-hit message

```text
risk rule industry-blocklist 已命中: code=000001 industry=银行 blocked=银行,地产
```

## Architecture Boundary

### Risk service remains the orchestrator

`RiskService::check_buy()` remains the single orchestration entry point for buy checks.

Its evaluation order becomes:

1. refresh state
2. enforce buy lock from `daily-loss-limit`
3. enforce `position-limit`
4. enforce `volatility-limit`
5. enforce `industry-blocklist`

### Add a focused industry module

Add a focused risk-side industry layer, for example:

- `src/risk/industry.rs`
- `src/risk/industry_store.rs`

Responsibilities:

- latest-source industry resolution
- monthly snapshot lookup
- latest available snapshot lookup
- month-snapshot freezing
- blocklist matching
- formatted error construction

### Keep ClickHouse/market details out of `RiskService`

`RiskService` should not embed SQL or market-service details directly.

Use a thin resolver boundary such as:

- `IndustryResolver`

Inputs:

- `code`
- `query_date`

Outputs:

- resolved industry name
- optionally metadata about which tier was used

That keeps the risk layer composable and allows later industry-limit work to reuse the same resolution path.

## CLI And Output Semantics

### Rule list and status

`risk rule list` and the `[规则]` section of `risk status` should display the rule directly:

```text
industry-blocklist    银行,地产    enabled
```

### No new log event type

This slice does not add dedicated industry-triggered event types to `risk log`.

Rationale:

- current `risk log` is for durable state changes
- an industry rejection is a point-in-time buy decision
- logging every rejection would create strategy/daemon noise

Only rule-management events remain durable:

- `rule-set`
- `rule-enabled`
- `rule-disabled`

## Testing Scope

The minimum test matrix is:

1. `risk rule set --type industry-blocklist --value 银行,地产` succeeds
2. rule value parses into `TextList([\"银行\", \"地产\"])`
3. latest-source industry lookup hits a blocked industry and rejects the buy
4. latest-source lookup resolves an unblocked industry and allows the buy
5. latest-source failure falls back to the query-month snapshot
6. query-month miss falls back to the most recent snapshot
7. all three tiers missing reject the buy with an industry-resolution error
8. first successful latest-source resolution in a month freezes that month’s snapshot row
9. an existing month snapshot is not overwritten in v1
10. sell paths remain unaffected
11. `risk rule list` and `risk status` display `industry-blocklist`
12. strategy paper path surfaces the same rejection reason
13. strategy mock_live path surfaces the same rejection reason

## File Impact Preview

Expected primary touch points:

- `src/risk/models.rs`
- `src/risk/mod.rs`
- `src/risk/service.rs`
- `src/risk/industry.rs`
- `src/risk/industry_store.rs`
- `src/market/service.rs` or a thin adapter boundary reused from it
- `src/cli/tests/risk.rs`
- `src/cli/handlers.rs` (tests and possibly small injection glue only)
- `tests/risk_service_test.rs`
- `tests/risk_industry_test.rs`
- `README.md`
- `docs/USER_MANUAL.md`
- `tests/repo_hygiene_test.rs`

## Acceptance Criteria

This slice is complete when:

1. users can configure `industry-blocklist` through the existing `risk rule` CLI
2. buys are rejected when the resolved industry is blocked
3. the resolver follows latest-source → query-month snapshot → latest snapshot precedence
4. successful latest-source resolution freezes a month snapshot for that code
5. all-tier resolution failure rejects the buy instead of silently bypassing the rule
6. sell paths, buy-lock semantics, and live-import semantics remain unchanged
7. docs and hygiene tests reflect the new supported risk rule
