# 外部系统契约文档设计

> 本文档是 *契约文档本身的设计 spec*。最终交付物为 `docs/contracts/external-systems.md` + `tests/contract_doc_sync_test.rs`。

## 目标 (Goal)

为 quantix-rust 对接的全部外部依赖产出一份**参考级中文契约文档**，同时配套**同步测试**防止文档与代码漂移。

受众三个：新人上手 / 防回归审计 / 跨团队对接。

## 范围 (Scope)

覆盖 `src/sources/` 与 `src/db/` 下所有外部依赖：

- **三大主系统**（独立章节）：Bridge HTTP / OpenStock HTTP / ClickHouse
- **其他外部依赖**（合并一章）：Postgres / TDengine / 上游 MySQL / 文件源（TDX / AkShare / EastMoney）

不覆盖：项目内部模块边界、CLI 命令、纯计算逻辑。

## 详细度

参考级 (Reference) — 每个系统包含：

- 概述与用途
- 完整端点清单 或 表清单（method/path 或 表名 + 用途一行说明）
- 鉴权机制
- 关键错误模型
- 配置 env 变量
- Contract Version

不列：每字段完整类型注解（除 9 个 spot-check 项）、示例请求、版本演进史。

## 受众与用途

| 受众 | 主要查询路径 |
|------|------------|
| 新开发者 | §1 文档说明 + §2 系统全景图 + 目标系统章节概述 |
| 防回归审计 | 任一章节的 Contract Version + 端点/表清单 + 附录 PR checklist |
| 跨团队对接 | Bridge/OpenStock 章节中的鉴权/版本/错误模型小节 |

## 同步策略

- 文档形式：手写 Markdown，放 `docs/contracts/external-systems.md`
- 漂移防护：单文件同步测试 `tests/contract_doc_sync_test.rs`，CI 随 `cargo test` 运行
- 失败行为：清单或字段不匹配 → **硬失败**，阻止合并

---

## 文档结构 (Output Document TOC)

最终交付 `docs/contracts/external-systems.md` 的目录：

```
# 外部系统契约参考

## 1. 文档说明
   受众、用途、同步策略、Contract Version 规则

## 2. 系统全景图
   Mermaid/ASCII 图：quantix-rust 中央节点 → 8 个外部依赖
   一句话总览每个依赖

## 3. Bridge HTTP 接口
   3.1 概述（用途、版本、契约来源）
   3.2 鉴权（X-Quantix-Api-Key / Bearer / contract_version）
   3.3 端点清单（13 个 /api/v1/* 路径 + method + 一行用途）
   3.4 关键数据模型（BridgeQmtOrderRequest / BridgeTaskExecuteRequest / BridgeKlineBarPayload 字段表）
   3.5 错误模型（BridgeError / BridgeFailureCode）
   3.6 配置（env vars + CliRuntime.bridge 字段）
   3.7 Contract Version（当前 miniqmt.v1）

## 4. OpenStock HTTP 接口
   4.1 概述（NAS 部署、版本来源）
   4.2 鉴权（OPENSTOCK_API_KEY 头）
   4.3 端点清单（WORKDAYS / TRADE_DATES / STOCK_CODES / ALL_STOCKS / DAILY_KLINE / MINUTE_SHARE / TICK_DATA / ...）
   4.4 关键数据模型（OpenStockEnvelope 字段 + DAILY_KLINE 响应形状）
   4.5 错误模型（OpenStockKlineParseError 等）
   4.6 配置（OPENSTOCK_BASE_URL / OPENSTOCK_API_KEY / QUANTIX_OPENSTOCK_*_APPLY）

## 5. ClickHouse 数据契约
   5.1 概述（连接 URL、库 quantix、ON CLUSTER '{cluster}' 模板）
   5.2 表清单（11 张表，按 src/db/clickhouse/schema.rs + models.rs 分类）
       - stock_info / stock_realtime_quotes / kline_data / minute_klines / minute_shares
       - limit_up_events / gbbq_events
       - sector_daily / north_flow_daily / market_sentiment_daily / market_fundamentals_daily
   5.3 关键表 Schema（kline_data / minute_klines / minute_shares / import_state 列名+类型）
   5.4 物化视图与 shadow 表（shadow_kline.rs）
   5.5 配置（QUANTIX_CLICKHOUSE_*）

## 6. 其他外部依赖
   6.1 Postgres（live_import_* / industry_reference_* / risk_industry_snapshots / import_state 表清单）
   6.2 TDengine（库 + supertable + 子表规则 + 写入协议）
   6.3 上游 MySQL（mystocks 库、读取边界）
   6.4 文件源
       - TDX 文件路径与命名约定
       - AkShare / EastMoney HTTP API surface

## 7. 附录
   7.1 同步测试运行说明（cargo test --test contract_doc_sync_test）
   7.2 修改外部系统代码的 PR checklist
   7.3 术语表
```

## 同步测试设计 (tests/contract_doc_sync_test.rs)

单文件，硬失败，无需网络/数据库。两层结构：

### Layer 1 — 清单匹配 (Inventory)

对每个系统，从文档解析条目集合、从代码解析对应条目集合，断言两个集合无序相等。

| 测试函数 | 文档解析目标 | 代码解析目标 |
|---------|------------|------------|
| `bridge_endpoints_in_doc_match_code` | §3.3 端点表 (METHOD + path) | `src/bridge/client.rs` 中 `/api/v[0-9]/...` 字符串字面量 |
| `openstock_endpoints_in_doc_match_code` | §4.3 数据类别表（OpenStock 用统一 `/data/fetch` 端点 + `data_category` 参数区分） | `src/sources/openstock_client*.rs` 中 `pub async fn fetch_*` 方法名 + `data_category` 字符串字面量 |
| `clickhouse_tables_in_doc_match_code` | §5.2 表清单 | `src/db/clickhouse/*.rs` + `src/tasks/openstock_import/*.rs` 中 `CREATE TABLE IF NOT EXISTS <name>` |
| `postgres_tables_in_doc_match_code` | §6.1 表清单 | `src/risk/import_store.rs` + `src/risk/industry_store.rs` 中 `CREATE TABLE IF NOT EXISTS` |
| `env_vars_in_doc_match_code` | 全文中 `OPENSTOCK_*` / `QUANTIX_CLICKHOUSE_*` / `QUANTIX_OPENSTOCK_*_APPLY` 等 | `src/` 中 `env::var("...")` 调用的字符串字面量 |
| `doc_covers_all_source_modules` | 全文中每个外部系统小节列出的源文件名（在 §3.x/§4.x/§5.x/§6.x 小节标题或正文以 `src/sources/<name>.rs` 反引号格式出现） | `ls src/sources/*.rs src/db/clickhouse/*.rs` 文件列表 |

**实现约束**：
- 文档侧解析：用 fenced code block 或 markdown 表格作为结构化来源，避免散文匹配。建议在 §3.3 / §5.2 等小节用统一格式的表格，每行 `| path | 用途 |`，便于 regex 解析。
- 代码侧解析：用 `include_str!` + regex，不引入外部解析依赖。

### Layer 2 — Spot-Check Schema (字段级)

对高 churn / 高风险项，解析文档字段表 + 代码字段定义，断言字段名集合一致。

| 测试函数 | 文档来源 | 代码来源 |
|---------|--------|--------|
| `kline_data_columns_match` | §5.3 中 kline_data 字段表 | `src/db/clickhouse/schema.rs` 中 `CREATE TABLE kline_data` 列定义 |
| `minute_klines_columns_match` | §5.3 | `schema.rs` |
| `minute_shares_columns_match` | §5.3 | `schema.rs` |
| `import_state_columns_match` | §5.3 | `src/tasks/openstock_import/state.rs` |
| `bridge_qmt_order_request_fields_match` | §3.4 | `src/bridge/models.rs` 中 `struct BridgeQmtOrderRequest` |
| `bridge_task_execute_request_fields_match` | §3.4 | `src/bridge/models.rs` |
| `bridge_kline_bar_payload_fields_match` | §3.4 | `src/bridge/models.rs` |
| `openstock_envelope_fields_match` | §4.4 | `src/sources/openstock_envelope.rs` |
| `openstock_daily_kline_response_fields_match` | §4.4 | `src/sources/openstock.rs` 中 `parse_daily_kline_json` 解析的 JSON shape |

**规则**：
- 字段名集合比较（无序），不比较顺序/可选性（避免文档过细导致 churn 过大）
- **类型大类校验（必做）**：除字段名外，必须校验类型大类集合（数值/字符串/时间/自定义）一致；
  这是抓静默漂移（Decimal → Float64、i64 → f64、DateTime ↔ String）的主要网。
- 自定义类型（如 `Decimal`、`serde_json::Value`、`BridgeTaskExecuteParams`）按 `Custom` 归类，
  文档侧注明实际承载的语义（如 "Decimal = 高精度金额，禁用 Float64 替代"）

### 失败诊断

测试失败时打印：

```
[field-set mismatch] minute_shares
Missing in doc:    ["avg_price"]
Missing in source: ["turnover"]
Hint: update §5.3 in docs/contracts/external-systems.md or src/db/clickhouse/schema.rs
```

类型大类失败时打印：

```
[type-class mismatch] kline_data.close
Doc says:    Float (Float64 implies float price)
Source says: Decimal
Hint: Decimal→Float64 is a silent-precision-loss change; bump Contract Version Major
```

## Spot-Check 项总览

按"破坏契约是否导致生产事故/静默数据腐败"原则选取 9 项：

| 类别 | 项 | 理由 |
|------|----|------|
| ClickHouse | kline_data | 主行情表，所有策略读取 |
| ClickHouse | minute_klines | 分钟策略依赖 |
| ClickHouse | minute_shares | OpenStock 新近落地 |
| ClickHouse | import_state | 幂等导入状态机核心 |
| Bridge | BridgeQmtOrderRequest | 实盘下单请求体 |
| Bridge | BridgeTaskExecuteRequest | 任务化下单契约 |
| Bridge | BridgeKlineBarPayload | 行情回放数据点 |
| OpenStock | OpenStockEnvelope | 所有响应公共头 |
| OpenStock | DAILY_KLINE 响应 | 最常用 fixture 路径 |

其他系统（Postgres / TDengine / MySQL / 文件源）只进清单层，不做字段 spot-check。

### L2 准入矩阵 (Admission Criteria)

"破坏契约是否导致生产事故/静默数据腐败"是历史判断，落地为以下可执行的判定规则。
**新增对象满足任意一条即必须进入 L2 字段级 spot-check**：

| 规则 | 例子 |
|------|------|
| **R1 涉及资金/订单/持仓** | 下单请求、撤单响应、持仓快照、清算回报 |
| **R2 行情主数据核心表** | kline / minute 系列 / tick / share |
| **R3 被 ≥3 个上游模块依赖的公共数据结构** | `OpenStockEnvelope`（被 importer / aggregator / shadow 同时读取） |
| **R4 近 3 个月内有字段变更** | git log 显示字段层改动即触发 |
| **R5 幂等/状态机的状态字段** | `import_state.status`、`BridgeTaskLifecycleStatus` 变体集 |

L2 变更记录在附录 D「分层准入变更记录」中维护，每次进/出 L2 写一行理由。

**L1（清单层）的准入门槛建制**：所有外部依赖（含 Postgres / TDengine / MySQL / 文件源）
必须进入 L1 清单与 L1 测试；不进 L1 等于脱离契约治理。L1 至少覆盖：源文件存在性、
表名清单、env 变量清单、关键 feature flag。

## Contract Version 策略

每个章节顶部一行：`**Contract Version:** vX.Y`

- **Major (vX)**：删除端点、删除表、删除/重命名必需字段、**字段语义变更**（见下）
- **Minor (vY)**：新增可选字段、新增端点、新增表

### 字段语义变更 = 破坏性变更

字段名不变、但下列任一属性变化，视为 **Major bump**：

| 属性 | 破坏性变更例子 |
|------|----------------|
| **类型大类** | `Decimal → Float64`（精度丢失）、`i64 → f64`（语义漂移）、`DateTime ↔ String`（解析失败） |
| **单位** | 成交量「手」↔「股」、价格「元」↔「分」、时间戳「秒」↔「毫秒」 |
| **语义** | `status` 字段的枚举值集合变化（新增非可选变体）、`code` 字段允许格式从 `600000` 扩展到 `sh600000` |
| **可选性** | 可选字段变为必填（反向变更视情况，但默认按 Major 处理） |

> 字段语义变更不会触发编译错误，但会导致策略计算结果静默失真。
> **类型大类校验在 L2 spot-check 中必须覆盖**（见「同步测试设计」Layer 2 章节）。

### 废弃流程 (Deprecation)

废弃字段/端点/变体时不直接删除，按以下流程：

1. **标记**：文档中加 `@deprecated v<版本>` + 替代方案；源码加 `#[deprecated]` 属性
2. **保留**：至少保留 1 个 Minor 版本周期（约 4 周），给下游适配时间
3. **移除**：下个 Major 版本移除；移除时在 PR checklist 显式列「废弃项清理」
4. **通知**：Bridge 服务方变更需通过 bridge_contract_version 字段 + BridgeFailureCode::LiveBridgeUnsupportedContractVersion 双向协商

### 版本协商 (Bridge 侧)

- 客户端请求带 `contract_version` 头（`miniqmt.v1`）
- 服务端返回 `BridgeFailureCode::LiveBridgeUnsupportedContractVersion` 时，客户端必须降级到只调用 capabilities 端点，不再下单
- 文档需记录「服务端拒绝 X.Y 时，客户端安全的只读端点集合」

## PR Checklist (附录 7.2)

```
修改外部系统对接代码时：
[ ] 增删端点 → 更新 §3.3 端点清单 → 测试: bridge_endpoints_in_doc_match_code (未实现，见 #1)
[ ] 增删表/列 → 更新 §5.2 表清单 → 测试: l1_full_inventory_all_paths_exist / l2_*_columns_match
[ ] 改鉴权/env 变量 → 更新 §3.6 / §4.6 / 附录 A → 测试: env_vars_in_doc_match_code
[ ] 改 BridgeQmt* / OpenStockEnvelope / 关键表字段 → 更新附录 C → 测试: l2_*_fields_match
[ ] 改字段类型/单位/语义 → bump Contract Version (Major) → 更新附录 C 类型列
[ ] 废弃字段 → 加 @deprecated 标记 + 保留 1 个 Minor 周期
[ ] cargo test --test contract_doc_sync_test --test contract_doc_field_sync_test 全绿
```

## 文件清单

新增文件：

| 路径 | 用途 | 行数预估 |
|------|------|---------|
| `docs/contracts/external-systems.md` | 契约文档主体 | 600-900 |
| `tests/contract_doc_sync_test.rs` | 同步测试 | 400-600 |

修改文件：无（纯新增）。文档与测试相互引用，不改动现有源码。

## 验证 (Verification)

- `cargo test --test contract_doc_sync_test` 全绿
- `cargo fmt --check` / `cargo clippy -- -D warnings` 通过
- 文档被人通读一遍，确认三个受众视角都能找到自己需要的信息
- 故意改一处代码（如新增 ClickHouse 表），确认测试失败并给出清晰提示

## 其他系统的最低契约治理（#6）

「其他外部依赖」（Postgres / TDengine / 上游 MySQL / 文件源）不做字段级 spot-check，
但**不得脱离契约治理**。最低要求：

| 系统 | L1 清单 | L1 测试 | 行为契约（≥1 条） |
|------|---------|---------|--------------------|
| Postgres | 表清单 + schema 名（`quantix.*`） | `postgres_tables_in_doc_match_code` | 写入幂等：所有 `live_import_*` 表用 `ON CONFLICT DO NOTHING` 或 dedupe 索引 |
| TDengine | supertable + 子表规则 | `tdengine_schema_in_doc_match_code` | 子表命名规则：`<symbol>_<period>`；只追加，不更新 |
| 上游 MySQL | 库名 + 只读边界 | `mysql_readonly_boundary_in_doc_match_code` | 严格只读；任何写操作必须经 review 在文档中显式列出 |
| 文件源（TDX / AkShare / EastMoney） | 路径模板 / 命名规则 | `file_source_paths_in_doc_match_code` | 文件命名约定（如 `SH600000.day`）+ 读取失败降级策略 |

env 变量扫描必须覆盖 `src/db/postgresql.rs` / `src/db/tdengine.rs` / `src/sources/*.rs`，
不止 3 个核心模块。`env_vars_in_doc_match_code` 测试用例应列出全部扫描路径。

## 延后决策 (Deferred)

以下建议长期成立但本期不实施，记录为未来演进方向：

### D1. 统一常量文件 + `syn` 解析（review #1）

**现状**：Bridge 端点是 `format!("{}/api/v1/task/execute", self.base_url)` 字面量散落在
`src/bridge/client.rs` 各方法中。OpenStock 用统一 `/data/fetch` + `data_category` 路由。
当前 L1 测试只做 `Path::exists` 校验，**不解析端点字面量**——所以 review 中担心的
"format! 拼接 / 重构常量化 → 正则失效"场景对当前测试不成立。

**延后理由**：把 13 个端点收敛到 `src/bridge/endpoints.rs` 常量是纯重构，需改 ~400 行
client.rs，触及 13 个调用点，**收益仅在后续要测端点集合时才兑现**。本期 spot-check 用
字段级 + Appendix C 锚点的方式抓结构性漂移；端点路径漂移靠 Bridge server 侧契约版本
头协商兜底。

**触发条件**：若未来 6 个月内出现 ≥2 次端点契约漂移事故，或开始测端点集合，
则启动常量化 + `syn` 解析。

### D2. 代码为单一事实源（SSOT）+ 文档自动生成（review #8）

**现状**：以 Markdown 为契约源、代码为实现、测试做对齐。

**延后理由**：codegen 方案在 review 中也明确标注"当前阶段可维持现状"。
当前 9 项 spot-check 维护成本可接受；code-gen 工具链（含类型注释语法、build script、
生成后 diff 校验）需要至少 1 人周投入。文档侧人工维护 + 强校验测试是当前 ROI 最优。

**触发条件**：spot-check 项扩展到 >20 项、或人工同步每月出错 ≥1 次。

## 风险与权衡 (Risks)

| 风险 | 缓解 |
|------|------|
| 文档与代码仍可能同步漂移（同 PR 都改但改错） | spot-check 字段级 + 类型大类测试是更细粒度的网 |
| 同步测试成为开发负担 | 清单层匹配快速且失败信息明确；spot-check 只 9 项 |
| 文档太长不易维护 | 三层结构 + Reference 颗粒度，避免过度展开 |
| Bridge server 端契约变化无法被代码侧捕获 | 不在本次范围；需 Bridge 服务方自己维护对外契约 |
| 字段语义变更静默通过（类型/单位漂移） | L2 类型大类校验 + 文档 §类型大类变更规则 |
| 其他系统治理盲区（Postgres/TDengine/MySQL） | §其他系统的最低契约治理 表强制 L1 + 至少 1 条行为约束 |

## 不在范围 (Out of Scope)

- 代码生成文档（codegen）—— 见 D2，延后决策
- 服务端契约验证（Bridge/OpenStock 自身的对外 spec）—— 由服务方维护
- 字段可选性的全量 spot-check —— churn 过大；L2 仅覆盖类型大类
- 历史版本演进表 —— 用 git log 即可追溯
