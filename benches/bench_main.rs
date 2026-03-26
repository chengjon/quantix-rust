// quantix-cli 性能基准测试套件
//
// Phase 18: 性能测试与优化
//
// 使用方法:
//   cargo bench --all-features
//
// 基准测试覆盖:
// - 数据导入导出性能
// - 技术指标计算性能
// - 回测引擎性能
// - 策略执行性能
// - 内存使用分析

use chrono::NaiveDate;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use quantix_cli::{
    analysis::{indicators_benches::*, performance::*},
    data::models::*,
    io::*,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::time::Duration;

/// 生成测试用K线数据
fn generate_test_klines(count: usize) -> Vec<Kline> {
    (0..count)
        .map(|i| {
            let i_f64 = i as f64;
            Kline {
                code: "000001".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
                open: Decimal::from_f64_retain(10.0 + i_f64 * 0.01).unwrap(),
                high: Decimal::from_f64_retain(11.0 + i_f64 * 0.01).unwrap(),
                low: Decimal::from_f64_retain(9.0 + i_f64 * 0.01).unwrap(),
                close: Decimal::from_f64_retain(10.5 + i_f64 * 0.01).unwrap(),
                volume: 1000000 + i as i64 * 1000,
                amount: Some(Decimal::from_f64_retain(10500000.0 + i_f64 * 100.0).unwrap()),
                adjust_type: AdjustType::None,
            }
        })
        .collect()
}

/// 基准测试：技术指标计算
fn bench_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("indicators");

    for size in [100, 1000, 10000].iter() {
        let klines = generate_test_klines(*size);
        let closes: Vec<Decimal> = klines.iter().map(|k| k.close).collect();

        // SMA 计算
        group.bench_with_input(BenchmarkId::new("sma_5", size), &closes, |b, data| {
            b.iter(|| calculate_sma(black_box(data), 5))
        });

        group.bench_with_input(BenchmarkId::new("sma_20", size), &closes, |b, data| {
            b.iter(|| calculate_sma(black_box(data), 20))
        });

        // EMA 计算
        group.bench_with_input(BenchmarkId::new("ema_12", size), &closes, |b, data| {
            b.iter(|| calculate_ema(black_box(data), 12))
        });

        group.bench_with_input(BenchmarkId::new("ema_26", size), &closes, |b, data| {
            b.iter(|| calculate_ema(black_box(data), 26))
        });

        // RSI 计算
        group.bench_with_input(BenchmarkId::new("rsi_14", size), &closes, |b, data| {
            b.iter(|| calculate_rsi(black_box(data), 14))
        });

        // MACD 计算
        group.bench_with_input(BenchmarkId::new("macd", size), &closes, |b, data| {
            b.iter(|| calculate_macd(black_box(data), 12, 26, 9))
        });
    }

    group.finish();
}

/// 基准测试：数据导出
fn bench_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export");
    group.sample_size(10); // 导出操作较慢，减少样本数

    for size in [1000, 10000, 100000].iter() {
        let klines = generate_test_klines(*size);
        let temp_dir = tempfile::tempdir().unwrap();

        // CSV 导出
        group.bench_with_input(BenchmarkId::new("csv", size), &klines, |b, data| {
            b.iter(|| {
                let exporter = DataExporter::with_defaults();
                let output_path = temp_dir.path().join("test.csv");
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(exporter.export_klines(black_box(data), &output_path))
                    .unwrap()
            })
        });

        // JSON 导出
        let config = ExportConfig {
            format: ExportFormat::JSON,
            ..Default::default()
        };
        group.bench_with_input(BenchmarkId::new("json", size), &klines, |b, data| {
            b.iter(|| {
                let exporter = DataExporter::new(config.clone());
                let output_path = temp_dir.path().join("test.json");
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(exporter.export_klines(black_box(data), &output_path))
                    .unwrap()
            })
        });
    }

    group.finish();
}

/// 基准测试：数据验证
fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    for size in [100, 1000, 10000].iter() {
        let klines = generate_test_klines(*size);
        let validator = DataValidator::with_defaults();

        group.bench_with_input(
            BenchmarkId::new("validate_klines", size),
            &klines,
            |b, data| b.iter(|| validator.validate_klines(black_box(data))),
        );

        group.bench_with_input(
            BenchmarkId::new("quality_report", size),
            &klines,
            |b, data| b.iter(|| validator.quality_report(black_box(data))),
        );
    }

    group.finish();
}

/// 基准测试：性能指标计算
fn bench_performance_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance");

    for size in [100, 500, 1000].iter() {
        // 使用更小的规模避免溢出
        // 生成权益曲线（使用更小的基数和收益率避免溢出）
        let base_equity = dec!(10000);
        let returns: Vec<Decimal> = (0..*size)
            .map(|i| {
                // 生成 -1% 到 +1% 的收益率（更保守）
                let pct = (i as f64 * 0.00002 - 0.01).max(-0.01).min(0.01);
                Decimal::from_f64_retain(pct).unwrap()
            })
            .collect();

        // 从收益率计算权益曲线
        let mut equity_curve = vec![base_equity];
        for ret in returns.iter() {
            let prev = equity_curve.last().unwrap();
            equity_curve.push(*prev * (dec!(1) + *ret));
        }

        // 总收益率计算
        group.bench_with_input(
            BenchmarkId::new("total_return", size),
            &equity_curve,
            |b, data| b.iter(|| calculate_total_return(black_box(data))),
        );

        // 最大回撤计算
        group.bench_with_input(
            BenchmarkId::new("max_drawdown", size),
            &equity_curve,
            |b, data| b.iter(|| calculate_max_drawdown(black_box(data))),
        );

        // 夏普比率计算
        let returns: Vec<Decimal> = equity_curve
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        // 确保收益率不会导致溢出
        let safe_returns: Vec<Decimal> = returns
            .iter()
            .map(|r| {
                let val = r.abs();
                if val > dec!(0) && val < dec!(1) {
                    *r
                } else {
                    dec!(0)
                }
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("sharpe_ratio", size),
            &safe_returns,
            |b, data| b.iter(|| calculate_sharpe_ratio(black_box(data), dec!(0.03))),
        );
    }

    group.finish();
}

/// 基准测试：批处理性能
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");
    group.sample_size(10);

    for size in [10000, 100000, 1000000].iter() {
        let klines = generate_test_klines(*size);
        let temp_dir = tempfile::tempdir().unwrap();
        let processor = BatchProcessor::with_defaults();

        group.bench_with_input(
            BenchmarkId::new("process_in_batches", size),
            &klines,
            |b, data| {
                b.iter(|| {
                    processor.process_in_batches(black_box(data.clone()), |chunk| {
                        // 模拟处理
                        chunk.len()
                    })
                })
            },
        );
    }

    group.finish();
}

/// 配置基准测试选项
fn benchmark_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10))
        .significance_level(0.05) // 95% 置信度
        .noise_threshold(0.02) // 2% 噪声阈值
        .sample_size(100) // 每个基准 100 次迭代
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets =
        bench_indicators,
        bench_export,
        bench_validation,
        bench_performance_metrics,
        bench_batch_processing
}

criterion_main!(benches);
