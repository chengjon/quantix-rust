use super::*;

pub async fn run_data_command(cmd: DataCommands) -> Result<()> {
    match cmd {
        DataCommands::Source(subcommand) => match subcommand {
            DataSourceCommands::List { config_dir } => {
                list_data_sources(&config_dir)?;
            }
            DataSourceCommands::Add {
                config_dir,
                source_type,
                hosts,
                port,
                timeout,
                base_url,
                rate_limit,
            } => {
                add_data_source(
                    &config_dir,
                    source_type,
                    hosts,
                    port,
                    timeout,
                    base_url,
                    rate_limit,
                )?;
            }
            DataSourceCommands::SetDefault { config_dir, name } => {
                set_default_data_source(&config_dir, name)?;
            }
            DataSourceCommands::Test { config_dir, name } => {
                test_data_source(&config_dir, name).await?;
            }
        },
        DataCommands::OpenStock(subcommand) => match subcommand {
            OpenStockCommands::ValidateFixture { file } => {
                validate_openstock_fixture(&file)?;
            }
            OpenStockCommands::ValidateLive {
                payload,
                symbol,
                period,
                start,
                end,
                limit,
            } => {
                validate_openstock_live(&payload, &symbol, &period, &start, &end, limit)?;
            }
            OpenStockCommands::PersistLive {
                payload,
                symbol,
                period,
                start,
                end,
                limit,
                apply,
            } => {
                persist_openstock_live(&payload, &symbol, &period, &start, &end, limit, apply)
                    .await?;
            }
            OpenStockCommands::ShadowRollback { batch_id } => {
                shadow_rollback(&batch_id).await?;
            }
            OpenStockCommands::ShadowVerify { batch_id } => {
                shadow_verify(&batch_id).await?;
            }
            OpenStockCommands::ValidateCodes { payload, kind } => {
                validate_openstock_codes(&payload, kind.as_deref())?;
            }
            OpenStockCommands::ValidateCalendar { payload, kind } => {
                validate_openstock_calendar(&payload, &kind)?;
            }
            OpenStockCommands::ValidateIndex {
                payload,
                symbol,
                start,
                end,
            } => {
                validate_openstock_index(&payload, &symbol, start.as_deref(), end.as_deref())?;
            }
            OpenStockCommands::FetchCodes => {
                let rt = CliRuntime::load();
                fetch_openstock_codes(&rt.openstock).await?;
            }
            OpenStockCommands::FetchCalendar { year, start, end } => {
                let rt = CliRuntime::load();
                fetch_openstock_calendar(&rt.openstock, year, start.as_deref(), end.as_deref())
                    .await?;
            }
            OpenStockCommands::FetchIndex { symbol, start, end } => {
                let rt = CliRuntime::load();
                fetch_openstock_index(&rt.openstock, &symbol, start.as_deref(), end.as_deref())
                    .await?;
            }
            OpenStockCommands::FetchKlines {
                symbol,
                period,
                adjust,
                start,
                end,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_klines(
                    &rt.openstock,
                    &symbol,
                    &period,
                    &adjust,
                    start.as_deref(),
                    end.as_deref(),
                )
                .await?;
            }
            OpenStockCommands::FetchMinuteKlines {
                symbol,
                period,
                date,
                start,
                end,
                adjust,
                stream,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_minute_klines(
                    &rt.openstock,
                    symbol,
                    period,
                    date,
                    start,
                    end,
                    adjust,
                    stream,
                )
                .await?;
            }
            OpenStockCommands::FetchMinuteShare {
                symbol,
                date,
                start,
                end,
                stream,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_minute_share(&rt.openstock, symbol, date, start, end, stream)
                    .await?;
            }
            OpenStockCommands::ImportMinuteKlines {
                code,
                period,
                adjust,
                start,
                end,
                apply,
            } => {
                let rt = CliRuntime::load();
                import_openstock_minute_klines(
                    &rt.openstock,
                    code,
                    period,
                    adjust,
                    start,
                    end,
                    apply,
                )
                .await?;
            }
            OpenStockCommands::ImportMinuteShare {
                code,
                start,
                end,
                apply,
            } => {
                let rt = CliRuntime::load();
                import_openstock_minute_share(&rt.openstock, code, start, end, apply).await?;
            }
            OpenStockCommands::ImportMinuteAll {
                date,
                format,
                dry_run,
            } => {
                let rt = CliRuntime::load();
                let pg_url = resolve_pg_url()?;
                import_openstock_minute_all(&rt.openstock, &pg_url, date, format, dry_run).await?;
            }
            OpenStockCommands::ImportStatus { date, format } => {
                let pg_url = resolve_pg_url()?;
                query_import_status(&pg_url, date, format).await?;
            }
            OpenStockCommands::FetchAllStocks { day } => {
                let rt = CliRuntime::load();
                fetch_openstock_all_stocks(&rt.openstock, day.as_deref()).await?;
            }
            OpenStockCommands::FetchWorkdays {
                action,
                date,
                start,
                end,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_workdays(
                    &rt.openstock,
                    &action,
                    date.as_deref(),
                    start.as_deref(),
                    end.as_deref(),
                )
                .await?;
            }
        },
        DataCommands::ImportFundamentals { input } => {
            import_market_fundamentals(input).await?;
        }
        DataCommands::Query {
            code,
            start,
            end,
            r#type,
            limit,
        } => {
            query_kline_data(code, start, end, r#type, limit).await?;
        }
        DataCommands::Export {
            code,
            format,
            output,
        } => {
            export_data(code, format, output).await?;
        }
        DataCommands::ImportTicks { code, date, apply } => {
            super::openstock_handler::import_openstock_ticks(&code, date.as_deref(), apply).await?;
        }
        DataCommands::ImportKlines {
            code,
            r#type,
            start,
            end,
            apply,
        } => {
            super::openstock_handler::import_openstock_klines(
                &code,
                &r#type,
                start.as_deref(),
                end.as_deref(),
                apply,
            )
            .await?;
        }
    }
    Ok(())
}

pub(super) async fn run_data_sync_menu() -> Result<()> {
    let items = vec![
        "查询K线数据     — 从ClickHouse查询指定股票的K线数据",
        "导出数据        — 导出K线数据为CSV或Parquet格式",
        "数据源管理      — 查看/添加/测试数据源配置",
        "返回",
    ];

    println!("\n  💡 数据同步功能需要：1) ClickHouse数据库运行中  2) 已配置数据源 (Bridge/TDX)");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            println!("  💡 请确保ClickHouse中已有K线数据 (可通过 quantix data query 验证)");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let limit = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("查询条数")
                .default("10".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let limit: usize = limit.parse().unwrap_or(10);
            query_kline_data(code, None, None, "day".to_string(), limit).await?;
        }
        1 => {
            println!("  💡 支持CSV (Excel可打开) 和 Parquet (大数据分析) 两种格式");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let items = vec!["CSV (Excel兼容)", "Parquet (大数据分析)"];
            let fmt_sel = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact()
                .map_err(|e| QuantixError::Other(format!("选择失败: {}", e)))?;

            let fmt = if fmt_sel == 0 { "csv" } else { "parquet" };
            export_data(code, fmt.to_string(), "./data".to_string()).await?;
        }
        2 => {
            println!("\n  📋 数据源操作：");
            println!("    quantix data source list          查看已配置的数据源");
            println!("    quantix data source add            添加数据源");
            println!("    quantix data source test --name X  测试数据源连通性");
            println!("\n  💡 数据源配置文件位于 ../config/ 目录");
        }
        3 => {}
        _ => {}
    }

    Ok(())
}
