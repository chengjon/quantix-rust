# Design: openstock-data-consumption-p0-13d

Full design rationale: `docs/superpowers/specs/2026-07-03-openstock-p0-13d-streaming-design.md`
(commit `21484df`, R1-revised). This file is a quick-reference summary; the spec is authoritative.

## Key Decisions

### D1 — Stream type: `impl Stream<Item = Result<Vec<T>>>`

Return `impl futures::Stream<Item = Result<Vec<...>, QuantixError>>` instead of
`Pin<Box<dyn Stream>>`, channel, or callback.

- Compile-time monomorphization, zero heap allocation (beyond the Vec itself)
- Idiomatic caller pattern `while let Some(r) = s.next().await`
- `Vec<T>` per batch reduces `next()` call count vs single-item streams

**Rejected**: callback (cannot await), Paginator struct (boilerplate), tokio mpsc
(complex error propagation), `Pin<Box<dyn Stream>>` (runtime overhead).

### D2 — Klines chunking: fixed 7-day segments

`chunk_range_weekly` slices by `start + 7 days`, not natural calendar weeks.

- No `chrono::Weekday` dependency; unambiguous across calendars
- Uniform segments, each ≤ 7 days
- Pure function, easy to test

**Rejected**: natural week (Weekday dependency), monthly (uneven length), daily
(too fragmented), adaptive (unnecessary complexity — 1m weekly ≈ 1.2k bars, far
below threshold).

### D3 — Share chunking: one batch per day, non-trading days yield empty Vec

Each `NaiveDate` yields one `Vec<MinuteShare>`; non-trading days yield `vec![]`.

- Perfectly reuses P0.13c `fetch_minute_share_single`
- Batch count == calendar-day count → callers can do completeness checks

**Rejected**: skip non-trading days (loses day-level signal), typed
`Trading/NonTrading` enum (extra type complexity), N-day accumulation
(unnecessary knob).

### D4 — Error semantics: first error terminates the stream

Any batch failure → yield `Err(...)` → subsequent `next()` returns `None`.

- Semantically consistent with batch API
- Prior batches already yielded are completed side effects (same property as
  batch API)
- Simple and predictable

**Rejected**: error-not-terminating (easy to swallow), tail-aggregation
(non-standard stream shape), in-stream retry (orthogonal to D5).

### D5 — Retry/circuit-breaker: inherit existing path behavior

Stream batches each go through their existing underlying path; no new
retry/breaker wrapper is introduced.

- klines path currently has no retry (`fetch_minute_klines` directly calls
  `self.http.post()`) — stream preserves current behavior
- share path already has retry+breaker (`fetch_minute_share_single` →
  `fetch::<T>()`) — stream inherits automatically
- Consistency: stream behavior aligns with batch (INV-1A prerequisite)
- Avoids introducing a "stream is more robust than batch" asymmetry

**Rejected**: add retry to stream klines batches (scope creep), remove retry
from stream share (regression).

### D6 — Dual API side-by-side

`fetch_minute_klines` (Vec) and `fetch_minute_klines_stream` (Stream) coexist;
each implements independently.

- Zero churn: all P0.13a/b/c wiremock / live / governance unchanged
- INV-4A directly guaranteed
- Equivalence verified by INV-1A (S5 test) + L1 live

**Rejected**: `fetch_minute_klines = stream.collect()` (DRY but no batch memory
benefit, introduces churn risk, violates user decision), deprecate batch API
(large-scale churn, all P0.13b/c callers need migration).

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| **R1** | Stream type signature introduces a new public trait bound, causing downstream compile errors | Public methods return `impl Stream + '_` (no `Send`); callers `use futures::StreamExt`; CI `cargo build --workspace` is the backstop |
| **R2** | `chunk_range_weekly` boundary bug (off-by-one, gap in coverage) | S4 end-to-end coverage test |
| **R3** | CLI `--stream` flag ignored on the batch path → ambiguous behavior | Design guarantee: flag defaults to `false`; when `flag=true` the handler takes a fully independent stream branch |
| **R4** | klines stream has no retry; a transient 5xx fails large-range jobs | Recorded in D5; accepted as a known asymmetry; candidate follow-up for P0.13e |

> **R1 revision** (from spec §7): the original R2 ("`futures` crate not in
> workspace") is removed — verified `Cargo.toml:38-39` already declares
> `futures = "0.3"` and `futures-util = "0.3"`, no new dependency needed.
> Original R3-R5 were renumbered to R1-R4 above.

## Alternatives Considered

The design alternatives (callback, Paginator struct, tokio mpsc,
`Pin<Box<dyn Stream>>`, natural-week chunking, monthly chunking, daily klines
chunking, skip-non-trading-days, typed Trading/NonTrading enum, N-day share
accumulation, error-not-terminating, tail-aggregation, in-stream retry,
`fetch_minute_klines = stream.collect()`, deprecate batch API) are discussed in
full in `docs/superpowers/specs/2026-07-03-openstock-p0-13d-streaming-design.md`
§6 (single source of truth).
