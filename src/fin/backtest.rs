//! 研究侧回测门面（fin Backtester 引擎）。
//! 策略桥（方案 C）：quantix 策略经同步 `signal_at` 预生成信号序列，
//! `FinStrategy` 按序回放给 fin 同步引擎——无 block_on、无跨 runtime。
//! 成本口径：仅 commission_rate（上游冲击/延迟模型未接入引擎，见 spec §12）。

use std::collections::{HashMap, VecDeque};

use fin_primitives::backtest::{
    BacktestConfig, Backtester, ParamRange, Signal as FinSignal, SignalDirection,
    Strategy as FinBacktestStrategy, WalkForwardConfig, WalkForwardOptimizer,
};
use fin_primitives::ohlcv::OhlcvBar;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::Signal;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::fin::adapter::{ohlcv_to_kline, to_ohlcv_bars};
use crate::strategy::ma_cross::MACrossStrategy;

/// 信号映射配置：方向映射 Buy/Sell→fin，固定下单数量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FinStrategyConfig {
    pub position_size: Decimal,
}

/// fin 引擎的策略适配器：回放预计算信号序列（桥内部件，fin 类型不出门面签名）。
pub(crate) struct FinStrategy {
    signals: VecDeque<FinSignal>,
}

impl FinStrategy {
    /// 回放契约：按 bar 顺序逐个弹出信号；**信号耗尽后返回 hold()（静默降级）**，
    /// 调用方保证 `signals` 与 bars 等长（precomputed_signals 已按同一序列产出）。
    pub(crate) fn new(signals: Vec<FinSignal>) -> Self {
        Self {
            signals: signals.into(),
        }
    }
}

impl FinBacktestStrategy for FinStrategy {
    fn on_bar(&mut self, _bar: &OhlcvBar) -> Option<FinSignal> {
        Some(self.signals.pop_front().unwrap_or_else(FinSignal::hold))
    }
}

fn map_signal(s: Signal, cfg: &FinStrategyConfig) -> FinSignal {
    match s {
        Signal::Buy => FinSignal::new(SignalDirection::Buy, cfg.position_size),
        Signal::Sell => FinSignal::new(SignalDirection::Sell, cfg.position_size),
        Signal::Hold => FinSignal::hold(),
    }
}

fn precomputed_signals(
    klines: &[Kline],
    short_period: usize,
    long_period: usize,
    cfg: &FinStrategyConfig,
) -> Vec<FinSignal> {
    let mut strategy = MACrossStrategy::new(short_period, long_period);
    klines
        .iter()
        .map(|k| map_signal(strategy.signal_at(k), cfg))
        .collect()
}

/// quantix 化单次回测摘要（镜像 fin `BacktestResult`，fin 类型不出门面签名）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinBacktestSummary {
    /// 区间总收益率 `(final_equity - initial_capital) / initial_capital`
    pub total_return: Decimal,
    /// 年化 Sharpe（252 交易日口径；收益标准差为 0 时取 0）
    pub sharpe_ratio: Decimal,
    /// 最大回撤小数口径（0.15 = 15%）
    pub max_drawdown: Decimal,
    /// 已平仓交易中盈利占比
    pub win_rate: Decimal,
    /// 成交笔数（上游口径：含失败的 fill）
    pub trade_count: u64,
    /// 期末净值
    pub final_equity: Decimal,
    /// 逐 bar 净值曲线
    pub equity_curve: Vec<Decimal>,
    /// 回测 bar 数
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
) -> Result<FinBacktestSummary> {
    let bars = to_ohlcv_bars(klines)?;
    let cfg = FinStrategyConfig { position_size };
    let signals = precomputed_signals(klines, short_period, long_period, &cfg);
    let mut strategy = FinStrategy::new(signals);
    let config =
        BacktestConfig::new(initial_capital, commission_rate).map_err(QuantixError::Fin)?;
    let result = Backtester::new(config)
        .run(&bars, &mut strategy)
        .map_err(QuantixError::Fin)?;
    let fin_primitives::backtest::BacktestResult {
        total_return,
        sharpe_ratio,
        max_drawdown,
        win_rate,
        trade_count,
        final_equity,
        equity_curve,
    } = result;
    Ok(FinBacktestSummary {
        total_return,
        sharpe_ratio,
        max_drawdown,
        win_rate,
        trade_count,
        final_equity,
        equity_curve,
        bar_count: bars.len(),
    })
}

/// quantix 化 walk-forward 单窗口报告（镜像 fin `WfPeriod`；省略嵌套 fin
/// `BacktestResult` 的 `oos_result`，OOS 收益/回撤取聚合层预算值）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardPeriodReport {
    pub train_start: usize,
    pub train_end: usize,
    pub test_start: usize,
    pub test_end: usize,
    /// 该窗口最优参数（is-Sample Sharpe 最大者）
    pub best_params: HashMap<String, f64>,
    pub in_sample_sharpe: f64,
    pub out_of_sample_sharpe: f64,
}

/// quantix 化 walk-forward 报告（镜像 fin `WalkForwardResult`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardReport {
    pub periods: Vec<WalkForwardPeriodReport>,
    /// 全窗口 OOS Sharpe 均值
    pub aggregate_sharpe: f64,
    /// 稳定性：OOS Sharpe > 0 的窗口占比，范围 [0,1]
    pub stability_score: f64,
    /// 全窗口 OOS 总收益均值
    pub mean_oos_return: f64,
    /// 全窗口最差（最大）OOS 回撤
    pub worst_oos_drawdown: f64,
}

impl WalkForwardReport {
    /// 稳健性判定：OOS Sharpe 均值 > 0 且稳定性 ≥ `min_stability`
    /// （同步自 fin-primitives walk_forward.rs:214，v4.0.1）。
    pub fn is_robust(&self, min_stability: f64) -> bool {
        self.aggregate_sharpe > 0.0 && self.stability_score >= min_stability
    }
}

/// walk-forward 网格与窗口参数（short/long 为闭区间步进网格，窗口为 bar 数）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardParams {
    /// (min, max, step) 闭区间步进网格
    pub short_range: (usize, usize, usize),
    /// (min, max, step) 闭区间步进网格
    pub long_range: (usize, usize, usize),
    /// 训练（in-sample）窗口 bar 数
    pub train_window: usize,
    /// 测试（out-of-sample）窗口 bar 数
    pub test_window: usize,
    /// 窗口推进步长 bar 数
    pub step: usize,
}

/// ma_cross walk-forward：short/long 双参数网格 + 滚动训练/测试窗口。
pub fn walk_forward_ma_cross(
    klines: &[Kline],
    params: &WalkForwardParams,
    initial_capital: Decimal,
    commission_rate: Decimal,
    position_size: Decimal,
) -> Result<WalkForwardReport> {
    let bars = to_ohlcv_bars(klines)?;
    let cfg = FinStrategyConfig { position_size };
    let param_space = vec![
        ParamRange {
            name: "short_period".into(),
            min: params.short_range.0 as f64,
            max: params.short_range.1 as f64,
            step: params.short_range.2 as f64,
        },
        ParamRange {
            name: "long_period".into(),
            min: params.long_range.0 as f64,
            max: params.long_range.1 as f64,
            step: params.long_range.2 as f64,
        },
    ];
    let config =
        BacktestConfig::new(initial_capital, commission_rate).map_err(QuantixError::Fin)?;
    let wf_config = WalkForwardConfig {
        train_window: params.train_window,
        test_window: params.test_window,
        step: params.step,
        param_space,
    };
    let optimizer = WalkForwardOptimizer::new(wf_config, config).map_err(QuantixError::Fin)?;

    // 闭包参数 params（HashMap）遮蔽外层 &WalkForwardParams，经 wf_params 别名引用网格 fallback。
    // 注意：filter_map 的 .ok() 失败会缩短 ks 使后续 signals[i] 错位——当前 bars
    // 由 to_ohlcv_bars 同源产出，结构性不可达；异源接入前应改 fail-fast
    // （collect::<Result<Vec<_>, _>>()）而非静默过滤。
    let wf_params = params;
    let mut make_strategy =
        |bars_slice: &[OhlcvBar], params: &HashMap<String, f64>| -> Box<dyn FinBacktestStrategy> {
            let short = params
                .get("short_period")
                .copied()
                .unwrap_or(wf_params.short_range.0 as f64) as usize;
            let long = params
                .get("long_period")
                .copied()
                .unwrap_or(wf_params.long_range.0 as f64) as usize;
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
    let periods = result
        .periods
        .into_iter()
        .map(|p| WalkForwardPeriodReport {
            train_start: p.train_start,
            train_end: p.train_end,
            test_start: p.test_start,
            test_end: p.test_end,
            best_params: p.best_params,
            in_sample_sharpe: p.in_sample_sharpe,
            out_of_sample_sharpe: p.out_of_sample_sharpe,
        })
        .collect();
    Ok(WalkForwardReport {
        periods,
        aggregate_sharpe: result.aggregate_sharpe,
        stability_score: result.stability_score,
        mean_oos_return: result.mean_oos_return,
        worst_oos_drawdown: result.worst_oos_drawdown,
    })
}

#[cfg(test)]
mod tests {
    use crate::data::models::AdjustType;
    use crate::fin::adapter::to_ohlcv_bar;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

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
            &klines,
            2,
            3,
            Decimal::from(10000),
            Decimal::ZERO,
            Decimal::from(100),
        )
        .unwrap();
        assert_eq!(out.bar_count, 10);
        assert_eq!(out.final_equity, Decimal::from_str("9700").unwrap());
        assert_eq!(out.trade_count, 2);
    }

    #[test]
    fn walk_forward_produces_periods_and_stability() {
        let mut closes: Vec<i64> = (10..=20).collect(); // 11 根缓涨
        closes.extend(8..=15); // 8 根缓跌
        let klines: Vec<Kline> = closes
            .iter()
            .enumerate()
            .map(|(i, c)| kline_at((i + 1) as u32, *c))
            .collect();
        let out = walk_forward_ma_cross(
            &klines,
            &WalkForwardParams {
                short_range: (2, 3, 1),
                long_range: (3, 4, 1),
                train_window: 8,
                test_window: 4,
                step: 4,
            },
            Decimal::from(10000),
            Decimal::ZERO,
            Decimal::from(100),
        )
        .unwrap();
        assert!(!out.periods.is_empty(), "至少应切出一个窗口");
        // stability ∈ [0,1]：OOS>0 占比
        assert!(out.stability_score >= 0.0 && out.stability_score <= 1.0);
        // is_robust 需阈值参数（walk_forward.rs:214，镜像语义）
        let _ = out.is_robust(0.5);
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
        let signals: Vec<FinSignal> = klines
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
        assert_eq!(out.trade_count as usize, buy_sell);
    }

    /// 回放契约钉子（桥核心约定）：signals 为空 → 每 bar 恒 hold()（静默降级），
    /// 且 on_bar 永不返回 None（fin 引擎语义：None=跳过交易，与 hold 不同）。
    #[test]
    fn fin_strategy_empty_signals_always_holds() {
        let bar = to_ohlcv_bar(&kline_at(1, 10)).unwrap();
        let mut s = FinStrategy::new(Vec::new());
        for _ in 0..3 {
            let sig = s.on_bar(&bar).unwrap();
            assert_eq!(sig.direction, SignalDirection::Hold);
            assert_eq!(sig.quantity, Decimal::ZERO);
        }
    }

    /// 回放契约钉子：signals 短于喂入 bar 数 → 首根回放真实信号，耗尽后恒 hold()。
    #[test]
    fn fin_strategy_short_signals_hold_after_exhaustion() {
        let bar = to_ohlcv_bar(&kline_at(1, 10)).unwrap();
        let buy = FinSignal::new(SignalDirection::Buy, Decimal::from(100));
        let mut s = FinStrategy::new(vec![buy]);
        let first = s.on_bar(&bar).unwrap(); // 第 1 根：回放 Buy
        assert_eq!(first.direction, SignalDirection::Buy);
        assert_eq!(first.quantity, Decimal::from(100));
        for _ in 0..2 {
            let sig = s.on_bar(&bar).unwrap(); // 耗尽后：hold 静默降级
            assert_eq!(sig.direction, SignalDirection::Hold);
            assert_eq!(sig.quantity, Decimal::ZERO);
        }
    }
}
