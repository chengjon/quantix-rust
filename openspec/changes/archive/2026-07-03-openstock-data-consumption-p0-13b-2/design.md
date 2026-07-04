# Design: openstock-data-consumption-p0-13b-2

Full design rationale: `docs/superpowers/specs/2026-07-02-openstock-p0-13b-2-minute-share-design.md`
(R1 revision `d25410d`).

## Key Decisions

- **D1**: Inline category string `"MINUTE_DATA"` (matches fetch_stock_codes style)
- **D2**: `RawMinuteRecord` uses `Option<Decimal>` directly (no from_f64_retain hop)
- **D3**: `parse_minute_share` returns `Option` (not `Result`) to support INV-2C skip
- **D4**: `parse_time_minutes` accepts both "0930" and "09:30" formats (R2 mitigation)
- **D5**: Parser inline in `openstock_client.rs` (consistent with P0.13b-1)
- **D6**: All price/amount/avg_price fields use `Decimal` (project-wide consistency)

## Risks

- **R1**: MINUTE_DATA actual schema unverified -> `#[serde(default)]` + wiremock-first
- **R2**: `time_minutes` format ambiguity -> D4 dual-format acceptance
- **R3**: String-vs-number serde drift -> fall back to `serde_json::Value` + `parse_decimal`
- **R4**: Envelope-level failure (5xx, retry exhausted) -> whole batch fails (different
       dimension from INV-2C single-record skip)
- **R5**: Concurrent modification with P0.13b-1 -> none (P0.13b-1 merged)

## Invariants

- **INV-1A**: Must use `self.fetch::<T>()` envelope path (not direct reqwest)
- **INV-1B**: Request params must be `{code, date}` only (no period/adjust)
- **INV-2C**: Single record missing field -> warn+skip (not batch fail)
- **INV-3**: `price`/`amount`/`avg_price` are `Decimal`; `volume` is `i64`
- **INV-4**: Never bypass envelope retry/circuit breaker
