use super::*;
use crate::core::Result;
use crate::sources::tdx_api::{KlineType, TdxApiClient};

fn client() -> Result<TdxApiClient> {
    TdxApiClient::from_env()
}

fn parse_kline_type(s: &str) -> KlineType {
    match s {
        "minute1" => KlineType::Min1,
        "minute5" => KlineType::Min5,
        "minute15" => KlineType::Min15,
        "minute30" => KlineType::Min30,
        "hour" => KlineType::Hour,
        "week" => KlineType::Week,
        "month" => KlineType::Month,
        _ => KlineType::Day,
    }
}

pub(crate) async fn run_tdx_api_command(cmd: TdxApiCommands) -> Result<()> {
    match cmd {
        TdxApiCommands::Health => {
            let c = client()?;
            let resp = c.health().await?;
            println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
        }
        TdxApiCommands::Quote { code } => {
            let c = client()?;
            let q = c.get_quote(&code).await?;
            println!("{code}: 价格={:.3} 涨幅={:.2}% 成交量={:.0} 成交额={:.0}",
                q.price, q.change_percent, q.volume, q.amount);
        }
        TdxApiCommands::Kline { code, r#type, limit } => {
            let c = client()?;
            let kt = parse_kline_type(&r#type);
            let resp = c.get_kline_raw(&code, kt, limit).await?;
            println!("K线 {} 共 {} 条:", r#type, resp.count);
            for item in resp.list.iter().rev().take(20) {
                let date = item.time.split('T').next().unwrap_or(&item.time);
                let o = item.open as f64 / 1000.0;
                let h = item.high as f64 / 1000.0;
                let l = item.low as f64 / 1000.0;
                let cl = item.close as f64 / 1000.0;
                println!("  {date} O={o:.2} H={h:.2} L={l:.2} C={cl:.2} V={}", item.volume);
            }
        }
        TdxApiCommands::KlineThs { code, r#type } => {
            let c = client()?;
            let kt = parse_kline_type(&r#type);
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
                println!("  之后: {}", resp.next.iter().map(|d| d.iso.as_str()).collect::<Vec<_>>().join(", "));
            }
            if !resp.previous.is_empty() {
                println!("  之前: {}", resp.previous.iter().map(|d| d.iso.as_str()).collect::<Vec<_>>().join(", "));
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
        TdxApiCommands::Income { code, start_date, days } => {
            let c = client()?;
            let resp = c.get_income(&code, &start_date, &days).await?;
            println!("收益计算 {} from {}:", code, start_date);
            for item in &resp.list {
                println!("  +{}日: 涨跌额={:.3} 收益率={:.4}%", item.offset, item.rise, item.rise_rate * 100.0);
            }
        }
        TdxApiCommands::MarketStats => {
            let c = client()?;
            let stats = c.get_market_stats().await?;
            for (name, s) in [("沪", &stats.sh), ("深", &stats.sz), ("京", &stats.bj)] {
                println!("  {name}: 总={total} 涨={up} 跌={down} 平={flat}",
                    total = s.total, up = s.up, down = s.down, flat = s.flat);
            }
        }
        TdxApiCommands::Tasks => {
            let c = client()?;
            let tasks = c.list_tasks().await?;
            if tasks.is_empty() {
                println!("无运行中的任务");
            } else {
                for t in &tasks {
                    println!("  [{}] {} - {} ({})",
                        t.id.chars().take(8).collect::<String>(),
                        t.task_type, t.status, t.started_at);
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
            if let Some(e) = &t.ended_at { println!("结束: {e}"); }
            if let Some(e) = &t.error { println!("错误: {e}"); }
        }
    }
    Ok(())
}
