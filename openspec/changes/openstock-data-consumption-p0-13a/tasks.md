# Tasks

## 0. Baseline

- [x] Spec: `docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md`
- [x] Plan: `docs/superpowers/plans/2026-07-02-openstock-p0-13a-multi-period-klines-plan.md`
- [x] Governance card: `.governance/programs/project-governance/cards/P0.13a.yaml`

## 1. Phase 1 — Client method

- [x] Add `BarPeriod` enum + `FromStr` + `AdjustType::as_openstock_param` + `FromStr` + 5 unit tests
- [x] Add `OpenStockClient::fetch_klines` + 3 wiremock tests
- [x] Quality gates green (fmt / clippy / cargo test --lib openstock)

## 2. Phase 2 — CLI wiring

- [x] Add `FetchKlines` variant to `OpenStockCommands`
- [x] Add `fetch_openstock_klines` handler
- [x] Wire dispatcher arm + re-export
- [x] Quality gates green (cargo test --workspace)
- [x] Offline CLI smoke (`--period invalid` fails fast; `--help` renders)

## 3. Phase 3 — Live tests + closeout

- [x] Add 3 live `#[ignore]` tests (T6 day+None, T7 week+qfq, T8 month+hfq)
- [ ] Manual live smoke against `http://192.168.123.104:8040` (when reachable)
- [x] Update HANDOFF report B-group row ❌ → ✅
- [x] Update FUNCTION_TREE.md
- [x] `openspec validate openstock-data-consumption-p0-13a --strict`
- [x] `openspec validate --all --strict`
- [x] Archive OpenSpec change
