use super::*;
use crate::core::{QuantixError, Result};
use crate::sources::tdx_api::{KlineType, TdxApiClient};

fn client() -> Result<TdxApiClient> {
    TdxApiClient::from_env()
}

fn parse_kline_type(s: &str) -> Result<KlineType> {
    match s {
        "minute1" => Ok(KlineType::Min1),
        "minute5" => Ok(KlineType::Min5),
        "minute15" => Ok(KlineType::Min15),
        "minute30" => Ok(KlineType::Min30),
        "hour" => Ok(KlineType::Hour),
        "week" => Ok(KlineType::Week),
        "month" => Ok(KlineType::Month),
        "day" => Ok(KlineType::Day),
        _ => Err(QuantixError::DataParse(format!(
            "不支持的 tdx-api K线周期: {s}"
        ))),
    }
}

pub(crate) async fn run_tdx_api_command(cmd: TdxApiCommands) -> Result<()> {
    match cmd {
        TdxApiCommands::Health => {
            let c = client()?;
            let health = c.health_status().await?;
            let server = c.server_status().await?;
            println!(
                "tdx-api: healthy={} status={} connected={} version={}",
                health.is_healthy(),
                server.status,
                server.connected,
                server.version.as_deref().unwrap_or("unknown")
            );
        }
        TdxApiCommands::Quote { code } => {
            let c = client()?;
            let q = c.get_quote(&code).await?;
            println!(
                "{code}: 价格={:.3} 涨幅={:.2}% 成交量={:.0} 成交额={:.0}",
                q.price, q.change_percent, q.volume, q.amount
            );
        }
        TdxApiCommands::Kline {
            code,
            r#type,
            limit,
        } => {
            let c = client()?;
            let kt = parse_kline_type(&r#type)?;
            let resp = c.get_kline_raw(&code, kt, limit).await?;
            println!("K线 {} 共 {} 条:", r#type, resp.count);
            for item in resp.list.iter().rev().take(20) {
                let date = item.time.split('T').next().unwrap_or(&item.time);
                let o = item.open as f64 / 1000.0;
                let h = item.high as f64 / 1000.0;
                let l = item.low as f64 / 1000.0;
                let cl = item.close as f64 / 1000.0;
                println!(
                    "  {date} O={o:.2} H={h:.2} L={l:.2} C={cl:.2} V={}",
                    item.volume
                );
            }
        }
        TdxApiCommands::KlineThs { code, r#type } => {
            let c = client()?;
            let kt = parse_kline_type(&r#type)?;
            let klines = c.get_kline_all_ths(&code, kt).await?;
            println!("THS 前复权 {} 共 {} 条:", r#type, klines.len());
            for k in klines.iter().rev().take(20) {
                println!("  {} C={}", k.date, k.close);
            }
        }
        TdxApiCommands::Minute { code, date } => {
            let c = client()?;
            let resp = c.get_minute(&code, date.as_deref()).await?;
            println!("分时 {} 共 {} 条:", resp.date, resp.count);
            for m in resp.list.iter().take(10) {
                let p = m.price as f64 / 1000.0;
                println!("  {} 价格={:.2} 成交量={}", m.time, p, m.number);
            }
            if resp.count > 10 {
                println!("  ... 共 {} 条", resp.count);
            }
        }
        TdxApiCommands::Search { keyword } => {
            let c = client()?;
            let results = c.search_codes(&keyword).await?;
            for r in &results {
                println!("  {} ({}) - {}", r.code, r.exchange, r.name);
            }
            println!("共 {} 条结果", results.len());
        }
        TdxApiCommands::Workday { date, count } => {
            let c = client()?;
            let ds = date.unwrap_or_else(|| chrono::Local::now().format("%Y%m%d").to_string());
            let resp = c.get_workday(&ds, count).await?;
            println!("{}: 交易日={}", resp.date.iso, resp.is_workday);
            if !resp.next.is_empty() {
                println!(
                    "  之后: {}",
                    resp.next
                        .iter()
                        .map(|d| d.iso.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if !resp.previous.is_empty() {
                println!(
                    "  之前: {}",
                    resp.previous
                        .iter()
                        .map(|d| d.iso.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        TdxApiCommands::WorkdayRange { start, end } => {
            let c = client()?;
            let dates = c.get_workday_range(&start, &end).await?;
            println!("交易日 {} ~ {} 共 {} 天:", start, end, dates.len());
            for d in &dates {
                println!("  {}", d);
            }
        }
        TdxApiCommands::Income {
            code,
            start_date,
            days,
        } => {
            let c = client()?;
            let resp = c.get_income(&code, &start_date, &days).await?;
            println!("收益计算 {} from {}:", code, start_date);
            for item in &resp.list {
                println!(
                    "  +{}日: 涨跌额={:.3} 收益率={:.4}%",
                    item.offset,
                    item.rise,
                    item.rise_rate * 100.0
                );
            }
        }
        TdxApiCommands::MarketStats => {
            let c = client()?;
            let stats = c.get_market_stats().await?;
            for (name, s) in [("沪", &stats.sh), ("深", &stats.sz), ("京", &stats.bj)] {
                println!(
                    "  {name}: 总={total} 涨={up} 跌={down} 平={flat}",
                    total = s.total,
                    up = s.up,
                    down = s.down,
                    flat = s.flat
                );
            }
        }
        TdxApiCommands::Tasks => {
            let c = client()?;
            let tasks = c.list_tasks().await?;
            if tasks.is_empty() {
                println!("无运行中的任务");
            } else {
                for t in &tasks {
                    println!(
                        "  [{}] {} - {} ({})",
                        t.id.chars().take(8).collect::<String>(),
                        t.task_type,
                        t.status,
                        t.started_at
                    );
                }
            }
        }
        TdxApiCommands::TaskInfo { id } => {
            let c = client()?;
            let t = c.get_task(&id).await?;
            println!("ID: {}", t.id);
            println!("类型: {}", t.task_type);
            println!("状态: {}", t.status);
            println!("开始: {}", t.started_at);
            if let Some(e) = &t.ended_at {
                println!("结束: {e}");
            }
            if let Some(e) = &t.error {
                println!("错误: {e}");
            }
        }
        TdxApiCommands::PullKline {
            codes,
            start_date,
            limit,
        } => {
            use crate::sources::tdx_api::PullKlineRequest;

            let c = client()?;
            let req = PullKlineRequest {
                codes,
                tables: Vec::new(),
                dir: String::new(),
                limit,
                start_date: start_date.unwrap_or_default(),
            };
            let task_id = c.create_pull_kline_task(&req).await?;
            println!("K线拉取任务已创建: {}", task_id);
            println!("使用 tdx-api task-info --id {} 查看进度", task_id);
        }
        TdxApiCommands::PullTrade {
            code,
            start_year,
            end_year,
        } => {
            use crate::sources::tdx_api::PullTradeRequest;

            let c = client()?;
            let req = PullTradeRequest {
                code,
                dir: String::new(),
                start_year,
                end_year,
            };
            let task_id = c.create_pull_trade_task(&req).await?;
            println!("成交拉取任务已创建: {}", task_id);
            println!("使用 tdx-api task-info --id {} 查看进度", task_id);
        }
        TdxApiCommands::CancelTask { id } => {
            let c = client()?;
            let resp = c.cancel_task(&id).await?;
            println!("任务 {} 已取消: {}", id, resp);
        }
        TdxApiCommands::ImportTicks { code, date } => {
            use crate::core::config::AppConfig;
            use crate::db::TDengineClient;

            let c = client()?;
            let resp = c.get_trades(&code, date.as_deref()).await?;
            if resp.list.is_empty() {
                println!("{} 无成交数据", code);
                return Ok(());
            }
            println!("获取到 {} 条逐笔成交数据", resp.list.len());

            let config =
                AppConfig::load().map_err(|e| QuantixError::Other(format!("加载配置失败: {e}")))?;
            let td = config.database.tdengine;
            let token = format!("{}:{}", td.username, td.password);
            let tde = TDengineClient::new(&format!("http://{}:{}", td.host, td.port), &token)?;
            tde.check_connection().await?;
            tde.create_tick_table().await?;

            let ticks: Vec<(i64, f64, i32, f64, i32)> = resp
                .list
                .iter()
                .map(|t| {
                    let price = t.price as f64 / 1000.0;
                    let amount = price * t.volume as f64 * 100.0;
                    (0i64, price, t.volume, amount, t.status)
                })
                .collect();

            tde.insert_ticks(&code, &ticks).await?;
            println!("已导入 {} 条逐笔数据到 TDengine", ticks.len());
        }
        TdxApiCommands::SyncCalendar { year } => {
            use crate::core::trading_calendar::TradingCalendar;
            use std::path::Path;

            let y =
                year.unwrap_or_else(|| chrono::Datelike::year(&chrono::Local::now().date_naive()));
            let c = client()?;

            // API 每次最多返回 100 条，按季度分批获取
            let mut trading_days = Vec::new();
            for (ms, me) in [
                ("0101", "0331"),
                ("0401", "0630"),
                ("0701", "0930"),
                ("1001", "1231"),
            ] {
                let s = format!("{y}{ms}");
                let e = format!("{y}{me}");
                if let Ok(mut batch) = c.get_workday_range(&s, &e).await {
                    trading_days.append(&mut batch);
                }
            }

            let mut cal = TradingCalendar::default();
            cal.sync_trading_days(y, trading_days);

            // 持久化到 config/holidays.json
            let config_path = Path::new("config/holidays.json");
            if let Some(parent) = config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // 读取已有配置或创建新配置
            let mut config: serde_json::Value = if config_path.exists() {
                serde_json::from_str(&std::fs::read_to_string(config_path).unwrap_or_default())
                    .unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            let holidays_arr: Vec<String> = cal
                .holidays_for_year(y)
                .iter()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .collect();
            let workdays_arr: Vec<String> = cal
                .workdays_on_weekend_for_year(y)
                .iter()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .collect();

            let year_key = y.to_string();
            config["years"][&year_key] = serde_json::json!({
                "holidays": holidays_arr,
                "early_close": [],
                "workdays_on_weekend": workdays_arr
            });

            std::fs::write(
                config_path,
                serde_json::to_string_pretty(&config).unwrap_or_default(),
            )
            .map_err(|e| QuantixError::Other(format!("写入日历失败: {e}")))?;

            println!("已同步 {} 年交易日历 → config/holidays.json", y);
            println!(
                "  节假日: {} 天, 调休日: {} 天",
                holidays_arr.len(),
                workdays_arr.len()
            );
        }
        TdxApiCommands::ImportKlines {
            code,
            all,
            exchange,
            r#type,
            force,
        } => {
            use crate::db::ClickHouseClient;

            let c = client()?;
            let kt = parse_kline_type(&r#type)?;
            let ch = ClickHouseClient::with_default_config().await?;
            ch.check_connection().await?;

            let codes = if all {
                let ex = exchange.as_deref();
                let resp = c.get_codes(ex).await?;
                println!(
                    "获取到 {} 只股票代码{}",
                    resp.codes.len(),
                    ex.map(|e| format!(" (交易所: {e})")).unwrap_or_default()
                );
                resp.codes.into_iter().map(|e| e.code).collect::<Vec<_>>()
            } else {
                match code {
                    Some(c) => vec![c],
                    None => {
                        return Err(QuantixError::Other(
                            "请指定 --code <代码> 或 --all".to_string(),
                        ));
                    }
                }
            };

            let total = codes.len();
            let mut imported = 0usize;
            let mut skipped = 0usize;
            let mut failed = 0usize;

            for (i, stock_code) in codes.iter().enumerate() {
                // 增量检查: 获取最新日期，后续过滤只插入新数据
                let latest_date = if !force {
                    ch.get_latest_kline_date(stock_code, &r#type, "THS_QFQ")
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                if total > 1 {
                    println!("[{}/{}] 正在获取 {} ...", i + 1, total, stock_code);
                }

                match c.get_kline_all_ths(stock_code, kt).await {
                    Ok(klines) => {
                        if klines.is_empty() {
                            skipped += 1;
                            continue;
                        }

                        // 过滤: 只保留最新日期之后的数据
                        let new_klines: Vec<_> = match latest_date {
                            Some(cutoff) => {
                                klines.into_iter().filter(|k| k.date > cutoff).collect()
                            }
                            None => klines,
                        };

                        if new_klines.is_empty() {
                            skipped += 1;
                            continue;
                        }

                        if let Err(e) = ch
                            .insert_kline_data_batch_with_source(&new_klines, &r#type, "THS_QFQ")
                            .await
                        {
                            failed += 1;
                            eprintln!("  导入 {} 失败: {}", stock_code, e);
                            continue;
                        }

                        imported += new_klines.len();
                        println!(
                            "  {} → {} 条新增 (累计: {})",
                            stock_code,
                            new_klines.len(),
                            imported
                        );
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("  获取 {} 失败: {}", stock_code, e);
                    }
                }

                // 限流: 避免对 tdx-api 造成压力
                if total > 1 && i < total - 1 {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }

            println!(
                "导入完成: {} 条记录, {} 只跳过, {} 只失败",
                imported, skipped, failed
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kline_type_rejects_unknown_period() {
        let err = parse_kline_type("quarter").unwrap_err();

        assert!(
            err.to_string().contains("不支持的 tdx-api K线周期"),
            "unexpected error: {err}"
        );
    }
}
