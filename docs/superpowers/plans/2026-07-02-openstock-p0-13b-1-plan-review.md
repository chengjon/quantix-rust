# 审核意见：P0.13b-1 分钟蜡烛实施计划

> 审核日期：2026-07-02
> 审核范围：`docs/superpowers/plans/2026-07-02-openstock-p0-13b-1-minute-candles-plan.md` 全文（1019 行）
> 审核基线：HEAD（P0.13a 已合并），对照 `src/sources/openstock_client.rs`、`src/data/models.rs`、`src/cli/commands/data.rs`、`src/cli/handlers/app_shell.rs`、`src/cli/handlers/mod.rs`

---

## 总体评价

计划非常详尽——四组 Task、逐步骤的 TDD 循环（写失败测试→实现→验证→clippy/fmt→commit）、完整的 commit message 模板、OpenSpec change 全套文件内容都给了。Global Constraints 正确纳入了上一轮设计审核的全部修正（`MinuteBar` 命名、`OpenStockSettings` 类型、`from_settings` 调用、adjust 字段省略）。但有一处**阻断级**代码错误和几条行号细节需要修正。

---

## 阻断级问题

### 1. Wiremock 测试使用了不存在的 `test_client_for` helper（**CRITICAL**）

**位置**：Task 2 Step 1（L252、L289、L310）

计划中三个 wiremock 测试的写法是：

```rust
let client = test_client_for(&server);
```

但 `test_client_for` **不存在**于 `src/sources/openstock_client.rs` 的测试模块中。实际的 P0.13a wiremock 测试（例如 L1093、L1134、L1182）使用的模式是：

```rust
let server = MockServer::start().await;
let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
```

**影响**：三个 wiremock 测试（`fetch_minute_klines_1m_none_sends_period_1m_and_date`、`fetch_minute_klines_5m_qfq_sends_adjust_and_stamps_records`、`fetch_minute_klines_propagates_4xx`）全部编译失败。

**同时**：计划的 Note for implementer（L326）写的是：

> The `test_client_for` helper and imports... are already in scope... If `test_client_for` is named differently, grep the file first.

这个提示方向是对的（让实现者自己 grep），但**作为实施计划，应该直接给出正确的函数名**而非让实现者现场排查。计划的其他部分精确到了具体行号和 import 路径，唯独这一处"差不多就行"——对于可执行计划来说不够。

**建议**：三个测试的 `test_client_for(&server)` 全部改为 `OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build")`，与 P0.13a 的 wiremock 测试保持一致。

---

## 需要修正

### 2. 行号引用可能导致实现者定位偏差

计划中有几处行号与实际代码不完全对齐。这些不是阻断性的（实现者可以用 grep），但在可执行计划里会影响效率：

| 位置 | 计划行号 | 实际情况 |
|------|---------|---------|
| L44 `models.rs:39-104` 插入点 | "after `BarPeriod` impl, before `AdjustType::as_openstock_param`" | 描述比行号更可靠——实现者应搜 `impl std::str::FromStr for BarPeriod` 定位 |
| L206 `openstock_client.rs` "after `fetch_klines_propagates_4xx` around L1200" | ~L1200 | 实际在 ~L1176 |
| L335 "after `fetch_klines` method's closing `}` around L675" | ~L675 | `fetch_klines` 的结束 `}` 在 ~L675-680 范围，但需确认 `fetch_klines` 之后没有其他方法插入 |

**建议**：关键插入点改为「搜索 `fn fetch_klines(` 定位，在其后插入」而非硬编码行号。行号作为参考标注即可。

### 3. Handler 中 `use crate::core::runtime::OpenStockSettings` 冗余 import

**位置**：Task 3 Step 2（L520）

```rust
use crate::core::runtime::OpenStockSettings;
```

该 import 已在 `openstock_handler.rs:4` 文件顶部导入。在函数体内二次 `use` 不会导致编译错误（Rust 允许 shadow/repeat），但会产生 clippy warning。计划在 L560 的 Note 中也提到了防御性 import——但建议把 `OpenStockSettings` 从这个 inline block 中移除，因为它是 100% 已存在的。

---

## 已验证正确的部分

| 检查项 | 结论 | 证据 |
|--------|------|------|
| `MinuteBar` 命名（非 `MinuteKline`） | ✅ | Global Constraints + Task 1 Step 3 明确注释了 TDengine 碰撞原因 |
| `OpenStockSettings` 类型（非 `OpenStockClientSettings`） | ✅ | L512 使用正确类型 |
| `from_settings(settings)` 调用 | ✅ | L531 使用正确方法 |
| adjust 字段省略逻辑 | ✅ | L372-374 的 `if let Some(adj)` 条件判断 |
| `MinutePeriod::from_str` 严格白名单 | ✅ | L133-145 仅接受 5 个 token |
| TDD 循环结构 | ✅ | 每组 Task 都是 write-failing-test → verify-fail → implement → verify-pass → clippy/fmt → commit |
| Commit message 模板 | ✅ | 每步有完整的 `git commit -m` 模板，含 Co-Authored-By |
| `mod.rs:130` 插入位置 | ✅ | `fetch_openstock_klines,` 与 `fetch_openstock_workdays,` 之间 |
| CLI 路径 `data openstock fetch-minute-klines` | ✅ | L603 验证命令正确 |
| Global Constraints 覆盖 | ✅ | 命名、wire token、adjust omission、no retry、Kline immutability、error pattern、file size、test/live patterns |
| OpenSpec change 内容 | ✅ | proposal/tasks/design/spec/card 五件套内容完整 |

---

## 次要意见

| 位置 | 意见 |
|------|------|
| L5 标题 | "`/data/bars?period=1m`" — `/data/bars` 是 POST，不是 GET+query string。内容正确（架构描述），标题中的 `?` 会误导 |
| L363 `FetchKlines` pattern reference | `data.rs:363-385` — `FetchKlines` 从 L363 开始约 22 行，插入 `FetchMinuteKlines` 后文件总行在 ~390 附近，确在合理范围内 |
| L607 `cargo test --workspace` | 注册测试总数会从当前值增加 6+3=9 个（6 unit/wiremock + 3 live ignored），计划没有预计总测试数——实际上无关紧要 |

---

## 总结

| 维度 | 评价 |
|------|------|
| TDD 结构完整性 | ✅ 每组 Task 6 步骤循环：测试→失败→实现→通过→lint→commit |
| 代码片段准确性 | ❌ `test_client_for` 不存在——3 个 wiremock 测试编译失败 |
| 行号精度 | ⚠️ 几处偏差在 ±20 行内，描述式定位比行号更可靠 |
| 上游设计对齐 | ✅ 全部 6 条 Global Constraints 正确纳入了 R1 审核修正 |
| OpenSpec 交付物 | ✅ 五件套内容完整，可直接 `mkdir -p` + 贴入 |
| 可执行性 | ⚠️ 修正 `test_client_for` 后即可逐步骤执行 |

**结论**：修正 `test_client_for` → `OpenStockClient::new(fast_test_cfg(...))` 后，这份计划可以直接交给 agent 执行。
