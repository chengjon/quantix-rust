# 审核意见：P0_11B_IMPLEMENTATION_REVIEW_2026-07-01.md

> 审核日期：2026-07-01
> 审核范围：`docs/reports/P0_11B_IMPLEMENTATION_REVIEW_2026-07-01.md` 全文
> 审核基线：commit `47747c5`，对照源码 `src/sources/openstock_ticks.rs`、`src/sources/openstock_client.rs`、`src/cli/handlers/tdx_api_handler.rs`、`src/cli/commands/data.rs`、`src/data/models.rs`、`src/db/tdengine.rs`、`tests/openstock_tick_data_live_test.rs`、`openspec/changes/openstock-data-consumption-p0-11/tasks.md`

---

## 总体评价

这是一份诚实的实施审核文档——七个问题各附证据和代码位置，自我列出了六条待审核要点（§七），未完成事项（§五）也标注清楚。以下三条需要修正或补充，其余验证通过。

---

## 必须修正

### 1. 问题 4（TradeDirection → status）：与 tdx-api legacy 路径的 status 语义不一致

报告将 `TradeDirection::{Buy→1, Sell→-1, Neutral→0}` 描述为「选择 1/-1/0 符合 A 股逐笔成交方向惯例」，标注「需要审核确认」——标注本身是对的，但**没有指出同文件中 legacy tdx-api 路径用的是完全不同的语义**。

**证据**：handler 第 403 行，tdx-api legacy 分支：

```rust
let amount = price * t.volume as f64 * 100.0;
(ts_ms, price, t.volume, amount, t.status)  // ← 直接用原始 t.status，不做映射
```

`t.status` 是 `i32`，来自 tdx-api 的原始协议字节，其语义未知（可能不是买/卖方向，可能是成交类型、撤单标记等）。openstock 新路径用 1/-1/0 映射 `TradeDirection`，两个路径写进同一 TDengine 表的 `direction TINYINT` 列，但**语义完全不同**——下游消费者如果按 status 列筛选，会拿到两组不可比较的值。

**建议**：在文档 §二问题 4 中补充这一差异，并在 §七审核要点 1 中明确写出「与 legacy tdx-api 路径的 `t.status` 语义不一致，需要确认能否统一或至少文档化两套语义」。

---

### 2. 问题 5：`decimal_to_f64` 失败降级 `0.0` 比报告描述的风险更高

报告写「失败时降级为 0.0（与 legacy 路径 `unwrap_or(0)` 一致）」——这是不准确的对照。

**证据**：

- Legacy tdx-api 路径（handler:403）的 `unwrap_or(0)` 作用于**整数**算术（`price * volume * 100`），`0` 作为金额在整型上下文中表示「未计算」。
- Openstock 路径的 `decimal_to_f64` 作用于 `Decimal` → `f64` 转换，`0.0` 会**静默替代真实价格/金额**写入 TDengine——数据损坏。

**场景**：`Decimal` 值超出 `f64` 范围（例如 `1e400`）时 `to_f64()` 返回 `None`，触发 `unwrap_or(0.0)`。结果：TDengine 里这条 tick 的 `price` 列变成 `0.0`，但原始数据其实是有效的。运维侧无法通过查询区分「真实价格为 0」和「转换失败降级」。

**建议**：要么改成返回 `Result<f64>` 让调用方决策（报告 §七第 5 条已提到），要么在文档中把风险等级从「可接受」上调为「需监控」（至少加一条 `tracing::warn!`）。

---

## 需要补充

### 3. `TickEntry` / `TickMeta` 缺少 `#[serde(flatten)] extra` catch-all——与项目 parser 约定不一致

项目的 parser 约定（见 `src/sources/openstock_codes.rs` 注释「do not hardcode field names beyond the minimal parsing set」）是每个 record struct 都带 `extra: HashMap<String, Value>` catch-all。`StockCodeRecord`、`StockListRecord`、`TradeDateRecord`、`WorkdayRecord` 都遵守了。

但 `TickEntry` 和 `TickMeta` 没有 `extra` 字段。runtime 返回的 `order_count`、`status`、`price_milli`、`price_delta_raw` 等字段被 serde 的 `#[serde(default)]` 静默丢弃，且无法在日志中看到丢弃了什么。

报告 §5.1 提到了这个风险，但侧重「后续切片扩展」，没有指出**与项目 parser 约定的不一致**。如果 runtime 未来在 `TICK_DATA` 响应中新增字段（例如 `price_base=1000` 改为 `price_base=10000`），当前代码不仅会丢弃新字段，dry-run 输出也无法提醒操作员。

**建议**：在 §二或 §五补充一条，指出 `TickEntry` 和 `TickMeta` 应增加 `#[serde(flatten)] extra` 以对齐项目 parser 约定。

---

## 已验证准确的部分

| 报告声明 | 结论 | 证据 |
|----------|------|------|
| 问题 1：参数名 `symbol` 而非 `code` | ✅ | `fetch_tick_data` 签名 `symbol: &str`，JSON key `"symbol"`，doc comment 明确说明与 `fetch_index_klines` 不同 |
| 问题 2：嵌套信封形状 `{meta, ticks}` | ✅ | `TickEnvelopeRecord { meta, ticks }` 匹配 runtime shape |
| 问题 3：数值字段 `Option<Value>` + `parse_decimal` | ✅ | `TickEntry.price/volume/amount` 全是 `Option<serde_json::Value>`，`parse_decimal` 处理 `String`/`Number` 双分支 |
| 问题 6：`QUANTIX_OPENSTOCK_TICK_APPLY` 独立于 kline 变量 | ✅ | 硬编码字符串比较，不与 `QUANTIX_OPENSTOCK_KLINE_APPLY` 共享 |
| 问题 7：`--source` 默认 `openstock` | ✅ | `#[arg(long, default_value = "openstock")]`，legacy 分支保留 + deprecation warning |
| 8 个单元测试 | ✅ | 全部存在于 `openstock_ticks.rs:203-345`，名称和覆盖场景与报告一致 |
| Quality gates | ✅ | 报告声明 1446 passed / 0 failed / 16 ignored |
| 未完成事项（spec.md / design.md） | ✅ | 四处 gap 标注准确，与 openspec tasks.md 的 2b.10 待办一致 |
| P0.11c 预估行数 | ✅ | `tdx_api.rs` 1309 + `tdx_api_handler.rs` 476 = 1785 行，报告写 ~1800 |

---

## 对 §七审核要点的逐条回应

| # | 审核要点 | 意见 |
|---|---------|------|
| 1 | status 映射 (Buy=1/Sell=-1/Neutral=0) | ⚠️ 需确认与 legacy tdx-api `t.status` 语义是否一致（见上文必须修正 1） |
| 2 | `--source` 默认切 openstock | 风险可控：spec.md 有条件语句，legacy path 有 deprecation warning。建议加一条 release note |
| 3 | `QUANTIX_OPENSTOCK_TICK_APPLY` vs 统一 `QUANTIX_OPENSTOCK_APPLY` | **独立变量是对的**——tick 写 TDengine、kline 写 ClickHouse，风险隔离。统一变量会导致操作员误以为一次确认覆盖两个目标 |
| 4 | `TickEntry` 丢弃 `order_count` 等字段 | 短期可接受（向前兼容）。建议补 `extra` catch-all 以对齐项目 parser 约定（见上文需要补充 3） |
| 5 | `decimal_to_f64` 降级 `0.0` | 风险比报告描述的高（见上文必须修正 2）。建议加 warn 日志或返回 Result |
| 6 | `(TickMeta, Vec<Tick>)` 元组 → 命名结构体 | 同意——未来加 `quality_flags` 会破坏所有调用点。建议 P0.11c 或独立 refactor 切片处理 |

---

## 总结

| 维度 | 评价 |
|------|------|
| 证据充分性 | ✅ 每条发现都有代码行号、commit hash、测试名称 |
| 自我审查诚实度 | ✅ §七主动列出六条待审核点，未完成事项标注清晰 |
| 技术准确性 | ⚠️ 问题 4/5 的低估风险需要修正；`TickEntry` 缺少 `extra` 需补充 |
| 可操作性 | ✅ 改动清单、测试覆盖、P0.11c 启动条件都写得具体 |

**结论**：修正问题 4（status 语义不一致）和问题 5（`0.0` 降级风险）的描述，补充 `extra` catch-all 缺失后，这份文档即可作为 P0.11b 的闭合审核依据。
