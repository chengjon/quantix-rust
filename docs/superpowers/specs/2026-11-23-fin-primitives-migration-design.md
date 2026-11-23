# fin-primitives 研究库集成设计

- 日期: 2026-11-23
- 状态: v2 修订版（响应 `2026-11-23-fin-primitives-migration-design-review.md`，NEEDS_REVISION → 修订待审）
- 上游库: fin-primitives v4.0.1（MIT, github.com/Mattbusel/fin-primitives, MSRV 1.81, edition 2021）
- 修订记录: 见文末 §12

## 1. 背景与目标

quantix-rust 的指标层（`analysis/indicators/` 538 行，不含 tests）与量化风控/研究能力薄弱。fin-primitives 提供 749 个流式指标（TA-Lib parity 测试背书）、PositionLedger 组合分析、DrawdownTracker/VaR 风险库、bar-by-bar 回测引擎（WalkForwardOptimizer）、事件研究（events）。两库技术栈兼容（rust_decimal/chrono/tokio 同线，上游无 polars/ndarray/nalgebra 重依赖）。

目标：以 cargo 依赖方式引入 fin-primitives，在 `src/fin/` 建立薄集成层，交付 3 个 CLI 入口（fin 引擎指标查询 / 回撤分析报告 / walk-forward 回测），落地 README 主线任务的「回撤控制」。

## 2. 非目标

- 不替换现有 `analysis/portfolio.rs`（A股 T+1 手续费记账）与 `analysis/backtest.rs`（A股 T+1 模拟闭环）——研究侧与业务侧双轨并存。
- 不引入 fin-primitives 核心类型体系（Price/Quantity/Symbol/OrderBook/Tick/OhlcvBar）为公共 API——这些类型仅出现在 `src/fin/` 内部。
- 不接入上游 impact/latency/execution_cost/microstructure 模块：它们是独立模块，未被 fin `Backtester`/`WalkForwardOptimizer` 集成，自行为其接线超出本期范围（列二期可选）。
- 本期不做：events 的 CLI 入口（仅库层）、`signals::float` f64 批量截面接入 `factor/`、`news/` 事件源接入、crypto/options/fixed-income/credit/tax/yield-curve/montecarlo/clustering 模块。`async` feature 上游默认即关闭，不启用。

## 3. 决策记录

| # | 决策 | 理由 |
|---|------|------|
| D1 | portfolio 并存：fin PositionLedger/attribution/performance = 研究侧；自研 analysis/portfolio.rs = 业务侧 | 保持 A股费用语义单一来源，互不侵蚀 |
| D2 | backtest 双引擎并存：fin 引擎（bar-by-bar + walk-forward）= 研究侧；回测成本口径为佣金率（commission_rate），冲击/延迟不接（见 §2） | 与 indicators 生态配套；现有引擎不动；成本模型表述与上游实际 API 对齐 |
| D3 | 落地深度 = 库层 + 3 个精选 CLI；events 仅库层 | 最小可用面，二期按需扩 CLI |
| D4 | 引入方式 = cargo 依赖，版本精确锁 `=4.0.1` | 跟随上游修复；vendor 为退路 |
| D5 | 策略单源引擎双轨 = **信号预计算桥**：运行回测前按 bar 顺序异步预驱动 quantix `Strategy`（`#[async_trait]`），生成 `Vec<Option<fin Signal>>` 缓存；fin 同步 `Strategy::on_bar` 仅按索引查表。无 `block_on`、无嵌套 runtime | quantix 策略为 async、fin 策略为 sync，直接委托会 panic；预计算使回测可在任意上下文运行，且保持逐 bar 推进顺序（有状态策略安全） |

### D5 补充：Signal 映射与数量规则

- quantix `core::signal::Signal`（Buy/Sell/Hold 枚举）→ fin `backtest::Signal`：
  - `Buy` → `Signal::new(SignalDirection::Buy, cfg.position_size)`
  - `Sell` → `Signal::new(SignalDirection::Sell, cfg.position_size)`
  - `Hold` → `Signal::hold()`（fin 引擎对 Hold/零数量自动跳过：backtest/mod.rs:188-192）
- quantix Signal 无数量概念 → `FinStrategyConfig { position_size: Decimal }` 显式指定每次下单数量（一期固定数量；比例仓位二期扩展）。
- quantix `on_bar` 返回 `Result<Signal, Box<dyn Error>>`：`Err` 或预驱动失败 → 记录该 bar 为 Hold 并累计错误计数，回测结束后并入 `QuantixError` 上抛（不静默吞错）。

## 4. 架构

新增 `src/fin/` 集成模块（与 `bridge/` 同级，定位为「研究库桥」）：

```
src/fin/
├── mod.rs         # 模块声明 + 公共出口（仅 quantix 类型）
├── adapter.rs     # Kline → BarInput（指标路径）+ Kline → OhlcvBar（回测路径，fallible）
├── indicators.rs  # 常用指标强类型门面
├── portfolio.rs   # PositionLedger / performance / attribution 门面
├── risk.rs        # DrawdownTracker / VaR / scenario 门面
├── backtest.rs    # FinEngine 封装 + FinStrategy 预计算桥 + walk-forward
└── events.rs      # 事件研究门面（库层，无 CLI）
```

数据流：

```
ClickHouse/DB → Vec<Kline> ─┬─ adapter::to_bar_inputs ──→ fin signals::indicators（指标路径）
                            └─ adapter::to_ohlcv_bars ──→ fin Backtester/EventStudy（回测/事件路径）
           → quantix 类型（Decimal/QuantixResult）→ CLI 输出
```

## 5. 组件明细

### 5.1 adapter.rs（fallible）

`Kline`（src/data/models/types.rs:6）到 fin 类型的全部构造规则：

| Kline 字段 | fin 目标 | 转换规则 |
|---|---|---|
| `code: String` | `OhlcvBar.symbol: Symbol` | `Symbol::new(code)` |
| `date: NaiveDate` | `OhlcvBar.ts_open / ts_close: NanoTimestamp` | 当日 09:30 / 15:00 CST → unix nanos |
| `open/high/low/close: Decimal` | `Price` / `BarInput` 对应字段 | 直传（`Price::try_new`） |
| `volume: i64` | `Quantity` / `BarInput.volume: Decimal` | `Decimal::from(v)`（`Quantity::try_new`） |
| —（Kline 无此概念） | `OhlcvBar.tick_count: u64` | 恒填 `1` |

- `to_bar_inputs(&[Kline]) -> Vec<BarInput>`（指标路径，infallible 除 volume 转换）。
- `to_ohlcv_bars(&[Kline]) -> Result<Vec<OhlcvBar>>`（回测路径）：`try_new` 失败或 `OhlcvBar::validate`（high≥max(open,close)、low≤min(open,close)）失败 → `FinError::InvalidInput` → `QuantixError::Fin` 上抛，附 code+date 定位信息。
- 契约：调用方保证按日期升序；复权类型由调用方选择（adapter 不处理 adjust_type）。

### 5.2 indicators.rs
- **强类型门面**：一期精修 10–20 个常用指标（RSI/MACD/BOLL/ATR/MA 家族/Stochastic 系等——上游无 KDJ，K/J 线如需由 Stochastic %K/%D 自行组合，二期门面再议），输入 `&[Kline]`，输出 quantix Decimal 结构。
- **扩展约定**（取代原「re-export 逃生舱」，消除与 §5.7 的矛盾）：未被门面覆盖的指标，通过**在 `src/fin/indicators.rs` 内新增薄封装函数**接入（每个 3–5 行：构造 fin 指标 → 喂 `BarInput` → 取值 → quantix 类型返回）。fin 类型一律不出 `src/fin/`；749 个指标按需逐步封装，不改公共边界。

### 5.3 portfolio.rs（研究侧）
- 从成交序列/逐 bar 净值构建 fin `PositionLedger` 的门面；导出 performance 统计与 attribution 结果为 quantix 结构。
- 不写账、不落库——纯研究分析输出。

### 5.4 risk.rs
- `DrawdownTracker` 门面：喂逐 bar 净值序列 → 当前回撤/最大回撤/回撤区间/恢复统计。
- VaR 门面：`VaRCalculator`（historical/parametric/monte_carlo/conditional/cornish_fisher）。精度口径：上游 VaR 基于 **f64 收益序列**，门面在 Decimal→f64 转换处注明（研究统计用途，f64 可接受；f64 结果不回转 Decimal 记账）。
- 服务 CLI「回撤分析报告」入口（见 §7）。

### 5.5 backtest.rs（研究侧）
- `FinStrategy` 预计算桥（D5）：`precompute_signals(&mut dyn quantix Strategy, &[Kline], cfg) -> Result<Vec<Option<fin Signal>>>` + 查表实现 fin `Strategy` trait。错误处理见 D5 补充。
- `FinEngine` 封装：fin `Backtester`（`BacktestConfig { initial_capital, commission_rate }`，与上游实际 API 一致）。冲击/延迟/微观结构不接（§2）。
- `walk_forward` 门面：fin `WalkForwardOptimizer` 参数网格 + 训练/验证窗口切分 + OOS 稳定性报告（quantix 结果结构）。

### 5.6 events.rs（库层）
上游 `EventStudy::compute` 语义（events/mod.rs:68）：单事件、价格序列 `&[(u64, f64)]`（unix 秒, 价格）、**必需 benchmark 序列**、返回单事件 `EventResult`。门面按此对齐：

```rust
pub struct EventSpec { pub code: String, pub date: NaiveDate, pub event_type: String, pub description: String }

/// 单事件：price/benchmark 为 (unix_sec, f64 价格) 升序序列
pub fn run_event_study(event: &EventSpec, security_prices: &[(u64, f64)], benchmark_prices: &[(u64, f64)], window: EventWindowSpec) -> Result<EventReport>

/// 多事件批量：按 code 查 Kline 序列，close Decimal→f64；benchmark 取指数代码序列；循环调用 compute
pub fn run_event_studies(events: &[EventSpec], klines_by_code: &HashMap<String, Vec<Kline>>, benchmark_code: &str, window: EventWindowSpec) -> Result<Vec<EventReport>>
```

- 门面职责：`EventSpec` → fin `MarketEvent`（NaiveDate → unix 秒，event_id = `{code}:{date}`）；Kline close → f64 价格序列（研究统计精度口径，f64 不回转记账）。
- 事件输入源：CSV/数据库事件表；`news/` 接入为二期。

### 5.7 mod.rs
- `pub mod` 声明；对 CLI 层只暴露 quantix 类型门面函数，**任何 fin 类型不 re-export**（与 §5.2 扩展约定一致）。

## 6. 错误处理

- `QuantixError`（`src/core/error.rs:8`，统一 `Result<T>` 别名在 :53）新增 variant，遵循现有 `#[from]` 风格：

```rust
#[error("fin-primitives 错误: {0}")]
Fin(#[from] fin_primitives::FinError),
```

- 纪律：`FinError` 不越过 `src/fin/` 边界——门面函数一律返回 `core::Result<T>`，调用方只见 `QuantixError`。adapter 的 fallible 构造（§5.1）错误同样经 `FinError::InvalidInput` → `QuantixError::Fin` 汇聚，消息含 code+date 定位。

## 7. CLI 集成（3 个入口）

| 入口 | 挂载点 | 行为 |
|------|--------|------|
| fin 引擎指标查询 | **扩展既有** `analyze indicators`（`src/cli/commands/analysis.rs` `AnalyzeCommands::Indicators`）新增 `--engine {native,fin}` 选项 | 默认 `native` 行为完全不变（misc.rs `calculate_indicators`）；`--engine fin` 走 fin 指标门面。不新增重复子命令 |
| 回撤分析报告 | `analyze drawdown --code ...`（挂 **analysis 命令族**） | 逐代码价格/净值序列 → 回撤统计 + VaR 报告。原 v1 设计挂 risk 命令族，但 `RiskCommands` 全部为实盘风控（Import/Sync/Rebuild/Rule/Log/Lock/Status/Pnl/Position），研究分析挂入语义错配，故改挂 analysis |
| walk-forward 回测 | `backtest walkforward --strategy ma_cross --grid ...`（挂 backtest 命令族，不变） | 参数网格 + OOS 稳定性报告 |

## 8. Cargo 依赖变更

```toml
[dependencies]
fin-primitives = { version = "=4.0.1", default-features = false, features = [
  "serde", "indicators", "portfolio", "risk", "backtest", "ml",
] }
```

- `ml` feature 门控 events（顺带带入 ml_features、alternative_data、pairs_trading，无副作用）。
- `default-features = false` 后仅上述白名单生效；options/fixed-income/derivatives/crypto/credit/tax/yield-curve/clustering/montecarlo 不启用（`async` 本就不在 default，无需处理）。
- 版本精确锁；如上游破坏性行为漂移且急需，vendor 源码为退路（MIT 允许，上游耦合 DAG 干净易裁剪）。

## 9. 测试策略

1. **adapter 单测**：Kline→BarInput/OhlcvBar 字段映射（volume i64→Decimal、时间戳规则、tick_count=1）；非法高低价 → `FinError::InvalidInput` 路径。
2. **门面合成数据单测**：各模块用合成 Kline 序列验证输出结构与非 NaN。
3. **金丝雀快照测试**：对固定输入记录 fin 指标/回撤/walk-forward 输出快照，防上游升级行为漂移（配合 D4 精确锁版本）。
4. **预计算桥单测**：有状态 quantix 策略（计数器）经桥接后逐 bar 按序推进；`Err` 返回 → Hold + 错误计数路径。
5. **双引擎一致性测试**：同一 ma_cross 策略跑 quantix 引擎与 fin 引擎，**信号序列一致**（成交结果不要求相等——两引擎成交假设/费用模型不同）。
6. **walk-forward 集成测试**：小网格端到端跑通并校验报告字段。
7. **质量门**：`cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test` 全绿。

## 10. 实施纪律

- 改动现有符号（cli/commands/*.rs、core/error.rs）前跑 GitNexus impact analysis 并报告波及面（HIGH/CRITICAL 需先警告用户）；提交前跑 `gitnexus_detect_changes()`。
- 功能完成后在 `FUNCTION_TREE.md` 登记（该文件是功能唯一注册表）。
- 提交遵循仓库规范 `<type>(<scope>): <subject>`，feat/docs 分开。
- Graphiti：设计定稿与调试结论需回填对应 group_id（本会话 MCP 未连接，实施会话处理）。

## 11. 二期展望（非本期承诺）

- events CLI 入口 + `news/` 事件源接入。
- 上游 impact/execution_cost 模块与 fin `Backtester` 的显式接线（上游未集成，需自建）。
- `signals::float` f64 批量指标接入 `factor/` 截面计算（polars 列式批处理路径）。
- 常用指标门面扩展至 30+；Stochastic 组合 K/J 线；比例仓位规则；ETF 期权场景再评估 options feature。

## 12. 修订记录

### v2（2026-11-23，响应 design-review）
- **C1**：D5 重写为信号预计算桥（异步预驱动 + 同步查表），补 Signal 映射表、`FinStrategyConfig.position_size` 数量规则、`Err`→Hold+错误计数处理（§3/§5.5/§9.4）。
- **C2**：删除「成本/冲击/延迟模型参数入口」表述；明确回测成本口径 = commission_rate，impact 等为上游独立模块未接入引擎，列二期（§1/§2/§5.5/§11）。
- **C3**：events 门面签名改为 `run_event_study(event, security_prices, benchmark_prices, window)`（单事件 + benchmark + f64），多事件批量循环 compute；补 code→序列映射与时间戳转换（§5.6）。
- **M4**：指标查询改为扩展既有 `analyze indicators --engine fin`，默认 native 不变，不新增重复子命令（§7）。
- **M5**：回撤报告从 risk 命令族改挂 analysis 命令族（`analyze drawdown`），理由 = RiskCommands 全实盘风控语义（§7）。
- **M6**：adapter 补 `Kline→OhlcvBar` fallible 转换 + 构造规则表（Symbol/NanoTimestamp 09:30/15:00 CST/tick_count=1）+ validate 错误路径（§5.1/§9.1）。
- **M7**：消除 re-export 矛盾——逃生舱改为「src/fin 内薄封装扩展约定」，fin 类型一律不出 src/fin（§5.2/§5.7）。
- **M8**：门面清单 KDJ → Stochastic 系，注明 K/J 线需自行组合（§5.2）。
- **L9/L10**：ml feature 注释补全 alternative_data；删除 async 相关排除表述（默认即关）（§8）。
- **L11**：指标层行数口径改为 538 行不含 tests（§1）。
- **L12**：VaR f64 精度口径注明（§5.4/§5.6）。
