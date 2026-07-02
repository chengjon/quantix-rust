# P0.13b-2 设计：OpenStock MINUTE_DATA 时间共享点消费

> 切片：P0.13b 的第二个子切片（P0.13b-1 已合并于 `8a1a8f2`）
> 父设计：`docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md`（覆盖双子切片的全局决策）
> 范围：通过 `/data/fetch` envelope 路径消费 `MINUTE_DATA` category（分时点序列）
> 实现方式：子代理驱动开发（与 P0.13b-1 一致）

---

## 1. 背景

P0.13b-1 已交付分钟蜡烛（KLINES minute periods，走 `/data/bars` 直 reqwest 路径）。本切片 P0.13b-2 是兄弟切片——消费 OpenStock 的 `MINUTE_DATA` category，即「分时点序列」（intraday time-share tick sequence），与分钟蜡烛在语义上完全不同：

- 分钟蜡烛：OHLC + volume（开盘/最高/最低/收盘 + 成交量）
- 分时点：单一 `price` + `avg_price`（逐分钟成交均价，用于绘制分时图）

**重要澄清**（父设计 §2.2 已确立）：`MINUTE_DATA` 是分时点序列，**不是分钟 K 线**。两者在 HANDOFF 报告行 35/38 已正确标注区分。本切片实现的是行 38 的 P2 分时图路径。

## 2. 目标与非目标

### 目标

- 新增 `MinuteShare` 模型，承载分时点序列
- 新增 `fetch_minute_share(code, date)` client 方法，走 `/data/fetch` envelope
- 新增 `parse_minute_share` 解析器：8 字段裁剪到 5 业务字段，关键字段缺失 warn+skip（INV-2C）
- 新增 CLI 子命令 `data openstock fetch-minute-share`
- 新增 `#[ignore]` 实时测试 3 个（覆盖 T6/T7/T8）
- 新增 OpenSpec change `openstock-data-consumption-p0-13b-2`
- 新增 governance card `P0.13b-2.yaml`

### 非目标

- 多日范围查询（推迟到 P0.13c）
- ClickHouse 写入 / shadow persistence 接入
- 其他 category（REALTIME_QUOTES 等）
- 重构 envelope 路径或 retry/circuit breaker 逻辑
- 迁移已有 parsers 到独立模块（跨切片重构，单独处理）
- 修改 P0.13b-1 已交付代码（`MinuteBar` / `fetch_minute_klines`）

## 3. 架构

### 3.1 数据流

```
CLI: data openstock fetch-minute-share --symbol sh600000 --date 2026-06-30
   ↓
Handler: fetch_openstock_minute_share(settings, symbol, date)
   ↓  OpenStockClient::from_settings(settings)?
OpenStockClient::fetch_minute_share(code, date)
   ↓  POST /data/fetch { category: "MINUTE_DATA", code, date }
   ↓  envelope + retry + circuit breaker（与 fetch_stock_codes 同路径）
OpenStockEnvelope<RawMinuteRecord>（response.records 是 8 字段数组）
   ↓  parse_minute_share(code, &raw) -> Option<MinuteShare>
Vec<MinuteShare>（5 业务字段 + code）
   ↓
Handler 输出（与 fetch-minute-klines 同形状，无 Period/Adjust 行）
```

### 3.2 与 P0.13b-1 的对比

| 维度 | P0.13b-1（已合并） | P0.13b-2（本切片） |
|------|------------------|------------------|
| 业务语义 | 分钟蜡烛 OHLC | 分时点序列 |
| OpenStock category | `KLINES`（minute periods） | `MINUTE_DATA` |
| HTTP 端点 | `/data/bars` | `/data/fetch` envelope |
| Retry/Circuit breaker | ❌ 直 reqwest，无 retry | ✅ 复用现有 envelope 路径 |
| 请求参数 | symbol + period + date + adjust | symbol + date（无 period/adjust） |
| 返回类型 | `Vec<MinuteBar>`（OHLC + volume） | `Vec<MinuteShare>`（price + avg_price） |
| 字段精度 | OHLC + volume（i64） | price/amount/avg_price（Decimal）+ volume（i64） |
| INV-2C skip 语义 | 不适用（bar 缺字段直接报错） | 适用（Option 字段 + parser skip） |

## 4. 组件设计

### 4.1 `MinuteShare` 结构体（`src/data/models.rs`）

```rust
/// 分时点序列（P0.13b-2 新增）。
///
/// 对应 OpenStock `MINUTE_DATA` category。与 `MinuteBar` 区别：
/// - 无 OHLC（仅单一 `price`）
/// - 含 `avg_price`（均价，业务关键字段）
///
/// **Option 字段说明**：业务字段全部用 `Option` 包裹以支持 INV-2C
/// （单条记录字段缺失时 warn + skip，不中断整批）。serde 反序列化
/// 在 Option 字段缺失时返回 None 而非失败；parser 阶段检查关键字段
/// （price/volume/amount/avg_price），任一为 None 则 warn + skip。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteShare {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub price: Option<Decimal>,
    pub volume: Option<i64>,
    pub amount: Option<Decimal>,
    pub avg_price: Option<Decimal>,
}
```

**字段精度决策**：`price/amount/avg_price` 使用 `Decimal`（与 `Kline`/`MinuteBar` 一致）。eLtdx 源的 float 字段 serde 兼容（自动反序列化为 Decimal），无精度损失。

### 4.2 `RawMinuteRecord` 中间类型（`src/sources/openstock_client.rs`）

```rust
/// MINUTE_DATA 原始记录（8 字段，未裁剪）。
///
/// OpenStock envelope `records` 数组元素的反序列化目标。
/// 字段名对应 eLtdx MINUTE_DATA 输出：
///   - time_minutes: "0930" 或 "09:30" 格式
///   - price/volume/amount/avg_price: 业务字段（保留）
///   - index/time/price_milli: 冗余字段（丢弃）
#[derive(Debug, Deserialize)]
struct RawMinuteRecord {
    time_minutes: String,
    price: Option<f64>,
    volume: Option<i64>,
    amount: Option<f64>,
    avg_price: Option<f64>,
    // 以下字段解析后丢弃，不放入 struct；用 `#[serde(default)]` 容忍缺失
}
```

实际实现可用 `serde_json::Value` + 手动字段提取，或定义带 `#[serde(default)]` 的 struct。决策点见 §6 D2。

### 4.3 `fetch_minute_share` 方法（`src/sources/openstock_client.rs`）

```rust
/// 消费 MINUTE_DATA category（分时点序列）。
///
/// 走 `/data/fetch` envelope 路径，复用 retry + circuit breaker。
/// 与 `fetch_stock_codes` / `fetch_trade_dates` 同路径。
///
/// **category 无 period/adjust 维度**——请求体仅 `{category, code, date}`。
///
/// 解析：response.records 是 8 字段的数组，parse_minute_share 裁剪到 5 业务字段。
/// 单条记录关键字段缺失 → warn + skip（INV-2C）。
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date: NaiveDate,
) -> Result<Vec<MinuteShare>> {
    let body = serde_json::json!({
        "category": "MINUTE_DATA",
        "code": code,
        "date": date.format("%Y-%m-%d").to_string(),
    });
    let envelope = self.fetch::<RawMinuteRecord>(body).await?;
    let records = envelope.records;
    let mut out = Vec::with_capacity(records.len());
    for raw in records {
        if let Some(share) = parse_minute_share(code, &raw) {
            out.push(share);
        } else {
            tracing::warn!(
                code = code,
                date = %date,
                time_minutes = %raw.time_minutes,
                "MINUTE_DATA record missing required field, skipping"
            );
        }
    }
    Ok(out)
}
```

### 4.4 `parse_minute_share` 解析器（内联 `openstock_client.rs` 私有 helper）

```rust
/// 解析 MINUTE_DATA 单条记录为 `MinuteShare`。
///
/// 丢弃字段：`index`（内部序号）、`time`（ISO 冗余）、`price_milli`（毫表示）。
/// 保留字段：`time_minutes, price, volume, amount, avg_price`。
///
/// 返回 `Option<MinuteShare>`：当 4 个关键字段（price/volume/amount/avg_price）
/// 任一为 None 时返回 None，调用方 warn + skip（INV-2C）。
///
/// **timestamp 解析**：`time_minutes` 可能是 "0930" 或 "09:30" 格式；
/// 与传入 `date` 组装为 `NaiveDateTime`（HH:MM:SS）。
fn parse_minute_share(code: &str, raw: &RawMinuteRecord, date: NaiveDate) -> Option<MinuteShare> {
    let price = raw.price?;
    let volume = raw.volume?;
    let amount = raw.amount?;
    let avg_price = raw.avg_price?;

    // time_minutes 可能是 "0930" 或 "09:30"；归一化为 (HH, MM)
    let (hh, mm) = parse_time_minutes(&raw.time_minutes)?;
    let timestamp = date.and_hms_opt(hh, mm, 0)?;

    Some(MinuteShare {
        code: code.to_string(),
        timestamp,
        price: Some(Decimal::from_f64_retain(price)?),
        volume: Some(volume),
        amount: Some(Decimal::from_f64_retain(amount)?),
        avg_price: Some(Decimal::from_f64_retain(avg_price)?),
    })
}
```

**时间格式决策**：`time_minutes` 的实际格式未知（R2 风险）。wiremock 测试先行探路两种格式（"0930" 和 "09:30"），live 测试验证生产格式。若两种都不匹配，parse_time_minutes 返回 None → 整条 skip。

### 4.5 CLI 子命令 + Handler

```rust
// src/cli/commands/data.rs（追加到 OpenStockCommands enum）
FetchMinuteShare {
    #[arg(long)] symbol: String,
    #[arg(long)] date: String,
}

// src/cli/handlers/openstock_handler.rs
pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockSettings,
    symbol: String,
    date: String,
) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| QuantixError::Other(format!("invalid date '{}': {}", date, e)))?;
    let started = std::time::Instant::now();
    let shares = client.fetch_minute_share(&symbol, date).await?;
    let latency_ms = started.elapsed().as_millis();

    println!("OpenStock MINUTE_DATA (time-share ticks)");
    println!("  Code:     {}", symbol);
    println!("  Date:     {}", date);
    println!("  Endpoint: {}/data/fetch", settings.base_url);
    println!("  Records:  {}", shares.len());
    if let (Some(first), Some(last)) = (shares.first(), shares.last()) {
        println!("  First:    {:?}", first);
        println!("  Last:     {:?}", last);
    }
    println!("  latency_ms: {}", latency_ms);
    Ok(())
}
```

Handler 输出形状与 `fetch-minute-klines` 类似，但无 Period/Adjust 行（MINUTE_DATA 不支持这些维度）。

## 5. 不变量（Invariants）

### INV-1A — envelope 路径
`fetch_minute_share` 必须走 `/data/fetch` envelope（POST + JSON body），复用 `OpenStockClient::fetch<T>()` generic envelope 方法。**禁止**直 reqwest（与 P0.13b-1 不同，P0.13b-1 走 `/data/bars` 直 reqwest 是设计决策 D2）。

**验证**：wiremock 测试断言请求是 POST `/data/fetch`，body 包含 `category: "MINUTE_DATA"`。

### INV-1B — 请求体形状
请求体 JSON 必须为 `{category, code, date}` 三字段。**不含** `period`/`adjust`（MINUTE_DATA 不支持）。`date` 格式为 `"YYYY-MM-DD"`。

**验证**：wiremock 测试 mock server 断言收到的 body。

### INV-2C — 单条记录字段缺失 warn+skip
当 `parse_minute_share` 返回 `None`（关键字段缺失或时间格式无效）时，**不**让整个 `fetch_minute_share` 失败，而是 warn log + skip 该条，继续处理剩余记录。

**实现**：Option 字段 + parser 返回 Option。
**验证**：wiremock 测试构造一个 mixed records 数组（部分完整、部分缺字段），断言返回 Vec 长度 < 输入长度，且完整记录被正确解析。

### INV-3 — 字段精度
`price/amount/avg_price` 必须为 `Decimal`（非 `f64`）。serde 兼容 eLtdx 输出的 float 字段。`volume` 为 `i64`（成交量为整数）。

**验证**：单元测试覆盖 `Decimal::from_f64_retain` 路径；serde 集成测试反序列化 eLtdx-shaped JSON。

### INV-4 — envelope retry 不被绕过
**禁止**给 `fetch_minute_share` 加 `.expect(1)` 或绕过 envelope 的 retry/circuit breaker。envelope 路径的 retry 是 OpenStock 运维契约（`/data/fetch` 可能返回 transient 失败）。

**验证**：实现 review 确认调用的是 `self.fetch::<T>()` 而非自定义 reqwest。

## 6. 决策记录

### D1：MINUTE_DATA category 名常量
**决策**：在 `src/sources/openstock_client.rs` 内联 `"MINUTE_DATA"` 字符串（不抽到常量）。

**原因**：现有 `fetch_stock_codes` / `fetch_trade_dates` 等也是内联字符串（"ALL_STOCKS" / "TRADE_DATES" / "INDEX_KLINES"）。统一风格，避免引入新抽象。

**替代方案（拒绝）**：抽到 `OpenStockCategory::MinuteData` enum。理由：现有代码没用此模式，超出 P0.13b-2 范围。

### D2：RawMinuteRecord 实现方式
**决策**：定义显式 struct `RawMinuteRecord` 带所有 8 字段（保留 + 丢弃），丢弃字段用 `#[serde(default)]` 容忍缺失。

**原因**：
- 显式 struct 比 `serde_json::Value` 性能更好（无运行时字段查找）
- 编译时字段名检查
- 与 P0.13b-1 的 `RawMinuteBar` 模式（如果存在）保持一致

**替代方案（拒绝）**：用 `serde_json::Value` + 手动 `get()`。理由：性能差，类型不安全。

### D3：parser 返回 `Option<MinuteShare>` 而非 `Result`
**决策**：parser 返回 `Option`。关键字段缺失（任一 None 或时间格式无效）返回 `None`，调用方 warn+skip。

**原因**：实现 INV-2C "不中断整批" 语义。`Result` 会强制错误冒泡。

**替代方案（拒绝）**：parser 返回 `Result`，调用方 match。理由：冗长，且 INV-2C 本质是 "skip"，不是 "error"。

### D4：timestamp 解析容错
**决策**：`parse_time_minutes` 同时接受 "0930" 和 "09:30" 两种格式。

**原因**：eLtdx MINUTE_DATA 实际输出格式未在文档中明确（R2 风险）。两种都接受避免 wiremock 测试通过但 live 失败。

**替代方案（拒绝）**：硬编码单一格式。理由：live 测试一旦失败需要迭代修复，浪费一轮。

### D5：内联 parser（不抽到独立模块）
**决策**：`parse_minute_share` 作为 `openstock_client.rs` 内的私有 `fn`，与 `parse_minute_bar`（P0.13b-1）同模式。

**原因**：与 P0.13b-1 实现范式统一；不新增文件、不产生跨模块迁移工作量。集中迁移所有 parser 至独立模块属于跨切片重构，单独起专项处理。

### D6：Decimal 精度
**决策**：`price/amount/avg_price` 使用 `Decimal`（rust_decimal）。

**原因**：全行情数据数值层统一高精度类型，规避浮点精度失真；serde 支持 float 自动反序列化为 Decimal，无解析兼容问题；维持项目数据模型一致性规范（`Kline` / `MinuteBar` 也用 Decimal）。

## 7. 风险

### R1：MINUTE_DATA 实际 schema 未验证
**风险**：设计基于 HANDOFF 报告 §B 的 8 字段描述（`time_minutes/price/volume/amount/avg_price/index/time/price_milli`），但实际 OpenStock 运行时可能返回更多/更少字段。

**缓解**：
- `RawMinuteRecord` 用 `#[serde(default)]` 容忍额外字段
- wiremock 测试先行覆盖设计文档描述的 schema
- live 测试（`#[ignore]`）验证生产 schema；若发现差异，迭代修复（与本切片同 PR）

### R2：`time_minutes` 格式歧义
**风险**：可能是 "0930"、"09:30"、或 "09:30:00" 格式。

**缓解**：D4 — `parse_time_minutes` 容错接受多种格式；wiremock 测试覆盖前两种，live 验证。

### R3：`from_f64_retain` 精度损失
**风险**：`Decimal::from_f64_retain(price)` 对某些 float 值可能返回 None（NaN/Infinity）或精度损失。

**缓解**：parser 中 `Decimal::from_f64_retain(x)?` 返回 None → skip 该条（INV-2C 兜底）。

### R4：envelope 失败时的整批语义
**风险**：envelope 5xx / retry 耗尽 → 整批 `fetch_minute_share` 失败（与 INV-2C 的单条 skip 不同维度）。

**缓解**：这是 envelope 路径的标准行为（与 `fetch_stock_codes` 一致）。用户可重试。文档说明：INV-2C 仅适用于「envelope 成功但单条 record 残缺」场景，不适用于 envelope 级失败。

### R5：与 P0.13b-1 代码冲突
**风险**：P0.13b-2 修改 `src/sources/openstock_client.rs` 与 P0.13b-1 同文件。

**缓解**：P0.13b-1 已合并（`8a1a8f2`），无并发修改风险。P0.13b-2 仅追加新方法（`fetch_minute_share`、`parse_minute_share`、`RawMinuteRecord`、`parse_time_minutes`），不修改 P0.13b-1 代码。

## 8. 测试矩阵

### 8.1 Unit / Wiremock 测试（src/sources/openstock_client.rs 内部）

| ID | 描述 | 验证 |
|----|------|------|
| T1 | `MinuteShare` 序列化往返 | serialize → deserialize 等价 |
| T2 | `parse_minute_share` 完整记录 | 5 业务字段 + timestamp 正确 |
| T3 | `parse_minute_share` 缺字段 → None | 任一关键字段 None 返回 None |
| T4 | `parse_time_minutes` 多格式 | "0930" 和 "09:30" 都返回 (9, 30) |
| T5 wiremock | `fetch_minute_share` 发送 MINUTE_DATA category | mock 断言 body `{category: "MINUTE_DATA", code, date}` |
| T6 wiremock | `fetch_minute_share` skip 残缺记录 | mixed records → Vec 长度正确 |
| T7 wiremock | `fetch_minute_share` 4xx propagation | envelope 404 → Err 冒泡 |

### 8.2 Live 测试（tests/openstock_live_minute_share.rs，全部 `#[ignore]`）

| ID | 描述 | 数据 |
|----|------|------|
| L1 | sh600000 当日 MINUTE_DATA | 断言非空，首/末 timestamp 在交易时段内 |
| L2 | 周末/非交易日 | 断言空 Vec 或特定错误 |
| L3 | 未知代码 | 断言 envelope 错误冒泡 |

Live tests 由 `QUANTIX_OPENSTOCK_LIVE=1` env var 门控，CI 跳过。

## 9. OpenSpec 变更

新增 `openspec/changes/openstock-data-consumption-p0-13b-2/`：
- `proposal.md` — Why / What Changes / Impact / Non-Goals
- `tasks.md` — 编号任务步骤（baseline/governance → models → client → CLI → tests → openspec → verification）
- `design.md` — D1-D6 决策 + R1-R5 风险（与本文档 §6/§7 同步）
- `specs/openstock-data-consumption/spec.md` — `## ADDED Requirements` 块，覆盖 MINUTE_DATA scenarios

## 10. Governance Card

新增 `.governance/programs/project-governance/cards/P0.13b-2.yaml`：
- `scope.allowed_paths`: P0.13b-2 涉及的所有文件
- `scope.forbidden_paths`: P0.13b-1 的关键 symbol（`MinuteBar`、`fetch_minute_klines`、`MinutePeriod`），防止误改
- `acceptance_gates`: cargo fmt / clippy / test / openspec validate
- `non_goals`: 多日范围、ClickHouse writes、其他 category

## 11. 文件清单

| 文件 | 操作 | 行数预估 |
|------|------|---------|
| `src/data/models.rs` | Modify（追加 MinuteShare struct + tests） | +60 |
| `src/sources/openstock_client.rs` | Modify（追加 fetch_minute_share / parse_minute_share / RawMinuteRecord / parse_time_minutes + wiremock tests） | +180 |
| `src/cli/commands/data.rs` | Modify（追加 FetchMinuteShare 变体） | +6 |
| `src/cli/handlers/openstock_handler.rs` | Modify（追加 fetch_openstock_minute_share） | +30 |
| `src/cli/handlers/mod.rs` | Modify（追加 re-export） | +1 |
| `src/cli/handlers/app_shell.rs` | Modify（追加 dispatcher arm） | +3 |
| `tests/openstock_live_minute_share.rs` | Create | +60 |
| `openspec/changes/openstock-data-consumption-p0-13b-2/proposal.md` | Create | +30 |
| `openspec/changes/openstock-data-consumption-p0-13b-2/tasks.md` | Create | +60 |
| `openspec/changes/openstock-data-consumption-p0-13b-2/design.md` | Create | +80 |
| `openspec/changes/openstock-data-consumption-p0-13b-2/specs/openstock-data-consumption/spec.md` | Create | +60 |
| `.governance/programs/project-governance/cards/P0.13b-2.yaml` | Create | +30 |

总改动：~600 行新增，0 删除。

## 12. 验证

```bash
# Quality gates
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli openstock
cargo test --test openstock_live_minute_share        # skipped without QUANTIX_OPENSTOCK_LIVE=1
cargo test --workspace                                # regression

# Spec + governance
openspec validate openstock-data-consumption-p0-13b-2 --strict
openspec validate --all --strict
gitnexus detect_changes                               # expect LOW risk on client + handlers

# Manual live smoke (only when OpenStock runtime is reachable)
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_minute_share -- --ignored

# CLI smoke
cargo run -q -- data openstock fetch-minute-share --symbol sh600000 --date 2026-06-30
```

## 13. 后续事项（超出 P0.13b-2 范围）

合并后：
- `git push origin master`
- `gitnexus analyze`
- `openspec archive openstock-data-consumption-p0-13b-2`
- 翻转 governance card P0.13b-2.yaml → `completed`
- 开始 P0.13c（多日范围查询）头脑风暴
