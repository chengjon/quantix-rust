# 外部系统合同文档与同步测试 — review2md 审核报告

> **审核类型**: evidence-driven document review  
> **审核日期**: 2026-07-11  
> **审核人**: CodeWhale (review2md flow)  
> **工作空间**: quantix-rust  
> **审核对象**:
> - `docs/contracts/external-systems.md` (604 行, 21KB)
> - `tests/contract_doc_sync_test.rs` (214 行, 7KB)

---

## 1. 审核概要

| 维度 | 结果 |
|------|------|
| Layer 1 — 全量清单完整性 | ✅ **PASS** (27/27 系统, 32/32 路径存在) |
| Layer 2 — 定点抽查 (9 项) | ✅ **PASS** (9/9 通过，验证后已修正 2 项源文件缺陷) |
| 文档一致性 | ✅ **PASS** (环境变量/feature flag/合约均与代码一致) |
| 测试可执行性 | ✅ **PASS** (`cargo test --test contract_doc_sync_test` 11/11 通过) |
| **总评** | **✅ PASS — 通过** |

---

## 2. 审核方法

| 步骤 | 做法 |
|------|------|
| 源码勘察 | 遍历 `src/` 下 522+ 个 `.rs` 文件，识别所有 HTTP 请求器、TCP 连接、DB 驱动、API 调用 |
| 归类 | 按 7 大类 (Storage/Market/Execution/AI/News/Fundamental/Notification) 归纳 |
| 回写文档 | 三层结构 (系统层/服务层/合约层) + 附录 (环境变量/feature flag) |
| 编写测试 | Layer 1 全量清单 + Layer 2 9 项定点抽查 (hard-fail) |
| 运行验证 | `cargo test --test contract_doc_sync_test` 验证 11/11 通过 |
| 产生报告 | 本文件 |

---

## 3. Layer 1 — 全量清单完整性 (CP)

### CP1: 外部系统总数

**要求**: 文档列举全部外部系统

**证据** (源码勘察结果):

| 分类 | 文档计数 | 源码确认 | 状态 |
|------|---------|---------|------|
| 数据存储 | 4 | ClickHouse, PostgreSQL, TDengine, SQLite | ✅ |
| 市场行情 | 7 | OpenStock, TDX TCP, Bridge TDX, EastMoney HTTP, AkShare, WebSocket, Kline Aggregator | ✅ |
| 交易执行 | 4 | Windows Bridge, QMT Live, QMT Preview, MiniQMT | ✅ |
| AI/LLM | 3 | DeepSeek, OpenAI, Ollama | ✅ |
| 新闻搜索 | 3 | Tavily, SerpAPI, Bocha | ✅ |
| 基本面 | 1 | EastMoney Fundamental | ✅ |
| 通知推送 | 5 | Feishu, WeChat Work, Desktop, Webhook, Log | ✅ |
| **合计** | **27** | **27** | ✅ |

**修正记录**: 初版文档写 23，源码勘察实为 27，已修正为 27。根因：通知推送 5 个 + AI 3 个 + 市场行情 `KlineAggregator` 和 `WebSocket` 被漏数。

### CP2: 关键源码路径

**要求**: 每个系统至少有一个关键源码路径在测试中验证

**证据**: `L1_ENTRIES` 数组定义了 32 个路径 (27 系统 × 平均 1.2 路径/系统)

| 系统 | 验证的路径 | 存在? |
|------|-----------|-------|
| ClickHouse | `src/db/clickhouse/mod.rs` | ✅ |
| PostgreSQL | `src/db/postgresql.rs` | ✅ |
| TDengine | `src/db/tdengine.rs` | ✅ |
| SQLite | `Cargo.toml` | ✅ |
| OpenStock API | `client.rs`, `envelope.rs` | ✅ |
| TDX TCP | `src/sources/tdx.rs` | ✅ |
| Bridge TDX | `src/sources/bridge_tdx.rs` | ✅ |
| EastMoney HTTP | `src/sources/eastmoney.rs` | ✅ |
| AkShare | `src/sources/akshare.rs` | ✅ |
| WebSocket | `src/sources/websocket.rs` | ✅ |
| Kline Aggregator | `src/sources/kline_aggregator.rs` | ✅ |
| Windows Bridge | `client.rs`, `models.rs` | ✅ |
| QMT Live | `qmt_live_adapter.rs` | ✅ |
| QMT Preview | `qmt_bridge.rs` | ✅ |
| MiniQMT | `miniqmt_market.rs` | ✅ |
| DeepSeek | `openai_compat.rs` | ✅ |
| OpenAI | `openai_compat.rs` | ✅ |
| Ollama | `openai_compat.rs` | ✅ |
| Tavily | `tavily.rs` | ✅ |
| SerpAPI | `serpapi.rs` | ✅ |
| Bocha | `bocha.rs` | ✅ |
| EastMoney Fundamental | `eastmoney.rs` | ✅ |
| Feishu | `feishu.rs` | ✅ |
| WeChat Work | `wechat_work.rs` | ✅ |
| Desktop | `desktop.rs` | ✅ |
| Webhook | `webhook.rs` | ✅ |
| Log | `log.rs` | ✅ |

**测试验证**: `l1_full_inventory_all_paths_exist` → 32/32 路径 ✅

---

## 4. Layer 2 — 定点抽查 (CA)

### CA1: SC1 — Bridge 默认 URL 端口

| 维度 | 值 |
|------|-----|
| 文档声称 | `http://127.0.0.1:17580` (2.12 节) |
| 源码证据 | `src/core/runtime/settings.rs:34` → `DEFAULT_BRIDGE_BASE_URL = "http://127.0.0.1:17580"` |
| 测试 | `l2_sc1_bridge_default_port` → ✅ |
| **修正记录**: 初版测试搜索 `src/bridge/client.rs`，实际位于 `src/core/runtime/settings.rs`。已修正。 |

### CA2: SC2 — Contract Version 默认值

| 维度 | 值 |
|------|-----|
| 文档声称 | `miniqmt.v1` (3.1 节) |
| 源码证据 | `src/core/runtime/settings.rs:36` → `DEFAULT_BRIDGE_CONTRACT_VERSION = "miniqmt.v1"` |
| 测试 | `l2_sc2_contract_version_miniqmt` → ✅ (搜索 3 个文件) |
| **修正记录**: 初版仅搜索 `bridge/client.rs` + `runtime.rs`，未包含 `settings.rs`。已修正。 |

### CA3: SC3 — ClickHouse 默认端口

| 维度 | 值 |
|------|-----|
| 文档声称 | `http://localhost:8123` (2.1 节) |
| 源码证据 | `src/db/clickhouse/mod.rs` 含 `"8123"` 默认 URL |
| 测试 | `l2_sc3_clickhouse_default_port` → ✅ |

### CA4: SC4 — OpenStock 默认重试次数

| 维度 | 值 |
|------|-----|
| 文档声称 | `3` 次重试 (2.5 节) |
| 源码证据 | `src/sources/openstock_client.rs` → `DEFAULT_MAX_RETRIES` |
| 测试 | `l2_sc4_openstock_retries` → ✅ |

### CA5: SC5 — OpenStock Circuit Breaker 阈值

| 维度 | 值 |
|------|-----|
| 文档声称 | `5` 次连续失败/30s 冷却 (2.5 节) |
| 源码证据 | `src/sources/openstock_client.rs` → `DEFAULT_CIRCUIT_BREAK_THRESHOLD` |
| 测试 | `l2_sc5_openstock_circuit_breaker` → ✅ |

### CA6: SC6 — BridgeTaskLifecycleStatus 枚举变体

| 维度 | 值 |
|------|-----|
| 文档声称 | 4 种: Pending, Completed, Failed, BridgeTaskAccepted (2.12 节) |
| 源码证据 | `src/bridge/models.rs` → `pub enum BridgeTaskLifecycleStatus` 含全部 4 变体 |
| 测试 | `l2_sc6_bridge_lifecycle_status` → ✅ |

### CA7: SC7 — BridgeFailureCode 枚举变体

| 维度 | 值 |
|------|-----|
| 文档声称 | 7 种 (3.2 节) |
| 源码证据 | `src/bridge/models.rs` → `pub enum BridgeFailureCode` 含全部 7 变体 |
| 测试 | `l2_sc7_bridge_failure_codes` → ✅ |

### CA8: SC8 — ClickHouse 默认批次大小

| 维度 | 值 |
|------|-----|
| 文档声称 | 1000 行 (2.1 节) |
| 源码证据 | `src/db/clickhouse/mod.rs:50` → `DEFAULT_BATCH_SIZE: usize = 1000` |
| 测试 | `l2_sc8_clickhouse_batch_size` → ✅ |

### CA9: SC9 — PostgreSQL 连接池上限

| 维度 | 值 |
|------|-----|
| 文档声称 | 10 (2.2 节) |
| 源码证据 | `src/db/postgresql.rs` → `max_connections(10)` |
| 测试 | `l2_sc9_postgres_max_connections` → ✅ |

---

## 5. 一致性检查 (CS)

### CS1: 环境变量索引 vs 代码

**要求**: 附录 A 列出的环境变量在源码中有定义

| 环境变量 | 源码搜索 | 状态 |
|----------|---------|------|
| `QUANTIX_CLICKHOUSE_URL` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_CLICKHOUSE_DB` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_CLICKHOUSE_USER` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_CLICKHOUSE_PASSWORD` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_POSTGRES_URL` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_TDENGINE_URL` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_TDENGINE_TOKEN` | `src/core/runtime/settings.rs` | ✅ |
| `QUANTIX_BRIDGE_URL` | `src/core/runtime/settings.rs:28` | ✅ |
| `QUANTIX_BRIDGE_API_KEY` | `src/core/runtime/settings.rs:29` | ✅ |
| `QUANTIX_BRIDGE_BEARER_TOKEN` | `src/core/runtime/settings.rs:30` | ✅ |
| `QUANTIX_BRIDGE_CONTRACT_VERSION` | `src/core/runtime/settings.rs:31` | ✅ |
| `QUANTIX_BRIDGE_TIMEOUT_MS` | `src/core/runtime/settings.rs:32` | ✅ |
| `QUANTIX_BRIDGE_POLL_INTERVAL_MS` | `src/core/runtime/settings.rs:33` | ✅ |
| `QUANTIX_BRIDGE_POLL_TIMEOUT_MS` | `src/core/runtime/settings.rs` | ✅ |

**结论**: 14/14 环境变量与代码一致 ✅

### CS2: Feature Flag vs Cargo.toml

| Feature | Cargo.toml 行 | 文档附录 B | 状态 |
|---------|--------------|-----------|------|
| `postgresql` | `sqlx/postgres` | ✅ | ✅ |
| `sqlite` | `sqlx/sqlite` | ✅ | ✅ |
| `tdengine-rest` | feature 定义 | ✅ | ✅ |
| `tdengine-ws` | feature 定义 | ✅ | ✅ |
| `tui` | `ratatui+crossterm` | ✅ | ✅ |

**结论**: 5/5 feature flag 一致 ✅

### CS3: 文档内部一致性

| 检查项 | 结果 |
|--------|------|
| 系统计数与分类总和不一致 | ❌ **初版 23≠4+7+4+3+3+1+5=27 → 已修正** |
| 目录锚点 vs 实际标题 | ✅ 所有锚点匹配 |
| 附录 A 引用章节号 vs 实际 | ✅ 2.1, 2.2, 2.3, 2.12 均正确 |
| 第三层合约引用第二层 | ✅ 所有第三层概念在第二层有定义 |

---

## 6. 发现项总表

| ID | 严重度 | 状态 | 说明 |
|----|--------|------|------|
| F1 | **Hard** | ✅ **已修复** | 文档系统计数 23 错误，实为 27。根因：漏数 AI 3 个+通知 5 个中的部分项。 |
| F2 | **Hard** | ✅ **已修复** | SC1 测试搜索路径 `bridge/client.rs` 不含 `17580`，实际在 `settings.rs`。 |
| F3 | **Hard** | ✅ **已修复** | SC2 测试搜索 `bridge/client.rs`+`runtime.rs` 不含 `miniqmt.v1`，需加 `settings.rs`。 |
| F4 | Info | ✅ 非缺陷 | `sqlite` feature 仅依赖 `Cargo.toml` 存在，没有独立源码文件 — 合理（feature-gated）。 |
| F5 | Info | ✅ 非缺陷 | AkShare 标记为"骨架/未接入" — 与代码实际匹配 (`Unsupported` 错误)。 |

---

## 7. 测试执行结果

```
running 11 tests
test l1_doc_file_exists              ... ok
test l1_full_inventory_all_paths_exist ... ok
test l2_sc1_bridge_default_port      ... ok
test l2_sc2_contract_version_miniqmt ... ok
test l2_sc3_clickhouse_default_port  ... ok
test l2_sc4_openstock_retries        ... ok
test l2_sc5_openstock_circuit_breaker ... ok
test l2_sc6_bridge_lifecycle_status  ... ok
test l2_sc7_bridge_failure_codes     ... ok
test l2_sc8_clickhouse_batch_size    ... ok
test l2_sc9_postgres_max_connections ... ok

test result: ok. 11 passed; 0 failed; 0 ignored
```

---

## 8. 结论

**总评: ✅ PASS**

| 考核维度 | 得分 | 说明 |
|---------|------|------|
| 覆盖完整性 | **100%** | 27/27 外部系统全部覆盖，无遗漏 |
| 源码对齐 | **100%** | 9/9 定点抽查全部通过，已修复 2 项路径缺陷 |
| 文档一致性 | **通过** | 环境变量、feature flag、合约均与代码一致 |
| 测试可执行性 | **通过** | `cargo test` 11/11 全部通过，`hard-fail` 语义生效 |

**修复总结**: 审核过程中发现了 3 个硬缺陷并全部修复:
1. F1: 文档系统计数 23→27
2. F2: SC1 测试路径 `bridge/client.rs` → `runtime/settings.rs`
3. F3: SC2 测试搜索文件集补充 `settings.rs`

**维护建议**: 后续修改任何外部系统集成（新增/修改/移除），应同时更新本文档 Layer 2 的合同参数，并确保 `tests/contract_doc_sync_test.rs` 中的对应测试项随之更新。
