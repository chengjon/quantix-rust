# openstock_client.rs split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `src/sources/openstock_client.rs` (2364 lines, 4.7× module force-split limit) into a module directory whose largest file is under the 500-line warn threshold.

**Architecture:** Pure-internal reorganization. `OpenStockClient` struct + its config stay in the parent file as the canonical home; the generic `fetch<T>` HTTP core + circuit-breaker helpers also stay (they own private fields). The 12 family fetch methods + 2 private range/single helpers move into sibling files under `src/sources/openstock_client/`, each as a separate `impl OpenStockClient { ... }` block (Rust permits multiple impl blocks for the same type across files). The 1271-line `#[cfg(test)] mod tests` block is split by family into sibling test files using the same pattern. **No public API change** — every existing import path (`crate::sources::openstock_client::OpenStockClient`) continues to work, because the parent file remains at the same path and just gains `mod` declarations.

**Tech Stack:** Rust 1.83, reqwest, serde, futures (existing deps only — no new dependencies).

## Global Constraints

- **Public API frozen**: `OpenStockClient`, `OpenStockClientConfig`, all 12 `fetch_*` method signatures, `new`/`from_env`/`from_settings`, return types — unchanged. The 22 external files (12 src + 10 tests) that reference `OpenStockClient` must compile without edits.
- **No field visibility changes**: Sibling files call only `self.fetch(...)` and the public `fetch_*` methods. They must **not** touch `self.circuit`/`self.config`/`self.http`/`self.base_url`/`self.api_key` directly. The one exception is `from_settings`/`fetch_daily_klines`/`fetch_klines`/`fetch_minute_klines_range` which currently read `self.api_key`/`self.config` — those stay in the parent file alongside the fields they touch.
- **File-size targets after split** (each under 500-line warn):
  - `openstock_client.rs` (parent): struct + config + ctor + `fetch<T>` + circuit helpers ≈ ≤ 500 lines (production only; tests move out)
  - `openstock_client/klines.rs`: `fetch_historical_klines`, `fetch_daily_klines`, `fetch_klines`, `fetch_index_klines`, `fetch_tick_data` ≈ ≤ 350 lines
  - `openstock_client/minute.rs`: `fetch_minute_klines`, `fetch_minute_klines_range`, `fetch_minute_share`, `fetch_minute_share_single`, `fetch_minute_klines_stream`, `fetch_minute_share_stream` ≈ ≤ 500 lines
  - `openstock_client/reference.rs`: `fetch_stock_codes`, `fetch_all_stocks`, `fetch_trade_dates`, `fetch_workdays` ≈ ≤ 150 lines
  - Test files: `openstock_client/tests_core.rs`, `tests_klines.rs`, `tests_minute.rs`, `tests_reference.rs` — each ≤ 500 lines
- **One impl block per family**: Each sibling file contains exactly one `impl OpenStockClient { ... }` block (or one `impl OpenStockClient` block grouped by purpose). No struct redefinition.
- **Tests in sibling files via `#[cfg(test)] mod tests_*`**: The parent file declares `#[cfg(test)] mod tests_core;` etc. Each `tests_*.rs` file is a plain `mod` (not `#[cfg(test)]` itself — the parent declaration gates it).
- **Each task ends green**: `cargo build -p quantix-cli`, `cargo test -p quantix-cli openstock`, `cargo fmt --check`, `cargo clippy -p quantix-cli --tests -D warnings` all pass before commit.
- **Atomic commits**: One commit per task. Commit message format `<type>(sources): <subject>` per CLAUDE.md.
- **No Cargo.toml changes**: This is a pure file move.

---

## File Structure

```
src/sources/
  openstock_client.rs              # parent: struct + config + ctor + fetch<T> + circuit + mod decls
  openstock_client/
    mod.rs                         # re-exports + `impl OpenStockClient {}` placeholder (empty) if needed
    klines.rs                      # impl OpenStockClient { fetch_historical_klines, fetch_daily_klines, fetch_klines, fetch_index_klines, fetch_tick_data }
    minute.rs                      # impl OpenStockClient { fetch_minute_klines, fetch_minute_klines_range, fetch_minute_share, fetch_minute_share_single, fetch_minute_klines_stream, fetch_minute_share_stream }
    reference.rs                   # impl OpenStockClient { fetch_stock_codes, fetch_all_stocks, fetch_trade_dates, fetch_workdays }
    tests_core.rs                  # #[cfg(test)] in-file; gated by parent
    tests_klines.rs
    tests_minute.rs
    tests_reference.rs
```

Note on `mod.rs`: when `openstock_client.rs` and `openstock_client/` coexist, Rust treats `openstock_client.rs` as the parent (no `mod.rs` needed in the directory — Rust 2018+ convention). If for some reason Rust complains, fall back to making the parent file `openstock_client/mod.rs`. **Plan assumes the `openstock_client.rs` + `openstock_client/` layout** (standard Rust 2018 style).

---

### Task 1: Scaffold the module directory (no logic moves yet)

**Goal:** Establish the target files as empty stubs so subsequent tasks can move code in increments. Verify the project still builds and all tests pass with the empty modules declared.

**Files:**
- Create: `src/sources/openstock_client/klines.rs`
- Create: `src/sources/openstock_client/minute.rs`
- Create: `src/sources/openstock_client/reference.rs`
- Create: `src/sources/openstock_client/tests_core.rs`
- Create: `src/sources/openstock_client/tests_klines.rs`
- Create: `src/sources/openstock_client/tests_minute.rs`
- Create: `src/sources/openstock_client/tests_reference.rs`
- Modify: `src/sources/openstock_client.rs` — add `mod` declarations at the bottom (above existing `#[cfg(test)] mod tests` block)

**Interfaces:**
- Consumes: nothing (scaffolding only)
- Produces: empty child modules that compile; parent still owns all current logic

- [ ] **Step 1: Create the three production stub files**

Each file contains only a header comment and a private impl block placeholder:

`src/sources/openstock_client/klines.rs`:
```rust
//! K-line family fetch methods for [`crate::sources::openstock_client::OpenStockClient`].
//!
//! Impl block is populated in a later task; this file is a scaffold.
```

`src/sources/openstock_client/minute.rs`:
```rust
//! Minute-data family fetch methods for [`crate::sources::openstock_client::OpenStockClient`].
//!
//! Impl block is populated in a later task; this file is a scaffold.
```

`src/sources/openstock_client/reference.rs`:
```rust
//! Reference-data family fetch methods (codes, all_stocks, trade_dates, workdays)
//! for [`crate::sources::openstock_client::OpenStockClient`].
//!
//! Impl block is populated in a later task; this file is a scaffold.
```

- [ ] **Step 2: Create the four test stub files**

Each test stub file is a Rust module (not gated with `#[cfg(test)]` itself — that gate lives in the parent):

`src/sources/openstock_client/tests_core.rs`:
```rust
//! Tests for the HTTP core (fetch, retry, circuit breaker, constructors).
//! Populated by a later task.
```

Repeat for `tests_klines.rs`, `tests_minute.rs`, `tests_reference.rs` with appropriate doc-comment text.

- [ ] **Step 3: Declare the child modules in the parent file**

In `src/sources/openstock_client.rs`, immediately **before** the existing `#[cfg(test)] mod tests {` block (currently at L1094), add:

```rust
mod klines;
mod minute;
mod reference;

#[cfg(test)]
mod tests_core;
#[cfg(test)]
mod tests_klines;
#[cfg(test)]
mod tests_minute;
#[cfg(test)]
mod tests_reference;
```

- [ ] **Step 4: Verify build + tests + lint**

Run:
```bash
cargo build -p quantix-cli
cargo test -p quantix-cli openstock
cargo fmt --check
cargo clippy -p quantix-cli --tests -- -D warnings
```
Expected: build succeeds (child modules are empty but declared — Rust allows empty modules). All existing openstock tests still pass. Clippy clean.

- [ ] **Step 5: Commit**

```bash
git add src/sources/openstock_client.rs src/sources/openstock_client/
git commit -m "$(cat <<'EOF'
refactor(sources): scaffold openstock_client module directory

Create empty child modules (klines, minute, reference, tests_*) for the
upcoming openstock_client.rs split. No logic moves yet; parent file
unchanged apart from mod declarations.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Move reference-data methods to `reference.rs`

**Goal:** Move `fetch_stock_codes` (L323-335), `fetch_trade_dates` (L336-357), `fetch_all_stocks` (L436-451), `fetch_workdays` (L452-481) — and their doc comments — from the parent file's `impl OpenStockClient { ... }` block into a new `impl OpenStockClient { ... }` block in `reference.rs`.

**Why this task first:** These four methods are the simplest (no private helpers, no field access, just `self.fetch(...)` calls). They establish the move pattern that the klines/minute tasks follow.

**Files:**
- Modify: `src/sources/openstock_client/reference.rs`
- Modify: `src/sources/openstock_client.rs`

**Interfaces:**
- Consumes: `self.fetch<T>()` public method on `OpenStockClient`
- Produces: 4 methods accessible on `OpenStockClient` exactly as before

- [ ] **Step 1: Copy methods (with their `///` doc comments) into `reference.rs`**

In `reference.rs`, add the imports the methods need (look at the parent file's existing imports — likely just `serde_json` and `crate::core::Result`):

```rust
use serde_json::json;

use crate::core::Result;
use crate::sources::openstock_codes::StockCodeRecord;
// (other imports as needed by the four methods)

impl super::OpenStockClient {
    // <paste fetch_stock_codes here, doc-comment included>
    // <paste fetch_trade_dates here>
    // <paste fetch_all_stocks here>
    // <paste fetch_workdays here>
}
```

Use `super::OpenStockClient` because the struct lives in the parent module.

- [ ] **Step 2: Delete the same methods from the parent file's `impl` block**

Remove the four methods (with doc comments) from `src/sources/openstock_client.rs`. Leave all other methods untouched.

- [ ] **Step 3: Verify build + tests + lint**

Run the same four commands as Task 1 Step 4. Expected: build clean, the reference-data tests (`fetch_stock_codes` / `fetch_trade_dates` / `fetch_all_stocks` / `fetch_workdays` paths) still pass.

- [ ] **Step 4: Commit**

```bash
git add src/sources/openstock_client.rs src/sources/openstock_client/reference.rs
git commit -m "$(cat <<'EOF'
refactor(sources): move reference-data fetch methods to child module

Move fetch_stock_codes, fetch_trade_dates, fetch_all_stocks, fetch_workdays
out of openstock_client.rs into openstock_client/reference.rs. Public API
unchanged; OpenStockClient type lookup is unchanged.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Move K-line family methods to `klines.rs`

**Goal:** Move `fetch_index_klines` (L358-386), `fetch_historical_klines` (L387-419), `fetch_tick_data` (L420-435), `fetch_daily_klines` (L482-581), `fetch_klines` (L582-693) into `klines.rs`.

**Caveat:** `fetch_daily_klines` and `fetch_klines` currently access `self.api_key` directly. **They must continue to compile after the move.** If `api_key` is a private field on `OpenStockClient`, the move will fail with E0616 (private field). Resolve by either:
- (preferred) Inspecting why `api_key` is read directly — if it's a logging/debug breadcrumb, refactor to not require the field; OR
- (fallback) Adding a `pub(super) fn api_key(&self) -> &HeaderValue` accessor on `OpenStockClient` in the parent file. **Only do this if the direct field access is unavoidable.** Document the reason in the commit message.

**Files:**
- Modify: `src/sources/openstock_client/klines.rs`
- Modify: `src/sources/openstock_client.rs` (only if adding an accessor)

**Interfaces:**
- Consumes: `self.fetch<T>()` and possibly `self.api_key` (via new accessor if needed)
- Produces: 5 K-line family methods accessible as before

- [ ] **Step 1: Inspect `fetch_daily_klines`/`fetch_klines` bodies for `self.api_key` usage**

Read those two methods' bodies. Determine whether the `self.api_key` access is:
- (a) Required for a request header — in which case it should already be set up in the generic `fetch<T>` core (the `new` ctor stores `api_key` as a `HeaderValue` on the struct and the `fetch` core attaches it to every request). If the methods read it for header setup themselves, that's redundant — remove the redundancy.
- (b) Required for a non-HTTP purpose (logging, etc.) — add a `pub(super)` accessor.

Document the finding in the report.

- [ ] **Step 2: Copy methods (with doc comments) into `klines.rs`**

Pattern identical to Task 2 Step 1. Add any imports the methods need (`rust_decimal`, `serde_json`, `crate::data::models::*`, etc. — check parent file's imports).

```rust
impl super::OpenStockClient {
    // <fetch_index_klines>
    // <fetch_historical_klines>
    // <fetch_tick_data>
    // <fetch_daily_klines>
    // <fetch_klines>
}
```

- [ ] **Step 3: Delete moved methods from the parent file**

- [ ] **Step 4: If accessor was needed, add it to parent file**

If Step 1 finding was (b), add to the parent file inside `impl OpenStockClient { ... }`:
```rust
pub(super) fn api_key(&self) -> &reqwest::header::HeaderValue {
    &self.api_key
}
```

- [ ] **Step 5: Verify build + tests + lint**

Same four commands. Expected: all klines tests pass (`fetch_klines_day_none_sends_period_day_and_omits_adjust`, `fetch_klines_qfq_sends_adjust_qfq_and_stamps_records`, `fetch_klines_propagates_4xx`, plus index/tick tests).

- [ ] **Step 6: Commit**

```bash
git add src/sources/openstock_client.rs src/sources/openstock_client/klines.rs
git commit -m "$(cat <<'EOF'
refactor(sources): move klines family fetch methods to child module

Move fetch_index_klines, fetch_historical_klines, fetch_tick_data,
fetch_daily_klines, fetch_klines out of openstock_client.rs into
openstock_client/klines.rs.

<If accessor added:> Add pub(super) api_key() accessor on OpenStockClient
to support the klines methods' direct field access without widening
public API.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Move minute-data family methods to `minute.rs`

**Goal:** Move `fetch_minute_klines` (L694-760), `fetch_minute_klines_range` (L761-872, private helper), `fetch_minute_share` (L873-930), `fetch_minute_share_single` (L931-967, private helper), `fetch_minute_klines_stream` (visible in earlier scan, ~L720+), `fetch_minute_share_stream` (after fetch_minute_share) into `minute.rs`.

**Caveat:** `fetch_minute_klines_range` accesses `self.api_key` directly (same issue as Task 3). Apply the same resolution.

**Files:**
- Modify: `src/sources/openstock_client/minute.rs`
- Modify: `src/sources/openstock_client.rs` (only if accessor not already added in Task 3)

**Interfaces:**
- Consumes: `self.fetch<T>()`, possibly `self.api_key()` accessor (added in Task 3)
- Produces: 6 minute family methods accessible as before

- [ ] **Step 1: Copy methods (with doc comments) into `minute.rs`**

Includes the two private helpers (`fetch_minute_klines_range`, `fetch_minute_share_single`) — they stay `fn` (no `pub`), only accessible via the public methods in the same `impl` block.

```rust
impl super::OpenStockClient {
    // <fetch_minute_klines>
    // <fetch_minute_klines_range>  (private helper, no pub)
    // <fetch_minute_share>
    // <fetch_minute_share_single>  (private helper, no pub)
    // <fetch_minute_klines_stream>
    // <fetch_minute_share_stream>
}
```

- [ ] **Step 2: Delete moved methods from the parent file**

- [ ] **Step 3: Verify build + tests + lint**

Expected: all minute tests pass (the 18 minute-family tests identified in the scan).

- [ ] **Step 4: Commit**

```bash
git add src/sources/openstock_client.rs src/sources/openstock_client/minute.rs
git commit -m "$(cat <<'EOF'
refactor(sources): move minute-data family fetch methods to child module

Move fetch_minute_klines, fetch_minute_klines_range, fetch_minute_share,
fetch_minute_share_single, fetch_minute_klines_stream,
fetch_minute_share_stream out of openstock_client.rs into
openstock_client/minute.rs. Private helpers stay private within the
impl block.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Verify parent file is now under 500 lines (production) + audit test split

**Goal:** After Tasks 2-4 the parent file's `impl OpenStockClient { ... }` should only contain: `new`, `from_env`, `from_settings`, `reset_circuit`, `record_circuit_failure`, `fetch<T>`. Verify the line count; if still over 500 (warn), decide whether to further split or accept.

**Files:**
- Read-only audit: `src/sources/openstock_client.rs`

- [ ] **Step 1: Measure parent file size**

Run:
```bash
wc -l src/sources/openstock_client.rs
```
Record the count.

- [ ] **Step 2: Determine split status**

- If production code (everything before `#[cfg(test)] mod tests`) is ≤ 500 lines → green. The 1271-line test block remains in the parent file (test code doesn't trigger the file-size rule per CLAUDE.md, but see Task 6 to split it for maintainability).
- If production code is still > 500 lines → STOP and ask the human partner before proceeding. Do not auto-split further without confirmation.

Report the production-only line count in the task report.

- [ ] **Step 3: No commit** (audit-only task); proceed to Task 6.

---

### Task 6: Move test code into sibling test files

**Goal:** Distribute the 1271-line `#[cfg(test)] mod tests` block across four sibling test files by family. The parent file's `#[cfg(test)] mod tests` block is **removed entirely**; the four `tests_*.rs` files declared in Task 1 take its place.

**Files:**
- Modify: `src/sources/openstock_client/tests_core.rs`
- Modify: `src/sources/openstock_client/tests_klines.rs`
- Modify: `src/sources/openstock_client/tests_minute.rs`
- Modify: `src/sources/openstock_client/tests_reference.rs`
- Modify: `src/sources/openstock_client.rs` — delete the entire `#[cfg(test)] mod tests { ... }` block

**Test family mapping** (based on the 38 test functions scanned):

- `tests_core.rs`: ctor tests (`from_envelope_*` ×3, `from_settings_*` ×3), helper fns (`fast_test_cfg`, `success_body`), HTTP core tests (`fetch_*` retry/4xx/corrupt, `circuit_breaker_*` ×4, `success_resets_circuit`) — 17 tests total
- `tests_klines.rs`: `fetch_klines_day_none_*`, `fetch_klines_qfq_*`, `fetch_klines_propagates_4xx` — 3 tests
- `tests_minute.rs`: 18 tests covering `fetch_minute_klines*`, `fetch_minute_share*`, `parse_time_minutes`, streams
- `tests_reference.rs`: tests for `fetch_stock_codes`, `fetch_trade_dates`, `fetch_all_stocks`, `fetch_workdays` (currently 1 visible in scan, but check if more exist between L1415-1477 where the gap appears)

- [ ] **Step 1: Move tests_core.rs contents**

Copy from parent file's tests block: the 3 `from_envelope_*` tests, 3 `from_settings_*` tests, 2 helpers (`fast_test_cfg`, `success_body`), and 9 fetch/circuit-breaker tests into `tests_core.rs`.

Add the module-level setup that the parent's `mod tests {` block had (typically `use super::*;` plus any test-only imports). Make `tests_core.rs` begin with:

```rust
use super::*;
// additional test-only imports as needed
```

The parent's `#[cfg(test)] mod tests_core;` declaration (added in Task 1) makes this compile under `cfg(test)` only.

- [ ] **Step 2: Move tests_klines.rs, tests_minute.rs, tests_reference.rs contents**

Same pattern. Each file starts with `use super::*;` and contains the relevant test fns.

- [ ] **Step 3: Delete the parent file's entire `#[cfg(test)] mod tests { ... }` block**

After the move, the parent file's tests block should be completely removed. Only production code + the `mod` declarations (added in Task 1) remain.

- [ ] **Step 4: Verify build + tests + lint**

Run the four commands. Expected: **all 38 openstock client tests still pass**, distributed across the four new files. The total test count for the workspace is unchanged.

- [ ] **Step 5: Verify final file sizes**

Run:
```bash
wc -l src/sources/openstock_client.rs src/sources/openstock_client/*.rs
```
All files should be under the 500-line warn threshold (or, for `tests_minute.rs` which holds 18 tests, acceptably close — if > 500, flag it).

- [ ] **Step 6: Commit**

```bash
git add src/sources/openstock_client.rs src/sources/openstock_client/
git commit -m "$(cat <<'EOF'
refactor(sources): split openstock_client tests by family

Distribute the 1271-line #[cfg(test)] mod tests block across four sibling
test files (tests_core, tests_klines, tests_minute, tests_reference) to
match the production-code split. Parent file's tests block removed
entirely.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 7: Final gate + summary

**Goal:** Confirm the split is complete and all quality gates are green.

- [ ] **Step 1: Run the full quality gate**

```bash
cargo fmt --check
cargo clippy --workspace --tests -- -D warnings
cargo test
cargo build --release
```

Expected: all four green. The total test count is unchanged from before the split (modulo any tests added by other in-flight work).

- [ ] **Step 2: Run GitNexus detect_changes**

```bash
gitnexus analyze   # refresh index
```
Then via MCP:
```
gitnexus_detect_changes(scope: "compare", base_ref: "master~7")
```
Expected: changed symbols confined to `src/sources/openstock_client*`. No external symbols flagged.

- [ ] **Step 3: Update progress ledger**

Append to `.superpowers/sdd/progress.md`:
```
## Tech-debt sweep — openstock_client.rs split (2026-07-09)
- Task 1-7 complete (commits <base>..<head>)
- Final file sizes: <list>
- Public API unchanged; 38 client tests pass; 22 caller files unmodified
```

- [ ] **Step 4: No commit** (ledger is gitignored). Report to human partner.
