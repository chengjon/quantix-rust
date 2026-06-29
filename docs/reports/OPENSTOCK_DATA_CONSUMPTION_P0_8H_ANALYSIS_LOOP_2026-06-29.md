# OpenStock 数据消费 P0.8h — analysis/backtest 更宽 fixture 链路（indicators + strategy）

日期：2026-06-29
分支：`test/openstock-p0-8h-analysis-loop`
FUNCTION_TREE 节点：`P0.8h: OpenStock analysis wider fixture loop`（status: `approved-for-implementation`，test-only）
前置切片：P0.8d（窄链路 `parse_daily_kline_json → analysis::sma`）已落地；P0.8e/f/g 已合并入 master

## 1. 决策

P0.8h 是**test-only 切片**。在 P0.8d 已经证明 `parse_daily_kline_json → Vec<Kline> → analysis::sma(window=2)` 的最窄链路之后，P0.8h 把 fixture 消费端扩展到更宽的指标/策略侧：

- **指标扇出**：把同一份 fixture 的 close/high/low/volume 序列送入 `analysis::{sma, ema, wma, bollinger_bands, atr, obv, cci, williams_r}`
- **策略入场**：把每个 `Kline` 顺序喂入 `MACrossStrategy::new(short, long)` 的 `Strategy::on_bar`，收集 `Signal` 序列

本切片不批准、不实现任何 `src/` 修改。

## 2. 范围

允许（本切片）：

- 编写本文档（设计 + scope gate）
- 创建治理节点 `P0.8h` 并推进到 `approved-for-implementation`
- 后续实现步骤仅落地：
  - `tests/fixtures/openstock/daily_kline_30d.json`（~30 个交易日，code `600000` 的合成 OHLCV）
  - `tests/openstock_analysis_wider_loop_test.rs`（指标扇出 + 策略入场）
- 同步 OpenSpec tasks、CHANGELOG、README、FUNCTION_TREE（落地阶段）

禁止（本切片）：

- 不修改任何 `src/` 生产 Rust 代码（card 明确列出 `src/**` 为 forbidden_paths）
- 不修改 `Cargo.toml` / `Cargo.lock`
- 不写 ClickHouse
- 不替换生产数据源路由
- 不触达 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- 不做 live OpenStock 网络调用
- 不恢复 `.unwrap()` 清理
- 不修改或复用 `ControlledPersistencePolicy`（GitNexus impact HIGH）
- 不修改 `Kline` 定义（CRITICAL hub，只读消费）

## 3. 公共 API（read-only 消费）

| 模块 | 符号 | 用途 |
|---|---|---|
| `sources::openstock` | `parse_daily_kline_json` | fixture JSON → `Vec<Kline>`（沿用 P0.8d/P0.8f 路径） |
| `analysis::indicators` | `sma`、`ema`、`wma`、`bollinger_bands`、`atr`、`obv`、`cci`、`williams_r` | 指标扇出 |
| `strategy::ma_cross::MACrossStrategy` | `MACrossStrategy::new(short, long)` | 实例化 MA 交叉策略 |
| `strategy::trait_def::Strategy` | `Strategy::on_bar(&Kline) -> Signal` | 逐 bar 驱动策略 |
| `core::signal::Signal` | `Buy \| Sell \| Hold` | 策略输出契约 |

所有 API 均为既有公开符号；本切片仅只读消费，不修改。

## 4. Backtest 显式推迟

P0.8h 的初版范围曾考虑 `analysis::backtest` 入场。调研发现 `BacktestEngine`（`src/analysis/backtest.rs`）当前仅暴露：

- `pub fn new(config: BacktestConfig) -> Self`
- `pub fn with_default_config() -> Self`
- `pub fn portfolio_snapshot(&self) -> &Portfolio`

**没有**公开 `run` / `feed_bar` / `on_bar` 入口；引擎通过内部 trait 消费。把 fixture 喂入 `BacktestEngine` 必须修改 `src/analysis/backtest.rs` 暴露新公共方法，这违反 P0.8h 的 test-only non-goals。

**决定**：backtest 路径推迟到独立切片（候选名 `P0.8h-bt`），该切片必须：

1. 跑 fresh GitNexus impact on `BacktestEngine`（`src/analysis/backtest.rs`）和 `PerformanceCalculator`
2. 单独治理 card（production-code slice，`source_edits_authorized: true`）
3. TDD RED→GREEN，定义 `pub fn feed_bar(&mut self, bar: &Kline) -> Result<...>` 或等价入口
4. 评估是否影响 `Strategy` trait 的 on_bar 消费契约

## 5. Fixture 设计

`tests/fixtures/openstock/daily_kline_30d.json` 需要满足：

- ~30 条 daily 记录（足够支持 `period ≤ 10` 的指标窗口和 MA cross 策略）
- 单一 code `600000`（避免跨 code 干扰）
- 每条字段对齐 P0.8f 的 envelope：`symbol / time / open / high / low / close / volume / amount / period`
- 收盘价序列需要精心选择，使得：
  - EMA(5) 最后一个窗口输出为 `Some(...)` 且非 NaN
  - `MACrossStrategy::new(2, 5)` 在 30 bar 中至少触发一次非 `Hold` 信号（短窗均线上穿/下穿长窗）
  - Bollinger Bands(5, 2) 的 `upper/mid/lower` 在最后窗口均 `Some(...)`

Fixture 是合成数据（非真实市场数据），用于验证 pipeline 形状而非金融意义。

## 6. 测试断言（落地阶段）

`tests/openstock_analysis_wider_loop_test.rs`：

1. `parse_daily_kline_json(FIXTURE).unwrap()`，断言记录数 ≈ 30，所有 `kline.code == "600000"`
2. 指标扇出：对每条指标函数，断言返回 `Vec` 长度 == 30，且最后一个索引处为 `Some(...)`，无 panic
3. 策略：构造 `MACrossStrategy::new(2, 5)`，遍历 `Vec<Kline>` 调用 `on_bar(&kline).await.unwrap()`，收集 `Vec<Signal>`，断言长度 == 30 且至少包含一个 `Buy` 或 `Sell`

## 7. GitNexus Impact（本设计 gate）

本切片不修改任何代码。落地阶段的目标符号只读消费：

| 候选目标 | 当前状态 | 用途 | P0.8h 决策 |
|---|---|---|---|
| `src/sources/openstock.rs::parse_daily_kline_json` | LOW (P0.8f 已 green) | 输入解析 | 只读消费 |
| `src/analysis/indicators.rs` 公开 fn (8 个) | LOW | 指标扇出 | 只读消费 |
| `src/strategy/ma_cross.rs::MACrossStrategy::new` | LOW | 策略实例化 | 只读消费 |
| `src/strategy/trait_def.rs::Strategy::on_bar` | n/a (trait method) | 策略驱动 | 只读消费 |
| `src/analysis/backtest.rs::BacktestEngine` | 推迟 | — | 不在本片触及；留待 P0.8h-bt |

落地切片启动时必须重跑 `gitnexus_detect_changes`，并校验 `src/` 符号触及数 = 0。

## 8. 验收标准（设计 gate）

- 治理节点 `P0.8h` 状态推进到 `approved-for-implementation`
- OpenSpec 任务 `5h.1`–`5h.7` 创建并标记完成计划
- GitNexus `detect_changes` 确认仅 docs/governance 范围
- PR CI 通过

## 9. Non-Goals（继承 P0.8 系列）

- 不修改任何生产 Rust 源码（本切片）
- 不修改 `Cargo.toml` / `Cargo.lock`
- 不写 ClickHouse
- 不触达 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- 不修改或复用 `ControlledPersistencePolicy`（GitNexus HIGH）
- 不修改 `Kline` 定义（CRITICAL hub，只读）
- 不做 live OpenStock 网络调用
- 不实现 OpenStock provider contract（留待 P0.8g-impl 之后）
- 不驱动 `BacktestEngine`（推迟到 P0.8h-bt）

## 10. 下一步

- **P0.8h（实现切片）**：本设计 gate 合并后启动；按 tasks.md §5h.2–5h.6 落地 fixture + 测试；TDD 形态可选（断言形状先于实现，但实现均在使用既有 API，无 RED 阶段）
- **P0.8h-bt（候选）**：`BacktestEngine` 公共入口 + fixture 驱动
- **P0.8g-impl**：依然独立的可选 ClickHouse 写入路径实现
