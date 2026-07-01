# OpenStock Data Consumption P0.11 Closeout

Date: 2026-07-02

Status: P0.11 (TdxApiClient removal) closed

OpenSpec change: `openstock-data-consumption-p0-11`

FUNCTION_TREE node: `sources/` data layer; `cli/handlers/` dispatcher;
`tasks/collect_scheduler.rs` runtime

Governance nodes:

- `project-governance/P0.11` (transitioned to `completed` per task 4.4)

## Closure Decision

P0.11 — the subtractive slice that removed the `TdxApiClient` REST adapter
and the 18 `data tdx-api` CLI subcommands — is closed. The OpenStock
provider chain (P0.8 → P0.10) is the canonical market data source; no
production code path still references `tdx-api`, `TdxApiClient`, or the
`tdx_api_fallback` scheduler field.

Two commits implement the closeout:

| Commit | Phase | Lines changed | Summary |
|---|---|---|---|
| `d1dda04` | Phase 4 (code removal) | 30 added / 2,656 deleted | Delete `tdx_api.rs` (1,309 L) + `tdx_api_handler.rs` (726 L) + 6 config helpers + 8 data_handler helpers + 4 tests; clean dead `tdx_api_fallback` field from `collect_scheduler` |
| `b03b93e` | Phase 5 (ecosystem docs) | 54 added / 84 deleted | Deprecation notices in `docker-compose.yml`, `FUNCTION_TREE.md`, `CLI_COMMAND_MANUAL.html`, `README.md`, `CHANGELOG.md`, `TDX_API_BRIDGE_GUIDE.md` |

Phase 1 (openstock branch migration, `d73f860`) and Phases 2-3 (schema
audit + scheduler reroute, `c5e2152`) shipped earlier in the slice.

## Decisions Revisited

| Decision | Choice | Rationale | Outcome |
|---|---|---|---|
| D1 — `collect_scheduler` tdx_api_fallback | **B** (delete) | grep confirmed zero callers; D1=A would be new feature work, not migration | Dead field removed; no replacement needed |
| D2 — handlers location | **A** (migrate to `openstock_handler.rs`) | Co-located with peer openstock handlers | `import_openstock_ticks` / `import_openstock_klines` already there |
| D3 — TDengine `direction TINYINT` column | **B** (audit-only no-op) | On-site grep showed the column had always been named `direction`; no migration needed | Phase 2 marked as no-op (3c.11-3c.13); 0.5 day saved |
| D4 — CLI shape | **A** (top-level promote) | `data import-ticks` / `data import-klines` more discoverable than nested `data tdx-api import-*` | Promoted in Phase 1, dispatcher arms updated |

## Consumer Reroutes

| Old path | New path | Notes |
|---|---|---|
| `data tdx-api import-ticks --source openstock` | `data import-ticks` | Phase 1 promotion (commit `d73f860`) |
| `data tdx-api import-klines --source openstock` | `data import-klines` | Phase 1 promotion |
| `CollectScheduler::set_tdx_api_fallback(...)` | (removed) | Dead code; no replacement |
| `TdxApiClient::fetch_*` | `OpenStockClient::fetch_*` wrappers | Covered in P0.9 / P0.10 |
| `DataSourceKind::TdxApi` | (removed) | Source-list/set-default/test commands now `tdx`/`akshare` only |
| `TDX_API_URL` env var | (removed from docker-compose) | OpenStock uses `OPENSTOCK_BASE_URL` + `OPENSTOCK_API_KEY` (P0.10) |

## Residual References (doc-only, expected)

`grep -rn "tdx_api|TdxApi|tdx-api" src/ tests/ --include="*.rs"` returns
empty — source tree is clean. Doc-only residue (kept as historical
record, not authoritative for current binary):

- `docs/CLI_COMMAND_MANUAL.html` — 152 historical matches in the
  deprecated `data tdx-api` section; top-level banner added in Phase 5
- `CHANGELOG.md` — historical 2026-06-09 entry preserved per changelog
  convention; new 2026-07-02 Removed entry added
- `docs/guides/TDX_API_BRIDGE_GUIDE.md` — guide preserved with banner
- `README.md` Phase 31 section — replaced with deprecation notice
- `docker-compose.yml` — tdx-api service block commented for one-release
  rollback per design.md R6; full removal scheduled for P0.12
- `FUNCTION_TREE.md` — all five reference points marked deprecated/removed

## Verification

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | clean |
| Lints | `cargo clippy --all-targets --workspace -- -D warnings` | 0 warnings |
| Tests | `cargo test --workspace` | 1431 passed, 16 ignored |
| OpenSpec | `openspec validate openstock-data-consumption-p0-11 --strict` | valid |
| Source grep | `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/ --include="*.rs"` | empty |
| Impact (gitnexus) | `gitnexus detect_changes` | 11 changed symbols, 5 affected processes (all `import_openstock_ticks` doc-cleanup ripple), medium risk per process-count threshold, no surprise files |

## Follow-ups (not blocking closure)

1. **P0.12 — full tdx-api service block removal from `docker-compose.yml`.**
   Wait one release cycle for rollback safety per R6.
2. **`scripts/daily-update.sh` rewrite.** Currently references deleted
   `data tdx-api sync-calendar` / `data tdx-api import-klines --all`.
   Either rewrite to OpenStock (`data openstock fetch-calendar` /
   `data import-klines`) or delete. Not in P0.11 scope.
3. **`docs/CLI_COMMAND_MANUAL.html` full HTML rewrite.** 152 historical
   refs kept as reference; full cleanup deferred (banner added in Phase 5).
4. **tdx-api Docker image itself.** Not in this repo. Image retirement
   tracked under the openstock repo (see `openstock/docs/operations`).
