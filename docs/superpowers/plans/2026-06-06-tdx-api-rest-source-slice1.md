# tdx-api REST Source Slice 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden Quantix's existing `tdx-api` REST client enough to use it as a reliable, read-only runtime source for health, server-status, quote, batch quote, and daily k-line follow-up work.

**Architecture:** Keep `tdx-api` as a direct REST runtime source, not an MCP runtime dependency. `src/core/config.rs::TdxApiConfig` remains the serde/app-config source of truth, while `src/sources/tdx_api.rs::TdxApiConfig` is a derived runtime client config. `TdxApiClient` gains typed health/server-status behavior and batch-size validation before later source-selection work.

**Tech Stack:** Rust, `reqwest`, `serde`, `tokio`, `wiremock`, existing `QuantixError`, existing `TdxApiClient`.

---

## Scope

This plan implements the first slice from
`docs/superpowers/specs/2026-06-05-tdx-api-rest-source-design.md`:

```text
config/health/server-status hardening -> batch cap -> CLI health smoke
```

This plan does not implement source selection, strategy fallback changes, async
task orchestration, or MCP.

## Pre-Execution Worktree Gate

Before executing this plan, confirm the implementation workspace is clean or
that existing dirty changes are explicitly owned by the current task.

At plan-writing time, this branch also contained uncommitted source changes in:

- `config/holidays.json`
- `src/cli/commands/data.rs`
- `src/cli/handlers/tdx_api_handler.rs`
- `src/core/trading_calendar.rs`

Do not overwrite, revert, or silently mix those changes into this plan. If they
are still present when execution starts, inspect them first, decide whether they
belong to this slice, and either commit them separately, move to a clean
worktree, or update this plan before changing code.

## Files

- Modify: `src/core/config.rs`
  - Add serde/app-config fields and defaults for `tdx-api` enablement,
    max batch size, and health timeout.
- Modify: `src/sources/tdx_api.rs`
  - Add runtime config fields.
  - Add config conversion helper.
  - Add typed health and server-status response structs.
  - Add `health_status()` and `server_status()`.
  - Update `check_connection()`.
  - Add batch quote validation.
  - Add unit and wiremock tests.
- Modify: `src/cli/handlers/tdx_api_handler.rs`
  - Make `tdx-api health` print liveness and upstream TDX connection state.
- Test: `src/sources/tdx_api.rs`
  - Extend existing in-module tests.
- Test: existing CLI handler tests only if a health-output test already exists
  nearby during implementation. Do not create a broad CLI test harness in this
  slice.

## Task 1: Config Ownership And Runtime Mapping

**Files:**
- Modify: `src/core/config.rs`
- Modify: `src/sources/tdx_api.rs`
- Test: `src/sources/tdx_api.rs`

- [ ] **Step 1: Add failing config tests**

Append these tests inside `src/sources/tdx_api.rs`'s existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn default_config_includes_batch_and_health_limits() {
    let config = TdxApiConfig::default();

    assert_eq!(config.base_url, "http://tdx-api:8080");
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.health_timeout, Duration::from_secs(5));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.max_batch_quote_size, 50);
    assert!(config.enabled);
}

#[test]
fn runtime_config_from_app_config_maps_extended_fields() {
    let app_config = crate::core::config::TdxApiConfig {
        base_url: "http://127.0.0.1:8089".to_string(),
        timeout_secs: 11,
        max_retries: 2,
        enabled: false,
        max_batch_quote_size: 25,
        health_timeout_secs: 3,
    };

    let runtime = TdxApiConfig::from_app_config(&app_config);

    assert_eq!(runtime.base_url, "http://127.0.0.1:8089");
    assert_eq!(runtime.timeout, Duration::from_secs(11));
    assert_eq!(runtime.health_timeout, Duration::from_secs(3));
    assert_eq!(runtime.max_retries, 2);
    assert_eq!(runtime.max_batch_quote_size, 25);
    assert!(!runtime.enabled);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
```

Expected: FAIL because `health_timeout`, `max_batch_quote_size`, `enabled`, and
`TdxApiConfig::from_app_config` do not exist yet.

- [ ] **Step 3: Add app-config fields and defaults**

In `src/core/config.rs`, extend `TdxApiConfig`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct TdxApiConfig {
    #[serde(default = "default_tdx_api_url")]
    pub base_url: String,
    #[serde(default = "default_tdx_api_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_tdx_api_retries")]
    pub max_retries: u32,
    #[serde(default = "default_tdx_api_enabled")]
    pub enabled: bool,
    #[serde(default = "default_tdx_api_max_batch_quote_size")]
    pub max_batch_quote_size: usize,
    #[serde(default = "default_tdx_api_health_timeout")]
    pub health_timeout_secs: u64,
}
```

Add defaults near the existing `default_tdx_api_*` helpers:

```rust
fn default_tdx_api_enabled() -> bool {
    std::env::var("TDX_API_ENABLED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(true)
}

fn default_tdx_api_max_batch_quote_size() -> usize {
    std::env::var("TDX_API_MAX_BATCH_QUOTE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
}

fn default_tdx_api_health_timeout() -> u64 {
    std::env::var("TDX_API_HEALTH_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}
```

- [ ] **Step 4: Add runtime config fields and conversion**

In `src/sources/tdx_api.rs`, extend the runtime config:

```rust
pub struct TdxApiConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub enabled: bool,
    pub max_batch_quote_size: usize,
    pub health_timeout: Duration,
}
```

Update `Default`:

```rust
Self {
    base_url: DEFAULT_BASE_URL.to_string(),
    timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    max_retries: MAX_RETRIES,
    enabled: true,
    max_batch_quote_size: 50,
    health_timeout: Duration::from_secs(5),
}
```

Update `from_env()`:

```rust
let enabled = std::env::var("TDX_API_ENABLED")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(true);
let max_batch_quote_size = std::env::var("TDX_API_MAX_BATCH_QUOTE_SIZE")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(50);
let health_timeout_secs = std::env::var("TDX_API_HEALTH_TIMEOUT_SECS")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(5);

Self {
    base_url,
    timeout: Duration::from_secs(timeout_secs),
    max_retries: MAX_RETRIES,
    enabled,
    max_batch_quote_size,
    health_timeout: Duration::from_secs(health_timeout_secs),
}
```

Add a conversion helper:

```rust
pub fn from_app_config(cfg: &crate::core::config::TdxApiConfig) -> Self {
    Self {
        base_url: cfg.base_url.clone(),
        timeout: Duration::from_secs(cfg.timeout_secs),
        max_retries: cfg.max_retries,
        enabled: cfg.enabled,
        max_batch_quote_size: cfg.max_batch_quote_size,
        health_timeout: Duration::from_secs(cfg.health_timeout_secs),
    }
}
```

Update `TdxApiClient::from_app_config`:

```rust
pub fn from_app_config(cfg: &crate::core::config::TdxApiConfig) -> Result<Self> {
    let runtime = TdxApiConfig::from_app_config(cfg);
    if !runtime.enabled {
        return Err(QuantixError::Unsupported(
            "tdx-api source is disabled".to_string(),
        ));
    }
    Self::new(runtime)
}
```

- [ ] **Step 5: Run focused config tests and commit**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
```

Expected: both tests PASS.

Then commit:

```bash
git add src/core/config.rs src/sources/tdx_api.rs
git commit -m "feat: extend tdx-api runtime config"
```

## Task 2: Typed Health And Server Status

**Files:**
- Modify: `src/sources/tdx_api.rs`
- Modify: `src/cli/handlers/tdx_api_handler.rs`
- Test: `src/sources/tdx_api.rs`

- [ ] **Step 1: Add failing health/server-status tests**

Add imports inside `src/sources/tdx_api.rs`'s test module:

```rust
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
```

Add tests:

```rust
#[tokio::test]
async fn health_status_accepts_observed_status_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "time": "2026-06-05T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let client = TdxApiClient::new(TdxApiConfig {
        base_url: server.uri(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        enabled: true,
        max_batch_quote_size: 50,
        health_timeout: Duration::from_secs(1),
    })
    .unwrap();

    let health = client.health_status().await.unwrap();
    assert!(health.is_healthy());
}

#[tokio::test]
async fn server_status_reads_connected_flag() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/server-status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0,
            "message": "success",
            "data": {
                "status": "running",
                "connected": true,
                "version": "1.0.0",
                "uptime": "unknown"
            }
        })))
        .mount(&server)
        .await;

    let client = TdxApiClient::new(TdxApiConfig {
        base_url: server.uri(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        enabled: true,
        max_batch_quote_size: 50,
        health_timeout: Duration::from_secs(1),
    })
    .unwrap();

    let status = client.server_status().await.unwrap();
    assert!(status.connected);
    assert_eq!(status.status, "running");
    assert_eq!(status.version.as_deref(), Some("1.0.0"));
}
```

- [ ] **Step 2: Run health tests and confirm they fail**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
```

Expected: FAIL because `health_status()` and `server_status()` do not exist.

- [ ] **Step 3: Add typed response structs**

Add near the existing response structs in `src/sources/tdx_api.rs`:

```rust
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TdxApiHealthStatus {
    pub status: Option<String>,
    pub code: Option<i64>,
    pub message: Option<String>,
}

impl TdxApiHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.status.as_deref() == Some("healthy") || self.code == Some(0)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TdxApiServerStatus {
    pub status: String,
    pub connected: bool,
    pub version: Option<String>,
    pub uptime: Option<String>,
}
```

- [ ] **Step 4: Add client methods**

Add methods near the existing `health()` method:

```rust
pub async fn health_status(&self) -> Result<TdxApiHealthStatus> {
    let url = format!("{}/api/health", self.config.base_url);
    let resp = self
        .client
        .get(&url)
        .timeout(self.config.health_timeout)
        .send()
        .await
        .map_err(QuantixError::Http)?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(QuantixError::DataSource(format!(
            "tdx-api health HTTP {}: {}",
            status, body
        )));
    }

    let health: TdxApiHealthStatus = resp
        .json()
        .await
        .map_err(|e| QuantixError::DataParse(format!("tdx-api health 解析失败: {e}")))?;

    if !health.is_healthy() {
        return Err(QuantixError::DataSource(format!(
            "tdx-api health unhealthy: {:?}",
            health
        )));
    }

    Ok(health)
}

pub async fn server_status(&self) -> Result<TdxApiServerStatus> {
    self.get("/api/server-status").await
}
```

Update `check_connection()`:

```rust
async fn check_connection(&self) -> Result<()> {
    self.health_status().await?;
    let status = self.server_status().await?;
    if !status.connected {
        return Err(QuantixError::DataSource(
            "tdx-api server is running but not connected to TDX upstream".to_string(),
        ));
    }
    Ok(())
}
```

- [ ] **Step 5: Update CLI health output**

In `src/cli/handlers/tdx_api_handler.rs`, update the `TdxApiCommands::Health`
arm from raw JSON output to typed liveness plus upstream state:

```rust
TdxApiCommands::Health => {
    let c = client()?;
    let health = c.health_status().await?;
    let server = c.server_status().await?;
    println!(
        "tdx-api: healthy={} status={} connected={} version={}",
        health.is_healthy(),
        server.status,
        server.connected,
        server.version.as_deref().unwrap_or("unknown")
    );
}
```

- [ ] **Step 6: Run health tests and commit**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
```

Expected: both tests PASS.

Then commit:

```bash
git add src/sources/tdx_api.rs src/cli/handlers/tdx_api_handler.rs
git commit -m "feat: add tdx-api server status health check"
```

## Task 3: Batch Quote Size Validation

**Files:**
- Modify: `src/sources/tdx_api.rs`
- Test: `src/sources/tdx_api.rs`

- [ ] **Step 1: Add failing over-limit test**

Add this test inside `src/sources/tdx_api.rs`'s test module:

```rust
#[tokio::test]
async fn batch_quote_rejects_requests_over_configured_limit() {
    let client = TdxApiClient::new(TdxApiConfig {
        base_url: "http://127.0.0.1:1".to_string(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        enabled: true,
        max_batch_quote_size: 1,
        health_timeout: Duration::from_secs(1),
    })
    .unwrap();

    let err = client.batch_quote(&["000001", "600519"]).await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("tdx-api batch quote"));
    assert!(msg.contains("2"));
    assert!(msg.contains("1"));
}
```

- [ ] **Step 2: Run test and confirm it fails**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests::batch_quote_rejects_requests_over_configured_limit
```

Expected: FAIL because `batch_quote()` does not validate `max_batch_quote_size`.

- [ ] **Step 3: Implement validation**

In `batch_quote()`, after the empty input check and before symbol conversion,
add:

```rust
if codes.len() > self.config.max_batch_quote_size {
    return Err(QuantixError::DataSource(format!(
        "tdx-api batch quote requested {} codes, exceeding max_batch_quote_size {}",
        codes.len(),
        self.config.max_batch_quote_size
    )));
}
```

- [ ] **Step 4: Run batch validation test and commit**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests::batch_quote_rejects_requests_over_configured_limit
```

Expected: PASS.

Then commit:

```bash
git add src/sources/tdx_api.rs
git commit -m "feat: validate tdx-api batch quote size"
```

## Task 4: Focused Regression Suite And Live Smoke Gate

**Files:**
- Modify only if needed: `src/sources/tdx_api.rs`
- No required code changes if Tasks 1-3 pass.

- [ ] **Step 1: Run focused in-repo tests**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
```

Expected: all `sources::tdx_api::tests` pass.

- [ ] **Step 2: Run CLI parser tests for tdx-api commands**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::tests::data
```

Expected: all CLI data parser tests pass, including the existing tdx-api command
parsing tests. These verify existing command parsing stays intact.

- [ ] **Step 3: Run bridge mapping tests**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test bridge_tdx_source_test
```

Expected: all bridge TDX source mapping tests pass. These are not direct REST
tests, but they guard current TDX model mapping assumptions.

- [ ] **Step 4: Run optional live REST smoke only when explicitly enabled**

Use the already validated live endpoint only if the operator sets both
environment variables:

```bash
QUANTIX_TDX_API_LIVE=1 TDX_API_URL=http://192.168.123.104:8089 \
cargo run --manifest-path /opt/claude/quantix-rust/Cargo.toml -- data tdx-api health
```

Expected output includes:

```text
tdx-api: healthy=true status=running connected=true version=1.0.0
```

If `QUANTIX_TDX_API_LIVE` is not set, skip this live smoke and record it as not
run. Do not make normal CI depend on the NAS service.

- [ ] **Step 5: Run GitNexus change detection**

Run:

```bash
gitnexus_detect_changes(scope="all")
```

Expected:

```text
changed files are limited to src/core/config.rs, src/sources/tdx_api.rs,
and src/cli/handlers/tdx_api_handler.rs; no unexpected high-risk flow impact
```

- [ ] **Step 6: Commit any final verification-only adjustments**

If no code changes were needed in Task 4, do not create an empty commit.

If a small test or output adjustment was required, commit it:

```bash
git add src/sources/tdx_api.rs src/cli/handlers/tdx_api_handler.rs
git commit -m "test: cover tdx-api REST source health"
```

## Implementation Notes

- Run `gitnexus impact` before editing each function that will be modified:
  - `TdxApiClient::from_app_config`
  - `TdxApiClient::batch_quote`
  - `TdxApiClient::check_connection`
  - `run_tdx_api_command`
- Do not modify the bridge protocol in this slice.
- Do not add MCP dependencies.
- Do not commit NAS credentials or service credentials.
- Keep live smoke checks opt-in. The repository test suite must pass without
  network access to the NAS service.

## Final Verification

Before opening a PR or claiming completion, run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib sources::tdx_api::tests
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::tests::data
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test bridge_tdx_source_test
cargo fmt --check
cargo clippy --manifest-path /opt/claude/quantix-rust/Cargo.toml --all-targets -- -D warnings
```

Then run:

```bash
gitnexus_detect_changes(scope="all")
```

If a commit is merged, refresh the index with local `gitnexus analyze` and
restore generated AGENTS/CLAUDE/GitNexus skill drift if it appears.

## Execution Choice

Plan complete and saved to
`docs/superpowers/plans/2026-06-06-tdx-api-rest-source-slice1.md`.

Two execution options:

1. Subagent-Driven (recommended): dispatch a fresh subagent per task, review
   between tasks, fast iteration.
2. Inline Execution: execute tasks in this session using executing-plans, with
   checkpoints after each task.
