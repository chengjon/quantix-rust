# 审核意见：P0_11C_PREFLIGHT_AUDIT_2026-07-01.md

> 审核日期：2026-07-01
> 审核范围：`docs/reports/P0_11C_PREFLIGHT_AUDIT_2026-07-01.md` 全文
> 审核基线：HEAD `47747c5`，对照 `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/` 全域检索 + 逐文件代码核实

---

## 总体评价

这份前置审计在结构上做得很扎实——204 处引用全域检索、四大阻塞决策逐一展开、五个 Phase 的删除顺序编排严谨。尤其是 §四 Phase 1 必须在 Phase 4 之前的硬约束标注，以及 §二 Decision 2「删 tdx_api_handler.rs 之前必须先迁出 openstock 分支」，是真正的 safety-critical 判断。但有三处需要修正，一处需要补充。

---

## 必须修正

### 1. `src/core/config.rs` 从清理清单中遗漏（**关键**）

**问题**：报告 §一标题声称「11 个文件」，全域检索实际命中 **15 个文件**。其中遗漏的最重要的是 `src/core/config.rs`——包含 15 处 `TdxApi` 引用，不是注释、不是 dead code，是**仍被加载的配置结构体**：

- `DataSourceConfig.tdx_api: Option<TdxApiConfig>` 字段（L54）
- `pub struct TdxApiConfig` 结构体定义，含 5 个字段（L66-78）
- 6 个 `default_tdx_api_*` 函数（L81-113）

P0.11c 删除 `src/sources/tdx_api.rs` 后，`TdxApiConfig` 类型失去定义来源，但 `DataSourceConfig` 仍引用它——如果不清理，**`cargo build` 会编译失败**。

**其他遗漏文件**：

| 遗漏文件 | 引用数 | 严重程度 |
|----------|--------|---------|
| `src/core/config.rs` | 15 | **CRITICAL** — 功能性代码，删除 tdx_api.rs 后编译断裂 |
| `src/core/trading_calendar.rs` | 1 | LOW — 仅注释 `可由 tdx-api 调用方注入` |
| `tests/openstock_import_klines_live_test.rs` | 1 | LOW — 仅 doc comment `quantix data tdx-api import-klines` |

**建议**：在 §一增加「1.2a `src/core/config.rs` 局部移除」条目，删除 `TdxApiConfig` 结构体 + 6 个 default 函数 + `DataSourceConfig.tdx_api` 字段。同时把「11 个文件」更正为「15 个文件（3 个仅含注释/无功能影响）」。

---

### 2. Phase 4 删除行数低估——实际应 ~2300 行而非 ~1800 行

**问题**：报告 §六写 Phase 4 删 1 800 行，但仅两个主文件就超出：

| 文件 | 实际行数 |
|------|---------|
| `src/sources/tdx_api.rs` | 1 309 |
| `src/cli/handlers/tdx_api_handler.rs` | 726 |
| **小计（整文件删除）** | **2 035** |

再加上局部删除：`data_handler.rs` 的 `tdx_api_*` helper（~140 行）、`commands/data.rs` 的 `TdxApiCommands` 枚举（~30 行）、`config.rs` 的 `TdxApiConfig`（~60 行）、`command_types.rs` 的 `DataSourceKind::TdxApi`（~2 行）——Phase 4 的删除量实际在 **~2 300 行**，而非 1 800 行。

报告 §六的净删量（-2 130 行）也需要上调到 ~-2 500 行。这不影响可行性判断，但影响估时准确性。

**建议**：把 §六 Phase 4 行数从「-1 800」修正为「-2 300」，总计净删从「1 720」修正为「~2 100」。

---

### 3. Phase 3（scheduler reroute）工作量估计与实际复杂度不一致

**问题**：报告 §六给 Phase 3 估 1.5 天 +200/-50 行，但 §二 Decision 1 自己写的是：

> Option A（rewire）— 高（需 P0.11d 规模的新 parser 切片）

Decision 1 明确说这是**新 parser 切片**级别的工作，但 Phase 3 把它压缩成一个 phase 子步骤（3c.7-3c.9），且估计仅有 3 个文件改动 + 200 行新增。

实际需要的工作量：

- 新增 `OpenStockClient::fetch_realtime_quotes()` wrapper → 需先 live-verify `REALTIME_QUOTES` category（design.md R5 标注「未 live-verified, shape 未知」）
- 新写 `src/sources/openstock_quotes.rs` parser → 独立模块，对标 `openstock_ticks.rs`（297 行）的规模
- `collect_scheduler.rs` 重接 → `TdxApiClient::collect_all_quotes()` 的返回类型与 OpenStock 的 category-based fetch 完全不同，需要 adapter 层

这是一个完整的 P0.11d 切片，不是 1.5 天能完成的。如果选 Decision 1 Option B（直接删 fallback），Phase 3 确实 0.5 天——但报告自己推荐的是 Option A。

**建议**：要么把 Phase 3 的默认估时上调到与 §二 Decision 1 一致（标记为 P0.11d 独立切片，3-5 天），要么把默认推荐从 Option A 改为 Option B（删除 fallback）以匹配 3.5 天的总估时。两者不能同时成立。

---

## 需要补充

### 4. Phase 2 硬编码了 Decision 3 Option B——其他选项的序列未展开

**问题**：§四 Phase 2 标题写「Decision 3 选 B 时」，步骤也全部基于 Option B（加 `direction` 列 + 改写入逻辑）。但如果用户选了 Option A（统一映射）或 Option C（`source` tag），Phase 2 的内容完全不同：

- Option A → 需要反向工程 tdx-api status 字节 + backfill 历史数据，Phase 2 膨胀到 ~2 天
- Option C → 加 `source VARCHAR` tag 列，与 Option B 工作量相当但 schema 不同

当前 §四只展开了一个分支，其他两个选项的序列没有对应编排。

**建议**：在 §四 Phase 2 前加一句「以下按默认推荐 Decision 3=B 展开；如选 A 或 C，Phase 2 的步骤和估时需要调整」，并附简表说明差异。

---

## 已验证准确的部分

| 报告声明 | 结论 | 证据 |
|----------|------|------|
| 全域检索 204 处引用 | ✅ | `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/ --include="*.rs" \| wc -l` → 204 |
| `tdx_api.rs` 1309 行 | ✅ | `wc -l` 确认 |
| `tdx_api_handler.rs` 726 行（设计文档曾写 476） | ✅ | `wc -l` 确认 726；P0.11a/b 期间膨胀正确归因 |
| `data_handler.rs` 795 行 | ✅ | `wc -l` 确认 |
| `collect_scheduler.rs` fallback 代码片段 | ✅ | L258-270 与报告代码逐字一致 |
| `data_handler.rs` 的 `DataSourceKind::TdxApi` 分支位置 | ✅ | L279/L300/L347 三处匹配 |
| Decision 2 硬约束：先迁出再删除 | ✅ | 正确且关键——如果跳过 Phase 1，openstock 的 import-* 分支会随文件删除丢失 |
| Decision 4 CLI 层级推荐 Option A | ✅ | design.md D5 也推荐顶层 promote，与报告一致 |
| 风险 R7（scheduler fallback） | ✅ | 真实风险，报告正确标注 |
| 风险 R9（CLI 路径变更破坏脚本） | ✅ | 报告正确建议 deprecation alias |

---

## 对 §七审核要点的逐条回应

| # | 审核要点 | 意见 |
|---|---------|------|
| 1 | Decision 1（scheduler fallback） | 选 A 需要 P0.11d 独立切片（见上文必须修正 3）；选 B 需确认无生产自动化依赖 scheduler。建议先做 B 快速删，P0.11d 再补 rewire |
| 2 | Decision 2（迁出目标） | Option A（`openstock_handler.rs`）合理——最小改动 |
| 3 | Decision 3（direction 列） | Option B（拆 `direction` + `status` 双列）语义最清晰。但需确认 TDengine schema migration 在 staging 环境可测试 |
| 4 | Decision 4（CLI 层级） | Option A 正确。deprecation alias 一个 release 是好的过渡策略（报告 R9 已提及） |
| 5 | 2b.10 live 验证 | 阻塞项，必须先跑通才能进 Phase 1 |
| 6 | 是否启动 Phase 1 | 需先完成：① 2b.10 通过 ② 四大决策确认 ③ 修正本审计的遗漏项后。Phase 1 本身是安全的（仅迁出，不删除） |

---

## 总结

| 维度 | 评价 |
|------|------|
| 全域检索完整性 | ⚠️ 遗漏 `src/core/config.rs`（功能性代码，编译关键路径） |
| 删除顺序编排 | ✅ §四五阶段顺序严谨，硬约束标注到位 |
| 估时准确性 | ⚠️ Phase 3（scheduler reroute）与 Decision 1 的复杂度描述自相矛盾；Phase 4 行数低估 ~500 行 |
| 决策框架 | ✅ 每个 decision 三选项对比，推荐默认值明确 |
| 风险覆盖 | ✅ 继承 design.md R1-R6 + 新增 R7-R9，矩阵完整 |

**结论**：修正三处问题（补 `config.rs` 到清理清单、上调估时/行数、澄清 Phase 3 复杂度）后，这份审计即可作为 P0.11c 的启动依据。当前状态下建议先解决 Decision 1 的分歧——要么选 B（删 fallback）让 Phase 3 降到 0.5 天、总计 ~2.5 天，要么承认 Phase 3 是独立 P0.11d 切片并重新排期。
