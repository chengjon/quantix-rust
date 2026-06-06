# tdx-api Import E2E Preflight Evidence

Date: 2026-06-06
OpenSpec change: `tdx-api-import-e2e-hardening`
Status: BLOCKED
Commit checked: `f169e6a docs: add tdx-api import e2e openspec`

## Scope

This preflight executed OpenSpec tasks 0 and 1 without running any import command that writes to ClickHouse or TDengine.

Selected environment discovered from the current workspace:

- tdx-api default runtime URL: not set in shell; quantix default resolved to `http://tdx-api:8080`.
- tdx-api live URL from the prior bridge evidence: `http://192.168.123.104:8089`.
- ClickHouse URL from `.env`: `http://192.168.123.104:8123`.
- TDengine URL from `config/default.toml`: `http://localhost:6041`.
- TDengine same-host probe: `http://192.168.123.104:6041`.

Secrets were not printed or stored.

## Results

| Check | Result | Evidence |
|-------|--------|----------|
| `quantix data tdx-api health` with default environment | BLOCKED for default URL | Exit 1 after timeout to `http://tdx-api:8080/api/health`; this Docker-internal host is not resolvable/reachable from the current shell. |
| `quantix data tdx-api health` with `TDX_API_URL=http://192.168.123.104:8089` | PASS | Exit 0; `tdx-api: healthy=true status=running connected=true version=1.0.0`. |
| ClickHouse `/ping` from `.env` URL | PASS | `http://192.168.123.104:8123/ping` returned HTTP 200 with body `Ok.`. |
| TDengine default config endpoint | BLOCKED | `http://localhost:6041/` and `http://localhost:6041/rest/sql` were not connectable from the current shell. |
| TDengine same-host probe | INCONCLUSIVE | `http://192.168.123.104:6041/` and `/rest/sql` returned HTTP 404, proving an HTTP listener exists but not proving the current quantix `import-ticks` configuration path can connect successfully. |

## Blocking Reason

The OpenSpec change cannot proceed to write-type E2E commands yet because `quantix data tdx-api import-ticks` uses `AppConfig::load("config")`, and the current checked config points TDengine to `localhost:6041`.

That endpoint is unavailable from this shell. Running `import-ticks` now would at best fail during TDengine connectivity, and at worst would mix partial live tdx-api reads with an unverified database target.

## Required Input To Unblock

Provide or configure a non-production TDengine target that is reachable through the same configuration path used by quantix:

- `config/default.toml` or the active config overlay must point `[database.tdengine]` at the intended host and port.
- The target database/table must be safe for E2E writes.
- Credentials must be available locally but must not be committed.

The tdx-api URL should also be selected explicitly for the E2E run:

```bash
TDX_API_URL=http://192.168.123.104:8089
```

## Next Attempt

After TDengine config is corrected, rerun dependency preflight before any import write:

```bash
TDX_API_URL=http://192.168.123.104:8089 quantix data tdx-api health
```

Then verify ClickHouse and TDengine through the same configuration paths used by:

```bash
quantix data tdx-api import-klines --all --exchange sh --type day
quantix data tdx-api import-ticks --code <code> --date <YYYYMMDD>
```
