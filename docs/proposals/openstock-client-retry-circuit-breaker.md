# Proposal: OpenStockClient — Retry + Circuit Breaker

> Status: APPROVED — proceed to implementation (2026-07-01)
> Reviewer: CodeWhale (deepseek-v4-pro) — `docs/proposals/openstock-client-retry-circuit-breaker-review.md`
> Scope: `/opt/claude/quantix-rust/src/sources/openstock_client.rs`
> Slice: 方向 2 of "依次解决上述 1, 2, 3"（Task #89）
> Reference pattern: `src/sources/tdx_api.rs:513-558` (`request_with_retry`)
>
> **决定汇总（§9 + 修正）**：
> - P1 ✅ 移除所有 `.expect()`，改 `map_err(...)?`
> - P2 ✅ `OpenStockClient` 新增 `config` + `circuit` 字段
> - P3 ✅ `Default for OpenStockClientConfig` 补齐 4 个新字段
> - P4 ✅ d=2 描述改为"全部在 tests/openstock_live_*.rs，无生产 handler"，风险降为 LOWER
> - P5 ✅ Non-Goals 改为"不实现带并发控制的完整 half-open"
> - Q1 保留 breaker / Q2 max_retries=3 / Q3 threshold=5 / Q4 cooldown=30s / Q5 不 env 化 / Q6 max_retries=0 时 breaker 仍开
> - S1 retry=`tracing::warn!`，circuit trip=`tracing::error!`
> - S2/S3 在字段 doc comment 中说明
> - S4 `.expect()` 仅限 `#[cfg(test)]`

## 1. Background

`OpenStockClient::fetch`（`src/sources/openstock_client.rs:97-147`）目前是一次性 HTTP 调用：

- 无 retry — 任何 send error / 5xx 直接返回错误
- 无 circuit breaker — 服务端持续不可用时，CLI/服务会无脑打满每次请求
- runtime 实际是远程服务（`http://192.168.123.104:8040`），存在瞬态网络抖动 / runtime 重启 / baostock 后端瞬时失败的可能

tdx-api 侧（`src/sources/tdx_api.rs:513`）已有成熟的 `request_with_retry`：指数退避 + 5xx/网络错误重试 / 4xx 不重试 / 业务错误不重试。本 slice 把同样的语义移植到 `OpenStockClient::fetch`，并额外加一个轻量 circuit breaker。

## 2. Goals

- **G1**：瞬态错误（网络错误 / 5xx）自动重试，指数退避
- **G2**：确定性错误（4xx / 业务错误 / envelope 解析失败）fail-fast，不浪费 retry 预算
- **G3**：连续失败 N 次后短路后续请求一段时间（circuit breaker），避免对挂掉的服务端持续打
- **G4**：可观测 — 每次 retry 与每次熔断触发都用 `tracing::warn!` 记录（CLAUDE.md 强制 lib 用 tracing 不用 println）
- **G5**：可关闭 — `max_retries = 0` 关闭 retry；`circuit_break_threshold = 0` 关闭熔断
- **G6**：零新依赖 — 仅用 `tokio::time::sleep` + `std::sync::Mutex` + `std::time::Instant`
- **G7**：API 兼容 — 5 个 wrapper（`fetch_stock_codes/fetch_trade_dates/fetch_index_klines/fetch_all_stocks/fetch_workdays`）签名不变；`OpenStockResponse` 不变；`from_env()` / `new(cfg)` 向后兼容

## 3. Non-Goals

- 不引入 `tower` / `backoff` / `failsafe` 等专用 crate（保持依赖最小）
- 不实现带并发控制的完整 half-open 状态机（D3 实现的是简化版：冷却到期后第一个请求作为探测，不限制并发）
- 不把 circuit breaker 状态提升到共享 `Arc`（CLI 短任务无影响；常驻服务化再提炼）
- 不引入 metrics（已有 `tracing`，prometheus 集成是后续 slice）
- 不改动上层 handler / CLI 命令（对调用方完全透明）

## 4. Impact Assessment (Pre-Edit)

`gitnexus impact` 结果（`Function:src/sources/openstock_client.rs:OpenStockClient.fetch#2`, upstream）:

- **Risk**: MEDIUM
- **Direct callers (d=1)**: 5 — `fetch_stock_codes`, `fetch_trade_dates`, `fetch_index_klines`, `fetch_all_stocks`, `fetch_workdays`
- **Transitive (d=2)**: 5 — **全部在 `tests/openstock_live_*.rs`**（`fetch_stock_codes_live` / `fetch_trade_dates_live` / `fetch_index_klines_live` / `fetch_all_stocks_live` / `fetch_workdays_today_live`），无生产 handler / CLI 命令经过 d=2 链
- **Affected processes**: 0（无注册的 execution flow 经过 fetch）
- **Modules affected**: 1（Sources）

**判断**：所有 d=1 caller 都是 thin pass-through（构造 params → 调 fetch → 包 `OpenStockResponse`），retry 状态机封装在 `fetch` 内部对它们透明；签名与返回类型不变。d=2 全部是测试，无生产代码受影响。MEDIUM 仅因直接 caller ≥ 5 触发阈值，**实际语义风险为 LOWER（仅影响测试，无生产路径）**。

## 5. Design Decisions

### D1 — 重试边界（哪些 retry，哪些 fail-fast）

| 条件 | Retry? | 理由 |
|---|---|---|
| reqwest send error（连接拒绝 / DNS / 超时） | ✅ | 瞬态 |
| HTTP 5xx | ✅ | 服务端瞬态 |
| HTTP 4xx | ❌ | 客户端错误，重试无用（参考 tdx_api.rs:532） |
| HTTP 2xx 但 envelope JSON 解析失败 | ❌ | 响应损坏，重试未必解决；fail-fast |
| HTTP 2xx + 业务层 error envelope（runtime 返回 200 + error 结构） | ❌ | 确定性错误（在 `fetch` 当前实现里走的是非 2xx 分支，但即便 runtime 把错误塞 2xx body 也不重试） |

### D2 — 配置字段（`OpenStockClientConfig` 扩展）

```rust
#[derive(Debug, Clone)]
pub struct OpenStockClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,

    /// 最大重试次数（不含首次）。
    ///
    /// 0 = 关闭 retry（仅尝试一次）。
    ///
    /// **注意**：`max_retries=0` 时 `circuit_break_threshold` 仍生效 ——
    /// "不重试" 不等于 "服务健康"，单次失败仍会被 circuit breaker 跟踪
    /// 避免对挂掉的 runtime 持续打。若想完全关闭保护，设
    /// `circuit_break_threshold = 0`。
    pub max_retries: u32,

    /// 指数退避基数。第 N 次重试等待 `base * 2^(N-1)`。
    pub retry_base_delay: Duration,

    /// 连续失败多少次后触发 circuit breaker。
    ///
    /// 0 = 关闭熔断（每次请求都跑完整 retry 循环）。
    ///
    /// **注意**：circuit breaker 在所有 category 之间**共享**。任一
    /// category 持续失败会阻塞全部 category 的请求。这是设计选择
    /// （runtime 是单实例，单点故障时全部 category 都不可用）。
    pub circuit_break_threshold: u32,

    /// 熔断后的冷却时间。冷却期内请求直接短路。
    pub circuit_break_cooldown: Duration,
}

impl Default for OpenStockClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_key: String::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(500),
            circuit_break_threshold: 5,
            circuit_break_cooldown: Duration::from_secs(30),
        }
    }
}
```

`Default` 实现保证 `from_env()` 行为不变（向后兼容 G7）。

### D3 — Circuit Breaker 状态机

```rust
#[derive(Debug, Default)]
struct CircuitState {
    consecutive_failures: u32,
    tripped_until: Option<Instant>,
}
```

**状态转移**：

```
                ┌──────────────────────────┐
                │ Closed (正常)             │
                │ consecutive_failures < T │
                └──┬───────────────────────┘
                   │ send error / 5xx
                   │ consecutive_failures += 1
                   ▼
                ┌──────────────────────────┐
                │ threshold reached        │
                │ → tripped_until = now+C  │
                ▼                          │
                ┌──────────────────────────┐
                │ Open (短路)              │
                │ now < tripped_until      │
                │ → 直接返回 CircuitOpen   │
                └──┬───────────────────────┘
                   │ now ≥ tripped_until
                   │ （冷却到期）
                   ▼
                ┌──────────────────────────┐
                │ Half-Open (探测)          │
                │ 放行一次请求：           │
                │   - 成功 → reset → Closed│
                │   - 失败 → trip again    │
                └──────────────────────────┘
```

> **说明**：这是简化版 half-open — 冷却到期后**第一个请求**作为探测，不限制并发（CLI 单进程，无并发问题；服务化场景需提炼到 `Arc<Mutex>` + permit，本 slice 不做）。

### D3.5 — `OpenStockClient` 结构体字段（修正 P2）

当前结构体只有 3 个字段；本 slice 新增 2 个：

```rust
pub struct OpenStockClient {
    base_url: Url,
    api_key: HeaderValue,
    http: reqwest::Client,
    config: OpenStockClientConfig,   // 新增：存储 retry/breaker 配置（new() 中保留一份）
    circuit: Mutex<CircuitState>,    // 新增：熔断器状态
}
```

`new(cfg)` 需要把 `cfg` clone 一份存入 `self.config`（原 `cfg` 中的 `base_url`/`api_key` 仍按原逻辑解析）。

### D4 — Mutex 选型

用 `std::sync::Mutex<CircuitState>`，不用 `tokio::sync::Mutex`。理由：

- 临界区仅 2 个字段读写（`consecutive_failures`, `tripped_until`），无 `await`
- std Mutex 在短临界区性能更好，且不会跨 `await` 持锁（否则编译错误）
- 不引入 `parking_lot`（保持依赖最小，Cargo.toml 已确认无）
- **Poison 处理**：按 CLAUDE.md L63 禁止 `.expect()`，所有 `.lock()` 都用 `map_err(|e| QuantixError::Other(format!("circuit mutex poisoned: {}", e)))?` —— poison 表示另一线程 panic 留下不一致状态，返回 `Err` 让上层决定，不 fail-loud panic。

### D5 — `fetch` 重写骨架

```rust
pub async fn fetch<T: DeserializeOwned>(
    &self,
    category: &str,
    params: Value,
) -> Result<OpenStockResponse<T>> {
    // 1. Circuit breaker check (read-only)
    {
        let guard = self.circuit.lock().map_err(|e| {
            QuantixError::Other(format!("circuit mutex poisoned: {}", e))
        })?;
        if let Some(until) = guard.tripped_until {
            if Instant::now() < until {
                return Err(QuantixError::Other(format!(
                    "openstock circuit breaker open until {:?} (category={})",
                    until, category
                )));
            }
        }
    }

    let endpoint = self.base_url.join("data/fetch")
        .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;
    let body = serde_json::json!({"data_category": category, "params": params});

    let mut last_err: Option<QuantixError> = None;
    for attempt in 0..=self.config.max_retries {
        if attempt > 0 {
            let delay = self.config.retry_base_delay * 2u32.pow(attempt - 1);
            tracing::warn!(
                attempt, max_retries = self.config.max_retries, category,
                delay_ms = delay.as_millis(), "openstock retry"
            );
            tokio::time::sleep(delay).await;
        }

        let send_result = self.http.post(endpoint.clone())
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send().await;

        match send_result {
            Err(e) => {
                last_err = Some(QuantixError::Network(format!("openstock request failed: {}", e)));
                continue; // retry
            }
            Ok(resp) => {
                let status = resp.status();
                let raw = resp.text().await.map_err(|e| {
                    QuantixError::Network(format!("openstock body read failed: {}", e))
                })?;

                if !status.is_success() {
                    let summary = match serde_json::from_str::<OpenStockErrorEnvelope>(&raw) {
                        Ok(env) => env.to_summary(),
                        Err(_) => format!(
                            "openstock: HTTP {} | body: {}",
                            status, raw.chars().take(200).collect::<String>()
                        ),
                    };
                    if status.is_server_error() {
                        last_err = Some(QuantixError::Other(summary));
                        continue; // retry 5xx
                    }
                    // 4xx: fail-fast, don't retry, don't trip circuit (client bug)
                    return Err(QuantixError::Other(summary));
                }

                // 2xx — try parse
                match serde_json::from_str::<OpenStockEnvelope<T>>(&raw) {
                    Ok(env) => {
                        self.reset_circuit()?;
                        return Ok(OpenStockResponse::from_envelope(env, &raw));
                    }
                    Err(e) => {
                        // fail-fast, do NOT retry (corrupted response)
                        return Err(QuantixError::Other(format!(
                            "openstock: cannot parse success envelope: {} | body: {}",
                            e, raw.chars().take(200).collect::<String>()
                        )));
                    }
                }
            }
        }
    }

    // All retries exhausted — record circuit failure
    self.record_circuit_failure(category)?;
    Err(last_err.unwrap_or_else(|| QuantixError::Other("openstock retry exhausted".to_string())))
}

fn reset_circuit(&self) -> Result<()> {
    let mut guard = self.circuit.lock().map_err(|e| {
        QuantixError::Other(format!("circuit mutex poisoned: {}", e))
    })?;
    guard.consecutive_failures = 0;
    guard.tripped_until = None;
    Ok(())
}

fn record_circuit_failure(&self, category: &str) -> Result<()> {
    if self.config.circuit_break_threshold == 0 {
        return Ok(()); // breaker disabled
    }
    let mut guard = self.circuit.lock().map_err(|e| {
        QuantixError::Other(format!("circuit mutex poisoned: {}", e))
    })?;
    guard.consecutive_failures += 1;
    if guard.consecutive_failures >= self.config.circuit_break_threshold {
        let cooldown = self.config.circuit_break_cooldown;
        guard.tripped_until = Some(Instant::now() + cooldown);
        tracing::error!(
            consecutive_failures = guard.consecutive_failures,
            threshold = self.config.circuit_break_threshold,
            cooldown_ms = cooldown.as_millis(),
            category, "openstock circuit breaker tripped"
        );
    }
    Ok(())
}
```

**关键点**：

- **Circuit 检查 vs. 状态更新分离** — 进入 fetch 时只读 `tripped_until`；重试耗尽时才 `record_circuit_failure`；成功时 `reset_circuit`。锁不跨 `await`。
- **Mutex poison 处理**：所有 `.lock()` 都用 `map_err(...)?` 返回 `QuantixError::Other` —— poison 表示另一线程 panic 留下不一致状态，返回 `Err` 让上层决定。**不使用 `.expect()`**（CLAUDE.md L63 强制）。
- **日志级别**：retry 用 `tracing::warn!`（瞬态，可观测）；circuit trip 用 `tracing::error!`（更醒目，需运维介入）。
- **`last_err.unwrap_or_else(...)`**：`Option::unwrap_or_else` 不在 CLAUDE.md 禁用列表（禁的是 `.unwrap()` / `.expect()`），合法。

### D6 — 边界：`max_retries = 0` 仍受 circuit breaker 保护

- `max_retries = 0` → 循环只跑 1 次（attempt=0），不重试
- 但失败仍会 `record_circuit_failure` — 避免单次失败就放行后续洪水
- 若想完全关闭保护，设 `circuit_break_threshold = 0`

## 6. Test Plan

`src/sources/openstock_client.rs` `#[cfg(test)] mod tests` 现有 3 个单元测试（`from_envelope_*`）保留不动。新增以下 `#[tokio::test]` 集成测试，使用现有 `wiremock = "0.6"` dev-dependency：

### 新增测试

| 名字 | 场景 | 断言 |
|---|---|---|
| `fetch_retries_on_5xx_then_succeeds` | mock 前两次返回 503，第三次返回 2xx envelope | `fetch` 返回 `Ok`；mock 收到 3 次请求 |
| `fetch_does_not_retry_on_4xx` | mock 返回 400 | `fetch` 返回 `Err`；mock 只收到 1 次请求 |
| `fetch_does_not_retry_on_corrupt_2xx` | mock 返回 2xx 但 body 不是合法 envelope JSON | `fetch` 返回 `Err`；mock 只收到 1 次请求 |
| `fetch_retries_on_network_error` | mock `Mock::is_failure` 模拟连接错误 | 重试 `max_retries` 次；耗尽后返回 `Err` |
| `circuit_breaker_trips_after_threshold` | 配置 `threshold=2, cooldown=50ms`；mock 持续返回 500 | 前 2 次 retry 耗尽 → 第 3 次 fetch 立即返回 circuit-open 错误；mock 只在前 2 轮被命中 |
| `circuit_breaker_resets_after_cooldown` | 同上配置；首次失败触发熔断；等待 60ms 后 mock 返回 2xx | 熔断到期后请求放行；成功后 `consecutive_failures = 0`（用 test-only accessor 验证） |
| `circuit_breaker_disabled_when_threshold_zero` | `threshold=0`；mock 持续 500 | 不短路，每个 fetch 都跑完 retry 循环 |
| `retry_resets_circuit_on_success` | 前 2 次 503 → 第 3 次 2xx；再发一次 2xx | 第 3 次成功后 circuit state 重置 |

### 测试辅助

```rust
fn make_test_client(server: &wiremock::MockServer) -> OpenStockClient {
    OpenStockClient::new(OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".into(),
        timeout: Duration::from_secs(1),
        max_retries: 2,
        retry_base_delay: Duration::from_millis(5), // 加速测试
        circuit_break_threshold: 5,
        circuit_break_cooldown: Duration::from_millis(50),
    }).expect("client build")
}
```

> 注：`expect("client build")` 在测试代码中允许（CLAUDE.md 限制针对 production code）。

### 现有测试不变

`from_envelope_records_source_and_artifact_hash` / `from_envelope_defaults_missing_source` / `from_envelope_artifact_hash_stable_for_same_body` — 3 个测试只测 `OpenStockResponse::from_envelope`，与 retry 无关，保持原样。

## 7. Verification Plan

```bash
# Quality gates
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli openstock
cargo test --test openstock_client

# Regression
cargo test --workspace

# Impact verification（pre-commit）
gitnexus detect_changes   # 期望：LOW，仅 openstock_client.rs + 不破坏 callers
git diff --check
```

## 8. Risks

- **R1 — MEDIUM impact 但语义 LOW**：5 个直接 caller，但全是 thin pass-through；签名不变。**Mitigation**：保持 wrapper 不动；CI 跑 `cargo test --workspace` 验证回归。
- **R2 — Mutex poison 行为**：`std::sync::Mutex::lock().expect()` 在 panic 后会 poison。**Mitigation**：fail-loud 是预期；若用户反对，可改为 `map_err` + 返回 `QuantixError`。
- **R3 — Half-open 不严格**：冷却到期后多个并发请求都会被放行（CLI 单进程无影响；服务化场景需重构）。**Mitigation**：文档标注；本 slice 不解决。
- **R4 — retry 隐藏了真实错误率**：用户看到的是耗尽后的最终错误，可能不知道中间 retry 了几次。**Mitigation**：`tracing::warn!` 每次重试都记录 attempt/delay/category；用 `RUST_LOG=quantix_cli::sources::openstock_client=warn` 可观测。
- **R5 — 测试耗时**：retry 测试默认会 sleep。**Mitigation**：测试 client 用 5ms base delay + 50ms cooldown，整套测试增加 < 1s。

## 9. Open Questions (RESOLVED 2026-07-01)

采纳 CodeWhale 审核建议（`docs/proposals/openstock-client-retry-circuit-breaker-review.md`）：

| Q | 决定 | 理由 |
|---|---|---|
| Q1 breaker 保留 | ✅ **保留** | 内网服务也可能重启/挂掉；breaker 增量小（CircuitState + 2 helper） |
| Q2 `max_retries` | ✅ **3** | 与 tdx-api 一致；内网延迟低，3 次覆盖 99% 瞬态 |
| Q3 `circuit_break_threshold` | ✅ **5** | 5 次连续失败 ≈ 27s 持续故障后熔断，合理 |
| Q4 `circuit_break_cooldown` | ✅ **30s** | runtime 重启通常 < 30s |
| Q5 env 化 | ❌ **本 slice 不做** | 配置层变更，与 retry/breaker 语义正交 |
| Q6 `max_retries=0` 时 breaker | ✅ **仍开** | "不重试" ≠ "服务健康"；在 D2 字段 doc 中说明 |

## 10. Files Touched

| 文件 | 变更 |
|---|---|
| `src/sources/openstock_client.rs` | 重写 `fetch`；扩展 `OpenStockClientConfig`；新增 `CircuitState` + 两个 helper 方法；新增 8 个测试 |
| 其他 | **无**（5 个 wrapper、handler、CLI 命令均不动） |

---

**Status**: 等待审核。请针对 §9 的 6 个开放问题给出决定（或确认按 default 走），确认后我开始实现。
