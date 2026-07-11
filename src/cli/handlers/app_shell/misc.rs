use super::*;

pub(super) async fn probe_tcp_async(url: &str) -> bool {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("mysql://"))
        .or_else(|| url.strip_prefix("mysqlx://"))
        .or_else(|| url.strip_prefix("tcp://"))
        .unwrap_or(url);

    let host_port = stripped.split('/').next().unwrap_or(stripped);
    let (host, port) = if let Some(colon) = host_port.rfind(':') {
        (
            &host_port[..colon],
            host_port[colon + 1..].parse::<u16>().unwrap_or(0),
        )
    } else if url.starts_with("https://") {
        (host_port, 443)
    } else {
        (host_port, 80)
    };

    if host.is_empty() || port == 0 {
        return false;
    }

    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect((host, port)),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

pub async fn run_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Indicators { code, indicators } => {
            calculate_indicators(code, indicators).await?;
        }
        AnalyzeCommands::Backtest { id } => {
            show_backtest_report(id)?;
        }
        AnalyzeCommands::CandlePattern {
            candle,
            code,
            tdx_root,
            market,
            day_file,
            start,
            end,
            r#type,
            limit,
            reference,
            previous_close,
        } => {
            analyze_candle_patterns(
                candle,
                code,
                tdx_root,
                market,
                day_file,
                start,
                end,
                r#type,
                limit,
                reference,
                previous_close,
            )
            .await?;
        }
        AnalyzeCommands::Screener(cmd) => {
            run_screener_command(cmd).await?;
        }
    }
    Ok(())
}

pub async fn run_status(health: bool) -> Result<()> {
    if health {
        println!("🏥 检查数据库连接...");

        match create_clickhouse_client().await {
            Ok(_) => println!("  ✅ ClickHouse: 连接正常"),
            Err(e) => println!("  ❌ ClickHouse: 连接失败 - {}", e),
        }
    } else {
        println!("📊 Quantix CLI 状态:");
        println!();
        println!("  版本: 0.1.0");
        println!("  模式: 共享数据库模式");
        println!("  Phase: 14/14 (CLI 命令实现)");
        println!();
        println!("  📦 已完成模块:");
        println!("    ✅ 数据采集基础 (Phase 1)");
        println!("    ✅ 竞价分析 (Phase 2)");
        println!("    ✅ K线管理 (Phase 3)");
        println!("    ✅ 回测引擎 (Phase 4)");
        println!("    ✅ 任务调度 (Phase 5)");
        println!("    ✅ TDX解析 (Phase 6)");
        println!("    ✅ GBBQ存储 (Phase 7)");
        println!("    ✅ 多周期查询 (Phase 8)");
        println!("    ✅ 东方财富 (Phase 9)");
        println!("    ✅ 批量优化 (Phase 10)");
        println!("    ✅ WebSocket (Phase 11)");
        println!("    ✅ 技术指标 (Phase 12)");
        println!("    ✅ Polars适配 (Phase 13)");
        println!("    ✅ CLI命令 (Phase 14)");
        println!();
    }

    Ok(())
}

pub(super) fn backtest_history_menu_command() -> crate::cli::command_types::BacktestCommands {
    crate::cli::command_types::BacktestCommands::List
}
