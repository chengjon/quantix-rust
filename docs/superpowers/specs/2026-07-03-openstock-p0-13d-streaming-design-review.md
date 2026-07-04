# 审核意见：P0.13d Streaming Fetch 设计文档

> 审核日期：2026-07-03
> 审核范围：`docs/superpowers/specs/2026-07-03-openstock-p0-13d-streaming-design.md` 全文
> 审核基线：HEAD（P0.13c 已合并），对照 `src/sources/openstock_client.rs`、`src/data/models.rs`、`Cargo.toml`

---

## 总体评价

设计在 P0.13c 基础上干净地叠加了流式层——双 API 并存（D6）、无新公共类型、`futures` 已在 workspace 中、`DateOrRange` + `iter_dates_inclusive` 全部复用。D1-D6 决策链路完整，INV 体系覆盖等价性/兼容性/错误语义。但有三处事实偏差需要修正。

---

## 需要修正

### 1. R2 风险已过期——`futures` 已在 `Cargo.toml` 中

**位置**：设计 §7 风险表 R2 + §9 文件清单 L349

> R2: `futures` crate 不在 workspace `[dependencies]`。Cargo.toml 加 `futures = "0.3"`

**实际状态**：`Cargo.toml:38-39` 已有：

```toml
futures = "0.3"
futures-util = "0.3"
```

**影响**：R2 风险不需要缓解——已经不存在。§9 文件清单中 `Cargo.toml (+2)` 的预估也不需要（0 行改动）。

**建议**：删除 R2，§9 去掉 Cargo.toml 行，总估时从 +580 下调到 ~578。

---

### 2. `DateOrRange::Range` 字段在设计中标为 `Option<NaiveDate>`——实际代码为非 Option

**位置**：设计 L118-119

```rust
// 设计中的写法（过期）
Range { start: Option<NaiveDate>, end: Option<NaiveDate> },
```

**实际代码**（`models.rs:287-294`）：

```rust
// D5 已拒绝半开区间——from_cli 绝不可能产生带 None 的 Range
pub enum DateOrRange {
    Date(chrono::NaiveDate),
    Range {
        start: chrono::NaiveDate,    // ← 非 Option
        end: chrono::NaiveDate,      // ← 非 Option
    },
}
```

**影响**：stream 实现代码（L127-129）的 `match` 解构与实际情况一致（直接解构 `start`/`end` 不 unwrap），**行为正确**。但设计文档中 enum 定义与代码不同步——reviewer 或 implementer 读到这里会困惑。`chunk_range_weekly(start, end)` 的签名也无需 Option。

**建议**：L116-119 的 `DateOrRange` 定义与 `models.rs` 同步，`start`/`end` 去掉 `Option`。

---

### 3. `FetchMinuteKlines` CLI struct 缺少 `default_value` 属性

**位置**：设计 L162-171

```rust
FetchMinuteKlines {
    #[arg(long)] symbol: String,
    #[arg(long)] period: String,          // ← 缺 default_value = "1m"
    #[arg(long)] adjust: Option<String>,  // ← 缺 default_value = "none"
    ...
```

当前 P0.13c 代码中 `period` 有 `default_value = "1m"`，`adjust` 有 `default_value = "none"`。设计片段省略了这些属性——如果是意图变更（去掉默认值），需要明确标注为 Decision；如果是缩写，实现者会误删。而且 `adjust` 从 `String` 改为 `Option<String>` 未标注——P0.13c 中是 `String` 有默认值。

**建议**：补全 `default_value` 属性，或加注释 `// (existing attrs unchanged)`。

---

## 已验证正确的部分

| 设计声明 | 结论 | 证据 |
|----------|------|------|
| D1 `impl Stream<Item = Result<Vec<T>>>` | ✅ | 编译期单态化、零堆分配、惯用 `while let Some` |
| D2 klines 固定 7 天切片 | ✅ | 1200 bars/周远低于阈值，纯函数易测 |
| D3 share 一日一 batch + 空 Vec | ✅ | 完美复用 `fetch_minute_share_single`，batch count == 日历天数 |
| D4 首个错误终止 | ✅ | 与 batch API 语义一致 |
| D5 继承现有 retry/breaker 行为 | ✅ | klines 无 retry，share 自动继承 envelope retry |
| D6 双 API 并存 | ✅ | 零 churn、INV-4A 保证、P0.13a/b/c 测试全不变 |
| INV-1A stream vs batch 等价 | ✅ | S5 端到端相等测试覆盖 |
| INV-1B/1C `chunk_range_weekly` 覆盖 | ✅ | S1-S4 四测试覆盖单日/7日/8日/长范围 |
| INV-2A/2B wire body 兼容 | ✅ | `start == end` → `date`；`start != end` → `start_date`/`end_date` |
| INV-3 类型不变 | ✅ | `MinuteBar`/`MinuteShare`/`MinutePeriod`/`DateOrRange`/`AdjustType` 全部不变 |
| INV-4A/4B 现有 API/CLI 不变 | ✅ | 仅新增 `--stream` flag 和 stream 方法 |
| S1-S4 `chunk_range_weekly` 测试 | ✅ | 单日/7日边界/8日分割/长范围全覆盖 |
| S5 batch vs stream 等价 | ✅ | 核心不变量，mock 注入验证 |
| S6 错误终止 | ✅ | 注入第 2 batch 失败，验证 stream 终止 |
| S7 非交易日空 Vec | ✅ | mock 返回空 records |
| W1-W3 wiremock 覆盖 | ✅ | 调用次数 + body 验证，不复验证 P0.13c wire shape |

---

## 总结

| 维度 | 评价 |
|------|------|
| API 设计 | ✅ 双 API 并存，零 churn，`impl Stream` 惯用 |
| 与 P0.13c 衔接 | ✅ 100% 复用 `DateOrRange` + `iter_dates_inclusive` + `fetch_minute_share_single` |
| 风险登记 | ⚠️ R2 已过期（`futures` 已在 Cargo.toml） |
| 文档同步 | ⚠️ `DateOrRange::Range` 字段类型 + CLI `default_value` 与实际代码不同步 |
| 决策完整性 | ✅ D1-D6 全部有 rejected alternatives |
| 测试覆盖 | ✅ 7 unit + 3 wiremock + 2 live = 12，INV 全覆盖 |

**结论**：修正 R2（删除）、同步 `DateOrRange` 定义、补全 CLI `default_value` 后，即可编写实施计划。
