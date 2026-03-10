# Phase 21 Watchlist Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Phase 21 P0 watchlist flow so users can add, remove, list, group, move, tag, inspect local history, and view best-effort price snapshots through the existing CLI.

**Architecture:** Implement a small `watchlist` domain with JSON-backed persistence, a thin service layer for group/tag/history rules, and a best-effort resolver layer for stock name and price enrichment. Extend the existing `clap` tree in `src/cli/mod.rs` and handler dispatch in `src/cli/handlers.rs`; do not introduce a second CLI framework, realtime daemon, or a fake snapshot dependency for P0.

**Tech Stack:** Rust 2024, `clap`, `serde`, `serde_json`, `chrono`, `tempfile`, existing `QuantixError/Result`, existing PostgreSQL and TDX source modules.

---

## Critical Review Before Execution

- The original draft assumed Phase 21 could reuse an existing query-ready latest-price reader. That is not true on the current branch.
- `src/sources/eastmoney.rs` still contains placeholder parsing and cannot be treated as a reliable P0 dependency.
- `src/db/clickhouse.rs` currently has quote insert paths but no small, ready-to-reuse latest-quote read API for watchlist display.
- Therefore, `watchlist list --with-price` must use a best-effort foreground query path and degrade gracefully when no quote source is available.
- `src/main.rs` does not need Phase 21 changes; all module wiring should happen in `src/lib.rs` and CLI modules.

## Task 1: Expand Watchlist Models And JSON Storage

**Files:**
- Create: `src/watchlist/mod.rs`
- Create: `src/watchlist/models.rs`
- Create: `src/watchlist/storage.rs`
- Modify: `src/lib.rs`
- Test: `tests/watchlist_storage_test.rs`

**Intent:**

- Keep one JSON file as the source of truth
- Store groups as the range-input core
- Add minimal per-stock metadata for tags and timestamps
- Add append-only local operation history

**Verification:**

- Red: `cargo test --test watchlist_storage_test -v`
- Green: `cargo test --test watchlist_storage_test -v`

## Task 2: Implement Service Rules For Group, Tag, And History

**Files:**
- Create: `src/watchlist/service.rs`
- Modify: `src/watchlist/mod.rs`
- Test: `tests/watchlist_service_test.rs`

**Behavior:**

- `add(code, group)` creates entry metadata if absent
- `remove(code)` removes the code from all groups and prunes unused metadata
- `move_code(code, group)` records a move event
- `create_group(name)` records a group creation event
- `add_tag(code, tag)` / `remove_tag(code, tag)` update entry tags
- `list(group, tag)` supports optional group filter and optional tag filter
- `history(code, limit)` returns newest-first filtered history

## Task 3: Add Best-Effort Resolver For Name And Price Enrichment

**Files:**
- Create: `src/watchlist/resolver.rs`
- Modify: `src/watchlist/mod.rs`
- Test: `tests/watchlist_resolver_test.rs`

**Resolver rules:**

- Stock name lookup: best-effort through PostgreSQL if `POSTGRES_URL` is available
- Price lookup: best-effort through foreground TDX batch quote fetch
- Failures must degrade into `None`, never fail the whole `watchlist list`
- No EastMoney dependency in P0
- No ClickHouse quote-read dependency in P0

## Task 4: Extend CLI Parsing For P0 Commands

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

**Required CLI surface:**

```bash
quantix watchlist add --code 000001 [--group core]
quantix watchlist remove --code 000001
quantix watchlist list [--group core] [--tag bank] [--with-price]
quantix watchlist move --code 000001 --group core
quantix watchlist group create --name core
quantix watchlist group list
quantix watchlist tag add --code 000001 --tag bank
quantix watchlist tag remove --code 000001 --tag bank
quantix watchlist tag list --code 000001
quantix watchlist history [--code 000001] [--limit 20]
```

## Task 5: Implement Watchlist CLI Handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/core/runtime.rs`
- Modify: `src/watchlist/storage.rs`
- Modify: `src/watchlist/service.rs`
- Modify: `src/watchlist/resolver.rs`
- Test: `tests/watchlist_handler_test.rs`

**Handler rules:**

- Load store from a stable file path
- Execute service mutation/query
- Persist updated store after write commands
- Enrich list rows for `--with-price`
- Keep write path independent from quote lookup
- `list --with-price` must remain a best-effort browse enhancement, not a monitoring runtime

## Task 6: Add End-To-End CLI Smoke Tests And User Docs

**Files:**
- Create: `tests/watchlist_cli_smoke_test.rs`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`

**Required smoke flows:**

1. add -> list -> remove
2. add -> tag add -> list --tag
3. add -> move -> history
4. list --with-price degrades gracefully when quote source is unavailable

## Recommended Delivery Order

1. Models and storage
2. Service rules
3. CLI parsing
4. Base handlers
5. Resolver and `--with-price`
6. Smoke tests and docs

## P0 Exit Criteria

- `watchlist` group/tag/history/price-display CLI is usable
- All mutations write JSON and append local history events
- `list --with-price` degrades gracefully
- Later phases can read watchlist as a stable universe input
