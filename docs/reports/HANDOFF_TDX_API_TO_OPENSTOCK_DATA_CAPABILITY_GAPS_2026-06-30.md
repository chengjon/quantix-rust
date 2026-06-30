# Handoff — TDX-API 数据能力缺口 → OpenStock 接管

日期：2026-06-30
方向：quantix-rust → openstock
触发：本项目 `quantix-rust` 仅作数据消费者，所有数据来自容器化部署的 `openstock`；明确不吸收 `tdxquant`、`miniqmt`（依赖 Windows 客户端，无法容器化）。

---

## 一、背景与决策

`quantix-rust` 历史引入了 `TdxApiClient`（`src/sources/tdx_api.rs`，33 个公开方法）作为 tdx-api Docker 服务的 REST 消费端。该服务在 `docker-compose.yml` 中定义为 `tdx-api:8080`，镜像构建自 `/opt/claude/tdx-api`，本质上是通达信本地客户端的 REST 封装。

明确边界：

- **quantix-rust 不构建数据源**，仅消费由 `openstock`（容器化）统一提供的数据。
- **tdxquant / miniqMT 不吸收**（依赖 Windows 客户端运行时，不能容器化）。
- **openstock 是合规的数据生产端**；quantix-rust 通过其 REST/JSON 协议消费。

因此，凡是 `TdxApiClient` 当前提供、而 openstock 暂未提供的数据能力，**剩余开发由 openstock 接管实现**；quantix-rust 这边只保留"消费方"角色。

---

## 二、当前覆盖对比

### 2.1 openstock（eltdx）已提供

| 能力 | quantix-rust 入口 | 备注 |
|---|---|---|
| 日 K 线（THS 前复权）解析 | `parse_daily_kline_json` (`src/sources/openstock.rs`) | P0.8a–h 全部落地，含 fixture、live shadow 验证、shadow persistence |
| 日 K 线影子表持久化（dry-run + 双门 opt-in） | `quantix data openstock persist-live` (`src/sources/openstock_shadow.rs`) | P0.8g-impl 已合并 |
| 实盘负载抓取与漂移验证 | `validate_live_shadow_payload` | P0.8f |

### 2.2 `TdxApiClient` 当前提供但 openstock 尚未提供

下表按**真实数据能力**归类（剔除 `new`/`from_env`/`from_app_config`/`invalidate_cache` 等纯客户端构造与缓存方法，这些不属于数据维度）。

#### A. 股票基础 K 线（非"日/前复权"周期或复权方式）

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `get_kline_raw` | `kline` | 分钟级 K 线（minute1/5/15/30/hour）+ 日/周/月，**TDX 原始价格（不复权）** | 高（量化必需分钟级） |
| `get_kline_history` | — | 历史 K 线（多周期） | 高 |
| `get_daily_kline` | — | 日 K 线（不复权专用） | 中 |
| `get_kline_all_tdx` | — | TDX 原版全历史 K 线 | 中 |
| `get_kline_all_ths` | `kline-ths` | 同花顺前复权全历史 | 高（openstock 已有日级；周/月可考虑） |
| `get_kline_ths_qfq` | — | THS 前复权单段 | 中 |

#### B. 指数 K 线（重要缺口）

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `get_index_kline` | **无 CLI 暴露** | 指数 K 线（如上证指数、沪深 300） | **高** — 量化基准必备 |

#### C. 实时行情

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `quote` | `quote` | 单股实时行情（价/涨幅/量/额） | 中（看策略是否需要实时） |
| `batch_quote` | — | 批量实时行情 | 中 |
| `collect_all_quotes` | — | 全市场行情快照 | 中（适合盘中扫描） |

#### D. 分时与逐笔

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `get_minute` | `minute` | 分时图（日内 1 分钟价格序列） | 中 |
| `get_trades` | `import-ticks` (写入 TDengine) | 逐笔成交（价/量/买卖盘标记） | 中（数据源切换见 §3，入仓选型属 quantix-rust 内部决策） |

#### E. 财务与市场统计

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `get_income` | `income` | 财务收入（含 OHLCV 字段） | 中 |
| `get_market_stats` | `market-stats` | 市场统计（涨跌停、成交额分布等） | 中 |
| `get_market_count` | — | 市场股数统计 | 低 |

#### F. 代码与交易日历

| 方法 | 当前 CLI | 数据能力 | 优先级建议 |
|---|---|---|---|
| `search_codes` | `search` | 代码/名称搜索 | 中 |
| `get_codes` | `import-klines --all` 内部调用 | 全代码列表 | 高（**openstock 必须提供等效列表 API，否则无法做全市场循环**） |
| `get_workday` / `get_workday_range` / `is_trading_day` | `workday` / `workday-range` / `sync-calendar` | 交易日历 | 高（**openstock 应提供交易日历 API，quantix-rust 不应依赖 tdx-api 的日历**） |

#### G. 异步任务（专属 tdx-api 的实现细节，不应迁移）

| 方法 | 当前 CLI | 处置建议 |
|---|---|---|
| `create_pull_kline_task` / `create_pull_trade_task` / `list_tasks` / `get_task` / `cancel_task` | `pull-kline` / `pull-trade` / `tasks` / `task-info` / `cancel-task` | **不迁移**。这是 tdx-api 内部"拉取数据到本地再读"的实现机制；openstock 作为生产端应直接提供同步 REST 数据 API，不需要 quantix-rust 这边管理异步任务。 |

---

## 三、quantix-rust 端的清理建议（待 openstock 接管后）

**本节为接收方（openstock）实现上述能力后，quantix-rust 这边的消费侧改造指引，不属于本次 handoff 的承诺时间表。**

### 3.1 数据源切换（不规定入仓选型）

**前提**：openstock 是数据生产端；**入仓到哪个数据库（TDengine / ClickHouse / PostgreSQL）是 quantix-rust 的内部决策，不属本 handoff 范围**。本节只描述数据来源切换，不规定存储后端。

- `import-ticks`：当前数据源为 `tdx-api`，待 openstock 提供逐笔 API（P3）后切换数据源；`src/db/tdengine.rs` 客户端本身不废弃，是否使用由 quantix-rust 自行决定。
  - 影响：`src/cli/handlers/tdx_api_handler.rs::ImportTicks`、`openspec/changes/tdx-api-import-e2e-hardening/` §3。
- `import-klines`：当前数据源为 `tdx-api` 直写 ClickHouse，待 openstock 提供全 K 线周期 API（P0/P1）后切换为 openstock → 入仓（具体入仓路径由 quantix-rust 决定，shadow persistence 已落地于 P0.8g-impl，是可选项之一但不强制）。
  - 影响：`src/cli/handlers/tdx_api_handler.rs::ImportKlines`、上述 OpenSpec §2。

### 3.2 可以保留为"兼容回退"直到 openstock 完全覆盖

- `TdxApiClient` 的只读能力（`quote` / `kline` / `minute` / `search` / `workday` / `income` / `market-stats`）可作为 openstock 同等能力上线前的过渡消费端。openstock 覆盖度上来后，quantix-rust 应切换为 openstock 消费端并移除 `tdx_api.rs`。

### 3.3 当前 OpenSpec 任务的处置

`openspec/changes/tdx-api-import-e2e-hardening/`（8/27 任务，停摆 23 天）的原始假设——"证明 tdx-api → ClickHouse / TDengine 真实链路可发布"——**已与本项目数据消费定位不符**。建议：

- **方案 A（推荐）**：归档该 OpenSpec change，理由：依赖路径已被 openstock 接管，本切片不再 releasable。
- **方案 B**：缩减为只验证 tdx-api 只读路径作为过渡消费端，删除 §2（ClickHouse 直写）和 §3（TDengine 直写）的入仓链路验证（入仓后端选型不属本 handoff 范围）。
- 由 openstock 团队决定后告知 quantix-rust 维护者。

---

## 四、给 openstock 团队的接管清单

按优先级排序，**P0 = quantix-rust 量化主线必需**：

| 优先级 | 能力 | 接口形态建议 | 备注 |
|---|---|---|---|
| **P0** | 全 A 股代码列表（含交易所标记 sh/sz/bj） | REST `GET /codes` → JSON | quantix-rust 全市场循环依赖此 |
| **P0** | 交易日历（某年节假日 + 调休日） | REST `GET /calendar?year=YYYY` → JSON | quantix-rust 不应再依赖 tdx-api 的日历 |
| **P0** | 指数 K 线（上证综指、沪深 300、中证 500 等） | REST `GET /index_kline?code=&period=&limit=` → JSON | 当前 quantix-rust 完全没有指数入口 |
| **P1** | 分钟级 K 线（minute1/5/15/30/hour） | REST `GET /kline?code=&period=minuteN&...` → JSON | 复用日 K 线的 payload 结构建议 |
| **P1** | 周线 / 月线 K 线 | REST `GET /kline?period=week\|month` → JSON | openstock 日级已通 |
| **P1** | 不复权 K 线（adjust_type=none） | 在现有日 K 线 payload 上扩展 `adjust_type` 字段 | openstock 已有 adjust_type 字段，扩展支持即可 |
| **P2** | 实时行情（单股 + 批量） | REST `GET /quote?code=` / `GET /quotes?codes=` → JSON | 看量化策略是否需要盘中 |
| **P2** | 分时图（日内分钟价格序列） | REST `GET /minute?code=&date=` → JSON | |
| **P3** | 逐笔成交 | REST `GET /trades?code=&date=` → JSON | 入仓由 quantix-rust 决定（存储选型不属 handoff 范围） |
| **P3** | 财务数据（收入/利润/ OHLCV 附字段） | REST `GET /income?code=` → JSON | |
| **P3** | 市场统计（涨跌停、成交额分布） | REST `GET /market_stats?date=` → JSON | |
| **P3** | 代码/名称搜索 | REST `GET /search?q=` → JSON | |

**不建议迁移**：`create_pull_kline_task` 等 tdx-api 异步任务机制。openstock 作为生产端应直接提供同步 REST 数据 API。

---

## 五、协议建议（重要）

为避免 openstock 重复 tdx-api 的设计债，建议 openstock 在提供上述能力时：

1. **统一 payload schema**：参照现有 `parse_daily_kline_json` 已定义的 `period / adjust_type / records[{code, date, open, high, low, close, volume, amount, ...}]` 结构，向后兼容地扩展 `period`（minuteN / day / week / month）与 `adjust_type`（none / qfq / hfq）枚举。
2. **统一错误形态**：HTTP 状态码 + 标准 JSON error envelope（code, message, request_id），避免 tdx-api 当前的隐式错误模式。
3. **明确 rate limit**：在响应头里给 `X-RateLimit-*`，便于 quantix-rust 自适应节流。
4. **明确幂等与增量语义**：日 K 线影子表已用 `batch_id + artifact_hash + source+period+code+date+adjust_type` 做去重；openstock 应在响应里提供稳定 `artifact_hash` 计算依据（例如 SHA-256 of canonical JSON），便于 quantix-rust 入仓时直接复用。
5. **明确身份标记**：响应里 `source: "openstock"` 而非 `"tdx-api"`，便于 quantix-rust 入仓后审计。

---

## 六、不迁移、不接管的项（明确边界）

- **Windows 客户端依赖项**（tdxquant / miniQMT）：本项目永不吸收，不进入 openstock。
- **实时 WebSocket 推送**：本 handoff 不涉及；若未来需要，应在 openstock 侧单独立项。
- **数据源统一抽象层**（Unified `DataSource` trait）：已在 tdx-api 价值分析报告中明确 defer，不在本 handoff 范围。
- **告警 / 通知 / 系统服务化 / systemd**：均 defer，不在本 handoff 范围。

---

## 七、quantix-rust 端下一步（待与 openstock 团队确认后）

1. 收到 openstock 接管时间表后，将本 handoff 中的接管清单条目逐项从"tdx-api 提供"迁移到"openstock 提供"，删除 `TdxApiClient` 对应方法。
2. 归档 `openspec/changes/tdx-api-import-e2e-hardening/`（方案 A）或缩减范围（方案 B）。
3. `import-ticks` 的数据源切换（tdx-api → openstock）需在 openstock 提供逐笔 API（P3）后执行；TDengine 客户端 `src/db/tdengine.rs` 是否保留由 quantix-rust 内部决定，不属本 handoff 范围。
4. 在 `FUNCTION_TREE.md` 中记录"数据消费统一从 openstock"的边界决策。

---

## 八、联系人 / 协作

- quantix-rust 维护者：通过本仓库 PR / issue。
- openstock 团队：接收本 handoff 后回填接管计划与上线时间。
