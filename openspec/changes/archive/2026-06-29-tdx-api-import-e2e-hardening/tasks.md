# tdx-api Import E2E Hardening Tasks

## 0. Baseline And Scope

- [x] 0.1 Confirm the active commit and record it in the E2E evidence report.
  - Status 2026-06-06: recorded `f169e6a docs: add tdx-api import e2e openspec` in `docs/reports/evidence/tdx-api-import-e2e-20260606/preflight-blocked.md`.
- [x] 0.2 Confirm `docs/reports/TDX_API_FOLLOWUP_VALUE_ANALYSIS_2026-06-06.md` is the value basis for this change.
  - Status 2026-06-06: confirmed and referenced in the preflight evidence.
- [x] 0.3 Confirm the E2E environment owns non-production tdx-api, ClickHouse, and TDengine targets or explicitly marks the run as BLOCKED.
  - Status 2026-06-06: marked BLOCKED because the current TDengine config path points to unavailable `localhost:6041`.
- [x] 0.4 Record the selected E2E scope:
  - tdx-api base URL.
  - ClickHouse host/database/table target.
  - TDengine host/database/table target.
  - K-line import market bound, such as `--exchange sh`, if full-market `--all` is too large for the gate.
  - Tick import `code` and `date`.
  - Status 2026-06-06: partial scope recorded in `docs/reports/evidence/tdx-api-import-e2e-20260606/preflight-blocked.md`; tick `code/date` intentionally not selected while TDengine is blocked.

## 1. Dependency Preflight

- [x] 1.1 Run `quantix data tdx-api health` against the selected `TDX_API_URL` and capture exit code plus response summary.
  - Status 2026-06-06: default URL timed out; `TDX_API_URL=http://192.168.123.104:8089` passed with `healthy=true status=running connected=true version=1.0.0`.
- [x] 1.2 Verify ClickHouse connectivity through the same configuration path used by `quantix data tdx-api import-klines`.
  - Status 2026-06-06: `.env` ClickHouse URL `http://192.168.123.104:8123/ping` returned HTTP 200 `Ok.`.
- [x] 1.3 Verify TDengine connectivity through the same configuration path used by `quantix data tdx-api import-ticks`.
  - Status 2026-06-06: checked and failed; current config path points to `localhost:6041`, which is not reachable from this shell.
- [x] 1.4 If any dependency is unavailable, stop the E2E gate and record BLOCKED with the missing dependency, command, endpoint, and error. Do not mark the gate passed.
  - Status 2026-06-06: stopped before import writes; BLOCKED evidence saved at `docs/reports/evidence/tdx-api-import-e2e-20260606/preflight-blocked.md`.

## 2. ClickHouse K-line Import E2E

- [ ] 2.1 Run the K-line E2E command through the `--all` code path:

```bash
TDX_API_URL=<tdx-api-url> quantix data tdx-api import-klines --all --type day
```

- [ ] 2.2 If full-market execution is too large for the release gate, rerun with an explicit approved exchange bound while still exercising `--all`:

```bash
TDX_API_URL=<tdx-api-url> quantix data tdx-api import-klines --all --exchange sh --type day
```

- [ ] 2.3 Capture command exit code, selected scope, total code count, imported row count, skipped count, failed count, and elapsed time.
- [ ] 2.4 Query ClickHouse for imported rows and record:
  - Non-zero row count.
  - Minimum and maximum trade date.
  - At least one imported symbol.
  - Source marker for THS front-adjusted data.
  - OHLCV sanity checks: high >= low, open/close within high/low when fields are present, volume >= 0.
- [ ] 2.5 Rerun the same command without `--force` and record incremental behavior. The rerun must either import zero new rows or only rows newer than the previous maximum date.

## 3. TDengine Tick Import E2E

- [ ] 3.1 Run the tick E2E command for the approved code/date:

```bash
TDX_API_URL=<tdx-api-url> quantix data tdx-api import-ticks --code <code> --date <YYYYMMDD>
```

- [ ] 3.2 Capture command exit code, code, date, imported tick count, and elapsed time.
- [ ] 3.3 Query TDengine for imported rows and record:
  - Non-zero row count.
  - Minimum and maximum timestamp.
  - Price > 0 for imported trade rows.
  - Volume >= 0.
  - The target table or stable name used by quantix.
- [ ] 3.4 If tdx-api returns no tick data for the selected code/date, rerun with a documented alternate trading day before marking the E2E gate blocked.

## 4. Evidence And Documentation

- [ ] 4.1 Save E2E evidence under `docs/reports/evidence/tdx-api-import-e2e-<YYYYMMDD>/`.
- [ ] 4.2 Include command lines, exit codes, dependency endpoints with secrets redacted, database target names, row counts, validation query summaries, and failure diagnostics.
- [ ] 4.3 Update the tdx-api bridge summary or follow-up report with the E2E result and evidence path.
- [ ] 4.4 Keep host-specific secrets, tokens, passwords, and raw large logs out of the repository.

## 5. Release And Closure Gates

- [ ] 5.1 Run `cargo test -p quantix-cli --quiet` after E2E hardening work and capture exit 0 evidence.
- [ ] 5.2 Run `cargo build --release --quiet` in a stable terminal/session and capture exit 0 evidence.
- [ ] 5.3 Run task-relevant clippy only if implementation changes Rust code.
- [ ] 5.4 Run GitNexus impact before any function, method, class, or refactor target edit.
- [ ] 5.5 Run GitNexus `detect_changes` before closure if any repository files are changed.
- [ ] 5.6 Record Graphiti review/debug conclusion memory after the E2E result converges, or leave a local backfill summary if Graphiti is rate-limited.
