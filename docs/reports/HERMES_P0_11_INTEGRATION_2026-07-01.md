# Hermes × P0.11 线整合分析

> 日期：2026-07-01
> 范围：将 Hermes 线（P1-P4，已应用未提交）的成果纳入 P0.11 编排，统一后续路线
> 基线：HEAD `eae7887` + 工作区 Hermes 未提交改动 + P0.11a `d5e9b75` + P0.11b `47747c5`/`c16ea8e`

---

## 一、两线交付物对照

### Hermes 线（P1-P4，2026-07-01 应用）

**核心改动：5 个 handler 切换到 OpenStock + ClickHouse day_kline 优先路由**

| 文件 | 改动 |
|------|------|
| `.env` | 追加 `OPENSTOCK_BASE_URL` + `OPENSTOCK_API_KEY=dev`（已修正为真实 key） |
| `src/db/clickhouse/kline.rs` | `get_kline_data()` 路由：day_kline 优先 → kline_data fallback；新增 `query_day_kline()` |
| `src/sources/openstock_market.rs`（新建）| `OpenStockMarketReader`：SECTOR_QUOTES / UPDOWN_DISTRIBUTION / NORTHBOUND_FLOW |
| `src/sources/openstock_client.rs` | 新增 `fetch_daily_klines()` 直连 `/data/bars` |
| `src/cli/handlers/shared_support.rs` | 新增 `get_kline_for_analysis()` 统一入口 |
| `src/cli/handlers/{backtest,analyze,data,market,screener}.rs` + `strategy_handler/catalog.rs` | 切到 `get_kline_for_analysis` / `OpenStockMarketReader` |

**未触及**：`TdxApiClient`、`collect_scheduler.rs`、`tdx_api_handler.rs`、`src/sources/tdx_api.rs`。

### P0.11 线（a/b 已合并，c 待启动）

| 子切片 | 状态 | 涉及 |
|--------|------|------|
| P0.11a（klines） | ✅ commit `d5e9b75` | `import-klines --source openstock` → ClickHouse `kline_data` 表 |
| P0.11b（ticks） | ✅ commit `47747c5` + `c16ea8e` | `import-ticks --source openstock` → TDengine；parser `openstock_ticks.rs` |
| P0.11c（删除） | 🔲 待启动 | 删 `TdxApiClient` + 18 CLI 子命令 + `tdx_api.rs` 1309 行 + `tdx_api_handler.rs` 726 行 + `config.rs::TdxApiConfig` |

---

## 二、交集分析：两线是否冲突？

**结论：无冲突。两线是正交的。**

| 维度 | Hermes 线 | P0.11 线 | 冲突? |
|------|-----------|----------|------|
| CLI 命令面 | `market`、`backtest`、`analyze`、`data query`、`screener`、`strategy` | `data tdx-api import-{klines,ticks}` | 无 — 不同命令树 |
| OpenStock 端点 | `/data/bars`（直连）、`/data/fetch`（SECTOR_QUOTES 等） | `/data/fetch`（INDEX_KLINES、HISTORICAL_KLINES、TICK_DATA） | 无 — 端点共存 |
| ClickHouse 写入 | 无（只读） | P0.11a 写 `kline_data`、P0.11b 写 TDengine | 无 |
| ClickHouse 读取 | `day_kline` 表（1150 万行历史） | P0.11a 不读，只写 | 互补 |
| TdxApiClient | 不引用 | P0.11c 删除 | **无新增引用** — grep 全域仍 204 处，与 P0.11c 准入审计一致 |
| 环境变量 | `OPENSTOCK_BASE_URL`、`OPENSTOCK_API_KEY` | 同上 + `QUANTIX_OPENSTOCK_KLINE_APPLY`、`QUANTIX_OPENSTOCK_TICK_APPLY` | 共享 base/key，无冲突 |

**关键证据**：
```
$ grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/ --include="*.rs" | wc -l
204                                                    ← 与 P0.11c 审计基线一致
```

Hermes 线**完全是 additive**：新增 OpenStock 消费者，不新增也不删除 tdx-api 引用。P0.11c 的删除范围（15 个文件 / 204 处引用）**完全不受影响**。

---

## 三、Hermes 线遗留的小修（本次完成）

| # | 问题 | 修复 |
|---|------|------|
| 1 | `.env` 中 `OPENSTOCK_API_KEY=dev`，但 live server 需要真实 key | 改为 `sk-ICdVlZ72X59SAIm2TfX3ZzM9qLan5bk5` |
| 2 | `screener_handler.rs::ClickHouseDailyKlineLoader.client` 字段 dead_code（P4 切换后未用） | 删字段、`new()` 改无参；同步更新 3 个 call site（`screener_handler.rs:23`、`strategy_handler.rs:584`、`strategy_handler/catalog.rs:29`） |
| 3 | `openstock_client.rs::BarRecord.symbol` 字段 dead_code（`fetch_daily_klines` 内部 struct） | 删字段 |

验证：
```
$ cargo check --workspace
cargo build (1 crates compiled)                      ← 0 warning
$ cargo test --workspace --lib
cargo test: 765 passed (1 suite, 12.48s)             ← 无回归
```

---

## 四、对 P0.11c 的影响

**直接影响：零。** Hermes 线未改变任何 P0.11c 的准入条件或删除范围。

**间接影响（正面）**：
1. P0.11c Decision 1（scheduler fallback）现在**风险更低** — Hermes 已证明 OpenStock `/data/bars` + `get_kline_for_analysis` 路由稳定（765 测试通过 + 5 个 handler 验证），同样的 fallback 模式可应用于 `collect_scheduler`。
2. P0.11c Phase 3（scheduler reroute to OpenStock）可以**复用 Hermes 的 `OpenStockClient` 扩展模式**（`fetch_*` wrapper + 内部 `BarRecord` 反序列化），不需要从零写。
3. ClickHouse `day_kline` 表的 1150 万行真实数据 + Hermes 的 `query_day_kline()` 路由 = **P0.11c 后 OpenStock 偶尔失败时的天然兜底**，不需要 `tdx-api` 备份数据。

---

## 五、合并后的统一路线

```
P0.11a/b ✅  →  Hermes P1-P4 ✅  →  P0.11c (启动)  →  Closeout
   ↑                ↑                      ↑
   已合并           工作区（本次修正）        待 Decision 1-4 确认
```

**P0.11c 准入检查（更新版）**：

| 条件 | 状态 |
|------|------|
| P0.11a 合并 | ✅ commit `d5e9b75` |
| P0.11b 合并 | ✅ commit `47747c5` + `c16ea8e` |
| 2b.10 live smoke | ✅ 2026-07-01 通过（`openstock:8040`，600000/20260630） |
| **Hermes 线 build green** | ✅ 0 warning，765 测试通过 |
| **Hermes 线不引入新 tdx-api 引用** | ✅ 全域仍 204 处 |
| Decision 1-4 确认 | 🔲 待用户 |

**建议**：把 Hermes 的工作区改动作为**独立 commit** 先落地（它已经是完整的、测试通过的、与 P0.11c 解耦的），然后再启动 P0.11c。这样：
- Hermes 线的 P1-P4 成果不与 P0.11c 的删除改动混在一个 PR 里（评审负担）
- P0.11c 启动时 Hermes 改动已在 master，可用 `gitnexus detect_changes` 干净对照
- 若 P0.11c Phase 3 需要 scheduler reroute，Hermes 的 `OpenStockClient` 扩展已是上游稳定的依赖

---

## 六、Commit 拆分建议（Hermes 线落地）

| Commit | 范围 | 类型 |
|--------|------|------|
| 1 | `.env` 真实 key + Hermes P1-P4 + 本次 3 处小修（dead_code 清理） | `feat(data): wire market/backtest/analyze/screener/strategy to OpenStock with ClickHouse day_kline fallback` |

或更细粒度拆 3 个（P1+P2 一个、P3 一个、P4 一个）— 取决于评审偏好。建议单 commit，因为：
- 改动主题统一（OpenStock 数据源切换）
- 跨 5 个 handler 的小改动单独 commit 反而难审
- 765 测试一次性通过，无需分阶段验证
