# tdx-api REST Source Design

Date: 2026-06-05

## Decision

Integrate `tdx-api` as a Quantix runtime data source through its REST API.

Do not require `tdx-api` to run as an MCP server for Quantix runtime use. MCP is
an optional future agent/tooling layer that can wrap the REST API, but it should
not be the production strategy, sync, or market-data transport.

## Context

The external `tdx-api` project exposes a Go REST service backed by TDX public
servers. The service is already running at the documented NAS endpoint and was
validated from this workspace.

Validated REST probes:

| Endpoint | Result |
| --- | --- |
| `/api/health` | HTTP 200, `status=healthy` |
| `/api/server-status` | HTTP 200, `code=0`, `status=running`, `connected=true`, `version=1.0.0` |
| `/api/quote?code=000001` | HTTP 200, `code=0`, 1 quote |
| `/api/batch-quote` | HTTP 200, `code=0`, 2 quotes |
| `/api/kline?code=000001&type=day` | HTTP 200, `code=0`, 8423 daily bars |
| `/api/search?keyword=...` | HTTP 200, `code=0`, 3 matches |

The health endpoint differs from the guide's documented response shape. The
guide says health returns `code/message`, but the running service returns
`status=healthy`. Quantix must support the observed shape for health checks.

`tdx-api/FUNCTION_TREE.md` records these relevant implemented capabilities:

- Quote and batch quote: `/api/quote`, `/api/batch-quote`
- K-line data: `/api/kline`, `/api/kline-history`, `/api/kline-all`,
  `/api/kline-all/tdx`, `/api/kline-all/ths`
- Intraday and trades: `/api/minute`, `/api/trade`, `/api/trade-history`,
  `/api/trade-history/full`, `/api/minute-trade-all`
- Code discovery: `/api/search`, `/api/codes`, `/api/stock-codes`,
  `/api/etf-codes`, `/api/etf`
- Index and market state: `/api/index`, `/api/index/all`,
  `/api/market-stats`, `/api/market-count`
- Calendar and income helpers: `/api/workday`, `/api/workday/range`,
  `/api/income`
- Async pull tasks: `/api/tasks`, `/api/tasks/pull-kline`,
  `/api/tasks/pull-trade`, `/api/tasks/{id}`

Known `tdx-api` issues to avoid in the first integration slice:

- `extend/pull-trade.go` multi-period merge bug
- `web/server.go` dead `getMinuteWithFallback` path
- `extend/codes-server.go` error-return bug

## Current Quantix Baseline

Quantix already has partial `tdx-api` support:

- `src/sources/tdx_api.rs` defines `TdxApiConfig` and `TdxApiClient`.
- `TdxApiClient` already exposes REST methods for quote, batch quote, k-line,
  minute, trades, code search, code lists, workday, market stats, index k-line,
  k-line history, full k-line, pull tasks, task inspection, cancellation, and
  health.
- `src/cli/handlers/tdx_api_handler.rs` exposes CLI-level TDX API commands.
- `src/bridge/client.rs` has bridge methods for TDX quotes and k-line.
- `tests/bridge_tdx_source_test.rs` validates quote and k-line model mapping.
- Strategy daemon fallback tests already exercise a primary source with TDX
  fallback behavior.

This means the implementation should consolidate and productize the existing
client instead of creating a new parallel data-source stack.

## Goals

- Make `tdx-api` a documented, configurable, read-only stable data source.
- Reuse `TdxApiClient` as the runtime REST client.
- Start with low-risk endpoints: health, quote, batch quote, daily k-line,
  code search, and code lists.
- Make source selection explicit in config and CLI behavior.
- Preserve current fallback behavior and make `tdx-api` eligible as a primary
  or fallback market-data source where the existing abstractions support it.
- Add tests around response-shape compatibility, mapping, error handling, and
  source selection.

## Non-Goals

- Do not build an MCP server in the first slice.
- Do not route Quantix runtime data through MCP.
- Do not use NAS/Docker credentials in application config.
- Do not integrate async pull tasks, trade-history full pulls, or known
  unstable `tdx-api` paths in the first slice.
- Do not replace every existing data source at once.

## Architecture

The first slice should use this runtime path:

```text
Quantix command / strategy / sync flow
  -> source selection
  -> TdxApiClient
  -> tdx-api REST endpoint
  -> Quantix domain model
```

Optional future agent path:

```text
AI agent
  -> tdx-api MCP wrapper
  -> tdx-api REST endpoint
```

The two paths should remain separate. Runtime source code must depend on REST
client behavior, not MCP tool availability.

## Configuration

Use or extend the existing `TdxApiConfig` shape rather than adding a new config
family.

Required config fields:

- `base_url`
- request timeout
- enabled/disabled flag

Recommended operational fields:

- max batch quote size, defaulting to the `tdx-api` documented limit of 50
- retry count for idempotent reads
- source priority when used as fallback
- health-check timeout

Configuration should support environment variables and app config, reusing the
existing `from_env` / `from_app_config` path where possible.

## Source Scope

Initial stable source capability:

| Capability | Endpoint | Quantix behavior |
| --- | --- | --- |
| Health | `/api/health`, `/api/server-status` | Accept `status=healthy` and `code=0` success shapes |
| Single quote | `/api/quote` | Map to existing `StockQuote` |
| Batch quote | `/api/batch-quote` | Preserve code order where possible, cap batch size |
| Daily k-line | `/api/kline?type=day` | Map to existing `Kline` |
| Code search | `/api/search` | Use for symbol discovery and diagnostics |
| Code list | `/api/codes`, `/api/stock-codes`, `/api/etf-codes` | Use for source validation and future sync work |

Deferred capability:

- minute and trade data
- full k-line history pull
- index and market-count integration
- async task orchestration
- MCP wrapper

## Error Handling

The REST client should classify failures into stable Quantix errors:

- endpoint unavailable or timeout
- HTTP non-success
- JSON decode failure
- upstream `code != 0`
- missing or unexpected `data` shape
- health endpoint not healthy
- unsupported endpoint in this first slice

Health behavior should use both:

- `/api/health` for liveness
- `/api/server-status` for upstream TDX connection state

`server-status.connected=false` should mark the source unhealthy for runtime
data, even if the HTTP service itself is reachable.

## Testing

Unit and integration tests should avoid requiring the NAS service unless they
are explicitly marked as live smoke checks.

Required test layers:

- pure mapping tests for quote and k-line response shapes
- health parser tests for both documented and observed response shapes
- client error classification tests using local test server fixtures
- source-selection tests showing `tdx-api` as primary and fallback source
- CLI parser tests for any new flags or config commands

Live smoke checks should be opt-in and guarded by environment variables such as
`QUANTIX_TDX_API_LIVE_BASE_URL`.

## Rollout Plan

1. Spec and implementation plan only.
2. First code slice: stabilize config, health parsing, and live smoke command.
3. Second code slice: promote quote/batch quote/daily k-line to stable source
   paths.
4. Third code slice: connect source selection and strategy fallback behavior.
5. Later slice: decide whether an MCP wrapper is useful for agent operations.

## Acceptance Criteria

- Quantix can verify the running `tdx-api` service through a health command.
- Quantix can fetch and map a single quote, batch quotes, and daily k-line data
  through `TdxApiClient`.
- Health checks pass with the observed `status=healthy` shape.
- Runtime data access does not require MCP.
- Tests cover config, mapping, health, failure classification, and source
  selection.
- No NAS credentials or deployment secrets are committed.

## Open Follow-Up

Before implementation planning, confirm the first code slice should be:

```text
health/config hardening -> quote/batch/kline stable source -> source selection
```

This keeps the initial implementation small enough to verify end to end while
still moving toward `tdx-api` as a stable data source.
