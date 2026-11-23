# fin-primitives 研究库集成设计

- 日期: 2026-11-23
- 状态: 已获用户批准（brainstorming 两段设计呈现均已确认）
- 上游库: fin-primitives v4.0.1（MIT, github.com/Mattbusel/fin-primitives, MSRV 1.81, edition 2021）

## 1. 背景与目标

quantix-rust 的指标层（`analysis/indicators/` 约 550 行）与量化风控/研究能力薄弱。fin-primitives 提供 725+ TA-Lib parity 流式指标、PositionLedger 组合分析、DrawdownTracker/VaR 风险库、bar-by-bar 回测引擎（WalkForwardOptimizer + 成本/冲击/延迟模型）、事件研究（events）。两库技术栈兼容（rust_decimal/chrono/tokio 同线，上游无 polars/ndarray/nalgebra 重依赖）。

目标：以 cargo 依赖方式引入 fin-primitives，在 `src/fin/` 建立薄集成层，交付 3 个 CLI 入口（指标查询 / 回撤风险报告 / walk-forward 回测），落地 README 主线任务的「回撤控制」。

## 2. 非目标

- 不替换现有 `analysis/portfolio.rs`（A股 T+1 手续费记账）与 `analysis/backtest.rs`（A股 T+1 模拟闭环）——研究侧与业务侧双轨并存。
- 不引入 fin-primitives 核心类型体系（Price/Quantity/Symbol/OrderBook/Tick）为公共 API——BarInput/OhlcvBar 仅出现在 `src/fin/` 内部。
- 本期不做：events 的 CLI 入口（仅库层）、`signals::float` f64 批量截面接入 `factor/`、`news/` 事件源接入、crypto/options/fixed-income/credit/tax/yield-curve/montecarlo/clustering 模块。

## 3. 决策记录

| # | 决策 | 理由 |
|---|------|------|
| D1 | portfolio 并存：fin PositionLedger/attribution/performance = 研究侧；自研 analysis/portfolio.rs = 业务侧 | 保持 A股费用语义单一来源，互不侵蚀 |
| D2 | backtest 双引擎并存：fin 引擎（bar-by-bar + walk-forward + 成本模型）= 研究侧 | 与 indicators + Signal trait 生态配套；现有引擎不动 |
| D3 | 落地深度 = 库层 + 3 个精选 CLI；events 仅库层 | 最小可用面，二期按需扩 CLI |
| D4 | 引入方式 = cargo 依赖，版本精确锁 `=4.0.1` | 跟随上游修复；vendor 为退路 |
| D5 | 策略单源引擎双轨：FinStrategy 适配器把 quantix `Strategy` trait 信号适配成 fin `Strategy` trait | ma_cross 等策略一份实现跑双引擎，避免策略分叉 |

## 4. 架构

新增 `src/fin/` 集成模块（与 `bridge/` 同级，定位为「研究库桥」）：

```
src/fin/
├── mod.rs         # 模块声明 + 公共 re-export（quantix 类型出口）
├── adapter.rs     # Kline ↔ BarInput 转换
├── indicators.rs  # 常用指标强类型门面 + 通用逃生舱
├── portfolio.rs   # PositionLedger / performance / attribution 门面
├── risk.rs        # DrawdownTracker / VaR / scenario 门面
├── backtest.rs    # FinEngine 封装 + FinStrategy 适配器 + walk-forward
└── events.rs      # 事件研究门面（库层，无 CLI）
```

数据流：

```
ClickHouse/DB → Vec<Kline> → adapter::to_bar_inputs → fin-primitives API
           → quantix 类型（Decimal/QuantixResult）→ CLI 输出
```

## 5. 组件明细

### 5.1 adapter.rs
- `impl From<&Kline> for fin_primitives::signals::BarInput`：`volume: i64 → Decimal::from(v)`，其余字段同名直传。
- `pub fn to_bar_inputs(klines: &[Kline]) -> Vec<BarInput>`（按日期升序，调用方保证）。
- 契约：`Kline.volume: i64` 非负；复权类型由调用方选择（adapter 不处理 adjust_type）。

### 5.2 indicators.rs
两层设计（避免逐个封装 725+ 指标的爆炸工作量）：
- **强类型门面**：一期精修 10–20 个常用指标（RSI/MACD/BOLL/ATR/MA 家族/KDJ 类等，实现期从 fin API 清单圈定），输入 `&[Kline]`，输出 quantix Decimal 结构。
- **通用逃生舱**：re-export fin-primitives `signals::indicators` 命名空间 + adapter，任何未被门面覆盖的指标可经 `BarInput` 直接实例化使用。

### 5.3 portfolio.rs（研究侧）
- 从成交序列/逐 bar 净值构建 fin `PositionLedger` 的门面；导出 performance 统计与 attribution 结果为 quantix 结构。
- 不写账、不落库——纯研究分析输出。

### 5.4 risk.rs
- `DrawdownTracker` 门面：喂逐 bar 净值序列 → 当前回撤/最大回撤/回撤区间/恢复统计。
- VaR 与 scenario 门面。
- 此模块服务 CLI「回撤风险报告」入口，落地主线任务「回撤控制」。

### 5.5 backtest.rs（研究侧）
- `FinStrategy` 适配器：实现 fin 的 bar-by-bar `Strategy` trait，内部委托 quantix `strategy::trait_def::Strategy` 的信号（D5，ma_cross 先行验证）。
- `FinEngine` 封装：fin `Backtester` + 成本/冲击模型参数入口。
- `walk_forward` 门面：参数网格 + 训练/验证窗口切分 + OOS 稳定性报告（quantix 结果结构）。

### 5.6 events.rs（库层）
- `run_event_study(events, klines_by_code, window) -> EventResult` 门面，包装 fin `EventStudy`/`EventWindow`/`AbnormalReturn`。
- 事件输入：CSV/数据库事件表（字段：code + date + 事件类型）；`news/` 接入为二期。

### 5.7 mod.rs
- `pub mod` 声明；对 CLI 层只暴露 quantix 类型门面函数，不 re-export fin 内部类型（BarInput 等止步于此）。

## 6. 错误处理

- `QuantixError`（`src/core/error.rs:8`，统一 `Result<T>` 别名在 :53）新增 variant，遵循现有 `#[from]` 风格：

```rust
#[error("fin-primitives 错误: {0}")]
Fin(#[from] fin_primitives::FinError),
```

- 纪律：`FinError` 不越过 `src/fin/` 边界——门面函数一律返回 `core::Result<T>`，调用方只见 `QuantixError`。

## 7. CLI 集成（3 个入口，挂现有命令组）

| 入口 | 挂载点 | 行为 |
|------|--------|------|
| 指标查询 | `src/cli/commands/analysis.rs` 新增 `indicator` 子命令 | 对单代码 Kline 序列计算指定指标，表格式输出 |
| 回撤风险报告 | `src/cli/commands/risk.rs` 新增子命令 | 逐代码净值/价格序列 → 回撤统计报告（落地主线「回撤控制」） |
| walk-forward 回测 | `src/cli/commands/backtest.rs` 新增 `walkforward` 子命令 | `backtest walkforward --strategy ma_cross --grid ...` 参数网格 + OOS 报告 |

## 8. Cargo 依赖变更

```toml
[dependencies]
fin-primitives = { version = "=4.0.1", default-features = false, features = [
  "serde", "indicators", "portfolio", "risk", "backtest", "ml",
] }
```

- `ml` feature 用于门控 `events`（顺带带入 ml_features/pairs_trading，无副作用）。
- `default-features = false` 排除 options/fixed-income/derivatives/crypto/credit/tax/yield-curve/clustering/montecarlo/async。
- 版本精确锁；如上游破坏性行为漂移且急需，vendor 源码为退路（MIT 允许，上游耦合 DAG 干净易裁剪）。

## 9. 测试策略

1. **adapter 单测**：Kline→BarInput 字段映射（含 volume i64→Decimal、边界值）。
2. **门面合成数据单测**：各模块用合成 Kline 序列验证输出结构与非 NaN。
3. **金丝雀快照测试**：对固定输入记录 fin 指标/回撤/walk-forward 输出快照，防上游升级行为漂移（配合 D4 精确锁版本）。
4. **FinStrategy 集成测试**：同一 ma_cross 策略跑 quantix 引擎与 fin 引擎，核对信号一致性口径。
5. **walk-forward 集成测试**：小网格端到端跑通并校验报告字段。
6. **质量门**：`cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test` 全绿。

## 10. 实施纪律

- 改动现有符号（cli/commands/*.rs、core/error.rs）前跑 GitNexus impact analysis 并报告波及面（HIGH/CRITICAL 需先警告用户）；提交前跑 `gitnexus_detect_changes()`。
- 功能完成后在 `FUNCTION_TREE.md` 登记（该文件是功能唯一注册表）。
- 提交遵循仓库规范 `<type>(<scope>): <subject>`，feat/docs 分开。
- Graphiti：设计定稿与调试结论需回填对应 group_id（本会话 MCP 未连接，实施会话处理）。

## 11. 二期展望（非本期承诺）

- events CLI 入口 + `news/` 事件源接入。
- `signals::float` f64 批量指标接入 `factor/` 截面计算（polars 列式批处理路径）。
- 常用指标门面扩展至 30+；ETF 期权场景再评估 options feature。
