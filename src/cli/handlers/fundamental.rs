use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::fundamental::dragon_tiger::DragonTigerFetcher;
use crate::fundamental::earnings::EarningsFetcher;
use crate::fundamental::institution::InstitutionFetcher;
use crate::fundamental::valuation::ValuationFetcher;
use crate::fundamental::{EastMoneyFundamentalProvider, FundamentalProvider};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};

// ============================================================
// 基本面数据命令
// ============================================================

/// 处理基本面命令
pub async fn run_fundamental_command(cmd: FundamentalCommands) -> Result<()> {
    match cmd {
        FundamentalCommands::Show { code } => run_fundamental_show(&code).await,
        FundamentalCommands::Valuation { code } => run_fundamental_valuation(&code).await,
        FundamentalCommands::Earnings { code, years } => {
            run_fundamental_earnings(&code, years).await
        }
        FundamentalCommands::Institution { code } => run_fundamental_institution(&code).await,
        FundamentalCommands::DragonTiger { code, days } => {
            run_fundamental_dragon_tiger(code.as_deref(), days).await
        }
        FundamentalCommands::Dividend { code, years } => {
            run_fundamental_dividend(&code, years).await
        }
    }
}

async fn run_fundamental_show(code: &str) -> Result<()> {
    println!("📊 基本面数据");
    println!("   代码: {}", code);
    println!();

    println!("⏳ 正在获取数据...");

    let provider = EastMoneyFundamentalProvider::new();

    match provider.get_fundamental(code).await {
        Ok(data) => {
            println!("✅ 数据获取成功");
            println!();

            // 显示估值指标
            if let Some(val) = &data.valuation {
                println!("📈 估值指标:");
                if let Some(pe) = val.pe_ttm {
                    println!("   市盈率(TTM): {:.2}", pe);
                }
                if let Some(pb) = val.pb {
                    println!("   市净率: {:.2}", pb);
                }
                if let Some(mc) = val.market_cap {
                    println!("   总市值: {:.2} 亿", mc);
                }
                if let Some(roe) = val.roe {
                    println!("   净资产收益率: {:.2}%", roe);
                }
                println!();
            }

            // 显示最新财报
            if let Some(earnings) = &data.latest_earnings {
                println!("📋 最新财报:");
                println!("   报告期: {}", earnings.report_date);
                println!("   报告类型: {}", earnings.report_type);
                if let Some(rev) = earnings.revenue {
                    println!("   营业收入: {:.2} 亿", rev);
                }
                if let Some(profit) = earnings.net_profit {
                    println!("   净利润: {:.2} 亿", profit);
                }
                println!();
            }

            // 显示机构持仓
            if !data.institution_holdings.is_empty() {
                println!("🏛️ 机构持仓 (共 {} 家):", data.institution_holdings.len());
                for holding in data.institution_holdings.iter().take(5) {
                    println!(
                        "   - {} [{}]: {:.2} 万股",
                        holding.institution_name, holding.institution_type, holding.shares
                    );
                }
                println!();
            }

            println!("📁 数据来源: {}", data.source);
            println!(
                "🕐 更新时间: {}",
                data.updated_at.format("%Y-%m-%d %H:%M:%S")
            );
        }
        Err(e) => {
            println!("❌ 获取数据失败: {}", e);
            println!();
            println!("💡 请检查股票代码是否正确，或稍后重试");
        }
    }

    Ok(())
}

async fn run_fundamental_valuation(code: &str) -> Result<()> {
    println!("📈 估值指标");
    println!("   代码: {}", code);
    println!();

    println!("⏳ 正在获取估值数据...");

    let fetcher = ValuationFetcher::new();
    match fetcher.fetch_from_eastmoney(code).await {
        Ok(val) => {
            println!("✅ 数据获取成功");
            println!();

            println!("┌─────────────────┬─────────────┐");
            println!("│ 指标            │ 数值        │");
            println!("├─────────────────┼─────────────┤");
            println!("│ 市盈率 (TTM)    │ {} │", format_decimal(&val.pe_ttm));
            println!("│ 市盈率 (静态)   │ {} │", format_decimal(&val.pe_static));
            println!("│ 市净率          │ {} │", format_decimal(&val.pb));
            println!("│ 市销率          │ {} │", format_decimal(&val.ps));
            println!(
                "│ 总市值          │ {} 亿 │",
                format_decimal(&val.market_cap)
            );
            println!(
                "│ 流通市值        │ {} 亿 │",
                format_decimal(&val.float_market_cap)
            );
            println!(
                "│ 股息率          │ {}% │",
                format_decimal(&val.dividend_yield)
            );
            println!("│ 每股收益        │ {} │", format_decimal(&val.eps));
            println!("│ 每股净资产      │ {} │", format_decimal(&val.bvps));
            println!("│ 净资产收益率    │ {}% │", format_decimal(&val.roe));
            println!(
                "│ 毛利率          │ {}% │",
                format_decimal(&val.gross_margin)
            );
            println!("│ 净利率          │ {}% │", format_decimal(&val.net_margin));
            println!("└─────────────────┴─────────────┘");
            println!();
            println!("📅 数据日期: {}", val.date);
        }
        Err(e) => {
            println!("❌ 获取数据失败: {}", e);
            println!();
            println!("💡 请检查股票代码是否正确，或稍后重试");
        }
    }

    Ok(())
}

async fn run_fundamental_earnings(code: &str, years: u32) -> Result<()> {
    println!("📋 财报数据");
    println!("   代码: {}", code);
    println!("   年数: {}", years);
    println!();

    println!("⏳ 正在获取财报数据...");

    let fetcher = EarningsFetcher::new();

    if years == 1 {
        // 只获取最新财报
        match fetcher.fetch_latest(code).await {
            Ok(report) => {
                println!("✅ 数据获取成功");
                println!();
                println!("📅 最新财报:");
                println!("   报告期: {}", report.report_date);
                println!("   报告类型: {}", report.report_type);
                println!();
                println!("┌─────────────────┬─────────────┐");
                println!("│ 指标            │ 数值        │");
                println!("├─────────────────┼─────────────┤");
                println!(
                    "│ 营业收入        │ {} 亿 │",
                    format_decimal(&report.revenue)
                );
                println!(
                    "│ 营收同比        │ {}% │",
                    format_decimal(&report.revenue_yoy)
                );
                println!(
                    "│ 净利润          │ {} 亿 │",
                    format_decimal(&report.net_profit)
                );
                println!(
                    "│ 净利润同比      │ {}% │",
                    format_decimal(&report.net_profit_yoy)
                );
                println!(
                    "│ 扣非净利润      │ {} 亿 │",
                    format_decimal(&report.net_profit_deducted)
                );
                println!(
                    "│ 经营现金流      │ {} 亿 │",
                    format_decimal(&report.operating_cash_flow)
                );
                println!(
                    "│ 总资产          │ {} 亿 │",
                    format_decimal(&report.total_assets)
                );
                println!(
                    "│ 净资产          │ {} 亿 │",
                    format_decimal(&report.net_assets)
                );
                println!(
                    "│ 资产负债率      │ {}% │",
                    format_decimal(&report.debt_ratio)
                );
                println!(
                    "│ 毛利率          │ {}% │",
                    format_decimal(&report.gross_margin)
                );
                println!(
                    "│ 净利率          │ {}% │",
                    format_decimal(&report.net_margin)
                );
                println!("└─────────────────┴─────────────┘");

                if let Some(announce_date) = report.announce_date {
                    println!();
                    println!("📢 公告日期: {}", announce_date);
                }
            }
            Err(e) => {
                println!("❌ 获取数据失败: {}", e);
                println!();
                println!("💡 请检查股票代码是否正确，或稍后重试");
            }
        }
    } else {
        // 获取历史财报
        match fetcher.fetch_history(code, years).await {
            Ok(reports) => {
                println!("✅ 获取到 {} 期财报数据", reports.len());
                println!();

                for report in reports {
                    println!("┌─────────────────────────────────────┐");
                    println!(
                        "│ 报告期: {} ({})          │",
                        report.report_date, report.report_type
                    );
                    println!("├─────────────────────────────────────┤");
                    if let Some(rev) = report.revenue {
                        println!("│ 营业收入: {:.2} 亿", rev);
                    }
                    if let Some(profit) = report.net_profit {
                        println!("│ 净利润: {:.2} 亿", profit);
                    }
                    if let Some(yoy) = report.net_profit_yoy {
                        let arrow = if yoy > Decimal::ZERO {
                            "↑"
                        } else if yoy < Decimal::ZERO {
                            "↓"
                        } else {
                            "→"
                        };
                        println!("│ 净利润同比: {} {:.2}%", arrow, yoy);
                    }
                    println!("└─────────────────────────────────────┘");
                    println!();
                }
            }
            Err(e) => {
                println!("❌ 获取数据失败: {}", e);
                println!();
                println!("💡 请检查股票代码是否正确，或稍后重试");
            }
        }
    }

    Ok(())
}

async fn run_fundamental_institution(code: &str) -> Result<()> {
    println!("🏛️ 机构持仓");
    println!("   代码: {}", code);
    println!();

    println!("⏳ 正在获取机构持仓数据...");

    let fetcher = InstitutionFetcher::new();
    match fetcher.fetch_holdings(code).await {
        Ok(holdings) => {
            println!("✅ 数据获取成功");
            println!();

            if holdings.is_empty() {
                println!("📊 机构持仓明细:");
                println!("   (暂无数据)");
                println!();
                println!("💡 该股票暂无机构持仓记录");
            } else {
                println!("📊 机构持仓明细 (共 {} 家):", holdings.len());
                println!();

                // 按机构类型统计
                let mut type_counts: HashMap<String, usize> = HashMap::new();
                for h in &holdings {
                    *type_counts.entry(h.institution_type.clone()).or_insert(0) += 1;
                }

                println!("📈 按机构类型统计:");
                for (itype, count) in &type_counts {
                    println!("   - {}: {} 家", itype, count);
                }
                println!();

                // 显示前10条持仓明细
                println!("┌────────────────────────────────────────────────────────────────┐");
                println!("│ 机构名称                              │ 类型    │ 持股(万股)  │");
                println!("├────────────────────────────────────────────────────────────────┤");

                for holding in holdings.iter().take(10) {
                    println!(
                        "│ {:<36} │ {:<6} │ {:>10.2} │",
                        truncate_str(&holding.institution_name, 36),
                        truncate_str(&holding.institution_type, 6),
                        holding.shares
                    );
                }
                println!("└────────────────────────────────────────────────────────────────┘");

                if holdings.len() > 10 {
                    println!();
                    println!("💡 显示前 10 条，共 {} 条记录", holdings.len());
                }
            }
        }
        Err(e) => {
            println!("❌ 获取数据失败: {}", e);
            println!();
            println!("💡 请检查股票代码是否正确，或稍后重试");
        }
    }

    Ok(())
}

async fn run_fundamental_dragon_tiger(code: Option<&str>, days: u32) -> Result<()> {
    if let Some(c) = code {
        println!("🐉 龙虎榜 - {}", c);
    } else {
        println!("🐉 今日龙虎榜");
    }
    println!("   天数: {}", days);
    println!();

    println!("⏳ 正在获取龙虎榜数据...");

    let fetcher = DragonTigerFetcher::new();

    let result = if let Some(c) = code {
        fetcher.fetch(c, days).await
    } else {
        fetcher.fetch_today().await
    };

    match result {
        Ok(items) => {
            println!("✅ 数据获取成功");
            println!();

            if items.is_empty() {
                println!("📊 龙虎榜数据:");
                println!("   (暂无数据)");
                println!();
                if code.is_some() {
                    println!("💡 该股票在指定时间内未上龙虎榜");
                } else {
                    println!("💡 今日暂无龙虎榜数据");
                }
            } else {
                println!("📊 龙虎榜数据 (共 {} 条):", items.len());
                println!();

                for item in items {
                    println!("┌─────────────────────────────────────────────────────────┐");
                    println!(
                        "│ {} ({})                                    ",
                        truncate_str(&item.name, 20),
                        item.code
                    );
                    println!("├─────────────────────────────────────────────────────────┤");
                    println!(
                        "│ 交易日期: {}  收盘价: {:.2}  涨跌幅: {:.2}%",
                        item.trade_date, item.close_price, item.change_pct
                    );
                    println!("│ 上榜原因: {}", truncate_str(&item.reason, 45));
                    println!("├─────────────────────────────────────────────────────────┤");
                    println!(
                        "│ 买入金额: {:>12.2} 万元    卖出金额: {:>12.2} 万元",
                        item.buy_amount, item.sell_amount
                    );
                    println!("│ 净买入:   {:>12.2} 万元", item.net_buy);
                    println!("└─────────────────────────────────────────────────────────┘");
                    println!();

                    // 显示买卖前5营业部
                    if !item.top_buyers.is_empty() {
                        println!("   📈 买入前5营业部:");
                        for buyer in item.top_buyers.iter().take(5) {
                            if let Some(amount) = buyer.buy_amount {
                                println!(
                                    "      - {} ({:.2} 万)",
                                    truncate_str(&buyer.broker_name, 30),
                                    amount
                                );
                            }
                        }
                    }

                    if !item.top_sellers.is_empty() {
                        println!("   📉 卖出前5营业部:");
                        for seller in item.top_sellers.iter().take(5) {
                            if let Some(amount) = seller.sell_amount {
                                println!(
                                    "      - {} ({:.2} 万)",
                                    truncate_str(&seller.broker_name, 30),
                                    amount
                                );
                            }
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("❌ 获取数据失败: {}", e);
            println!();
            println!("💡 请检查股票代码是否正确，或稍后重试");
        }
    }

    Ok(())
}

async fn run_fundamental_dividend(code: &str, years: u32) -> Result<()> {
    println!("💰 分红信息");
    println!("   代码: {}", code);
    println!("   年数: {}", years);
    println!();

    println!("🚧 功能开发中...");
    println!();
    println!("📊 分红数据获取功能即将上线，敬请期待！");
    println!();
    println!("💡 您可以先使用以下命令查看其他基本面数据:");
    println!("   - quantix fundamental valuation {}   # 估值指标", code);
    println!("   - quantix fundamental earnings {}    # 财报数据", code);
    println!("   - quantix fundamental institution {} # 机构持仓", code);

    Ok(())
}

/// Helper function to format optional Decimal values for display
fn format_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(v) => format!("{:.2}", v),
        None => "-".to_string(),
    }
}

/// Helper function to truncate a string to a maximum length
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
