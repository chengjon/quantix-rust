# P0.11c 前置审计报告 — TdxApiClient 全量下线准备

> **会话日期**: 2026-07-01
> **OpenSpec 变更**: `openstock-data-consumption-p0-11`（第三个子片 P0.11c）
> **状态**: 审计完成（r2 已应用审核反馈），待四大架构决策 + 2b.10 live 验证后启动代码清理
> **前置文档**: `P0_11B_IMPLEMENTATION_REVIEW_2026-07-01.md`（P0.11b 代码已完成）
> **设计依据**: `openspec/changes/openstock-data-consumption-p0-11/design.md` D1 / D2 / D3 / D5
>
> **修订**: r2 — 已应用 `P0_11C_PREFLIGHT_AUDIT_REVIEW_2026-07-01.md` 四条意见：① §一 补 `src/core/config.rs` 等 4 个遗漏文件（CRITICAL）；② §四 Phase 2 增加选项分支说明；③ §六 Phase 3/4 估时与行数修正；④ §二 Decision 1 与 §六 Phase 3 的复杂度口径对齐。

---

## 一、清理范围全量清单

全域检索 `tdx_api|TdxApi|tdx-api`（仅 `*.rs`）：**204 处引用，15 个文件**。其中 4 个文件仅含注释/doc-comment（无功能影响），功能性代码集中在 11 个文件。按改动类型分三类：

### 1.1 整文件删除（2 个文件 / 2 035 行）

| 文件 | 总行数 | TdxApi 引用 | 备注 |
|---|---|---|---|
| `src/sources/tdx_api.rs` | 1 309 | 62 | `TdxApiClient` 本体 + 33 个方法。删除前必须先确认没有外部调用残留 |
| `src/cli/handlers/tdx_api_handler.rs` | **726**（注 1） | 34 | 18 个 CLI subcommand handler。**注 1**：design.md D2 记为 476 行，但 P0.11a/b 期间把 `import-klines` / `import-ticks` 的 openstock 分支也写进了本文件，目前膨胀到 726 行。删除前必须先把这两个 openstock 分支迁出（见 §三） |

### 1.2 局部移除（8 个文件 / 204 引用中的 ~167 处）

| 文件 | 总行数 | TdxApi 引用 | 移除内容 |
|---|---|---|---|
| `src/cli/handlers/data_handler.rs` | 795 | 45 | L279-281 `DataSourceKind::TdxApi` add-source 分支、L300 配置检查、L347-351 健康检查、L42-47 `PersistedTdxApiConfig` 序列化结构、L543-605 全部 `tdx_api_*` helper（`tdx_api_configured` / `tdx_api_env_url` / `format_tdx_api_*` / `default_tdx_api_*`）、L678-712 两个 `set_default_tdx_api_*` 测试 |
| **`src/core/config.rs`** ⚠️ **CRITICAL** | - | 15 | **L54 `DataSourceConfig.tdx_api: Option<TdxApiConfig>` 字段、L66-78 `pub struct TdxApiConfig` 结构体（5 字段）、L81-113 六个 `default_tdx_api_*` 函数**。删除 `src/sources/tdx_api.rs` 后 `TdxApiConfig` 失去定义来源，但 `DataSourceConfig` 仍引用它——**不清理则 `cargo build` 编译断裂**。审核意见 1 指出本文件必须列入清理清单 |
| `src/cli/tests/data.rs` | 292 | 14 | TdxApi 相关的 CLI 集成测试用例 |
| `src/cli/commands/data.rs` | 538 | 11 | `TdxApiCommands` 枚举 + `TdxApi` parent variant；**保留** `ImportTicks` / `ImportKlines`（按 D5 决策 promote 到 `DataCommands` 顶层） |
| `src/sources/mod.rs` | 67 | 4 | `pub mod tdx_api;` 声明 + re-export |
| `src/cli/handlers/mod.rs` | 190 | 2 | `pub mod tdx_api_handler;` + re-export |
| `src/cli/command_types.rs` | 60 | 1 | `DataSourceKind::TdxApi` 枚举变体删除 |
| `src/cli/mod.rs` | 27 | 1 | 模块注册（待 Phase 4 grep 审计时复核确认具体行） |

### 1.3 逻辑 reroute（2 个文件 / 关键架构点）

| 文件 | 总行数 | TdxApi 引用 | reroute 内容 |
|---|---|---|---|
| `src/tasks/collect_scheduler.rs` | 368 | 10 | L83 `tdx_api_fallback` 字段、L104/121 初始化、L136 `set_tdx_api_fallback` setter、**L262 fallback 读取（生产路径，主采集器失败时切 HTTP 备用）** — 见 §二 Decision 1 |
| `src/cli/handlers/app_shell.rs` | 864 | 2 | `TdxApi(cmd) => ...` dispatcher 分支删除；如 D5 选顶层 promote，需新增 `DataCommands::ImportTicks` / `ImportKlines` 分支 |

### 1.4 仅注释 / doc-comment（3 个文件 / 无功能影响）

| 文件 | 引用 | 内容 | 处置 |
|---|---|---|---|
| `src/core/trading_calendar.rs` | 1 | L387 `/// 从交易日列表同步日历 (可由 tdx-api 调用方注入)` 注释 | P0.11c 顺手清理注释（不影响编译） |
| `tests/openstock_tick_data_live_test.rs` | 1 | doc comment 提及 tdx-api 历史路径 | 配合 Decision 4 CLI 改名时一并更新 |
| `tests/openstock_import_klines_live_test.rs` | 1 | doc comment `quantix data tdx-api import-klines` | 配合 Decision 4 CLI 改名（`tdx-api` parent 移除后，test 名也建议改 `openstock_import_klines_live_test` → 保留以减少 git rename 噪声，仅更新 doc） |

### 1.5 总数核对

- **15 个文件** = §1.1（2 整删）+ §1.2（8 局部移除）+ §1.3（2 reroute）+ §1.4（3 仅注释）= **15 唯一文件**
- **204 处引用** = 上述 15 文件汇总，无遗漏

---

## 二、四大阻塞决策点

四个决策都需要用户基于运维现状判断，无法自动选择。每个决策的 Option 对比、推荐默认值、风险如下。

### Decision 1：`collect_scheduler` fallback 处置（D2 Option A vs B）

**现状**：`src/tasks/collect_scheduler.rs:258-270` 在主 TDX 协议采集器 `collector.collect_all()` 失败时，回退到 `tdx_api_fallback` 字段持有的 `TdxApiClient::collect_all_quotes()` HTTP 备用采集。这是**生产路径**，不是死代码。

```rust
// L258-270 当前实现
let quotes = match self.collector.collect_all(&stocks).await {
    Ok(q) => q,
    Err(e) => {
        warn!("TDX 协议采集失败: {e}, 尝试 tdx-api 备用");
        let fb = self.tdx_api_fallback.read().await;
        if let Some(ref api) = *fb {
            api.collect_all_quotes(&codes).await?   // ← HTTP 备用
        } else {
            return Err(e);
        }
    }
};
```

| Option | 做法 | 工作量 | 风险 |
|---|---|---|---|
| **A（rewire）** | 改持有 `OpenStockClient`，调用新增的 `fetch_realtime_quotes()` wrapper + 新写 `parse_realtime_quotes()` parser | 高（需 P0.11d 规模的新 parser 切片） | OpenStock `REALTIME_QUOTES` 类别未 live-verified（design.md R5），shape 未知；fallback 在主采集失败时才触发，bug 不易在测试中暴露 |
| **B（删除）** | 直接删 `tdx_api_fallback` 字段、setter、读取点；主采集失败直接 `Err` 冒泡 | 低（~15 行改动） | **高运行时风险**：主采集器任何失败都会导致整批 quote 丢失，没有兜底；如果 scheduler 在生产自动化里运行，必须先确认有上层重试 |
| **C（保留 legacy tdx-api 作为 fallback）** | 不删 fallback，仅删 import-* 命令 | 0 改动 | 与 P0.11 整体目标冲突（仍需 `TdxApiClient` 存活），违背减法切片意图 |

**默认推荐**：**Option A**，但**前置**于 P0.11c 之前做一个独立 mini-slice（P0.11b.5？）live-verify `REALTIME_QUOTES`，确认 shape 后再 rewire。否则选 B 并接受失去兜底。

**回滚成本**：A 高（新 parser 写完后才发现 shape 不对，回滚要拆两处）；B 低（git revert 一个 commit）。

---

### Decision 2：`tdx_api_handler.rs` 的 openstock 分支迁出（执行顺序约束）

**现状**：P0.11a/b 把 `import-klines` 和 `import-ticks` 的 openstock 分支写在 `tdx_api_handler.rs` 内（与 legacy 分支并列）。P0.11c 要整文件删除，**必须先迁出这两个分支**，否则删除时会把 openstock 路径一起删掉，导致主功能丢失。

| Option | 迁出目标 | 工作量 |
|---|---|---|
| **A** | 迁到 `src/cli/handlers/openstock_handler.rs`（现有文件，与 P0.9/10 fetch handlers 同处） | 低：~120 行搬运 |
| **B** | 新建 `src/cli/handlers/import_handler.rs`（专门给 import-* 命令） | 中：新模块声明 + re-export |
| **C** | 按 D5 选顶层 promote 后，handler 也跟着提到 `data_handler.rs` 内 | 中：与 data_handler 已有的 791 行混合，文件超 800 行告警 |

**默认推荐**：**Option A**，最小改动。配合 D5 Decision 4 的 promote，handler 函数名从 `import_ticks` / `import_klines` 保持不变，仅 `app_shell.rs` dispatcher 改路由。

**硬约束**：删除 `tdx_api_handler.rs` 之前必须完成迁出，否则 broken main。tasks.md 3c 顺序需要显式编排（见 §四）。

---

### Decision 3：TDengine `direction` 列语义统一（D3 Decision 2）

**现状**：P0.11b 已 live-verified — `tdx_api_handler.rs::import_ticks` 的 openstock 分支写入 `direction` 列时映射 `Buy=1 / Sell=-1 / Neutral=0`（来自 `TradeDirection` 枚举）；同一文件的 legacy 分支写入 `t.status`（tdx-api 协议原始字节，语义未在 quantix 内说明）。两条路径写同一 TDengine `tick_data.direction` 列，值不可比。

P0.11c 删除 legacy 路径后，列里残留两批语义冲突的历史数据。

| Option | 做法 | 工作量 | 数据迁移 |
|---|---|---|---|
| **A（统一映射）** | 反向工程 tdx-api status 字节的真实含义，统一 legacy + openstock 映射 | 高（需要查 tdx-api 协议文档或代码） | 需要 backfill 历史数据 |
| **B（拆列）** | TDengine schema 加 `direction TINYINT`（新列，存 OpenStock 的 1/-1/0），保留 `status TINYINT` 给 legacy 字节 | 中（schema migration + 新写入逻辑） | 无需 backfill（物理隔离） |
| **C（source 标签）** | 加 `source VARCHAR` tag 列，downstream SQL 按 source 过滤 | 中（schema migration + 写入加 tag） | 无需 backfill（tag 维度区分） |

**默认推荐**：**Option B**（design.md 也推荐）。语义最清晰，下游消费代码改 1 个列名即可；接受一次性 TDengine schema migration 成本。

**前置依赖**：决策必须在 Phase 3（scheduler reroute + tick 写入分支）之前完成，否则改完 schema 又要回头改 handler。

---

### Decision 4：CLI 命令层级（D5）

**现状**：`quantix data tdx-api import-ticks` / `quantix data tdx-api import-klines`。删除 `TdxApi` parent 后，两个命令无家可归。

| Option | 新路径 | 工作量 | 用户体验 |
|---|---|---|---|
| **A（顶层 promote）** | `quantix data import-ticks` / `quantix data import-klines` | 低：`DataCommands` 枚举加 2 个 variant | 最佳（无 parent） |
| **B（迁到 openstock parent）** | `quantix data openstock import-ticks` | 中：复用现有 `OpenStockCommands` 枚举 | 命令名仍带 provider 信息，但 P0.11c 后 openstock 是唯一源，parent 多余 |
| **C（保持 tdx-api parent 占位）** | 不删 parent，仅清空内容 | 低 | 违背减法意图，命令名误导 |

**默认推荐**：**Option A**（design.md D5 也推荐）。CLI 破坏性变更需要更新 `docs/CLI_COMMAND_MANUAL.html` 的 section id（`cmd-data-tdx-api-import-ticks` → `cmd-data-import-ticks`）、侧边导航、目录条目、所有使用示例。

---

## 三、P0.11c 启动硬性前置条件

按 tasks.md 和 design.md D1 三子片约束推导，P0.11c 启动前必须满足：

### 3.1 代码层前置

1. ✅ P0.11a 已合并（commit `d5e9b75`）
2. ✅ P0.11b 代码已完成（commit `47747c5` + 审计修复 `c16ea8e`）
3. ⏳ **2b.10 live 验证未执行** — tasks.md 明确标注为 P0.11c 准入条件：
   ```bash
   QUANTIX_OPENSTOCK_LIVE=1 \
   OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
   OPENSTOCK_API_KEY=<key> \
   cargo test --test openstock_tick_data_live_test -- --ignored
   ```
   需要用户确认运行（需要 live 环境可达 + API key 有效）

### 3.2 决策层前置

4. ⏳ §二 Decision 1（scheduler fallback A/B/C）确认
5. ⏳ §二 Decision 2（openstock 分支迁出目标 A/B/C）确认
6. ⏳ §二 Decision 3（direction 列 A/B/C）确认
7. ⏳ §二 Decision 4（CLI 层级 A/B/C）确认

### 3.3 文档层前置

8. ⏳ P0.11b spec.md / design.md 补遗完成（commit `1c0f712`）✅
9. ⏳ 本审计文档归档 ✅

---

## 四、删除顺序编排（safety-critical）

P0.11c 是**减法切片**，删除顺序错误会留下 broken main。基于 design.md D1 三子片约束 + §二 Decision 2 的迁出硬约束，安全顺序分为 6 个 Phase。**精确步骤编号以 `tasks.md` §3c 为唯一权威源**（共 33 步，3c.1-3c.33）；本文档只标 Phase 名称与步骤要点，不重复编号，避免跨文档编号漂移。

### Phase 0 — 前置（tasks.md 3c.1-3c.6）

| 步骤 | 内容 | 状态 |
|---|---|---|
| 3c.1 | P0.11a + P0.11b 已合并验证 | ✅ |
| 3c.2 | **2b.10 live smoke 通过**（阻塞项） | ⏳ |
| 3c.3-3c.6 | Decision 1-4 全部确认 | ⏳ |

### Phase 1 — OpenStock 分支迁出（tasks.md 3c.7-3c.10，safety-critical）

必须先于 Phase 4。删除 `tdx_api_handler.rs` 前，P0.11a/b 写入该文件的 openstock 分支必须先迁到 `openstock_handler.rs`（Decision 2 = A 时）；`app_shell.rs` dispatcher 重路由到新位置（Decision 4 = A 时新增 `DataCommands::ImportTicks`/`ImportKlines` 顶层 variant）；迁出后 `cargo build + cargo test --workspace` 全绿。

### Phase 2 — TDengine schema 准备（tasks.md 3c.11-3c.13）

按 Decision 3 = B 展开。其他选项的步骤差异见 tasks.md Phase 2 标题说明：
- A（统一映射）：反向工程 status 字节 + backfill 历史数据，~2 天
- B（拆列）⭐默认：加 `direction TINYINT` 列 + 改 import_ticks 写入逻辑，~0.5 天
- C（source tag）：加 `source VARCHAR` tag 列，~0.5 天

**前置依赖**：决策必须在 Phase 3（scheduler reroute + tick 写入分支）之前完成，否则改完 schema 又要回头改 handler。

### Phase 3 — scheduler reroute（tasks.md 3c.14-3c.18）

⚠️ Decision 1 = A 时才执行。**实为 P0.11d 规模独立切片**（详见 §二 Decision 1），3-5 天：live-verify `REALTIME_QUOTES` → 新增 `OpenStockClient::fetch_realtime_quotes` wrapper → 新建 `src/sources/openstock_quotes.rs` parser 模块（对标 `openstock_ticks.rs` 297 行规模）→ rewire `collect_scheduler.rs` 字段类型与方法名，需 adapter 层转换 `TdxApiClient::collect_all_quotes` 返回类型为 OpenStock category-based fetch 语义。

Decision 1 = B 时跳过本 Phase。

### Phase 4 — 删除（tasks.md 3c.19-3c.29）

此时 legacy 已无生产引用。删除顺序（精确编号见 tasks.md）：
- `src/sources/tdx_api.rs`（1 309 行）+ `src/sources/mod.rs` 声明
- `src/cli/handlers/tdx_api_handler.rs`（726 行）+ `src/cli/handlers/mod.rs` 声明
- `src/cli/commands/data.rs` 的 `TdxApiCommands` 枚举 + `TdxApi` parent variant（保留 `ImportTicks`/`ImportKlines`）
- `src/cli/command_types.rs` 的 `DataSourceKind::TdxApi` 变体
- `src/cli/handlers/app_shell.rs` 的 `TdxApi(cmd) => ...` dispatcher 死分支
- `src/cli/handlers/data_handler.rs` 的全部 `tdx_api_*` helper + 测试
- ⚠️ **CRITICAL**：`src/core/config.rs` 的 `TdxApiConfig` 结构体 + 6 个 default 函数 + `DataSourceConfig.tdx_api` 字段
- §1.4 三个仅注释文件（`trading_calendar.rs:387`、两个 live test doc）
- grep 审计 + `cargo build --workspace + cargo test --workspace` 全绿
- `gitnexus detect_changes` 验证无意外文件

### Phase 5 — Ecosystem cleanup（tasks.md 3c.30-3c.33，main 已稳定后）

`docker-compose.yml` 注释 tdx-api 服务块 → `FUNCTION_TREE.md` 五处更新（L95/L212/L658/L781/L1126）→ `docs/CLI_COMMAND_MANUAL.html` 删除旧 section + 新增 `cmd-data-import-*` section → README/CHANGELOG/TDX_API_BRIDGE_GUIDE.md 加 deprecation banner。

**关键顺序约束**：
- Phase 1 不能跳过 → 否则删 `tdx_api_handler.rs` 时一并删掉 openstock 分支
- **Phase 4 删除 `config.rs::TdxApiConfig` 必须与删 `tdx_api.rs` 同步** → 否则 `DataSourceConfig.tdx_api` 字段引用悬空，`cargo build` 断裂（审核意见 1）
- Phase 4 必须在 Phase 1/2/3 全绿后 → 否则编译断裂
- Phase 5 在 Phase 4 后 → 文档与代码一致
- Phase 4 必须在 Phase 1/2/3 全绿后 → 否则编译断裂
- Phase 5 在 Phase 4 后 → 文档与代码一致

---

## 五、风险矩阵

继承 design.md R1-R6，新增 R7-R9（来自本审计发现的额外风险）：

| ID | Risk | Mitigation |
|----|------|-----------|
| R1-R6 | （见 design.md Risks Summary） | （见 design.md） |
| **R7** | **scheduler fallback 删除后主采集器失败无兜底**（Decision 1 Option B 风险） | 选 Option A；若选 B，先确认上层有重试机制，且增加采集失败告警 |
| **R8** | **TDengine schema migration 不可逆**（Decision 3 Option B 加列后，回滚不能删列） | 先在 staging 环境验证；准备 backfill 脚本以反向兼容 |
| **R9** | **CLI 路径变更破坏外部脚本**（Decision 4 选 A 后 `quantix data tdx-api import-*` 不存在） | 在 CHANGELOG 显式标注；保留一个 release 的 deprecation warning；考虑保持 `tdx-api` parent 作为隐藏 alias 一个版本 |

---

## 六、工作量预估

按上述 5 phase 拆解，假设 Decision 1=A / Decision 2=A / Decision 3=B / Decision 4=A（全部默认推荐）：

| Phase | 文件改动数 | 代码行变化 | commits | 估时 |
|---|---|---|---|---|
| Phase 1（迁出） | 4 | +120 / -120 | 2 | 0.5 天 |
| Phase 2（schema，Decision 3 = B） | 2 | +30 / -10 | 1 | 0.5 天 |
| Phase 3（scheduler reroute，**实为 P0.11d 独立切片**） | 3 | +400 / -50 | 3 | **3-5 天**（含 live-verify REALTIME_QUOTES + 新 parser 模块） |
| Phase 4（删除） | 9 | **-2 300** | 3 | 1 天 |
| Phase 5（文档） | 5 | +60 / -150 | 1 | 0.5 天 |
| **总计** | **23** | **+610 / -2 630**（净删 ~2 020 行） | **10** | **5.5-7.5 天**（Decision 1 选 A 时）/ **2.5 天**（Decision 1 选 B 时） |

**审核意见 2/3 修正说明**：
- Phase 4 行数从 `-1 800` 上调到 `-2 300`：仅 `tdx_api.rs`（1 309）+ `tdx_api_handler.rs`（726）两个整文件就 2 035 行，加上 `data_handler.rs` 的 ~140 行 helper、`commands/data.rs` 的 ~30 行枚举、`config.rs` 的 ~60 行配置结构、`command_types.rs` 的 ~2 行变体——合计 ~2 300 行。
- Phase 3 从 1.5 天上调到 3-5 天：与 §二 Decision 1 的「Option A 工作量：高（需 P0.11d 规模新 parser 切片）」口径对齐。新 parser 模块（`src/sources/openstock_quotes.rs`）需对标 `openstock_ticks.rs`（297 行）规模，且 REALTIME_QUOTES category 未 live-verified，需先做 smoke。
- **决策建议**：若想保持原 3.5 天总估时，则 Decision 1 应改选 B（删 fallback），Phase 3 降到 0.5 天，总计 2.5 天，但运行时失去 fallback 兜底（见 R7）。两种路径不可同时成立，需用户决策。

---

## 七、审核要点

请重点确认以下六项：

1. **Decision 1（scheduler fallback）**：当前 `collect_scheduler` 是否在生产自动化里运行？若是，强烈建议 Option A（rewire 到 OpenStock）；若否，Option B（直接删）可接受。
2. **Decision 2（openstock 分支迁出目标）**：是否同意 Option A（迁到现有 `openstock_handler.rs`）？还是希望另立 `import_handler.rs`？
3. **Decision 3（direction 列）**：是否同意 Option B（拆 `direction` + `status` 双列）并接受 TDengine schema migration？或者倾向 Option C（加 `source` tag）？
4. **Decision 4（CLI 层级）**：是否同意 Option A（顶层 promote 到 `quantix data import-*`）？是否需要保留 `tdx-api` parent 作为 deprecation alias 一个 release？
5. **2b.10 live 验证启动授权**：是否现在就跑 live smoke？需要你提供 OPENSTOCK_API_KEY 或确认 NAS 环境可达。
6. **是否启动 Phase 1**：四大决策确认 + 2b.10 通过后，是否立即开始 Phase 1 迁出？还是先做 P0.11d（live-verify REALTIME_QUOTES 作为 scheduler fallback 前置）？
