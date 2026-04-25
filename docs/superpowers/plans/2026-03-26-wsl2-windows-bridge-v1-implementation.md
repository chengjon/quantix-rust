# WSL2 Windows Bridge v1 Implementation Plan

> Historical context:
> This dated implementation plan reflects the intended v1 scope on 2026-03-26.
> Current project state has moved beyond the original preview-only QMT contract:
> guarded `qmt_live` real submission now exists, while generic `target_mode=live` remains unimplemented.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a working `TDX` Windows bridge source for `quantix-rust` and, at that time, a `QMT` preview-only broker contract that aligned with the then-current execution-request / execution-kernel architecture without enabling real live order submission.

**Architecture:** This feature spans two codebases and must be implemented as two coordinated tracks: a Windows-side `quantix-bridge` HTTP service and a Rust-side bridge client/integration layer. `TDX` ships as the first real remote capability, while this plan's original `QMT` scope ships only as a preview/validation path that consumes frozen execution snapshots and does not mutate request state or submit live orders.

**Tech Stack:** Rust 2024, tokio, reqwest, serde/serde_json, clap, chrono, rust_decimal, existing `CliRuntime`, existing execution/request models, existing watchlist/source code, Python 3.11+, FastAPI, pydantic, pytest, FastAPI `TestClient`, optional Rust dev-dependency `wiremock` for bridge client contract tests.

---

## Preflight

- Read the approved architecture doc in [WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md](/opt/claude/quantix-rust/docs/architecture/WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the slice is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})`. If GitNexus transport is unavailable, refresh it before editing or record the fallback explicitly.
- Before every commit in this repo, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Graphiti is mandatory for design/debug/handoff memories. If ingest fails, keep an equivalent local summary and mark `Graphiti backfill required`.
- This plan intentionally separates the work into two tracks:
  - Track A: Windows-side bridge service at `/mnt/d/mystocks/quantix/quantix_bridge`
  - Track B: Rust-side integration in this repository
- Do not start Track B Task 4 until Track A Task 2 has a stable `TDX` contract.
- Do not start Track B Task 6 until Track A Task 5 has a stable `QMT preview` contract.

## File Map

### Track A: Windows `quantix-bridge`

- `/mnt/d/mystocks/quantix/quantix_bridge/pyproject.toml`
  - Python project metadata and dependencies.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/main.py`
  - FastAPI application bootstrap and route registration.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/config.py`
  - Environment-backed bridge settings.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/security.py`
  - API key validation and host restrictions.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/models/common.py`
  - Shared response wrappers and capability payloads.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/models/tdx.py`
  - `TDX` quote / kline response models.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/models/qmt.py`
  - `QMT` preview and account-status models.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/health.py`
  - `/health`.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/capabilities.py`
  - `/api/v1/capabilities`.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/tdx.py`
  - `TDX` quote / kline endpoints.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/qmt.py`
  - `QMT` account status and preview endpoints.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/services/tdx_service.py`
  - Windows-side `TDX` quote / kline adapter boundary.
- `/mnt/d/mystocks/quantix/quantix_bridge/app/services/qmt_preview_service.py`
  - Windows-side `QMT` SDK probing and preview-only payload validation.
- `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_health.py`
  - Health and auth coverage.
- `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_tdx_routes.py`
  - `TDX` contract coverage.
- `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_qmt_preview.py`
  - `QMT preview` contract coverage.

### Track B: `quantix-rust`

- `src/bridge/mod.rs`
  - Bridge module exports.
- `src/bridge/client.rs`
  - HTTP client, auth injection, timeout handling, capability / `TDX` / `QMT preview` calls.
- `src/bridge/models.rs`
  - Rust-side request/response models for bridge payloads.
- `src/bridge/error.rs`
  - Bridge-specific error mapping.
- `src/core/runtime.rs`
  - Environment-backed bridge runtime settings.
- `src/lib.rs`
  - Re-export `bridge`.
- `src/sources/bridge_tdx.rs`
  - Remote `TDX` source and batch quote helpers.
- `src/sources/mod.rs`
  - Export `BridgeTdxSource`.
- `src/watchlist/resolver.rs`
  - Bridge-backed quote lookup implementation and wiring seam.
- `src/execution/qmt_bridge.rs`
  - Preview-only `QmtBridgePreviewAdapter`.
- `src/execution/mod.rs`
  - Export preview adapter.
- `src/cli/mod.rs`
  - Add `execution bridge` subcommands.
- `src/cli/handlers.rs`
  - Add bridge status / `QMT preview` handlers.
- `src/cli/tests/execution.rs`
  - Parser coverage for new bridge commands.
- `tests/bridge_client_test.rs`
  - HTTP contract tests for Rust bridge client.
- `tests/bridge_tdx_source_test.rs`
  - `TDX` source integration tests.
- `tests/qmt_bridge_preview_test.rs`
  - Preview adapter and CLI handler tests.
- `tests/watchlist_bridge_lookup_test.rs`
  - Watchlist lookup behavior when bridge-backed `TDX` is enabled.
- `README.md`
  - Operator-facing summary of bridge support.
- `docs/USER_MANUAL.md`
  - Bridge env vars, `TDX` usage, `QMT preview` workflow.

---

## Track A: Windows Bridge Service

### Task 1: Create the FastAPI skeleton with mandatory auth and capabilities

**Files:**
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/pyproject.toml`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/main.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/config.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/security.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/models/common.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/health.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/capabilities.py`
- Test: `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_health.py`

- [ ] **Step 1: Write the failing FastAPI contract tests**

Add tests that require:

```python
def test_health_is_public(client):
    response = client.get("/health")
    assert response.status_code == 200

def test_capabilities_requires_api_key(client):
    response = client.get("/api/v1/capabilities")
    assert response.status_code == 401

def test_capabilities_returns_tdx_and_qmt_modes(client, api_headers):
    response = client.get("/api/v1/capabilities", headers=api_headers)
    assert response.json()["tdx"]["enabled"] is True
    assert response.json()["qmt"]["mode"] == "preview_only"
```

- [ ] **Step 2: Run pytest to verify RED**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_health.py -q
```

Expected:
- FAIL because the bridge project and routes do not exist yet.

- [ ] **Step 3: Implement the skeleton**

Create:

```python
# /mnt/d/mystocks/quantix/quantix_bridge/app/security.py
from fastapi import Header, HTTPException

async def require_api_key(x_quantix_api_key: str | None = Header(default=None)):
    if x_quantix_api_key != settings.api_key:
        raise HTTPException(status_code=401, detail="invalid_api_key")
```

And:

```python
# /mnt/d/mystocks/quantix/quantix_bridge/app/routes/capabilities.py
@router.get("/api/v1/capabilities")
async def capabilities(_: None = Depends(require_api_key)):
    return {
        "tdx": {"enabled": True, "supports": ["quote", "batch_quotes", "kline"]},
        "qmt": {"enabled": True, "mode": "preview_only", "supports": ["account_status", "order_preview"]},
    }
```

- [ ] **Step 4: Re-run pytest to verify GREEN**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_health.py -q
```

Expected:
- PASS

- [ ] **Step 5: Commit the bridge skeleton**

Run:
```bash
git -C /mnt/d/mystocks/quantix/quantix_bridge add pyproject.toml app tests
git -C /mnt/d/mystocks/quantix/quantix_bridge commit -m "feat: add bridge service skeleton"
```

### Task 2: Add real `TDX` quote / kline HTTP contracts

**Files:**
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/models/tdx.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/tdx.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/services/tdx_service.py`
- Test: `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_tdx_routes.py`

- [ ] **Step 1: Write the failing `TDX` route tests**

Add tests that require:

```python
def test_batch_quotes_returns_normalized_symbols(client, api_headers, fake_tdx_service):
    response = client.post("/api/v1/data/tdx/quotes", json={"symbols": ["000001.SZ"]}, headers=api_headers)
    body = response.json()
    assert body["quotes"][0]["symbol"] == "000001.SZ"
    assert body["quotes"][0]["source"] == "tdx_bridge"

def test_kline_returns_period_and_bars(client, api_headers, fake_tdx_service):
    response = client.get("/api/v1/data/tdx/kline/000001.SZ?period=1d&start=2026-03-01&end=2026-03-26", headers=api_headers)
    body = response.json()
    assert body["period"] == "1d"
    assert len(body["bars"]) == 1
```

- [ ] **Step 2: Run pytest to verify RED**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_tdx_routes.py -q
```

Expected:
- FAIL because `TDX` routes and models do not exist yet.

- [ ] **Step 3: Implement the minimal `TDX` service layer**

Create a service boundary like:

```python
class TdxService:
    async def fetch_quotes(self, symbols: list[str]) -> list[QuotePayload]: ...
    async def fetch_kline(self, symbol: str, period: str, start: str, end: str) -> list[KlineBarPayload]: ...
```

Implement route handlers that:
- require API key
- validate `period`
- normalize symbols to `000001.SZ` / `600519.SH`
- return `source="tdx_bridge"`

- [ ] **Step 4: Re-run pytest to verify GREEN**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_tdx_routes.py -q
```

Expected:
- PASS

- [ ] **Step 5: Commit the `TDX` contract**

Run:
```bash
git -C /mnt/d/mystocks/quantix/quantix_bridge add app/models/tdx.py app/routes/tdx.py app/services/tdx_service.py tests/test_tdx_routes.py
git -C /mnt/d/mystocks/quantix/quantix_bridge commit -m "feat: add tdx bridge contracts"
```

### Task 3: Add `QMT` preview-only account-status and order-preview routes

**Files:**
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/models/qmt.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/routes/qmt.py`
- Create: `/mnt/d/mystocks/quantix/quantix_bridge/app/services/qmt_preview_service.py`
- Test: `/mnt/d/mystocks/quantix/quantix_bridge/tests/test_qmt_preview.py`

- [ ] **Step 1: Write the failing `QMT preview` tests**

Add tests that require:

```python
def test_qmt_account_status_reports_preview_mode(client, api_headers, fake_qmt_service):
    response = client.get("/api/v1/broker/qmt/account/status", headers=api_headers)
    assert response.json()["mode"] == "preview_only"

def test_qmt_preview_returns_adapter_contract_shape(client, api_headers, fake_qmt_service):
    payload = {
        "request_id": "req_1",
        "client_order_id": "cli_1",
        "symbol": "000001.SZ",
        "side": "buy",
        "quantity": 100,
        "price": "15.50",
        "order_type": "limit",
        "snapshot_metadata": {"source": "execution_request"},
    }
    response = client.post("/api/v1/broker/qmt/orders/preview", json=payload, headers=api_headers)
    body = response.json()
    assert body["latest_status"] == "accepted"
    assert body["filled_quantity"] == 0
    assert body["fill_details"] is None
```

- [ ] **Step 2: Run pytest to verify RED**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_qmt_preview.py -q
```

Expected:
- FAIL because the `QMT` routes and preview service do not exist yet.

- [ ] **Step 3: Implement `preview_only` behavior**

Implement:

```python
class QmtPreviewService:
    async def get_account_status(self) -> dict: ...
    async def preview_order(self, payload: PreviewOrderRequest) -> PreviewOrderResponse: ...
```

Rules:
- call Windows SDK only for availability / mapping validation
- never submit a real order
- return fields that map directly to Rust `OrderInitialResponse`
- reject malformed symbols / unsupported order types with `latest_status="rejected"`

- [ ] **Step 4: Re-run pytest to verify GREEN**

Run:
```bash
python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_qmt_preview.py -q
```

Expected:
- PASS

- [ ] **Step 5: Commit the preview-only `QMT` contract**

Run:
```bash
git -C /mnt/d/mystocks/quantix/quantix_bridge add app/models/qmt.py app/routes/qmt.py app/services/qmt_preview_service.py tests/test_qmt_preview.py
git -C /mnt/d/mystocks/quantix/quantix_bridge commit -m "feat: add qmt preview-only bridge contract"
```

---

## Track B: Rust Integration

### Task 4: Add bridge runtime settings and the Rust HTTP client

**Files:**
- Create: `src/bridge/mod.rs`
- Create: `src/bridge/client.rs`
- Create: `src/bridge/models.rs`
- Create: `src/bridge/error.rs`
- Modify: `src/core/runtime.rs`
- Modify: `src/lib.rs`
- Modify: `Cargo.toml`
- Test: `tests/bridge_client_test.rs`

- [ ] **Step 1: Run impact analysis for runtime and CLI entry symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "CliRuntime", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_execution_command", direction: "upstream"})
```

Expected:
- `CliRuntime` should show broad but manageable CLI/runtime consumers.
- `run_execution_command` should show CLI-only impact.

- [ ] **Step 2: Write the failing Rust bridge client tests**

Add tests that require:

```rust
let runtime = CliRuntime::load();
assert_eq!(runtime.bridge.base_url, "http://127.0.0.1:17580");

let capabilities = client.capabilities().await.unwrap();
assert!(capabilities.tdx.enabled);
assert_eq!(capabilities.qmt.mode, "preview_only");
```

Use `wiremock` (add it to `[dev-dependencies]`) so the client tests do not depend on a real Windows host.

- [ ] **Step 3: Run the focused tests to verify RED**

Run:
```bash
cargo test --test bridge_client_test -- --nocapture
```

Expected:
- FAIL because the `bridge` module and runtime settings do not exist yet.

- [ ] **Step 4: Implement runtime settings and client**

In `src/core/runtime.rs`, add:

```rust
pub const BRIDGE_BASE_URL_ENV: &str = "QUANTIX_BRIDGE_BASE_URL";
pub const BRIDGE_API_KEY_ENV: &str = "QUANTIX_BRIDGE_API_KEY";

pub struct BridgeRuntimeSettings {
    pub base_url: String,
    pub api_key: Option<String>,
    pub tdx_enabled: bool,
    pub qmt_preview_enabled: bool,
}
```

And in `src/bridge/client.rs`, add methods:
- `health()`
- `capabilities()`
- `fetch_tdx_quotes(...)`
- `fetch_tdx_kline(...)`
- `qmt_account_status()`
- `qmt_preview_order(...)`

- [ ] **Step 5: Re-run tests, detect changes, and commit**

Run:
```bash
cargo test --test bridge_client_test -- --nocapture
```

Then:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add Cargo.toml src/bridge src/core/runtime.rs src/lib.rs tests/bridge_client_test.rs
git commit -m "feat: add rust bridge client foundation"
```

### Task 5: Add `BridgeTdxSource` and integrate it into watchlist quote lookup

**Files:**
- Create: `src/sources/bridge_tdx.rs`
- Modify: `src/sources/mod.rs`
- Modify: `src/watchlist/resolver.rs`
- Test: `tests/bridge_tdx_source_test.rs`
- Test: `tests/watchlist_bridge_lookup_test.rs`

- [ ] **Step 1: Run impact analysis for `TdxWatchlistQuoteLookup`**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "TdxWatchlistQuoteLookup", direction: "upstream"})
```

Expected:
- watchlist/UI consumers only; no execution-kernel risk.

- [ ] **Step 2: Write the failing source and watchlist tests**

Add tests that require:

```rust
let quotes = source.fetch_quotes_batch(&[(0, "000001")]).await.unwrap();
assert_eq!(quotes[0].code, "000001.SZ");

let lookup = BridgeTdxWatchlistQuoteLookup::new(client);
let rows = lookup.lookup_quotes(&vec!["000001".to_string()]).await.unwrap();
assert!(rows.contains_key("000001.SZ"));
```

Cover:
- batch quotes map into existing `StockQuote`
- kline rows map into existing `Kline`
- watchlist lookup can switch to bridge-backed data without changing caller shape

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test bridge_tdx_source_test -- --nocapture
cargo test --test watchlist_bridge_lookup_test -- --nocapture
```

Expected:
- FAIL because `BridgeTdxSource` and bridge-backed lookup do not exist yet.

- [ ] **Step 4: Implement the bridge-backed source**

Create:

```rust
pub struct BridgeTdxSource {
    client: BridgeHttpClient,
}

impl BridgeTdxSource {
    pub async fn fetch_quotes_batch(&self, codes: &[(u16, &str)]) -> Result<Vec<StockQuote>> { ... }
}

#[async_trait]
impl Fetcher for BridgeTdxSource { ... }
```

In `src/watchlist/resolver.rs`, add a separate lookup type:

```rust
pub struct BridgeTdxWatchlistQuoteLookup {
    source: BridgeTdxSource,
}
```

Do not rewrite the existing `TdxWatchlistQuoteLookup`; add a new bridge-backed implementation and a narrow constructor seam so callers can opt in explicitly.

- [ ] **Step 5: Re-run tests, detect changes, and commit**

Run:
```bash
cargo test --test bridge_tdx_source_test -- --nocapture
cargo test --test watchlist_bridge_lookup_test -- --nocapture
```

Then:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/sources/bridge_tdx.rs src/sources/mod.rs src/watchlist/resolver.rs tests/bridge_tdx_source_test.rs tests/watchlist_bridge_lookup_test.rs
git commit -m "feat: add bridge-backed tdx source"
```

### Task 6: Add `execution bridge` CLI and `QMT` preview adapter

**Files:**
- Create: `src/execution/qmt_bridge.rs`
- Modify: `src/execution/mod.rs`
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/handlers.rs`
- Modify: `src/cli/tests/execution.rs`
- Test: `tests/qmt_bridge_preview_test.rs`

- [ ] **Step 1: Run impact analysis for `ExecutionCommands` and request handlers**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "ExecutionCommands", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_execution_command", direction: "upstream"})
```

Expected:
- CLI parser / handler impact only.

- [ ] **Step 2: Write the failing parser and preview tests**

Add parser tests that require:

```rust
let cli = Cli::try_parse_from(["quantix", "execution", "bridge", "status"]).unwrap();
let cli = Cli::try_parse_from(["quantix", "execution", "bridge", "qmt-preview", "--request-id", "req-1"]).unwrap();
```

Add adapter tests that require:

```rust
let response = adapter.preview_request(&request_record).await.unwrap();
assert_eq!(response.latest_status, OrderStatus::Accepted);
assert_eq!(response.filled_quantity, 0);
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test qmt_bridge_preview_test -- --nocapture
cargo test --lib cli::tests::execution -- --nocapture
```

Expected:
- FAIL because the bridge subcommands and preview adapter do not exist yet.

- [ ] **Step 4: Implement the preview-only Rust integration**

Extend CLI:

```rust
pub enum ExecutionCommands {
    Config(ExecutionConfigCommands),
    Daemon(ExecutionDaemonCommands),
    Bridge(ExecutionBridgeCommands),
}

pub enum ExecutionBridgeCommands {
    Status,
    QmtPreview { request_id: String },
}
```

Add `QmtBridgePreviewAdapter` with a method like:

```rust
pub async fn preview_request(
    &self,
    request: &ExecutionRequestRecord,
) -> Result<OrderInitialResponse>
```

Rules:
- load the frozen `execution_snapshot` from `request.payload_json`
- do not mutate request status
- do not call `ExecutionKernel::execute_request(...)`
- do not submit live orders

- [ ] **Step 5: Re-run tests, detect changes, and commit**

Run:
```bash
cargo test --test qmt_bridge_preview_test -- --nocapture
cargo test --lib cli::tests::execution -- --nocapture
```

Then:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/execution/qmt_bridge.rs src/execution/mod.rs src/cli/mod.rs src/cli/handlers.rs src/cli/tests/execution.rs tests/qmt_bridge_preview_test.rs
git commit -m "feat: add qmt bridge preview cli"
```

### Task 7: Document bridge usage and lock the operator contract

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Test: `tests/repo_hygiene_test.rs` or the repo’s existing documentation lock test file if one already covers these docs

- [ ] **Step 1: Write failing documentation lock tests**

Add assertions that require the docs to mention:

```text
QUANTIX_BRIDGE_BASE_URL
QUANTIX_BRIDGE_API_KEY
quantix execution bridge status
quantix execution bridge qmt-preview --request-id <ID>
```

- [ ] **Step 2: Run the documentation test to verify RED**

Run:
```bash
cargo test repo_hygiene -- --nocapture
```

Expected:
- FAIL because the new bridge wording is not present yet.

- [ ] **Step 3: Update docs**

Document:
- bridge env vars
- `TDX` bridge enablement
- `QMT preview` is preview-only and non-mutating
- troubleshooting for auth failure / bridge unavailable / Windows SDK unavailable

- [ ] **Step 4: Re-run the documentation test to verify GREEN**

Run:
```bash
cargo test repo_hygiene -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Detect changes and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add bridge operator guidance"
```

---

## Verification Checklist

- `/mnt/d/mystocks/quantix/quantix_bridge` tests pass:
  - `python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_health.py -q`
  - `python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_tdx_routes.py -q`
  - `python -m pytest /mnt/d/mystocks/quantix/quantix_bridge/tests/test_qmt_preview.py -q`
- Rust bridge client and source tests pass:
  - `cargo test --test bridge_client_test -- --nocapture`
  - `cargo test --test bridge_tdx_source_test -- --nocapture`
  - `cargo test --test watchlist_bridge_lookup_test -- --nocapture`
  - `cargo test --test qmt_bridge_preview_test -- --nocapture`
- CLI parser tests pass:
  - `cargo test --lib cli::tests::execution -- --nocapture`
- No request lifecycle regression:
  - `cargo test --test execution_daemon_test -- --nocapture`
  - `cargo test --test execution_kernel_test -- --nocapture`
- No unexpected blast radius:
  - `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`

---

## Notes For Execution

- Keep `QMT preview` intentionally outside `strategy request execute`; it is an operator-facing validation path, not a request consumer.
- Prefer explicit env-backed runtime settings in `CliRuntime` over introducing a second active config loader path for v1.
- Do not refactor `QuoteCollector` in the first pass unless the bridge-backed source is already green and the extra wiring is necessary for a user-visible outcome.
- If the sibling repo `/mnt/d/mystocks/quantix/quantix_bridge` does not yet exist, initialize it before Track A Task 1 and keep its commits separate from this repo.
