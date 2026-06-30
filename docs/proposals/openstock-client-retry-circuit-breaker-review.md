# Review: openstock-client-retry-circuit-breaker.md

> 审核日期：2026-07-01
> 审核人：CodeWhale（deepseek-v4-pro）
> 审核方法：全文通读提案（345 行）+ 对照现有代码 + GitNexus impact 验证 + CLAUDE.md 政策核查

---

## 总体评价：方向正确，设计严谨，可继续推进

提案结构清晰、范围受控。Goals / Non-Goals / Risks / Open Questions 分层明确。Impact Assessment 引用 GitNexus，设计决策每项有原因有对比。以下为逐项审核发现。

---

## 1. GitNexus Impact 验证

使用 `mcp_gitnexus_impact` 对 `Function:src/sources/openstock_client.rs:OpenStockClient.fetch#2` 进行上游影响分析，结果与提案一致：

| 维度 | 提案声明 | GitNexus 实际 |
|------|---------|---------------|
| Risk | MEDIUM | MEDIUM（`direct_callers: 5` 触发阈值） |
| d=1 callers | 5 个 wrapper | 5 — 同一文件，精确匹配 |
| d=2 callers | 5（"handler / test"） | 5 — **全部在 `tests/openstock_live_*.rs`**，非 handler |
| Affected processes | 0 | 0 |
| Modules | Sources | Sources（hits: 10） |

**发现**：提案 §4 的 "（5 个 wrapper 的下游 callers — handler / test）" 不精确。实际 d=2 的 5 个调用者全是测试文件中的 live test 函数（`fetch_all_stocks_live`、`fetch_trade_dates_live`、`fetch_stock_codes_live`、`fetch_index_klines_live`、`fetch_workdays_today_live`）。没有任何生产 handler 或 CLI 命令经过 d=2 链。**语义风险比 LOW 更低。**

---

## 2. CLAUDE.md 政策核查

CLAUDE.md L63 明文规定：

> **FORBIDDEN**: `.unwrap()`, `.expect()`, `panic!()` in production code

L115 显示之前 715 个 `.unwrap()` 已被清零。**不存在例外**。

---

## 3. 必须修正的问题

### P1（阻塞）— D5 骨架使用 `.expect()` 违反 CLAUDE.md

**位置**：§5 D5 骨架 L144, L226, L236

```rust
let guard = self.circuit.lock().expect("circuit mutex poisoned");
```

**问题**：CLAUDE.md L63 明确禁止生产代码中任何 `.unwrap()` / `.expect()` / `panic!()`。提案在 L250-253 试图论证 mutex poison 应 fail-loud 是 "例外"，然后在 L255 又说会用 `map_err` 替换——4 行内自相矛盾。

**修正**：
- 删除 L250-253 的 "例外" 论证
- 骨架中所有 `.expect(...)` 改为：

```rust
let guard = self.circuit.lock().map_err(|e| {
    QuantixError::Other(format!("circuit mutex poisoned: {}", e))
})?;
```

### P2（高）— 未明确 `OpenStockClient` 结构体变更

**问题**：提案 D5 骨架引用 `self.config.max_retries`、`self.circuit.lock()`，但当前结构体（L42-46）只有 3 个字段：

```rust
pub struct OpenStockClient {
    base_url: Url,
    api_key: HeaderValue,
    http: reqwest::Client,
}
```

**修正**：补充结构体定义，明确新增字段：

```rust
pub struct OpenStockClient {
    base_url: Url,
    api_key: HeaderValue,
    http: reqwest::Client,
    config: OpenStockClientConfig,       // 新增：存储 retry/breaker 配置
    circuit: Mutex<CircuitState>,         // 新增：熔断器状态
}
```

同步说明 `new()` 构造函数的修改（当前 `new(cfg)` 消费 `cfg`，需保留一份）。

### P3（高）— `OpenStockClientConfig` 新字段无 `Default` impl

**问题**：提案 D2 列出了 4 个新字段及其默认值，但当前 `Default` impl（L30-38）只处理 3 个字段。G7 声称 `from_env()` 向后兼容——需要 `Default` 给新字段赋合理默认值才能成立。

**修正**：补充完整的 `Default` impl：

```rust
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

### P4（中）— d=2 callers 描述需修正

**位置**：§4 Impact Assessment L42

> **Transitive (d=2)**: 5（5 个 wrapper 的下游 callers — handler / test）

**实际**：5 个全是 `tests/openstock_live_*.rs` 中的 live test。

**修正**：改为：

> **Transitive (d=2)**: 5 — 全部在 `tests/openstock_live_*.rs`（live test 函数）。无生产 handler。

同步将风险评估从 "实际语义风险为 LOW" → "实际语义风险为 LOWER（仅影响测试）"。

### P5（低）— Non-Goals 与 D3 的 half-open 措辞矛盾

**位置**：§3 Non-Goals L31 vs §5 D3 L125-126

L31 写 "不实现 half-open 状态机"，但 D3 L125-126 说明的正是简化版 half-open（冷却到期后第一个请求作为探测）。

**修正**：Non-Goals 改为：

> 不实现带并发控制的完整 half-open 状态机（D3 实现的是简化版：冷却到期后第一个请求探测，不限制并发）

---

## 4. 小建议

| # | 位置 | 建议 |
|---|------|------|
| S1 | §2 G4 | `tracing::warn!` 每次重试都记录——高频场景日志量可能大。考虑 `tracing::info!` 用于 retry 事件，`tracing::warn!` 保留给 circuit trip |
| S2 | §5 D6 | `max_retries=0` 仍受 breaker 保护——行为反直觉，建议在 `OpenStockClientConfig` 的字段文档注释中明确说明 |
| S3 | §5 D3 | 当前设计所有 category 共享一个 circuit breaker。一个 category 的连续故障会阻塞所有 category 的请求。这是设计选择但需文档说明 |
| S4 | §6 | 测试辅助函数中 `expect("client build")` 在测试代码允许，但建议与 CLAUDE.md 一致的写法（`.expect("...")` 的最低容忍范围只在 `#[cfg(test)]` 内） |

---

## 5. 对 6 个开放问题（§9）的建议

| Q | 问题 | 建议 |
|----|------|------|
| Q1 | Circuit breaker 是否需要？ | **保留**。内网服务也可能重启/挂掉，breaker 避免对挂掉的服务无脑打。改动量增量小（CircuitState + 2 个 helper 方法） |
| Q2 | Default `max_retries` | **保持 3**。与 tdx-api 一致，内网延迟低，3 次重试覆盖 99% 瞬态。 |
| Q3 | Default `circuit_break_threshold` | **保持 5**。连续 5 次失败 = 至少 5×500ms + 4×1s + 3×2s + 2×4s + 1×8s ≈ 27s 持续失败后熔断，合理。 |
| Q4 | Default `circuit_break_cooldown` | **保持 30s**。如果 runtime 重启通常 < 30s 就够；若你觉得 30s 太保守可改为 15s。 |
| Q5 | 是否通过 env var 暴露配置 | **本 slice 不做**。env var 化属于配置层变更，与本 slice 无关。后续可加 `OPENSTOCK_MAX_RETRIES` 等。 |
| Q6 | `max_retries=0` 时 breaker 是否也关闭 | **保持当前设计**（retry 关，breaker 仍开）。因为 "不重试" 不代表 "服务健康"——单次失败仍应被 breaker 跟踪防止洪水。但需在 `OpenStockClientConfig` 字段文档中说明此行为。 |

---

## 6. 验证计划确认

提案 §7 的验证步骤（`cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test --workspace` / `gitnexus detect_changes`）覆盖完整。补充两点：

- 测试 suite 中的 `openstock_live_*.rs`（d=2 callers）回归验证——已在 `cargo test --workspace` 覆盖
- `tdx_api.rs` 的 `request_with_retry` 不受影响——确认通过

---

## 审核结论

| 判定 | 说明 |
|------|------|
| **是否可推进** | ✅ 是 |
| **阻塞项** | P1（移除 `.expect`） |
| **高优先级** | P2（结构体字段）、P3（Default impl） |
| **中/低优先级** | P4（d=2 描述）、P5（措辞统一） |
| **建议** | 逐条回答 §9 的 6 个开放问题后再动手写代码 |
| **改完后** | 可直接进入实现，无其他阻塞 |

**修正后的推进顺序**：
1. 用户对 §9 的 6 个问题给出决策（或确认按建议）
2. 修正 P1-P5
3. 实现：重写 `fetch`、扩展 `OpenStockClientConfig`、新增 `CircuitState`、补 8 个测试
4. 运行 §7 验证计划 → `gitnexus detect_changes` → 提交
