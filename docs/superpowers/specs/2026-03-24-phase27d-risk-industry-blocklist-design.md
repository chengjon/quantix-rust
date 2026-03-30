# Phase 27D Risk Industry Blocklist Design

**Date:** 2026-03-24
**Status:** Draft for user file review
**Depends On:** Phase 27A local risk baseline, Phase 27B live import risk mirror, and Phase 27C risk volatility rule

> This document defines the next risk-rule slice: add an industry-level blocklist rule for new buys, while retaining multiple industry-classification standards in the system. Phase 27D v1 uses Shenwan first-level industry as the active runtime standard, and runs only on local project data stores (`ClickHouse + SQLite`). MySQL remains an upstream sync source, not a runtime query dependency.

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
- collapse the system down to a single permanent classification standard
- require direct runtime reads from a remote MySQL instance during buy checks

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. set a blocked industry list with the existing `risk rule set` command
2. enable or disable that rule with the existing toggles
3. see the rule in `risk rule list` and `risk status`
4. have new buys rejected when the symbol’s industry is blocked
5. explicitly refresh the local Shenwan reference tables from the upstream source
6. rely on a monthly static snapshot when the latest industry lookup is missing or unavailable

### Exact CLI boundary

This slice adds one explicit sync command and extends the existing rule-type domain:

```bash
quantix risk sync industry --standard shenwan
quantix risk rule set --type industry-blocklist --value 银行
quantix risk rule set --type industry-blocklist --value 银行,地产
quantix risk rule enable --type industry-blocklist
quantix risk rule disable --type industry-blocklist
quantix risk rule list
quantix risk status
```

Rules:

- `risk sync industry --standard shenwan` refreshes the local SQLite Shenwan reference tables from the upstream source
- `industry-blocklist` accepts a comma-separated industry-name list
- values are parsed as exact industry names, not fuzzy patterns
- the rule is visible in both `paper` and `live_import` rule listings
- only buy-evaluation paths execute this rule in v1
- Phase 27D v1 evaluates against **Shenwan first-level industry**

### Explicitly deferred

This slice does not include:

- industry whitelist rules
- industry exposure caps or portfolio-level sector limits
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

### Option C: Use one active runtime standard in SQLite plus monthly snapshots and historical fallback

Pros:

- keeps one explicit runtime standard for deterministic rule evaluation
- still allows the system to retain multiple classification standards
- provides fallback when the active source is missing
- preserves month-level historical industry mapping
- supports later audit and replay use cases
- keeps runtime risk checks local and deterministic

Cons:

- adds a small persistence layer for monthly snapshots
- requires a clear precedence rule

## Recommendation

Choose **Option C**.

The rule should resolve industry membership in this order:

1. active runtime standard lookup from local SQLite
2. the snapshot for the query month
3. historical fallback from the active standard in local SQLite
4. the most recent available local snapshot

Only when all configured resolution tiers fail should the buy be rejected as an industry-resolution error.

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

### Active runtime standard in v1

The system may retain multiple classification standards in storage, but `industry-blocklist` must evaluate against one active standard at runtime.

Phase 27D v1 uses:

- `Shenwan`
- `一级行业`

That means first-version blocklist values match names such as:

- `银行`
- `房地产`
- `医药生物`

### Evaluation target

The rule evaluates the target symbol of a pending buy only.

It does not:

- inspect total industry exposure of the whole account
- affect sell paths
- mutate any account lock state

## Industry Resolution And Monthly Snapshots

### Resolution order

For a given `code` and query date, industry resolution follows this fixed order:

1. current active-standard source
   - use a locally synchronized SQLite table derived from `mystocks.sw_industry_classification`
   - resolve `股票代码 -> 新版一级行业`

2. query-month snapshot
   - use the static snapshot for the month matching the query date, for example `2026-03`

3. nearest historical fallback snapshot
   - first try a locally synchronized SQLite history table derived from `mystocks.sw_stock_update` joined to `mystocks.sw_industry`
   - if that still misses, use the most recent available local snapshot

4. total failure
   - if all configured tiers are unavailable, reject the buy with a hard error

### Why monthly snapshots are required

The snapshot layer serves two goals:

1. resilience
   - latest industry lookup may be unavailable or incomplete

2. traceability
   - industry classifications can drift over time, especially on longer time horizons

The snapshot is not a new primary truth source. It is a fallback and audit substrate.

### Why SQLite is the runtime store in v1

The application should not depend on a remote MySQL instance during buy checks. Risk evaluation needs local, low-latency, deterministic reads with simple point lookups and month-level fallback logic. SQLite is a better runtime fit for that than ClickHouse, while ClickHouse remains the analysis warehouse.

Therefore, the external MySQL classification tables are treated as upstream source data that must be synchronized into local SQLite reference tables before or during normal project data-refresh workflows.

### Why Shenwan is the active standard in v1

The repository does not currently expose a reliable arbitrary-stock-code-to-industry mapping through `sector_daily`; that table contains board identity and a single leader stock, not full constituent membership.

By contrast, the available MySQL tables provide a practical first-version source path:

- `mystocks.sw_industry_classification`
  - current `股票代码 -> 申万一级/二级/三级行业`
- `mystocks.sw_stock_update`
  - historical industry membership changes with `计入日期`
- `mystocks.sw_industry`
  - industry code dictionary for historical rows

The system should still retain other standards such as CSRC, but Phase 27D v1 should evaluate one active runtime standard only, and Shenwan first-level is the best available choice.

### Runtime vs upstream database boundary

Phase 27D v1 should use:

- runtime reads:
  - local SQLite
- analysis warehouse:
  - ClickHouse
- upstream sync source:
  - MySQL

This keeps the runtime database set to `ClickHouse + SQLite` while still allowing MySQL-origin classification data to enter the project through a controlled sync path.

### Explicit runtime precondition

The expected first-version operator workflow is:

1. run `quantix risk sync industry --standard shenwan`
2. then enable and use `industry-blocklist`

The sync command reads upstream connection settings from:

- `QUANTIX_UPSTREAM_MYSQL_URL`
- `QUANTIX_UPSTREAM_MYSQL_DB`
- `QUANTIX_UPSTREAM_MYSQL_USER`
- `QUANTIX_UPSTREAM_MYSQL_PASSWORD`

and writes the local SQLite reference DB beside `risk_state.json`, for example `~/.quantix/risk/industry_reference.db`.

If the local SQLite Shenwan reference tables have not been synchronized yet, `industry-blocklist` checks fail closed with an industry-resolution error.

### Monthly snapshot update rule

The repository does not yet have a complete holiday-aware trading-calendar implementation, so v1 should not attempt to schedule updates on the exact “first trading day of the month.”

Instead:

- the first successful active-standard resolution for a code in a given month freezes that month’s snapshot row
- if that `(snapshot_month, code)` row already exists, do not overwrite it in v1

This yields deterministic month-level snapshots without depending on currently incomplete holiday data.

### Snapshot storage

Use lightweight risk-side SQLite tables rather than a JSON file.

Suggested table:

- `risk_industry_snapshots`

Additional local reference tables are expected for the active standard, for example:

- `industry_reference_current`
- `industry_reference_history`

Suggested columns:

- `standard`
- `level`
- `snapshot_month` (for example `2026-03`)
- `code`
- `industry_name`
- `source`
- `captured_at`

Unique key:

- `(standard, level, snapshot_month, code)`

### Source field

The snapshot row should record where the mapping came from when it was frozen, for example:

- `sw_current`
- `sw_history`
- later, potentially `csrc_2024`

That keeps the snapshot auditable without adding much complexity.

## Failure Semantics

The industry blocklist is fail-closed when all resolution tiers fail.

### Cases

1. rule not configured

- no industry check runs
- existing buy behavior stays unchanged

2. primary active-standard lookup succeeds from local SQLite

- evaluate against the blocklist
- if this is the first successful resolution for the month, write the month snapshot

3. active-standard lookup fails, but the query-month snapshot exists

- use the query-month snapshot
- do not error

4. query-month snapshot is missing, but local historical Shenwan membership or an older snapshot exists

- first use historical Shenwan fallback if available
- otherwise use the most recent available local snapshot
- do not error

5. all tiers fail

- reject the buy
- return a hard error
- do not write a `risk log` event

### Recommended error message

```text
risk rule industry-blocklist 检查失败: code=000001 原因=未找到行业归属（current/monthly/history/fallback 均为空）
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

- active-standard industry resolution
- historical Shenwan fallback lookup
- monthly snapshot lookup
- latest available snapshot lookup
- month-snapshot freezing
- blocklist matching
- formatted error construction
- reading only from local SQLite at runtime

### Keep source-specific storage details out of `RiskService`

`RiskService` should not embed SQL or market-service details directly.

Use a thin resolver boundary such as:

- `IndustryResolver`

Inputs:

- `code`
- `query_date`

Outputs:

- resolved industry name
- optionally metadata about which tier was used
- and which classification standard / level produced the result

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
3. current Shenwan first-level lookup hits a blocked industry and rejects the buy
4. current Shenwan first-level lookup resolves an unblocked industry and allows the buy
5. current lookup failure falls back to the query-month snapshot
6. query-month miss falls back to local historical Shenwan membership when available
7. if local historical Shenwan also misses, the resolver falls back to the most recent local snapshot
8. all tiers missing reject the buy with an industry-resolution error
9. first successful active-standard resolution in a month freezes that month’s snapshot row
10. an existing month snapshot is not overwritten in v1
11. sell paths remain unaffected
12. `risk rule list` and `risk status` display `industry-blocklist`
13. strategy paper path surfaces the same rejection reason
14. strategy mock_live path surfaces the same rejection reason

## File Impact Preview

Expected primary touch points:

- `src/risk/models.rs`
- `src/risk/mod.rs`
- `src/risk/service.rs`
- `src/risk/industry.rs`
- `src/risk/industry_store.rs`
- a thin sync/import path from Shenwan MySQL tables into local SQLite reference tables
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
2. users can run `quantix risk sync industry --standard shenwan` to populate the local SQLite Shenwan reference tables from the upstream source
3. buys are rejected when the resolved industry is blocked
4. the resolver follows active-standard current → query-month snapshot → historical → latest local snapshot precedence
5. successful active-standard resolution freezes a month snapshot for that code
6. all-tier resolution failure rejects the buy instead of silently bypassing the rule
7. runtime rule evaluation does not require direct MySQL access
8. the system retains room for multiple classification standards even though v1 evaluates Shenwan first-level only
9. sell paths, buy-lock semantics, and live-import semantics remain unchanged
10. docs and hygiene tests reflect the new supported risk rule and the explicit sync step
