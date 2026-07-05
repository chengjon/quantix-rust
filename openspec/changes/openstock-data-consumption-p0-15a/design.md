# Design Decisions

Source: docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md

## D1: `import-` prefix (not `fetch-` or `persist-`)

`fetch-*` = read-only. `persist-*` = shadow-write. `import-*` = canonical-write. Matches `ImportKlines` (`data.rs:74`) and `import_openstock_klines` (`openstock_handler.rs:981`).

## D2: Single env var for both subcommands

`QUANTIX_OPENSTOCK_MINUTE_APPLY` gates both. They are always used together in the future scheduler (every code gets both).

## D3: `compute_apply` env-aware helper

Reads env var internally. Tests must `std::env::set_var` the real name to pass — not a `&&` tautology.

## D4: stdout summary / stderr per-batch progress

Mirrors `fetch_openstock_minute_klines` `--stream` pattern. Lets operators redirect stdout to a file while seeing live progress.

## D5: Range-only (no --date shortform)

Mirrors `ImportKlines`. `from_cli(None, start, end)` enforces range-only when `date=None`.

## D6: Live tests in new file

`tests/openstock_live_import_minute.rs` is separate from `tests/openstock_live_minute_klines.rs` because the surfaces differ (import CLI vs fetch stream).

## Risks

See spec §9. R1 (lifetime) handled by inference; R2 (silent dry-run) handled by hint; R3 (partial failure) handled by INV-FLOW-1 documentation + per-batch output; R4 (huge range) handled by weekly chunking.
