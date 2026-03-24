/// 策略集成测试
///
/// 测试策略与回测引擎的集成
use quantix_cli::analysis::backtest::{BacktestConfig, BacktestEngine};
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::strategy::ma_cross::MACrossStrategy;
use quantix_cli::strategy::mean_reversion::{MeanReversionConfig, MeanReversionStrategy};
use quantix_cli::strategy::momentum::MomentumStrategy;

// 价格趋势枚举（本地副本，避免依赖 test_utils）
#[derive(Debug, Clone, Copy)]
enum PriceTrend {
    Up,
    Down,
    Sideways,
    Volatile,
}

// 生成本地价格序列（避免依赖 test_utils）
fn generate_price_series_local(start_price: f64, count: usize, trend: PriceTrend) -> Vec<Kline> {
    let mut klines = Vec::new();
    let mut price = start_price;

    for i in 0..count {
        match trend {
            PriceTrend::Up => price += 0.5,
            PriceTrend::Down => price -= 0.5,
            PriceTrend::Sideways => {
                if i % 2 == 0 {
                    price += 0.3
                } else {
                    price -= 0.3
                }
            }
            PriceTrend::Volatile => price += (i % 5) as f64 - 2.0,
        }

        if price < 1.0 {
            price = 1.0;
        }

        let date = make_test_date(i);
        klines.push(Kline {
            code: "000001".to_string(),
            date,
            open: rust_decimal::Decimal::from_str(price.to_string().as_str()).unwrap(),
            high: rust_decimal::Decimal::from_str((price + 1.0).to_string().as_str()).unwrap(),
            low: rust_decimal::Decimal::from_str((price - 1.0).to_string().as_str()).unwrap(),
            close: rust_decimal::Decimal::from_str(price.to_string().as_str()).unwrap(),
            volume: 1000000,
            amount: Some(
                rust_decimal::Decimal::from_str(price.to_string().as_str()).unwrap()
                    * rust_decimal::Decimal::from(1000000),
            ),
            adjust_type: AdjustType::None,
        });
    }

    klines
}
use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;

fn make_test_date(offset_days: usize) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .checked_add_signed(Duration::days(offset_days as i64))
        .unwrap()
}

/// 创建测试数据
fn create_test_data() -> HashMap<String, Vec<Kline>> {
    let mut data = HashMap::new();

    // 生成测试数据：先上涨后下跌
    let prices: Vec<f64> = (0..100)
        .map(|i| {
            if i < 50 {
                100.0 + i as f64 * 0.5 // 上涨
            } else {
                125.0 - (i as f64 - 50.0) * 0.5 // 下跌
            }
        })
        .collect();

    let klines: Vec<Kline> = prices
        .iter()
        .enumerate()
        .map(|(i, &price)| {
            let date = make_test_date(i);
            Kline {
                code: "000001".to_string(),
                date,
                open: Decimal::from_str(price.to_string().as_str()).unwrap(),
                high: Decimal::from_str((price + 1.0).to_string().as_str()).unwrap(),
                low: Decimal::from_str((price - 1.0).to_string().as_str()).unwrap(),
                close: Decimal::from_str(price.to_string().as_str()).unwrap(),
                volume: 1000000,
                amount: Some(
                    Decimal::from_str(price.to_string().as_str()).unwrap() * Decimal::from(1000000),
                ),
                adjust_type: AdjustType::None,
            }
        })
        .collect();

    data.insert("000001".to_string(), klines);
    data
}

/// 创建适用于 MA Cross 的测试数据
fn create_ma_cross_test_data() -> HashMap<String, Vec<Kline>> {
    let mut data = HashMap::new();
    let mut prices = Vec::new();
    let mut price = 100.0;

    // 先下跌，确保短均线位于长均线下方
    for _ in 0..20 {
        prices.push(price);
        price -= 0.5;
    }

    // 再上涨，触发金叉买入
    for _ in 0..40 {
        prices.push(price);
        price += 0.5;
    }

    // 最后回落，触发死叉卖出
    for _ in 0..40 {
        prices.push(price);
        price -= 0.5;
    }

    let klines: Vec<Kline> = prices
        .iter()
        .enumerate()
        .map(|(i, &price)| {
            let date = make_test_date(i);
            Kline {
                code: "000001".to_string(),
                date,
                open: Decimal::from_str(price.to_string().as_str()).unwrap(),
                high: Decimal::from_str((price + 1.0).to_string().as_str()).unwrap(),
                low: Decimal::from_str((price - 1.0).to_string().as_str()).unwrap(),
                close: Decimal::from_str(price.to_string().as_str()).unwrap(),
                volume: 1000000,
                amount: Some(
                    Decimal::from_str(price.to_string().as_str()).unwrap() * Decimal::from(1000000),
                ),
                adjust_type: AdjustType::None,
            }
        })
        .collect();

    data.insert("000001".to_string(), klines);
    data
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use quantix_cli::strategy::Strategy;

    #[test]
    fn test_create_test_data_supports_dates_beyond_january() {
        let data = create_test_data();
        let klines = data.get("000001").unwrap();

        assert_eq!(klines.len(), 100);
        assert_eq!(klines[0].date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(
            klines[31].date,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
        );
        assert_eq!(
            klines[99].date,
            NaiveDate::from_ymd_opt(2024, 4, 9).unwrap()
        );
    }

    #[tokio::test]
    async fn test_create_ma_cross_test_data_generates_buy_and_sell_signals() {
        let data = create_ma_cross_test_data();
        let klines = data.get("000001").unwrap();
        let mut strategy = MACrossStrategy::new(5, 10);
        let mut has_buy = false;
        let mut has_sell_after_buy = false;

        for kline in klines {
            match strategy.on_bar(kline).await.unwrap() {
                quantix_cli::strategy::trait_def::Signal::Buy => has_buy = true,
                quantix_cli::strategy::trait_def::Signal::Sell if has_buy => {
                    has_sell_after_buy = true;
                    break;
                }
                _ => {}
            }
        }

        assert!(has_buy, "MA Cross 测试夹具应该触发买入信号");
        assert!(
            has_sell_after_buy,
            "MA Cross 测试夹具应该在买入后触发卖出信号"
        );
    }

    /// 测试 MA Cross 策略与回测引擎集成
    #[tokio::test]
    async fn test_ma_cross_backtest_integration() {
        let mut engine = BacktestEngine::with_default_config();
        let mut strategy = MACrossStrategy::new(5, 10);
        let data = create_ma_cross_test_data();

        let result = engine.run(&mut strategy, &data).await.unwrap();

        // 验证回测结果
        assert!(result.final_equity > dec!(0));
        assert!(!result.trades.is_empty());

        // 验证策略产生交易
        assert!(result.trades.len() > 0, "MA Cross 策略应该产生交易");

        println!("MA Cross 回测结果:");
        println!("  总收益率: {}%", result.report.total_return * dec!(100));
        println!("  夏普比率: {}", result.report.sharpe_ratio);
        println!("  最大回撤: {}%", result.report.max_drawdown * dec!(100));
        println!("  交易次数: {}", result.trades.len());
        println!("  最终权益: {}", result.final_equity);
    }

    /// 测试 Mean Reversion 策略与回测引擎集成
    #[tokio::test]
    async fn test_mean_reversion_backtest_integration() {
        let config = BacktestConfig {
            initial_capital: dec!(1000000),
            ..Default::default()
        };
        let mut engine = BacktestEngine::new(config);

        let strategy_config = MeanReversionConfig {
            rsi_period: 10,
            rsi_overbought: dec!(70),
            rsi_oversold: dec!(30),
            bb_period: 15,
            bb_std_dev: 2,
            buy_deviation_pct: dec!(3),
            sell_deviation_pct: dec!(3),
        };
        let mut strategy = MeanReversionStrategy::new(strategy_config);
        let data = create_test_data();

        let result = engine.run(&mut strategy, &data).await.unwrap();

        // 验证回测结果
        assert!(result.final_equity > dec!(0));

        println!("Mean Reversion 回测结果:");
        println!("  总收益率: {}%", result.report.total_return * dec!(100));
        println!("  最终权益: {}", result.final_equity);
    }

    /// 测试 Momentum 策略与回测引擎集成
    #[tokio::test]
    async fn test_momentum_backtest_integration() {
        let mut engine = BacktestEngine::with_default_config();
        let mut strategy = MomentumStrategy::with_defaults();
        let data = create_test_data();

        let result = engine.run(&mut strategy, &data).await.unwrap();

        // 验证回测结果
        assert!(result.final_equity > dec!(0));

        println!("Momentum 回测结果:");
        println!("  总收益率: {}%", result.report.total_return * dec!(100));
        println!("  最终权益: {}", result.final_equity);
    }

    /// 测试多股票回测
    #[tokio::test]
    async fn test_multi_stock_backtest() {
        let mut engine = BacktestEngine::with_default_config();
        let mut strategy = MACrossStrategy::new(5, 10);

        // 创建多个股票的数据
        let mut data = HashMap::new();

        for code in &["000001", "000002", "000003"] {
            let prices: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 0.3).collect();

            let klines: Vec<Kline> = prices
                .iter()
                .enumerate()
                .map(|(i, &price)| {
                    let date = make_test_date(i);
                    Kline {
                        code: code.to_string(),
                        date,
                        open: Decimal::from_str(price.to_string().as_str()).unwrap(),
                        high: Decimal::from_str((price + 0.5).to_string().as_str()).unwrap(),
                        low: Decimal::from_str((price - 0.5).to_string().as_str()).unwrap(),
                        close: Decimal::from_str(price.to_string().as_str()).unwrap(),
                        volume: 1000000,
                        amount: Some(
                            Decimal::from_str(price.to_string().as_str()).unwrap()
                                * Decimal::from(1000000),
                        ),
                        adjust_type: AdjustType::None,
                    }
                })
                .collect();

            data.insert(code.to_string(), klines);
        }

        let result = engine.run(&mut strategy, &data).await.unwrap();

        // 验证回测结果
        assert!(result.final_equity > dec!(0));

        println!("多股票回测结果:");
        println!("  总收益率: {}%", result.report.total_return * dec!(100));
        println!("  最终权益: {}", result.final_equity);
    }

    /// 测试不同市场条件下的策略表现
    #[tokio::test]
    async fn test_strategy_performance_different_markets() {
        let test_cases = vec![
            ("牛市", PriceTrend::Up),
            ("熊市", PriceTrend::Down),
            ("震荡市", PriceTrend::Sideways),
        ];

        for (market_name, trend) in test_cases {
            let mut engine = BacktestEngine::with_default_config();
            let mut strategy = MACrossStrategy::new(5, 20);

            // 生成对应趋势的数据
            let series = generate_price_series_local(100.0, 100, trend);
            let klines: Vec<Kline> = series
                .iter()
                .enumerate()
                .map(|(i, kline)| {
                    let mut k = kline.clone();
                    k.code = "000001".to_string();
                    k
                })
                .collect();

            let mut data = HashMap::new();
            data.insert("000001".to_string(), klines);

            let result = engine.run(&mut strategy, &data).await.unwrap();

            println!("{} 市场 MA Cross 策略表现:", market_name);
            println!("  总收益率: {}%", result.report.total_return * dec!(100));
            println!("  夏普比率: {}", result.report.sharpe_ratio);
            println!("  最大回撤: {}%", result.report.max_drawdown * dec!(100));
            println!("  交易次数: {}", result.trades.len());

            // 验证结果合理
            assert!(result.final_equity > dec!(0));
        }
    }

    /// 测试策略性能基准
    #[tokio::test]
    async fn test_strategy_performance_benchmark() {
        let mut engine = BacktestEngine::with_default_config();
        let mut strategy = MACrossStrategy::new(5, 20);

        // 生成较大数据集（1000根K线）
        let prices: Vec<f64> = (0..1000)
            .map(|i| 100.0 + (i as f64 % 50.0 - 25.0))
            .collect();

        let klines: Vec<Kline> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| {
                let date = make_test_date(i);
                Kline {
                    code: "000001".to_string(),
                    date,
                    open: Decimal::from_str(price.to_string().as_str()).unwrap(),
                    high: Decimal::from_str((price + 1.0).to_string().as_str()).unwrap(),
                    low: Decimal::from_str((price - 1.0).to_string().as_str()).unwrap(),
                    close: Decimal::from_str(price.to_string().as_str()).unwrap(),
                    volume: 1000000,
                    amount: Some(
                        Decimal::from_str(price.to_string().as_str()).unwrap()
                            * Decimal::from(1000000),
                    ),
                    adjust_type: AdjustType::None,
                }
            })
            .collect();

        let mut data = HashMap::new();
        data.insert("000001".to_string(), klines);

        // 运行回测并测量时间
        let start = std::time::Instant::now();
        let result = engine.run(&mut strategy, &data).await.unwrap();
        let duration = start.elapsed();

        println!("性能基准测试 (1000根K线):");
        println!("  执行时间: {:?}", duration);
        println!(
            "  每根K线平均时间: {:.2}μs",
            duration.as_micros() as f64 / 1000.0
        );
        println!("  总收益率: {}%", result.report.total_return * dec!(100));
        println!("  最终权益: {}", result.final_equity);

        // 验证性能合理（应该在几秒内完成）
        assert!(duration.as_secs() < 10, "回测1000根K线应在10秒内完成");
    }
}
