# Tasks

## 1. Baseline And Governance

- [x] Create P0.15a governance card
- [x] Scaffold OpenSpec change (proposal/tasks/design)
- [ ] Add REQ-PERSIST-006..010 + scenarios (Task 7 of implementation plan)

## 2. CLI Variants

- [ ] Add `ImportMinuteKlines` variant on `OpenStockCommands`
- [ ] Add `ImportMinuteShare` variant on `OpenStockCommands`

## 3. Handler — compute_apply helper

- [ ] Add `MINUTE_APPLY_ENV` const + `pub(crate) fn compute_apply(apply: bool) -> bool`

## 4. Handler — import_openstock_minute_klines

- [ ] Parse period/adjust/daterange
- [ ] Dry-run branch (stream + count, no ClickHouse)
- [ ] Apply branch (construct sink, call P0.14 consumer)
- [ ] Hint message when `--apply` set but env var unset

## 5. Handler — import_openstock_minute_share

- [ ] Same shape as klines, no period/adjust

## 6. Dispatcher + re-exports

- [ ] Add 2 match arms in `app_shell.rs`
- [ ] Add 2 re-exports in `handlers/mod.rs`

## 7. Unit tests U1/U2/U3

- [ ] `import_minute_args_validate_period_and_adjust`
- [ ] `compute_apply_reads_env_var`
- [ ] `compute_apply_returns_false_when_apply_flag_false`

## 8. Live tests L1/L2

- [ ] `cli_import_minute_klines_round_trip`
- [ ] `cli_import_minute_share_round_trip`

## 9. OpenSpec requirements

- [ ] Add REQ-PERSIST-006..010 with scenarios

## 10. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-15a --strict`
- [ ] `openspec validate --all --strict`
