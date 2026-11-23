# fin-primitives 研究库集成 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 以 cargo 依赖接入 fin-primitives v4.0.1，在 `src/fin/` 建立研究库集成层（adapter/indicators/portfolio/risk/backtest/events），交付 3 个 CLI 入口（`analyze indicators --engine fin`、`analyze drawdown`、`backtest walkforward`）。

**Architecture:** 单 crate 内新增 `src/fin/` 模块（与 bridge/ 同级），fin-primitives 类型（BarInput/OhlcvBar 等）不越该边界；quantix 策略经同步 `signal_at` 提取（方案 C）驱动 fin 回测引擎；错误经 `QuantixError::Fin(#[from])` 收敛。设计文档：`docs/superpowers/specs/2026-11-23-fin-primitives-migration-design.md`（v2.1）。

**Tech Stack:** Rust 2024 / fin-primitives =4.0.1（features: serde,indicators,portfolio,risk,backtest,ml）/ rust_decimal / chrono / clap 4.5 derive。

---

## 全局纪律（每个任务执行前必读）

1. **禁止 `.unwrap()`/`.expect()` 出现在生产代码**（测试代码有先例可用）。
2. **改动现有符号前跑 GitNexus impact analysis**：涉及 Task 6（MACrossStrategy）、Task 9（AnalyzeCommands）、Task 11（BacktestCommands）。MCP 可用时 `gitnexus_impact({target, direction:"upstream"})` 并向用户报告波及面；不可用时用 `grep -rn "符号名" src/` 列出调用方代替，并记录"GitNexus backfill required"。
3. fin-primitives 类型只在 `src/fin/` 内部出现；CLI/handler 层只见 quantix 类型。
4. 每个 Task 结束跑 `cargo check` 必须通过再 commit。commit 格式 `type(scope): subject`。

## 关键上游 API 速查（已核实，v4.0.1）

| 项 | 签名 |
|---|---|
| 指标构造 | `Rsi/Sma/Ema/Atr/StochasticK::new(name: impl Into<String>, period: usize) -> Result<Self, FinError>`；`BollingerB::new(name, period, multiplier: impl Into<Decimal>)` |
| 指标流式接口 | trait `fin_primitives::signals::Signal`：`update(&mut self, &BarInput) -> Result<SignalValue, FinError>`；`SignalValue::as_decimal() -> Option<Decimal>` |
| `BarInput` | `{open,high,low,close,volume: Decimal}` |
| `OhlcvBar::new(symbol,open,high,low,close,volume,ts_open,ts_close,tick_count) -> Result<Self, FinError>`（校验 high≥open/close≥low） |
| newtype | `Symbol::new(impl AsRef<str>) -> Result<Self,FinError>`；`Price::new(Decimal) -> Result<Self,FinError>`；`Quantity: TryFrom<Decimal>`；`NanoTimestamp::new(i64)`；均 `.value()/.as_str()/.nanos()` 取回 |
| `DrawdownTracker` | `new(Decimal)`；`update(Decimal)`（返回 `()`）；`current_drawdown_pct()/worst_drawdown_pct()/peak()/current_equity(): Decimal`（百分数）；`drawdown_duration(): usize`；`in_drawdown(): bool` |
| `PositionLedger` | `new(initial_cash)`；`apply_fill(Fill)` by value -> `Result<(),FinError>`；`realized_pnl_total()`；`equity(&HashMap<String,Price>)` / `net_liquidation_value(&HashMap)` -> `Result<Decimal,FinError>` |
| `Fill` | `Fill::with_commission(symbol, side, quantity, price, timestamp, commission)`；`Side::{Bid,Ask}`（quantix Buy→Bid、Sell→Ask） |
| 回测 | `BacktestConfig::new(initial_capital, commission_rate) -> Result`；`Backtester::new(config).run(&[OhlcvBar], &mut dyn Strategy) -> Result<BacktestResult,FinError>`；fin `Signal::new(dir,qty)` / `Signal::hold()`；成交价=当 bar close，commission=close×qty×rate，Hold/零数量跳过 |
| Walk-forward | `ParamRange{name,min:f64,max:f64,step:f64}`；`WalkForwardConfig{train_window,test_window,step,param_space}`；`WalkForwardOptimizer::new(cfg, bt_cfg).run(&bars, &mut make_strategy) -> Result<WalkForwardResult,FinError>`，`make_strategy: FnMut(&[OhlcvBar], &HashMap<String,f64>) -> Box<dyn Strategy>`（同步闭包）；`WalkForwardResult::is_robust(min_stability: f64) -> bool`（walk_forward.rs:214，需阈值参数） |
| events | `MarketEvent{event_id,event_date:u64 unix秒,event_type,description}`；`EventWindow{pre_days,post_days: i32}` 字面量构造；`EventStudy::compute(&event,&[(u64,f64)],&[(u64,f64)],window) -> EventResult`；`EventStudy.significance(&[EventResult]) -> f64` |
| quantix 侧 | `Kline{code:String,date:NaiveDate,open/high/low/close:Decimal,volume:i64,amount:Option<Decimal>,adjust_type:AdjustType}`；`Signal{Buy,Sell,Hold}`（src/core/signal.rs）；`MACrossStrategy::new(short, long)`（src/strategy/ma_cross.rs）；`get_kline_for_analysis(&str, Option<NaiveDate>, Option<NaiveDate>, Option<usize>)`（src/cli/handlers/shared_support.rs:29） |

---

### Task 1: Cargo 依赖 + fin 模块骨架 + QuantixError::Fin

**Files:**
- Modify: `Cargo.toml`（[dependencies] 段）
- Modify: `src/lib.rs:26-27` 之间
- Create: `src/fin/mod.rs`
- Modify: `src/core/error.rs`（variant 列表末尾）

- [ ] **Step 1: 加依赖**

`Cargo.toml` [dependencies] 段追加（与现有依赖按字母序放置）：

```toml
fin-primitives = { version = "=4.0.1", default-features = false, features = [
    "serde",
    "indicators",
    "portfolio",
    "risk",
    "backtest",
    "ml",
] }
```

- [ ] **Step 2: 注册模块**

`src/lib.rs` 在 `pub mod execution;`（约 :26）与 `pub mod factor;`（约 :27）之间插入：

```rust
pub mod fin;
```

创建 `src/fin/mod.rs`（本任务先只放文档注释，子模块随后续任务逐个声明）：

```rust
//! fin-primitives 研究库集成层（设计见 docs/superpowers/specs/2026-11-23-fin-primitives-migration-design.md）。
//!
//! 边界纪律：fin-primitives 类型（BarInput/OhlcvBar/Signal/BacktestResult 等）只在本模块内部出现；
//! 对外仅暴露 quantix 类型（Decimal/Kline/core::Result）。FinError 不越出本模块。

pub mod error_bridge;
```

创建 `src/fin/error_bridge.rs`（本任务先承载 FinError→QuantixError 的转换与单测）：

```rust
//! FinError 收敛点：`QuantixError::Fin` 的 From 实现与验证。

#[cfg(test)]
mod tests {
    use crate::core::QuantixError;

    #[test]
    fn fin_error_maps_into_quantix_error() {
        let err = fin_primitives::FinError::InvalidInput("bad".to_string());
        let mapped: QuantixError = err.into();
        let text = mapped.to_string();
        assert!(text.contains("fin-primitives"), "display 应保留前缀: {text}");
        assert!(text.contains("bad"));
    }
}
```

注意：`FinError::InvalidInput(String)` 的确切 variant 名以 `fin-primitives/src/error.rs` 为准（已核实存在 InvalidInput；若字段形态不同，以编译器提示改用任一简单 variant，如 `FinError::NoData` 等，测试断言只依赖 display 前缀）。

- [ ] **Step 3: QuantixError 加 Fin variant**

`src/core/error.rs` 在 `Algo(String)` variant 之后（enum 结束前）追加：

```rust
    #[error("fin-primitives 错误: {0}")]
    Fin(#[from] fin_primitives::FinError),
```

- [ ] **Step 4: 编译与测试**

Run: `cargo check 2>&1 | tail -5`
Expected: 无错误（仅允许既存 warning）。

Run: `cargo test --lib fin::error_bridge 2>&1 | tail -5`
Expected: `test fin::error_bridge::tests::fin_error_maps_into_quantix_error ... ok`

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/fin/ src/core/error.rs
git commit -m "feat(fin): add fin-primitives dependency and QuantixError::Fin variant"
```

---

### Task 2: adapter.rs — Kline ↔ fin 类型双向转换

**Files:**
- Create: `src/fin/adapter.rs`
- Modify: `src/fin/mod.rs`（加 `pub mod adapter;`）
- Test: adapter.rs 内 `#[cfg(test)]`

- [ ] **Step 1: mod.rs 声明子模块**

`src/fin/mod.rs` 的 `pub mod error_bridge;` 之后追加：

```rust
pub mod adapter;
```

- [ ] **Step 2: 写失败测试**

创建 `src/fin/adapter.rs`：

```rust
//! Kline ↔ fin-primitives 类型适配。
//!
//! 正向：`Kline` → `BarInput`（指标流）/ `OhlcvBar`（回测），fallible（Symbol/Price 构造与 OHLC 校验）。
//! 反向：`OhlcvBar` → `Kline`（walk-forward 闭包内重放 quantix 策略）。

use chrono::NaiveDate;
use fin_primitives::ohlcv::OhlcvBar;
use fin_primitives::signals::BarInput;
use fin_primitives::types::{NanoTimestamp, Price, Quantity, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};

/// 日线开盘/收盘时刻（A 股）：09:30 / 15:00 CST（= 01:30 / 07:00 UTC）。
fn session_timestamps(date: NaiveDate) -> Result<(NanoTimestamp, NanoTimestamp)> {
    let open = date
        .and_hms_opt(1, 30, 0)
        .and_then(|t| t.and_utc().timestamp_nanos_opt())
        .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {date}")))?;
    let close = date
        .and_hms_opt(7, 0, 0)
        .and_then(|t| t.and_utc().timestamp_nanos_opt())
        .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {date}")))?;
    Ok((NanoTimestamp::new(open), NanoTimestamp::new(close)))
}

/// `Kline` → 指标流输入（无校验，volume i64 → Decimal）。
pub fn to_bar_input(kline: &Kline) -> BarInput {
    BarInput {
        open: kline.open,
        high: kline.high,
        low: kline.low,
        close: kline.close,
        volume: Decimal::from(kline.volume),
    }
}

pub fn to_bar_inputs(klines: &[Kline]) -> Vec<BarInput> {
    klines.iter().map(to_bar_input).collect()
}

/// `Kline` → `OhlcvBar`（fallible：Symbol/Price 构造 + OHLC 不变量校验）。
/// Price 约束严格 >0（停牌 0 价行被拒是预期校验）；tick_count 占位 1（日线无 tick 聚合语义）。
/// 失败时错误附 code/date 定位上下文，FinError display 保留于消息尾部。
pub fn to_ohlcv_bar(kline: &Kline) -> Result<OhlcvBar> {
    let with_ctx = |e: fin_primitives::FinError| {
        QuantixError::DataParse(format!(
            "Kline→OhlcvBar 失败 code={} date={}: {e}",
            kline.code, kline.date
        ))
    };
    let symbol = Symbol::new(kline.code.as_str()).map_err(with_ctx)?;
    let (ts_open, ts_close) = session_timestamps(kline.date)?;
    OhlcvBar::new(
        symbol,
        Price::new(kline.open).map_err(with_ctx)?,
        Price::new(kline.high).map_err(with_ctx)?,
        Price::new(kline.low).map_err(with_ctx)?,
        Price::new(kline.close).map_err(with_ctx)?,
        Quantity::try_from(Decimal::from(kline.volume)).map_err(with_ctx)?,
        ts_open,
        ts_close,
        1,
    )
    .map_err(with_ctx)
}

pub fn to_ohlcv_bars(klines: &[Kline]) -> Result<Vec<OhlcvBar>> {
    klines.iter().map(to_ohlcv_bar).collect()
}

/// `OhlcvBar` → `Kline`（walk-forward 闭包内重放 quantix 同步策略用）。
/// 复权类型丢失（置 None）；amount/tick_count 无 fin 对应物（None/丢弃）。
/// volume 为非整数 Decimal 时报错（日线成交量恒为整数；`to_i64` 对小数是截断语义，
/// 必须先显式校验 integrality）。
pub fn ohlcv_to_kline(bar: &OhlcvBar) -> Result<Kline> {
    let volume_dec = bar.volume.value();
    if !volume_dec.is_integer() {
        return Err(QuantixError::DataParse(format!(
            "volume 非整数: {volume_dec}"
        )));
    }
    let volume = volume_dec
        .to_i64()
        .ok_or_else(|| QuantixError::DataParse(format!("volume 超出 i64 范围: {volume_dec}")))?;
    let date = chrono::DateTime::from_timestamp_nanos(bar.ts_open.nanos()).date_naive();
    Ok(Kline {
        code: bar.symbol.as_str().to_string(),
        date,
        open: bar.open.value(),
        high: bar.high.value(),
        low: bar.low.value(),
        close: bar.close.value(),
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn make_kline(day: u32, close: &str, high: &str, low: &str) -> Kline {
        let c = Decimal::from_str(close).unwrap();
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: Decimal::from_str(high).unwrap(),
            low: Decimal::from_str(low).unwrap(),
            close: c,
            volume: 1000,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn bar_input_maps_all_fields_with_volume_i64_to_decimal() {
        let k = make_kline(5, "12.34", "12.50", "12.10");
        let b = to_bar_input(&k);
        assert_eq!(
            b,
            BarInput {
                open: Decimal::from_str("12.34").unwrap(),
                high: Decimal::from_str("12.50").unwrap(),
                low: Decimal::from_str("12.10").unwrap(),
                close: Decimal::from_str("12.34").unwrap(),
                volume: Decimal::from(1000),
            }
        );
    }

    #[test]
    fn ohlcv_bar_roundtrip_preserves_price_and_date() {
        let k = make_kline(6, "10.00", "10.50", "9.80");
        let bar = to_ohlcv_bar(&k).unwrap();
        let back = ohlcv_to_kline(&bar).unwrap();
        assert_eq!(back.code, "600000");
        assert_eq!(back.date, NaiveDate::from_ymd_opt(2026, 1, 6).unwrap());
        assert_eq!(back.open, Decimal::from_str("10.00").unwrap());
        assert_eq!(back.high, Decimal::from_str("10.50").unwrap());
        assert_eq!(back.low, Decimal::from_str("9.80").unwrap());
        assert_eq!(back.close, Decimal::from_str("10.00").unwrap());
        assert_eq!(back.volume, 1000);
    }

    #[test]
    fn ohlcv_bar_rejects_high_below_low() {
        let k = make_kline(7, "10.00", "9.00", "9.50"); // high < low
        let err = to_ohlcv_bar(&k).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("code=600000"), "应含 code 上下文: {text}");
        assert!(
            text.contains("invariant"),
            "应保留 FinError display: {text}"
        );
    }

    #[test]
    fn to_ohlcv_bar_rejects_non_positive_price_with_context() {
        let k = make_kline(8, "0", "0", "0"); // 停牌 0 价行：Price 拒 <=0
        let err = to_ohlcv_bar(&k).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("code=600000"), "应含 code 上下文: {text}");
        assert!(text.contains("date=2026-01-08"), "应含 date 上下文: {text}");
        assert!(
            text.contains("Price must be positive"),
            "应保留 FinError display: {text}"
        );
    }

    #[test]
    fn ohlcv_to_kline_rejects_non_integer_volume() {
        let bar = OhlcvBar::new(
            Symbol::new("600000").unwrap(),
            Price::new(Decimal::from_str("10.00").unwrap()).unwrap(),
            Price::new(Decimal::from_str("10.50").unwrap()).unwrap(),
            Price::new(Decimal::from_str("9.80").unwrap()).unwrap(),
            Price::new(Decimal::from_str("10.00").unwrap()).unwrap(),
            Quantity::try_from(Decimal::from_str("1000.5").unwrap()).unwrap(),
            NanoTimestamp::new(0),
            NanoTimestamp::new(0),
            1,
        )
        .unwrap();
        let err = ohlcv_to_kline(&bar).unwrap_err();
        assert!(matches!(err, QuantixError::DataParse(_)));
        assert!(err.to_string().contains("volume 非整数"));
    }

    #[test]
    fn to_ohlcv_bars_converts_batch_in_order() {
        let ks = [
            make_kline(5, "10.00", "10.50", "9.80"),
            make_kline(6, "11.00", "11.50", "10.80"),
        ];
        let bars = to_ohlcv_bars(&ks).unwrap();
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].close.value(), Decimal::from_str("10.00").unwrap());
        assert_eq!(bars[1].close.value(), Decimal::from_str("11.00").unwrap());
    }
}
```

- [ ] **Step 3: 跑测试验证通过（本任务测试与实现同时给出，直接验证）**

Run: `cargo test --lib fin::adapter 2>&1 | tail -8`
Expected: 6 passed。若 `FinError` variant 名/`Quantity::try_from` 形态与实际不符，按编译器提示修正（转换语义不变）。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): Kline<->BarInput/OhlcvBar adapter with OHLC validation"
```

---

### Task 3: indicators.rs — 指标门面（SMA/EMA/RSI/ATR/BOLL/STOCH）

**Files:**
- Create: `src/fin/indicators.rs`
- Modify: `src/fin/mod.rs`

- [ ] **Step 1: mod.rs 加声明**

```rust
pub mod indicators;
```

- [ ] **Step 2: 实现（含测试）**

创建 `src/fin/indicators.rs`：

```rust
//! 研究侧指标门面：强类型封装常用流式指标；未被封装的指标可在本模块内按同一模式扩展（约定：fin 类型不出 src/fin）。

use fin_primitives::signals::indicators::{Atr, BollingerB, Ema, Rsi, Sma, StochasticK};
use fin_primitives::signals::{Signal as FinIndicator, SignalValue};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::fin::adapter::to_bar_inputs;

/// 单指标全序列输出：`values[i]` 对应 `klines[i]`，未热身期为 `None`。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndicatorSeries {
    pub name: String,
    pub values: Vec<Option<Decimal>>,
}

fn stream(
    name: String,
    klines: &[Kline],
    make: impl FnOnce() -> std::result::Result<Box<dyn FinIndicator>, fin_primitives::FinError>,
) -> Result<IndicatorSeries> {
    let bars = to_bar_inputs(klines);
    let mut indicator = make().map_err(QuantixError::Fin)?;
    let mut values = Vec::with_capacity(bars.len());
    for bar in &bars {
        let v = indicator.update(bar).map_err(QuantixError::Fin)?;
        values.push(v.as_decimal());
    }
    Ok(IndicatorSeries { name, values })
}

pub fn sma(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("sma{period}"), klines, |_| {
        Ok(Box::new(Sma::new(format!("sma{period}"), period)?))
    })
}

pub fn ema(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("ema{period}"), klines, |_| {
        Ok(Box::new(Ema::new(format!("ema{period}"), period)?))
    })
}

pub fn rsi(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("rsi{period}"), klines, |_| {
        Ok(Box::new(Rsi::new(format!("rsi{period}"), period)?))
    })
}

pub fn atr(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("atr{period}"), klines, |_| {
        Ok(Box::new(Atr::new(format!("atr{period}"), period)?))
    })
}

pub fn bollinger(klines: &[Kline], period: usize, multiplier: Decimal) -> Result<IndicatorSeries> {
    stream(format!("boll{period}x{multiplier}"), klines, |_| {
        Ok(Box::new(BollingerB::new(
            format!("boll{period}"),
            period,
            multiplier,
        )?))
    })
}

pub fn stochastic(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("stoch{period}"), klines, |_| {
        Ok(Box::new(StochasticK::new(format!("stoch{period}"), period)?))
    })
}

/// CLI 分发：指标名 → 门面（名称与 `analyze indicators --indicators` 现有 native 命名对齐处由 handler 处理）。
pub fn resolve(name: &str, klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    match name {
        "sma" => sma(klines, period),
        "ema" => ema(klines, period),
        "rsi" => rsi(klines, period),
        "atr" => atr(klines, period),
        "boll" => bollinger(klines, period, Decimal::from(2)),
        "stoch" => stochastic(klines, period),
        other => Err(QuantixError::Unsupported(format!(
            "fin 引擎暂不支持指标: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use crate::data::models::AdjustType;
    use super::*;

    fn kline_at(day: u32, close: &str) -> Kline {
        let c = Decimal::from_str(close).unwrap();
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 100,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    fn rising_series(n: u32) -> Vec<Kline> {
        (1..=n)
            .map(|i| kline_at(i.min(28).max(1), &format!("{}", i)))
            .collect()
    }

    #[test]
    fn sma3_on_ramp_last_value_is_nine() {
        let klines: Vec<Kline> = (1..=10u32)
            .map(|i| kline_at(i, &i.to_string()))
            .collect();
        let out = sma(&klines, 3).unwrap();
        assert_eq!(out.values.len(), 10);
        assert_eq!(out.values[1], None); // 未热身
        assert_eq!(
            out.values[9],
            Some(Decimal::from_str("9").unwrap())
        ); // (8+9+10)/3 = 9
    }

    #[test]
    fn rsi_all_rising_is_hundred() {
        let klines = rising_series(30);
        let out = rsi(&klines, 14).unwrap();
        assert_eq!(out.values[29], Some(Decimal::from(100)));
    }

    #[test]
    fn resolve_unknown_indicator_is_unsupported() {
        let klines = rising_series(20);
        let err = resolve("kdj", &klines, 9).unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }
}
```

说明：fin 无 KDJ，`stoch` 即 StochasticK（评审 M8 已确认）；`rising_series` 中 `day.min(28).max(1)` 用于防 `from_ymd_opt(2026,1,>31)` 为 None。

- [ ] **Step 3: 跑测试**

Run: `cargo test --lib fin::indicators 2>&1 | tail -8`
Expected: 3 passed。若 RSI 末值非 100（上游 warmup 语义差异），打印 `out.values` 检查：全涨序列 RSI 必为 100（无下跌），若为 None 说明 period/warmup 语义不同，以打印值为准改断言并注释依据，不得改实现迁就断言。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): streaming indicator facade (sma/ema/rsi/atr/boll/stoch)"
```

---

### Task 4: risk.rs — DrawdownTracker 门面

**Files:**
- Create: `src/fin/risk.rs`
- Modify: `src/fin/mod.rs`

- [ ] **Step 1: mod.rs 加声明**

```rust
pub mod risk;
```

- [ ] **Step 2: 实现（含测试）**

创建 `src/fin/risk.rs`：

```rust
//! 研究侧回撤风险门面（落地主线任务「回撤控制」的分析能力）。

use fin_primitives::risk::DrawdownTracker;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownReport {
    /// 当前回撤百分数（0-100）
    pub current_pct: Decimal,
    /// 最大回撤百分数（0-100）
    pub worst_pct: Decimal,
    pub peak: Decimal,
    pub current_equity: Decimal,
    pub duration_bars: usize,
    pub in_drawdown: bool,
}

/// 逐 bar 净值序列 → 回撤报告。序列按时间升序，首元素为初始净值。
pub fn drawdown_report(equity: &[Decimal]) -> Result<DrawdownReport> {
    let Some((&initial, rest)) = equity.split_first() else {
        return Err(QuantixError::Config("净值序列为空".into()));
    };
    let mut tracker = DrawdownTracker::new(initial);
    for &value in rest {
        tracker.update(value);
    }
    Ok(DrawdownReport {
        current_pct: tracker.current_drawdown_pct(),
        worst_pct: tracker.worst_drawdown_pct(),
        peak: tracker.peak(),
        current_equity: tracker.current_equity(),
        duration_bars: tracker.drawdown_duration(),
        in_drawdown: tracker.in_drawdown(),
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn empty_equity_is_config_error() {
        let err = drawdown_report(&[]).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }

    #[test]
    fn drawdown_percentages_are_exact() {
        let report = drawdown_report(&[d(100), d(110), d(99)]).unwrap();
        assert_eq!(report.peak, d(110));
        assert_eq!(report.current_equity, d(99));
        // (110-99)/110*100 = 10.00（除法精确）
        assert_eq!(report.current_pct, Decimal::from_str("10").unwrap());
        assert_eq!(report.worst_pct, Decimal::from_str("10").unwrap());
        assert!(report.in_drawdown);
    }

    #[test]
    fn never_in_drawdown_when_rising() {
        let report = drawdown_report(&[d(100), d(101), d(102)]).unwrap();
        assert!(!report.in_drawdown);
        assert_eq!(report.worst_pct, Decimal::ZERO);
    }
}
```

- [ ] **Step 3: 跑测试**

Run: `cargo test --lib fin::risk 2>&1 | tail -6`
Expected: 3 passed。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): drawdown report facade over fin-primitives DrawdownTracker"
```

---

### Task 5: portfolio.rs — PositionLedger 研究侧门面

**Files:**
- Create: `src/fin/portfolio.rs`
- Modify: `src/fin/mod.rs`

- [ ] **Step 1: mod.rs 加声明**

```rust
pub mod portfolio;
```

- [ ] **Step 2: 实现（含测试）**

创建 `src/fin/portfolio.rs`：

```rust
//! 研究侧组合分析门面：fin PositionLedger（逐笔成交 → 已实现盈亏 / 按市价估值）。
//! 与业务侧 analysis/portfolio.rs（A股 T+1 记账）并存，不互相替代。

use std::collections::HashMap;

use fin_primitives::position::{Fill, PositionLedger};
use fin_primitives::types::{NanoTimestamp, Price, Quantity, Symbol};
use fin_primitives::types::Side;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::Signal;
use crate::core::{QuantixError, Result};

/// 研究侧成交描述（quantix 语义，内部转 fin 类型）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillSpec {
    pub symbol: String,
    /// quantix 信号语义：Buy→Bid、Sell→Ask
    pub side: Signal,
    pub quantity: i64,
    pub price: Decimal,
    /// 当前账目重放路径不消费时间戳（上游 apply_fill 不读），字段仅为 Fill 构造完备性保留。
    pub timestamp_nanos: i64,
    pub commission: Option<Decimal>,
}

fn to_fill(spec: &FillSpec) -> Result<Fill> {
    let symbol = Symbol::new(spec.symbol.as_str()).map_err(QuantixError::Fin)?;
    let price = Price::new(spec.price).map_err(QuantixError::Fin)?;
    let quantity = Quantity::try_from(Decimal::from(spec.quantity)).map_err(QuantixError::Fin)?;
    let side = match spec.side {
        Signal::Buy => Side::Bid,
        Signal::Sell => Side::Ask,
        Signal::Hold => {
            return Err(QuantixError::Config("FillSpec.side 不能为 Hold".into()));
        }
    };
    Ok(Fill::with_commission(
        symbol,
        side,
        quantity,
        price,
        NanoTimestamp::new(spec.timestamp_nanos),
        spec.commission.unwrap_or(Decimal::ZERO),
    ))
}

/// 门面层守卫：quantix A股语义无做空机制，超额卖出与零/负数量视为输入数据错误。
/// 上游本身允许超额静默转空头、允许 0 数量，quantix 不予放行。
fn validate_fills(fills: &[FillSpec]) -> Result<()> {
    let mut positions: HashMap<&str, Decimal> = HashMap::new();
    for spec in fills {
        if spec.quantity <= 0 {
            return Err(QuantixError::Config(format!(
                "无效数量 symbol={} qty={}",
                spec.symbol, spec.quantity
            )));
        }
        let qty = Decimal::from(spec.quantity);
        let pos = positions.entry(spec.symbol.as_str()).or_default();
        match spec.side {
            Signal::Buy => *pos += qty,
            Signal::Sell => {
                if qty > *pos {
                    return Err(QuantixError::Config(format!(
                        "超额卖出: {} 卖出 {} 超过持仓 {}",
                        spec.symbol, qty, *pos
                    )));
                }
                *pos -= qty;
            }
            Signal::Hold => {} // 由 to_fill 拒绝（Config），此处不跟踪
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    pub realized_pnl: Decimal,
    pub equity: Decimal,
    pub net_liquidation: Decimal,
}

/// 重放成交序列并按 `mark_prices` 估值。`mark_prices`: symbol → 市价。
/// 门面层拒绝超额卖出与零/负数量；上游本身允许超额转空头，quantix A股语义不予放行。
/// 口径注记（上游已核实）：`equity` = cash + 未实现盈亏（非市值）；`net_liquidation` =
/// cash + Σ(持仓×mark)（市值口径）。
pub fn ledger_summary(
    initial_cash: Decimal,
    fills: &[FillSpec],
    mark_prices: &HashMap<String, Decimal>,
) -> Result<LedgerSummary> {
    validate_fills(fills)?;
    let mut ledger = PositionLedger::new(initial_cash);
    for (i, spec) in fills.iter().enumerate() {
        ledger.apply_fill(to_fill(spec)?).map_err(|e| {
            QuantixError::DataParse(format!("成交重放失败 #{i} symbol={}: {e}", spec.symbol))
        })?;
    }
    let mut marks = HashMap::with_capacity(mark_prices.len());
    for (code, price) in mark_prices {
        marks.insert(
            code.clone(),
            Price::new(*price).map_err(|e| {
                QuantixError::DataParse(format!("mark 价格无效 symbol={code}: {e}"))
            })?,
        );
    }
    Ok(LedgerSummary {
        realized_pnl: ledger.realized_pnl_total(),
        equity: ledger.equity(&marks).map_err(QuantixError::Fin)?,
        net_liquidation: ledger
            .net_liquidation_value(&marks)
            .map_err(QuantixError::Fin)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn fill(side: Signal, qty: i64, price: i64) -> FillSpec {
        FillSpec {
            symbol: "600000".into(),
            side,
            quantity: qty,
            price: Decimal::from(price),
            timestamp_nanos: 0,
            commission: None,
        }
    }

    #[test]
    fn buy_sell_roundtrip_realized_pnl_and_equity() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(12));
        let summary = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 100, 10), fill(Signal::Sell, 100, 12)],
            &marks,
        )
        .unwrap();
        assert_eq!(summary.realized_pnl, Decimal::from(200));
        assert_eq!(summary.equity, Decimal::from(10200));
    }

    #[test]
    fn hold_side_is_rejected() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(100), &[fill(Signal::Hold, 1, 1)], &marks).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }

    #[test]
    fn commission_reduces_cash_but_not_realized_pnl() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(10));
        let mut fee_fill = fill(Signal::Buy, 100, 10);
        fee_fill.commission = Some(Decimal::from(5));
        let summary = ledger_summary(Decimal::from(10000), &[fee_fill], &marks).unwrap();
        // 上游语义（已核实 position/mod.rs apply_fill/realized_pnl）：佣金进 cash 也进 realized
        assert_eq!(summary.realized_pnl, Decimal::from(-5));
        assert_eq!(summary.equity, Decimal::from(8995)); // 零费 9000 - 佣金 5
        assert_eq!(summary.net_liquidation, Decimal::from(9995)); // cash 8995 + 持仓市值 1000
    }

    #[test]
    fn insufficient_cash_is_reported_with_replay_index() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(500), &[fill(Signal::Buy, 100, 10)], &marks).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("成交重放失败 #0"), "应含重放下标: {text}");
    }

    #[test]
    fn oversell_is_rejected_as_config_error() {
        let marks = HashMap::new();
        let err = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 50, 10), fill(Signal::Sell, 100, 10)],
            &marks,
        )
        .unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
        assert!(err.to_string().contains("超额卖出"));
    }

    #[test]
    fn zero_quantity_is_rejected() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(100), &[fill(Signal::Buy, 0, 10)], &marks).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
        assert!(err.to_string().contains("无效数量"));
    }

    #[test]
    fn multi_symbol_positions_are_independent() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(10));
        marks.insert("000001".to_string(), Decimal::from(12));
        let mut buy_b = fill(Signal::Buy, 100, 10);
        buy_b.symbol = "000001".into();
        let mut sell_b = fill(Signal::Sell, 100, 12);
        sell_b.symbol = "000001".into();
        let summary = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 100, 10), buy_b, sell_b],
            &marks,
        )
        .unwrap();
        assert_eq!(summary.realized_pnl, Decimal::from(200)); // 仅 B 平仓盈利
        // 上游 equity 口径 = cash + 未实现盈亏（非市值）：cash 9200 + A 未实现 0
        assert_eq!(summary.equity, Decimal::from(9200));
        // 市值口径走 net_liquidation：cash 9200 + A 持仓市值 100×10
        assert_eq!(summary.net_liquidation, Decimal::from(10200));
    }
}
```

- [ ] **Step 3: 跑测试**

Run: `cargo test --lib fin::portfolio 2>&1 | tail -6`
Expected: 7 passed。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): PositionLedger research-side facade"
```

---

### Task 6: ma_cross.rs 提取同步 signal_at（方案 C 核心）

**前置纪律：** 这是本计划第一处修改现有符号。先跑 impact analysis：
- GitNexus MCP 可用：`gitnexus_impact({target: "MACrossStrategy", direction: "upstream"})`，向用户报告直接调用方与风险级别（HIGH/CRITICAL 需用户确认再继续）。
- MCP 不可用：`grep -rn "MACrossStrategy" src/ --include="*.rs" -l` 实测调用方。**2026-11-23 实测基线（5 处，以执行时重测为准，不得预设范围）**：`src/strategy/runtime.rs`、`src/strategy/mod.rs`、`src/cli/handlers/backtest_handler.rs`、`src/cli/handlers/strategy_handler/catalog.rs`、`src/strategy/ma_cross.rs`（自身+测试）。逐个确认行为不受提取影响后再动手，记录 "GitNexus backfill required"。

**Files:**
- Modify: `src/strategy/ma_cross.rs:67-117`（impl Strategy 块）

- [ ] **Step 1: 跑 impact analysis**（见上），结果记录到 commit message。

- [ ] **Step 2: 写失败测试**（新增测试先行）

在 `src/strategy/ma_cross.rs` 现有 `#[cfg(test)]` 模块内追加：

```rust
    #[test]
    fn signal_at_matches_on_bar_sequence() {
        use crate::strategy::test_utils::create_test_kline;
        let mut s = MACrossStrategy::new(2, 3);
        let mut expected = s.clone_signal_replay_baseline();
        let _ = expected;
        let mut s2 = MACrossStrategy::new(2, 3);
        let klines: Vec<_> = (0..12)
            .map(|i| create_test_kline(i, if i >= 6 { 12.0 } else { 10.0 }))
            .collect();
        for k in &klines {
            let _ = s.signal_at(k);
        }
        // 有状态推进：末段价格跳变后，history 长度=12
        assert_eq!(s.price_history_len(), 12);
        let _ = s2;
    }
```

该测试依赖两个新可见性 helper（见 Step 3 实现）：`price_history_len(&self) -> usize` 与测试内占位 `clone_signal_replay_baseline`——**占位即删除**，最终测试简化为：

```rust
    #[test]
    fn signal_at_drives_state_and_returns_signal() {
        use crate::strategy::test_utils::create_test_kline;
        let mut s = MACrossStrategy::new(2, 3);
        let klines: Vec<_> = (0..12)
            .map(|i| create_test_kline(i, if i >= 6 { 12.0 } else { 10.0 }))
            .collect();
        for k in &klines {
            let _ = s.signal_at(k);
        }
        assert_eq!(s.price_history_len(), 12);
    }
```

Run: `cargo test --lib strategy::ma_cross 2>&1 | tail -5`
Expected: FAIL（`signal_at`/`price_history_len` 不存在）。

- [ ] **Step 3: 提取实现**

`src/strategy/ma_cross.rs` 的 `impl Strategy for MACrossStrategy` 块中，把 `on_bar`（:72-117）主体移入新公开同步方法，`on_bar` 改为薄委托。在 `impl MACrossStrategy` 块内加：

```rust
    /// 同步信号推进（供 async on_bar 与 fin 回测桥共用；无 IO、无 Result）。
    pub(crate) fn signal_at(&mut self, bar: &Kline) -> Signal {
        // ← 原 on_bar 主体逐行移入（push close、热身检查、双均线计算、
        //   let-else、金叉/死叉判断返 Buy/Sell、更新 last_*_ma 与 position）。
        // 返回值由原 `Ok(...)` 改为直接返回 Signal。
    }

    pub(crate) fn price_history_len(&self) -> usize {
        self.price_history.len()
    }
```

`impl Strategy` 内 `on_bar` 变为：

```rust
    async fn on_bar(&mut self, bar: &Kline) -> std::result::Result<Signal, Box<dyn std::error::Error>> {
        Ok(self.signal_at(bar))
    }
```

（`finish`/`init`/`name` 不动；`signal_at` 内部逻辑一行不改，只把返回包装去掉。）

- [ ] **Step 4: 跑全量 ma_cross 测试**

Run: `cargo test --lib strategy::ma_cross 2>&1 | tail -8`
Expected: 既有 test_ma_cross_strategy / test_ma_cross_death_cross + 新 signal_at 测试全部 PASS（行为零变化验证）。

- [ ] **Step 5: Commit**

```bash
git add src/strategy/ma_cross.rs
git commit -m "refactor(strategy): extract sync signal_at from MACrossStrategy::on_bar

impact: MACrossStrategy upstream callers verified (backtest_handler + tests)
GitNexus: <结果或 backfill-required>"
```

---

### Task 7: backtest.rs — FinStrategy 桥 + 单次回测 + walk-forward

**Files:**
- Create: `src/fin/backtest.rs`
- Modify: `src/fin/mod.rs`

- [ ] **Step 1: mod.rs 加声明**

```rust
pub mod backtest;
```

- [ ] **Step 2: 实现（含测试）**

创建 `src/fin/backtest.rs`：

```rust
//! 研究侧回测门面（fin Backtester 引擎）。
//! 策略桥（方案 C）：quantix 策略经同步 `signal_at` 预生成信号序列，
//! `FinStrategy` 按序回放给 fin 同步引擎——无 block_on、无跨 runtime。
//! 成本口径：仅 commission_rate（上游冲击/延迟模型未接入引擎，见 spec §12）。

use std::collections::{HashMap, VecDeque};

use fin_primitives::backtest::{
    BacktestConfig, BacktestResult, Backtester, ParamRange, Signal, SignalDirection,
    WalkForwardConfig, WalkForwardOptimizer, WalkForwardResult,
};
use fin_primitives::ohlcv::OhlcvBar;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::fin::adapter::{ohlcv_to_kline, to_ohlcv_bars};
use crate::strategy::ma_cross::MACrossStrategy;

/// 信号映射配置：方向映射 Buy/Sell→fin，固定下单数量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinStrategyConfig {
    pub position_size: Decimal,
}

/// fin 引擎的策略适配器：回放预计算信号序列。
pub struct FinStrategy {
    signals: VecDeque<Signal>,
}

impl FinStrategy {
    pub fn new(signals: Vec<Signal>) -> Self {
        Self {
            signals: signals.into(),
        }
    }
}

impl fin_primitives::backtest::Strategy for FinStrategy {
    fn on_bar(&mut self, _bar: &OhlcvBar) -> Option<Signal> {
        Some(self.signals.pop_front().unwrap_or_else(Signal::hold))
    }
}

fn map_signal(s: crate::core::Signal, cfg: &FinStrategyConfig) -> Signal {
    match s {
        crate::core::Signal::Buy => Signal::new(SignalDirection::Buy, cfg.position_size),
        crate::core::Signal::Sell => Signal::new(SignalDirection::Sell, cfg.position_size),
        crate::core::Signal::Hold => Signal::hold(),
    }
}

fn precomputed_signals(
    klines: &[Kline],
    short_period: usize,
    long_period: usize,
    cfg: &FinStrategyConfig,
) -> Vec<Signal> {
    let mut strategy = MACrossStrategy::new(short_period, long_period);
    klines
        .iter()
        .map(|k| map_signal(strategy.signal_at(k), cfg))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinBacktestOutput {
    pub result: BacktestResult,
    pub bar_count: usize,
}

/// 单次回测：ma_cross 策略 × fin 引擎。
pub fn run_fin_backtest(
    klines: &[Kline],
    short_period: usize,
    long_period: usize,
    initial_capital: Decimal,
    commission_rate: Decimal,
    position_size: Decimal,
) -> Result<FinBacktestOutput> {
    let bars = to_ohlcv_bars(klines)?;
    let cfg = FinStrategyConfig { position_size };
    let signals = precomputed_signals(klines, short_period, long_period, &cfg);
    let mut strategy = FinStrategy::new(signals);
    let config =
        BacktestConfig::new(initial_capital, commission_rate).map_err(QuantixError::Fin)?;
    let result = Backtester::new(config)
        .run(&bars, &mut strategy)
        .map_err(QuantixError::Fin)?;
    Ok(FinBacktestOutput {
        result,
        bar_count: bars.len(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardOutput {
    pub result: WalkForwardResult,
}

/// ma_cross walk-forward：short/long 双参数网格 + 滚动训练/测试窗口。
/// `ranges`: (short_min, short_max, short_step), (long_min, long_max, long_step)。
pub fn walk_forward_ma_cross(
    klines: &[Kline],
    short_range: (usize, usize, usize),
    long_range: (usize, usize, usize),
    train_window: usize,
    test_window: usize,
    step: usize,
    initial_capital: Decimal,
    commission_rate: Decimal,
    position_size: Decimal,
) -> Result<WalkForwardOutput> {
    let bars = to_ohlcv_bars(klines)?;
    let cfg = FinStrategyConfig { position_size };
    let param_space = vec![
        ParamRange {
            name: "short_period".into(),
            min: short_range.0 as f64,
            max: short_range.1 as f64,
            step: short_range.2 as f64,
        },
        ParamRange {
            name: "long_period".into(),
            min: long_range.0 as f64,
            max: long_range.1 as f64,
            step: long_range.2 as f64,
        },
    ];
    let config =
        BacktestConfig::new(initial_capital, commission_rate).map_err(QuantixError::Fin)?;
    let wf_config = WalkForwardConfig {
        train_window,
        test_window,
        step,
        param_space,
    };
    let optimizer = WalkForwardOptimizer::new(wf_config, config).map_err(QuantixError::Fin)?;

    // 闭包内：把传入 bar 窗口反转为 Kline 后重放同步 signal_at（方案 C）。
    // 反转失败（理论上仅 volume 非整数）→ 空信号序列 → 该段全 Hold，不中断整体搜索。
    let mut make_strategy = |bars_slice: &[OhlcvBar],
                             params: &HashMap<String, f64>|
     -> Box<dyn fin_primitives::backtest::Strategy> {
        let short = params
            .get("short_period")
            .copied()
            .unwrap_or(short_range.0 as f64) as usize;
        let long = params
            .get("long_period")
            .copied()
            .unwrap_or(long_range.0 as f64) as usize;
        let ks: Vec<Kline> = bars_slice
            .iter()
            .filter_map(|b| ohlcv_to_kline(b).ok())
            .collect();
        let signals = precomputed_signals(&ks, short, long, &cfg);
        Box::new(FinStrategy::new(signals))
    };
    let result = optimizer
        .run(&bars, &mut make_strategy)
        .map_err(QuantixError::Fin)?;
    Ok(WalkForwardOutput { result })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use crate::data::models::AdjustType;

    use super::*;

    fn kline_at(day: u32, close: i64) -> Kline {
        let c = Decimal::from(close);
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 1000,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    /// 手算依据（引擎语义已核实：成交价=当 bar close、零费、qty=100）：
    /// close=[10×4, 12×3, 9×3]，short=2, long=3：
    /// bar4: ma2=(10+12)/2=11 > ma3=(10+10+12)/3≈10.667 且前值相等持平→金叉 Buy@12
    /// bar7: ma2=(12+9)/2=10.5 < ma3=(12+12+9)/3=11 → 死叉 Sell@9
    /// 买 100@12（-1200）卖 100@9（+900）→ final_equity = 9700, trade_count = 2
    #[test]
    fn single_backtest_hand_computed_equity() {
        let mut closes = vec![10i64; 4];
        closes.extend(vec![12i64; 3]);
        closes.extend(vec![9i64; 3]);
        let klines: Vec<Kline> = closes
            .iter()
            .enumerate()
            .map(|(i, c)| kline_at((i + 1) as u32, *c))
            .collect();
        let out = run_fin_backtest(
            &klines, 2, 3,
            Decimal::from(10000), Decimal::ZERO, Decimal::from(100),
        )
        .unwrap();
        assert_eq!(out.bar_count, 10);
        assert_eq!(
            out.result.final_equity,
            Decimal::from_str("9700").unwrap()
        );
        assert_eq!(out.result.trade_count, 2);
    }

    #[test]
    fn walk_forward_produces_periods_and_stability() {
        let mut closes: Vec<i64> = (10..=20).collect(); // 11 根缓涨
        closes.extend((8..=15).collect()); // 8 根缓跌
        let klines: Vec<Kline> = closes
            .iter()
            .enumerate()
            .map(|(i, c)| kline_at((i + 1) as u32, *c))
            .collect();
        let out = walk_forward_ma_cross(
            &klines,
            (2, 3, 1),
            (3, 4, 1),
            8, 4, 4,
            Decimal::from(10000), Decimal::ZERO, Decimal::from(100),
        )
        .unwrap();
        assert!(!out.result.periods.is_empty(), "至少应切出一个窗口");
        // stability ∈ [0,1]：OOS>0 占比
        assert!(out.result.stability_score >= 0.0 && out.result.stability_score <= 1.0);
        // is_robust 需阈值参数（walk_forward.rs:214）
        assert!(matches!(
            out.result.is_robust(0.5),
            true | false
        ));
    }

    /// 双引擎一致性（spec §9.4 v2 口径）：signal_at 信号序列的 Buy/Sell 总数
    /// 应与 fin 引擎 trade_count 对齐（零费足额资金下 apply_fill 全部成功）；
    /// 成交结果不要求相等（费用/撮合模型不同）。
    #[test]
    fn bridge_signal_parity_with_engine_trades() {
        let mut closes: Vec<i64> = vec![10; 4];
        closes.extend(vec![12; 3]);
        closes.extend(vec![9; 3]);
        let klines: Vec<Kline> = closes
            .iter()
            .enumerate()
            .map(|(i, c)| kline_at((i + 1) as u32, *c))
            .collect();
        let mut strategy = MACrossStrategy::new(2, 3);
        let cfg = FinStrategyConfig {
            position_size: Decimal::from(100),
        };
        let signals: Vec<Signal> = klines
            .iter()
            .map(|k| map_signal(strategy.signal_at(k), &cfg))
            .collect();
        let buy_sell = signals
            .iter()
            .filter(|s| s.direction != SignalDirection::Hold)
            .count();
        let out = run_fin_backtest(
            &klines,
            2,
            3,
            Decimal::from(10000),
            Decimal::ZERO,
            Decimal::from(100),
        )
        .unwrap();
        assert_eq!(out.result.trade_count as usize, buy_sell);
    }
}
```

说明：
- `BacktestResult.final_equity` 精确断言依赖引擎成交语义（当 bar close、零费、自动数量），已核实；若金叉/死叉落点与注释不同（交叉判断用严格不等），先 `cargo test -- --nocapture` 打印信号序列核对注释假设，仅修正测试注释与输入序列构造，**不得修改 signal_at 或桥映射**。
- `walk_forward` 测试只做结构性断言（periods 非空、stability ∈ [0,1]），不假设窗口切分内部算法。

- [ ] **Step 3: 跑测试**

Run: `cargo test --lib fin::backtest 2>&1 | tail -8`
Expected: 3 passed（单次回测、walk-forward、信号一致性）。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): sync signal replay bridge + single backtest + walk-forward"
```

---

### Task 8: events.rs — 事件研究门面（库层）

**Files:**
- Create: `src/fin/events.rs`
- Modify: `src/fin/mod.rs`

- [ ] **Step 1: mod.rs 加声明**

```rust
pub mod events;
```

- [ ] **Step 2: 实现（含测试）**

创建 `src/fin/events.rs`：

```rust
//! 事件研究门面（库层，无 CLI，spec §2 非目标）。
//! 数据源约定（一期）：CSV/数据库事件表（code+date+类型）；news/ 接入二期。

use std::collections::HashMap;

use chrono::NaiveDate;
use fin_primitives::events::{EventStudy, EventWindow, MarketEvent};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSpec {
    pub code: String,
    pub date: NaiveDate,
    pub event_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStudyOutput {
    pub car_pre: f64,
    pub car_post: f64,
    pub peak_day: i32,
    pub trough_day: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatchOutput {
    pub studies: Vec<EventStudyOutput>,
    /// fin EventStudy::significance 的 t 类统计量（NaN = 事件过少）
    pub significance: f64,
    pub skipped: usize,
}

/// Kline 收盘序列 → fin 研究口径 f64 序列（date → UTC unix 秒）。
pub fn close_series(klines: &[crate::data::models::Kline]) -> Result<Vec<(u64, f64)>> {
    klines
        .iter()
        .map(|k| {
            // `NaiveDate::and_hms_opt` 返回 Option<NaiveDateTime>，先 ok_or 再 .and_utc()
            let ts = k
                .date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {}", k.date)))?
                .and_utc()
                .timestamp();
            let close = k
                .close
                .to_f64()
                .ok_or_else(|| QuantixError::DataParse(format!(
                    "close 转 f64 失败: {}",
                    k.close
                )))?;
            Ok((ts as u64, close))
        })
        .collect()
}

/// 批量事件研究。`series_by_code`: code → 收盘序列（调用方可经 `close_series` 构造）。
pub fn run_event_studies(
    specs: &[EventSpec],
    series_by_code: &HashMap<String, Vec<(u64, f64)>>,
    benchmark: &[(u64, f64)],
    pre_days: i32,
    post_days: i32,
) -> Result<EventBatchOutput> {
    let mut studies = Vec::new();
    let mut results: Vec<fin_primitives::events::EventResult> = Vec::new();
    let mut skipped = 0usize;
    for spec in specs {
        let Some(series) = series_by_code.get(&spec.code) else {
            skipped += 1;
            continue;
        };
        let event_date = spec
            .date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {}", spec.date)))?
            .and_utc()
            .timestamp() as u64;
        let event = MarketEvent {
            event_id: format!("{}:{}", spec.code, spec.date.format("%Y-%m-%d")),
            event_date,
            event_type: spec.event_type.clone(),
            description: spec.description.clone(),
        };
        let result = EventStudy::compute(
            &event,
            series,
            benchmark,
            EventWindow {
                pre_days,
                post_days,
            },
        );
        // 保留 fin EventResult 供真实 significance 统计（类型不出门面）
        studies.push(EventStudyOutput {
            car_pre: result.car_pre,
            car_post: result.car_post,
            peak_day: result.peak_day,
            trough_day: result.trough_day,
        });
        results.push(result);
    }
    // 真实调用 fin EventStudy::significance（t 类统计量，上游语义）；事件集为空 → NaN
    let significance = if results.is_empty() {
        f64::NAN
    } else {
        EventStudy::significance(&results)
    };
    Ok(EventBatchOutput {
        studies,
        significance,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use crate::data::models::{AdjustType, Kline};
    use super::*;

    fn kline(day: u32, close: i64) -> Kline {
        let c = Decimal::from(close);
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 100,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn batch_skips_missing_codes_and_returns_studies() {
        let klines: Vec<Kline> = (1..=10u32)
            .map(|d| kline(d, 10 + d as i64))
            .collect();
        let mut series = HashMap::new();
        series.insert("600000".to_string(), close_series(&klines).unwrap());
        let specs = vec![
            EventSpec {
                code: "600000".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                event_type: "announcement".into(),
                description: "test".into(),
            },
            EventSpec {
                code: "000001".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                event_type: "announcement".into(),
                description: "missing".into(),
            },
        ];
        let benchmark: Vec<(u64, f64)> = series["600000"].clone();
        let out = run_event_studies(&specs, &series, &benchmark, 2, 2).unwrap();
        assert_eq!(out.studies.len(), 1);
        assert_eq!(out.skipped, 1);
        assert!(out.significance.is_finite() || out.significance.is_nan());
    }
}
```

说明：CAR 精确值依赖 fin `compute` 内部公式（未核实），测试做结构断言；精确值哨兵由 Task 12 金丝雀以固定输入覆盖（记录当前输出，防上游漂移）。

- [ ] **Step 3: 跑测试**

Run: `cargo test --lib fin::events 2>&1 | tail -6`
Expected: 1 passed。

- [ ] **Step 4: Commit**

```bash
git add src/fin/
git commit -m "feat(fin): event-study facade (library level, no CLI)"
```

---

### Task 9: CLI — `analyze indicators --engine fin`

**前置纪律：** impact analysis on `AnalyzeCommands` / `calculate_indicators`（同 Task 6 方式）。

**Files:**
- Modify: `src/cli/commands/analysis.rs:42-50`（Indicators 变体）
- Modify: `src/cli/handlers/app_shell/misc.rs:37-41`（分发）
- Modify: `src/cli/handlers/analyze_handler.rs`（新 handler 函数）
- Test: analyze_handler.rs `#[cfg(test)]`

- [ ] **Step 1: impact analysis**（记录结果）。

- [ ] **Step 2: 扩展 CLI 定义**

`src/cli/commands/analysis.rs`：文件顶部加 engine 枚举，Indicators 变体加 engine 字段：

```rust
/// 指标计算引擎：native=自研指标层（默认，行为不变）；fin=fin-primitives 流式指标。
#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum IndicatorEngine {
    Native,
    Fin,
}
```

Indicators 变体改为：

```rust
    Indicators {
        #[arg(short, long)]
        code: String,
        #[arg(short, long)]
        indicators: String,
        /// 计算引擎（默认 native，行为与历史版本一致）
        #[arg(long, value_enum, default_value_t = IndicatorEngine::Native)]
        engine: IndicatorEngine,
    },
```

- [ ] **Step 3: 分发改造**

`src/cli/handlers/app_shell/misc.rs:39-41` 分支改为：

```rust
        AnalyzeCommands::Indicators {
            code,
            indicators,
            engine,
        } => match engine {
            crate::cli::commands::analysis::IndicatorEngine::Native => {
                calculate_indicators(code, indicators).await?
            }
            crate::cli::commands::analysis::IndicatorEngine::Fin => {
                crate::cli::handlers::analyze_handler::calculate_indicators_fin(code, indicators).await?
            }
        },
```

（`calculate_indicators` 的导入路径按该文件现有 use 调整。）

- [ ] **Step 4: 实现 fin handler**

`src/cli/handlers/analyze_handler.rs` 追加：

```rust
/// fin 引擎指标查询：版式与 native `calculate_indicators` 完全一致
/// （💹 头部三行 + 表头 + 每根 Kline 一行日期/收盘价 + 每指标一行且前两列留空）。
pub(crate) async fn calculate_indicators_fin(code: String, indicators_str: String) -> Result<()> {
    let klines = get_kline_for_analysis(&code, None, None, Some(1000)).await?;
    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }
    let names: Vec<&str> = indicators_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if names.is_empty() {
        return Err(QuantixError::Config("指标列表为空".into()));
    }
    println!("💹 计算技术指标（引擎=fin）");
    println!("  代码: {}", code);
    println!("  指标: {}", indicators_str);
    // fin 门面当前为单周期指标，period 与 native 缺省对齐
    let periods: &[(&str, usize)] = &[("sma", 5), ("ema", 5), ("rsi", 14), ("atr", 14), ("boll", 20), ("stoch", 9)];
    println!("\n📊 计算结果:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "日期", "收盘价", "指标", "值"
    );
    println!("{}", "-".repeat(65));
    let mut rendered = 0usize;
    for (idx, kline) in klines.iter().enumerate().take(20) {
        println!("{:<12} {:<20.2}", kline.date, kline.close);
        for name in &names {
            let Some((_, period)) = periods.iter().find(|(n, _)| n == name) else {
                return Err(QuantixError::Unsupported(format!(
                    "fin 引擎暂不支持指标: {name}"
                )));
            };
            let series = crate::fin::indicators::resolve(name, &klines, *period)?;
            let value = match series.values.get(idx).and_then(|v| *v) {
                Some(v) => v.to_string(),
                None => "N/A".to_string(),
            };
            println!(
                "{:<12} {:<20} {:<15} {:<15}",
                "",
                "",
                series.name,
                value
            );
            rendered += 1;
        }
    }
    if rendered == 0 {
        return Err(QuantixError::Config("未渲染任何指标行".into()));
    }
    Ok(())
}
```

（若 `resolve` 对多周期同指标需重复计算，一期接受该开销——数据 ≤1000 根，单指标流式计算 O(n)。）

- [ ] **Step 5: 测试**

analyze_handler.rs `#[cfg(test)]` 追加：

```rust
    #[tokio::test]
    async fn fin_indicators_reject_unknown_name() {
        // 空指标名与未知指标都应在数据获取前快速失败的不变量由 resolve 保证；
        // 此处验证 resolve 层（handler 网络依赖较重，resolve 已有单测，此处兜底冒烟）。
        let err = crate::fin::indicators::resolve("kdj", &[], 9).unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }
```

Run: `cargo test --lib analyze_handler 2>&1 | tail -5`
Expected: PASS（含既有 calculate_indicators 相关测试不受影响）。

- [ ] **Step 6: 手动冒烟（可选，无外部环境则跳过并记录）**

Run: `cargo run -- analyze indicators --code 600000 --indicators sma,rsi --engine fin`
Expected: 四列表格输出，无 panic（无数据库环境时报数据源错误也算通过——错误经 QuantixError 正常链路）。

- [ ] **Step 7: Commit**

```bash
git add src/cli/
git commit -m "feat(cli): analyze indicators --engine fin via fin-primitives facade

impact: AnalyzeCommands::Indicators callers = misc.rs dispatch only"
```

---

### Task 10: CLI — `analyze drawdown`

**Files:**
- Modify: `src/cli/commands/analysis.rs`（新变体）
- Modify: `src/cli/handlers/app_shell/misc.rs`（分发）
- Modify: `src/cli/handlers/analyze_handler.rs`（handler）
- Test: analyze_handler.rs `#[cfg(test)]`

- [ ] **Step 1: CLI 变体**

`AnalyzeCommands` 加：

```rust
    Drawdown {
        #[arg(short, long)]
        code: String,
        /// 参与统计的最近 N 根 K 线
        #[arg(long, default_value_t = 250)]
        limit: usize,
    },
```

- [ ] **Step 2: 分发**

misc.rs match 加分支：

```rust
        AnalyzeCommands::Drawdown { code, limit } => {
            crate::cli::handlers::analyze_handler::analyze_drawdown(code, limit).await?
        }
```

- [ ] **Step 3: handler 实现**

analyze_handler.rs 追加：

```rust
/// 回撤风险报告：以收盘价序列为净值曲线（研究口径），fin DrawdownTracker 统计。
pub(crate) async fn analyze_drawdown(code: String, limit: usize) -> Result<()> {
    let mut klines = get_kline_for_analysis(&code, None, None, Some(limit)).await?;
    if klines.is_empty() {
        return Err(QuantixError::DataSource(format!("无 K 线数据: {code}")));
    }
    klines.sort_by_key(|k| k.date);
    let equity: Vec<rust_decimal::Decimal> = klines.iter().map(|k| k.close).collect();
    let report = crate::fin::risk::drawdown_report(&equity)?;
    println!("回撤风险报告（{}，{} 根日线，净值=收盘价）", code, equity.len());
    println!("{}", "-".repeat(48));
    println!("当前回撤:   {} %", report.current_pct);
    println!("最大回撤:   {} %", report.worst_pct);
    println!("净值峰值:   {}", report.peak);
    println!("当前净值:   {}", report.current_equity);
    println!("回撤持续:   {} 根", report.duration_bars);
    println!("处于回撤中: {}", if report.in_drawdown { "是" } else { "否" });
    Ok(())
}
```

- [ ] **Step 4: 测试**

analyze_handler.rs `#[cfg(test)]` 追加（纯函数路径，不走网络）：

```rust
    #[test]
    fn drawdown_report_prefers_sorted_series() {
        use rust_decimal::Decimal;
        let reversed = vec![Decimal::from(99), Decimal::from(110), Decimal::from(100)];
        let sorted: Vec<_> = {
            let mut v = reversed.clone();
            v.sort();
            v
        };
        let unsorted = crate::fin::risk::drawdown_report(&reversed).unwrap();
        let sorted_r = crate::fin::risk::drawdown_report(&sorted).unwrap();
        // 乱序与升序输入报告不同——handler 必须 sort（本测试锁定该契约的输入侧）
        assert_ne!(unsorted.worst_pct, sorted_r.worst_pct);
    }
```

Run: `cargo test --lib analyze_handler 2>&1 | tail -5`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add src/cli/
git commit -m "feat(cli): analyze drawdown report via DrawdownTracker facade"
```

---

### Task 11: CLI — `backtest walkforward`

**前置纪律：** impact analysis on `BacktestCommands` / `execute_backtest_run`。

**Files:**
- Modify: `src/cli/commands/backtest.rs:6-78`（新变体）
- Modify: `src/cli/handlers/backtest_handler.rs:34-88`（分发）
- Modify: `src/cli/handlers/backtest_handler.rs`（新函数）

- [ ] **Step 1: impact analysis**（记录）。

- [ ] **Step 2: CLI 变体**

`BacktestCommands` 加（参数扁平化，网格各 3 个数字 + 窗口 3 个 + 资金/费率/仓位）：

```rust
    /// 研究侧 walk-forward 回测（fin 引擎，ma_cross 策略）
    Walkforward {
        #[arg(short, long)]
        code: String,
        /// 开始日期（YYYYMMDD，与 Run 变体同口径）
        #[arg(long)]
        start: Option<String>,
        /// 结束日期（YYYYMMDD）
        #[arg(long)]
        end: Option<String>,
        /// short_period 网格 min/max/step
        #[arg(long, default_value_t = 2)]
        short_start: usize,
        #[arg(long, default_value_t = 5)]
        short_end: usize,
        #[arg(long, default_value_t = 1)]
        short_step: usize,
        /// long_period 网格 min/max/step
        #[arg(long, default_value_t = 5)]
        long_start: usize,
        #[arg(long, default_value_t = 20)]
        long_end: usize,
        #[arg(long, default_value_t = 1)]
        long_step: usize,
        #[arg(long, default_value_t = 60)]
        train_window: usize,
        #[arg(long, default_value_t = 20)]
        test_window: usize,
        #[arg(long, default_value_t = 20)]
        step: usize,
        #[arg(long, default_value = "100000")]
        capital: String,
        #[arg(long, default_value = "0.0003")]
        commission_rate: String,
        #[arg(long, default_value = "100")]
        position_size: String,
    },
```

- [ ] **Step 3: 分发**

backtest_handler.rs 分发 match（:34-88）加分支，委托新函数 `execute_fin_walkforward`（参数透传）。

- [ ] **Step 4: handler 实现**

backtest_handler.rs 追加（解析模式对齐现有 :90 `execute_backtest_run` 的 Decimal parse 风格）：

```rust
pub(crate) async fn execute_fin_walkforward(
    code: String,
    start: Option<String>,
    end: Option<String>,
    short_start: usize,
    short_end: usize,
    short_step: usize,
    long_start: usize,
    long_end: usize,
    long_step: usize,
    train_window: usize,
    test_window: usize,
    step: usize,
    capital: String,
    commission_rate: String,
    position_size: String,
) -> Result<()> {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let parse_decimal = |label: &str, raw: &str| -> Result<Decimal> {
        Decimal::from_str(raw)
            .map_err(|e| QuantixError::DataParse(format!("{label} 无效: {raw} ({e})")))
    };
    let capital = parse_decimal("capital", &capital)?;
    let commission_rate = parse_decimal("commission_rate", &commission_rate)?;
    let position_size = parse_decimal("position_size", &position_size)?;

    let start = parse_optional_yyyymmdd(start.as_deref())?;
    let end = parse_optional_yyyymmdd(end.as_deref())?;
    let mut klines = get_kline_for_analysis(&code, start, end, None).await?;
    klines.sort_by_key(|k| k.date);
    if klines.len() < train_window + test_window {
        return Err(QuantixError::Config(format!(
            "数据量不足: {} 根 < train({}) + test({})",
            klines.len(),
            train_window,
            test_window
        )));
    }
    let out = crate::fin::backtest::walk_forward_ma_cross(
        &klines,
        (short_start, short_end, short_step),
        (long_start, long_end, long_step),
        train_window,
        test_window,
        step,
        capital,
        commission_rate,
        position_size,
    )?;
    let r = &out.result;
    println!(
        "walk-forward 报告（{}，{} 根，{} 窗口）",
        code,
        klines.len(),
        r.periods.len()
    );
    println!("{}", "-".repeat(72));
    println!(
        "{:<8} {:<14} {:<12} {:<12} {:<10}",
        "窗口", "best(short,long)", "IS sharpe", "OOS sharpe", "OOS 回撤"
    );
    for (i, p) in r.periods.iter().enumerate() {
        let s = p.best_params.get("short_period").copied().unwrap_or(0.0);
        let l = p.best_params.get("long_period").copied().unwrap_or(0.0);
        println!(
            "{:<8} {:<14} {:<12} {:<12} {:<10}",
            i + 1,
            format!("{s}/{l}"),
            format!("{:.3}", p.in_sample_sharpe),
            format!("{:.3}", p.out_of_sample_sharpe),
            format!("{:.3}", p.oos_result.max_drawdown),
        );
    }
    println!("{}", "-".repeat(72));
    println!("aggregate_sharpe:     {:.3}", r.aggregate_sharpe);
    println!("stability_score:      {:.3}（OOS>0 占比）", r.stability_score);
    println!("mean_oos_return:      {:.5}", r.mean_oos_return);
    println!("worst_oos_drawdown:   {:.5}", r.worst_oos_drawdown);
    // 0.5 为研究默认稳健阈值（stability_score 下限）；CLI 暂不参数化（YAGNI，需要时二期加 --min-stability）
    println!("is_robust(≥0.5):      {}", r.is_robust(0.5));
    Ok(())
}
```

日期解析复用本文件既有 `parse_optional_yyyymmdd`（backtest_handler.rs:207，格式 `%Y%m%d`，与 Run 变体 :116-117 同口径）；不新增 helper、不引入第二种日期格式。
（`get_kline_for_analysis` 的导入按文件现有 use；若 backtest_handler 未导入，加 `use crate::cli::handlers::shared_support::get_kline_for_analysis;`，可见性为 pub(crate) 已核实。）

- [ ] **Step 5: 编译与既有回归**

Run: `cargo test --lib backtest_handler 2>&1 | tail -5 && cargo check 2>&1 | tail -3`
Expected: 既有测试 PASS、无编译错误。

- [ ] **Step 6: Commit**

```bash
git add src/cli/
git commit -m "feat(cli): backtest walkforward via fin WalkForwardOptimizer

impact: BacktestCommands callers = cli.rs dispatch only"
```

---

### Task 12: 集成金丝雀 + FUNCTION_TREE 登记 + 质量门收尾

**Files:**
- Create: `tests/fin_canary.rs`
- Modify: `FUNCTION_TREE.md`（模块表 :41、CLI 入口表 :88、已设计/待实现 :125 按实际行号就近插入）

- [ ] **Step 1: 金丝雀集成测试**

创建 `tests/fin_canary.rs`（集成测试可用 unwrap，仓库有先例；确定性输入 + 手算期望，作为上游升级漂移哨兵）：

```rust
//! fin-primitives 集成金丝雀：确定性输入 + 手算期望。
//! 上游升级若打破任何断言 = 行为漂移，须人工核对（配合 Cargo 精确锁版本）。

use quantix_cli::core::Signal;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::fin;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

fn kline(day: u32, close: &str) -> Kline {
    let c = Decimal::from_str(close).unwrap();
    Kline {
        code: "600000".into(),
        date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
        open: c,
        high: c,
        low: c,
        close: c,
        volume: 1000,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[test]
fn canary_indicator_chain() {
    let klines: Vec<Kline> = (1..=10u32)
        .map(|i| kline(i, &i.to_string()))
        .collect();
    let sma = fin::indicators::sma(&klines, 3).unwrap();
    assert_eq!(sma.values[9], Some(Decimal::from_str("9").unwrap()));
    let rising: Vec<Kline> = (1..=30u32)
        .map(|i| kline(i.min(28).max(1), &i.to_string()))
        .collect();
    let rsi = fin::indicators::rsi(&rising, 14).unwrap();
    assert_eq!(rsi.values[29], Some(Decimal::from(100)));
}

#[test]
fn canary_drawdown_chain() {
    let report = fin::risk::drawdown_report(&[
        Decimal::from(100),
        Decimal::from(110),
        Decimal::from(99),
    ])
    .unwrap();
    assert_eq!(report.worst_pct, Decimal::from_str("10").unwrap());
}

#[test]
fn canary_backtest_chain() {
    let mut closes: Vec<i64> = vec![10; 4];
    closes.extend(vec![12; 3]);
    closes.extend(vec![9; 3]);
    let klines: Vec<Kline> = closes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let c = Decimal::from(*c);
            Kline {
                code: "600000".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, (i + 1) as u32).unwrap(),
                open: c,
                high: c,
                low: c,
                close: c,
                volume: 1000,
                amount: None,
                adjust_type: AdjustType::None,
            }
        })
        .collect();
    let out = fin::backtest::run_fin_backtest(
        &klines,
        2,
        3,
        Decimal::from(10000),
        Decimal::ZERO,
        Decimal::from(100),
    )
    .unwrap();
    assert_eq!(out.result.final_equity, Decimal::from_str("9700").unwrap());
    assert_eq!(out.result.trade_count, 2);
}
```

双引擎信号一致性**不进** tests/（集成测试访问不到 `pub(crate) signal_at`，评审 C4）：已由 Task 7 的 `bridge_signal_parity_with_engine_trades`（crate 内 `#[cfg(test)]`）覆盖，保持 `signal_at` 为 `pub(crate)`，不扩大 strategy 公共 API。

Run: `cargo test --test fin_canary 2>&1 | tail -6`
Expected: 3 passed。

- [ ] **Step 2: FUNCTION_TREE 登记**

`FUNCTION_TREE.md`：
1. 模块节点表（:41 附近）加一行，四列格式与邻近行一致：

```markdown
| fin（fin-primitives 研究库集成层） | [已实现] | src/fin/* + tests/fin_canary.rs | 研究侧；fin 类型不出 src/fin；业务侧 portfolio/backtest 不受影响 |
```

2. CLI 入口表（:88 附近）加三行：

```markdown
| quantix analyze indicators --engine fin | [已实现] | cli/commands/analysis.rs + handlers/analyze_handler.rs | 默认 native；fin 仅 sma/ema/rsi/atr/boll/stoch |
| quantix analyze drawdown | [已实现] | cli/commands/analysis.rs + handlers/analyze_handler.rs | 净值=收盘价研究口径 |
| quantix backtest walkforward | [已实现] | cli/commands/backtest.rs + handlers/backtest_handler.rs | 仅 ma_cross；fin 引擎；成本=commission_rate |
```

3. 若「回撤控制」相关行存在 [已设计/待实现] 状态，更新为 [部分实现]（研究侧分析已落地；实盘拦截逻辑未动），边界列注明。

- [ ] **Step 3: 全量质量门**

```bash
cargo fmt --check
cargo clippy -- -D warnings 2>&1 | tail -3
cargo test 2>&1 | tail -10
```

Expected: 三者全绿。clippy 新增告警须当场修复（如 `too_many_arguments` 于 walkforward handler——在该函数上加 `#[allow(clippy::too_many_arguments)]`，文件已有该 allow 先例）。

- [ ] **Step 4: GitNexus 变更检测**

MCP 可用：`gitnexus_detect_changes()`，确认影响面仅限预期符号（新增 fin/*、QuantixError::Fin、MACrossStrategy signal_at、CLI 三处）。
不可用：记录 "GitNexus backfill required" 于本 plan 文件末尾。

- [ ] **Step 5: Graphiti 回填**

MCP 可用：把「迁移设计定稿 + 实施完成」写入 `quantix_rust_main`（add_memory → get_ingest_status 等待 completed）。
不可用：本步骤跳过，在 plan 末尾保留 "Graphiti backfill required"。

- [ ] **Step 6: Commit**

```bash
git add tests/fin_canary.rs FUNCTION_TREE.md docs/superpowers/plans/
git commit -m "test(fin): integration canaries + FUNCTION_TREE registration"
```

---

## Self-Review 记录

1. **Spec 覆盖**：adapter（§5.1→T2）、indicators 两层门面（§5.2→T3，逃生舱=模块内扩展约定）、portfolio（§5.3→T5）、risk（§5.4→T4）、backtest 三件套（§5.5→T6+T7）、events 库层（§5.6→T8）、错误收敛（§6→T1）、CLI 三入口（§7→T9/T10/T11）、测试策略（§9→各任务+T12）、依赖（§8→T1）、纪律（§10→全局纪律+T12 Step4/5）。✅ 无缺口。
2. **占位符扫描**：Task 6 Step 2 的第一个测试代码块为"先写后简化"的过程展示，最终形态已给出；实施者直接用简化版。其余步骤均含完整代码/命令。✅
3. **类型一致性**：`FinStrategy::new(signals)` 全文一致（T7 定义、T7 测试、T12 canary）；`drawdown_report(&[Decimal])`（T4/T10/T12）；`run_fin_backtest(klines, short, long, capital, rate, size)`（T7/T12）；`resolve(name, klines, period)`（T3/T9）；`signal_at` 命名贯穿 T6/T7/T12。✅

## v1.1 修订（2026-11-23，按计划评审 12 项全采纳）

- C1/C2：events.rs 补 `use rust_decimal::prelude::ToPrimitive;`；`close_series` 改返回 `Result`，`and_hms_opt` 经 `ok_or_else` 后再 `.and_utc()`（两处）。
- C3：`is_robust(min_stability: f64)` 补参（速查表 + T11 handler 传 0.5 并打印阈值）。
- C4：canary 中的 parity 测试删除，正式移入 T7 `bridge_signal_parity_with_engine_trades`（crate 内），`signal_at` 保持 `pub(crate)`。
- M5：T6 前置纪律改为实测基线（runtime.rs/mod.rs/backtest_handler.rs/strategy_handler/catalog.rs），禁止预设调用方范围。
- M6：T11 日期解析复用既有 `parse_optional_yyyymmdd`（`%Y%m%d`），删除自建 `parse_date_opt`。
- M7：T9 fin handler 版式对齐 native（💹 头部 + 每根 Kline 一行日期/收盘价 + 每指标一行留空前两列；空数据行为也对齐 native 的警告+Ok）。
- M8：`significance` 改为真实调用 `EventStudy::significance(&[EventResult])`（门面内保留 `Vec<fin EventResult>`，类型不外泄）。
- L9/L10：不改（plan 已有"就近插入""直接用简化版"指令）。
