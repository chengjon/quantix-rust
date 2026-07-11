use super::*;

use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::qmt_live_gate::{QMT_LIVE_BRIDGE_COMMAND, QMT_LIVE_BRIDGE_MODE_REQUIREMENT};
use crate::execution::runtime_store::StrategyRuntimeStore;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal_macros::dec;

pub(crate) async fn run_strategy(name: String, mode: String, code: Option<String>) -> Result<()> {
    println!("🎯 运行策略: {} ({})", name, mode);
    if let Some(c) = &code {
        println!("📈 股票代码: {}", c);
    }

    match name.as_str() {
        "ma_cross" => {
            if mode == "backtest" {
                run_ma_cross_backtest(code).await?;
            } else if routes_through_execution_handler(&mode) {
                let runtime = CliRuntime::load();
                let runtime_store =
                    StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
                let summary = execute_strategy_run_with_components(
                    &name,
                    &mode,
                    code,
                    ClickHouseDailyKlineLoader::new(),
                    create_trade_store(),
                    create_risk_store(),
                    &runtime_store,
                )
                .await?;
                print_strategy_run_summary(&summary);
            } else if mode == "live" {
                println!(
                    "⚠️  strategy run 不支持 direct live；如需真实 QMT 提交，请先创建 qmt_live request，再走 {QMT_LIVE_BRIDGE_COMMAND} 路径，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}"
                );
            } else if mode == "qmt_live" {
                println!(
                    "⚠️  strategy run 不直接支持 qmt_live；如需真实 QMT 提交，请先创建 qmt_live request，再走 {QMT_LIVE_BRIDGE_COMMAND} 路径，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}"
                );
            } else {
                println!("⚠️  暂不支持该运行模式");
            }
        }
        _ => {
            println!("❌ 未知策略: {}", name);
            println!("💡 可用策略: ma_cross");
        }
    }

    Ok(())
}

fn routes_through_execution_handler(mode: &str) -> bool {
    matches!(mode, "paper" | "mock_live")
}

pub(crate) async fn run_ma_cross_backtest(code: Option<String>) -> Result<()> {
    let stock_code = code.unwrap_or_else(|| "000001".to_string());

    println!("🔙 开始回测: MA 交叉策略");
    println!("  股票: {}", stock_code);
    println!("  参数: MA5, MA20");

    let klines = get_kline_for_analysis(&stock_code, None, None, Some(10000)).await?;

    if klines.len() < 20 {
        return Err(QuantixError::Other(format!(
            "数据不足，至少需要20条，当前: {}",
            klines.len()
        )));
    }

    let config = BacktestConfig {
        initial_capital: dec!(100000),
        commission_rate: dec!(0.0003),
        slippage_bps: 10,
        max_positions: 5,
        max_position_ratio: dec!(0.2),
        risk_free_rate: dec!(0.03),
    };

    let mut engine = BacktestEngine::new(config);
    let mut strategy = crate::strategy::ma_cross::MACrossStrategy::new(5, 20);

    use crate::data::models::Kline;
    use std::collections::HashMap;

    let mut data_map = HashMap::new();
    let kline_data: Vec<Kline> = klines
        .into_iter()
        .map(|k| Kline {
            code: stock_code.clone(),
            date: k.date,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            amount: k.amount,
            adjust_type: crate::data::models::AdjustType::None,
        })
        .collect();
    data_map.insert(stock_code.clone(), kline_data);

    let total_klines = data_map.values().map(|v| v.len()).sum::<usize>();
    let progress = ProgressBar::new(total_klines as u64);
    let template = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} K线")
        .unwrap_or_else(|_| ProgressStyle::default_bar());
    progress.set_style(template);

    let result = engine
        .run(&mut strategy, &data_map)
        .await
        .map_err(|e| QuantixError::Other(format!("回测失败: {}", e)))?;

    progress.finish();

    println!("\n📊 回测结果:");
    println!("  总收益率: {:.2}%", result.report.total_return * dec!(100));
    println!("  夏普比率: {:.2}", result.report.sharpe_ratio);
    println!("  最大回撤: {:.2}%", result.report.max_drawdown * dec!(100));
    println!("  胜率: {:.2}%", result.report.win_rate * dec!(100));
    println!("  交易次数: {}", result.report.total_trades);

    Ok(())
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::routes_through_execution_handler;

    #[test]
    fn mock_live_routes_through_execution_handler() {
        assert!(routes_through_execution_handler("paper"));
        assert!(routes_through_execution_handler("mock_live"));
        assert!(!routes_through_execution_handler("live"));
        assert!(!routes_through_execution_handler("backtest"));
        assert!(!routes_through_execution_handler("qmt_live"));
    }
}

pub(crate) async fn list_strategies() -> Result<()> {
    let store = create_strategy_config_store();
    let config = store.load_or_create()?;
    print_strategy_catalog_and_instances(&config);
    Ok(())
}

pub(crate) async fn show_strategy(name: String) -> Result<()> {
    println!("📖 策略详情: {}", name);

    match name.as_str() {
        "ma_cross" => {
            println!();
            println!("  名称: 均线交叉策略");
            println!("  原理: 当短期均线(MA5)上穿长期均线(MA20)时买入，反之卖出");
            println!("  参数:");
            println!("    - 短期周期: 5");
            println!("    - 长期周期: 20");
            println!("  适用场景: 趋势明显的市场");
            println!("  风险: 震荡市场容易频繁交易");
        }
        _ => {
            println!("❌ 未知策略: {}", name);
        }
    }

    Ok(())
}

pub(crate) async fn run_strategy_menu() -> Result<()> {
    let items = vec!["运行策略", "查看策略列表", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => list_strategies().await?,
        2 => {}
        _ => {}
    }

    Ok(())
}
