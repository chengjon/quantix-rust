use super::*;
use crate::cli::handlers::market_output::print_market_strength_stock_ranking;
use std::path::Path;

use crate::core::runtime::OpenStockSettings;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketFoundationSummary, MarketOverview, MarketSentimentSnapshot, MarketService,
    MarketStrengthReport, NorthFlowSnapshot, StrongSectorStockRow,
    analyze_market_strength_with_reader, load_market_analysis_foundation,
};
use chrono::NaiveDate;

pub async fn run_market_command(cmd: MarketCommands) -> Result<()> {
    let runtime = CliRuntime::load();
    let output = match cmd {
        MarketCommands::Foundation => {
            let summary = load_market_analysis_foundation(&runtime.risk_path)
                .await?
                .summary;
            MarketCommandOutput::Foundation(summary)
        }
        other => {
            // 优先使用 OpenStock 作为 market 数据源，具备完整配置时走 OpenStock，
            // 否则回退到 ClickHouse
            let os_settings = OpenStockSettings::from_env();
            if os_settings.base_url.is_some() && os_settings.api_key.is_some() {
                match crate::sources::openstock_client::OpenStockClient::from_settings(&os_settings)
                {
                    Ok(os_client) => {
                        let reader =
                            crate::sources::openstock_market::OpenStockMarketReader::new(os_client);
                        execute_market_command_with_runtime(other, reader, Some(&runtime.risk_path))
                            .await?
                    }
                    Err(_) => {
                        let reader = create_clickhouse_client().await?;
                        execute_market_command_with_runtime(other, reader, Some(&runtime.risk_path))
                            .await?
                    }
                }
            } else {
                let reader = create_clickhouse_client().await?;
                execute_market_command_with_runtime(other, reader, Some(&runtime.risk_path)).await?
            }
        }
    };

    match output {
        MarketCommandOutput::Foundation(summary) => print_market_foundation_summary(&summary),
        MarketCommandOutput::BoardRows(rows) => print_market_board_rows(&rows),
        MarketCommandOutput::NorthFlow(snapshot) => print_north_flow_snapshot(snapshot.as_ref()),
        MarketCommandOutput::Sentiment(snapshot) => {
            print_market_sentiment_snapshot(snapshot.as_ref())
        }
        MarketCommandOutput::Leaders(rows) => print_market_leader_rows(&rows),
        MarketCommandOutput::Overview(overview) => print_market_overview(&overview),
        MarketCommandOutput::Strength(report) => print_market_strength_report(&report),
        MarketCommandOutput::StrengthStocks(ranking) => {
            print_market_strength_stock_ranking(&ranking)
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MarketCommandOutput {
    Foundation(MarketFoundationSummary),
    BoardRows(Vec<BoardRankRow>),
    NorthFlow(Option<NorthFlowSnapshot>),
    Sentiment(Option<MarketSentimentSnapshot>),
    Leaders(Vec<LeaderRow>),
    Overview(MarketOverview),
    Strength(MarketStrengthReport),
    StrengthStocks(MarketStrengthStockRankingOutput),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarketStrengthStockRankingOutput {
    pub metric: StrengthStockMetric,
    pub strong_top: usize,
    pub sector_filter: Option<String>,
    pub candidate_stock_count: usize,
    pub covered_count: usize,
    pub rows: Vec<StrongSectorStockRow>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) async fn execute_market_command_with_reader<R>(
    cmd: MarketCommands,
    reader: R,
) -> Result<MarketCommandOutput>
where
    R: MarketDataReader,
{
    execute_market_command_with_runtime(cmd, reader, None).await
}

#[cfg(test)]
pub(crate) async fn execute_market_command_with_test_payloads<R>(
    cmd: MarketCommands,
    reader: R,
    foundation_summary: Option<MarketFoundationSummary>,
    strength_report: Option<MarketStrengthReport>,
) -> Result<MarketCommandOutput>
where
    R: MarketDataReader,
{
    match cmd {
        MarketCommands::Foundation => foundation_summary
            .map(MarketCommandOutput::Foundation)
            .ok_or_else(|| QuantixError::Other("缺少 foundation 测试载荷".to_string())),
        MarketCommands::StrengthStocks {
            strong_top,
            sector,
            metric,
            ..
        } => strength_report
            .map(|report| {
                MarketCommandOutput::StrengthStocks(build_market_strength_stock_ranking_output(
                    report, metric, strong_top, sector,
                ))
            })
            .ok_or_else(|| QuantixError::Other("缺少 strength 测试载荷".to_string())),
        MarketCommands::Strength { .. } => strength_report
            .map(MarketCommandOutput::Strength)
            .ok_or_else(|| QuantixError::Other("缺少 strength 测试载荷".to_string())),
        other => execute_market_command_with_reader(other, reader).await,
    }
}

pub(crate) async fn execute_market_command_with_runtime<R>(
    cmd: MarketCommands,
    reader: R,
    risk_state_path: Option<&Path>,
) -> Result<MarketCommandOutput>
where
    R: MarketDataReader,
{
    match cmd {
        MarketCommands::Foundation => Ok(MarketCommandOutput::Foundation(
            load_market_analysis_foundation(require_risk_state_path(risk_state_path)?)
                .await?
                .summary,
        )),
        MarketCommands::Sector { top, date, sort_by } => {
            let service = MarketService::new(reader);
            let rows = service
                .get_board_rankings(
                    BoardType::Sector,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::Concept { top, date, sort_by } => {
            let service = MarketService::new(reader);
            let rows = service
                .get_board_rankings(
                    BoardType::Concept,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::North { date } => {
            let service = MarketService::new(reader);
            Ok(MarketCommandOutput::NorthFlow(
                service
                    .get_north_flow(parse_market_date(date.as_deref())?)
                    .await?,
            ))
        }
        MarketCommands::Sentiment { date } => {
            let service = MarketService::new(reader);
            Ok(MarketCommandOutput::Sentiment(
                service
                    .get_market_sentiment(parse_market_date(date.as_deref())?)
                    .await?,
            ))
        }
        MarketCommands::Leader {
            sector,
            concept,
            all,
            limit,
            date,
        } => {
            let service = MarketService::new(reader);
            let filter = build_leader_filter(sector, concept, all)?;
            let rows = service
                .get_leaders(filter, limit, parse_market_date(date.as_deref())?)
                .await?;
            Ok(MarketCommandOutput::Leaders(rows))
        }
        MarketCommands::Overview { top, date } => {
            let service = MarketService::new(reader);
            Ok(MarketCommandOutput::Overview(
                service
                    .get_overview(parse_market_date(date.as_deref())?, top)
                    .await?,
            ))
        }
        MarketCommands::Strength {
            date,
            strong_top,
            weak_top,
            stock_top,
        } => Ok(MarketCommandOutput::Strength(
            analyze_market_strength_with_reader(
                &reader,
                parse_market_date(date.as_deref())?,
                require_risk_state_path(risk_state_path)?,
                strong_top,
                weak_top,
                stock_top,
            )
            .await?,
        )),
        MarketCommands::StrengthStocks {
            date,
            strong_top,
            sector,
            metric,
            top,
        } => {
            let report = analyze_market_strength_with_reader(
                &reader,
                parse_market_date(date.as_deref())?,
                require_risk_state_path(risk_state_path)?,
                strong_top,
                1,
                top,
            )
            .await?;
            Ok(MarketCommandOutput::StrengthStocks(
                build_market_strength_stock_ranking_output(report, metric, strong_top, sector),
            ))
        }
    }
}

fn build_market_strength_stock_ranking_output(
    report: MarketStrengthReport,
    metric: StrengthStockMetric,
    strong_top: usize,
    sector_filter: Option<String>,
) -> MarketStrengthStockRankingOutput {
    let (rows, covered_count) = match metric {
        StrengthStockMetric::MarketCap => {
            (report.top_by_market_cap, report.market_cap_coverage_count)
        }
        StrengthStockMetric::Profit => (report.top_by_profit, report.profit_coverage_count),
    };
    let (candidate_stock_count, covered_count, rows) = match sector_filter.as_ref() {
        Some(sector_name) => {
            let filtered_rows = rows
                .into_iter()
                .filter(|row| row.sector_name == *sector_name)
                .collect::<Vec<_>>();
            let filtered_count = filtered_rows.len();
            (
                filtered_count,
                covered_count.min(filtered_count),
                filtered_rows,
            )
        }
        None => (report.candidate_stock_count, covered_count, rows),
    };

    MarketStrengthStockRankingOutput {
        metric,
        strong_top,
        sector_filter,
        candidate_stock_count,
        covered_count,
        rows,
    }
}

fn require_risk_state_path(risk_state_path: Option<&Path>) -> Result<&Path> {
    risk_state_path.ok_or_else(|| {
        QuantixError::Other("当前 market 命令需要运行时 risk_path 上下文".to_string())
    })
}

pub(crate) fn build_leader_filter(
    sector: Option<String>,
    concept: Option<String>,
    all: bool,
) -> Result<LeaderFilter> {
    let mut filter_count = 0usize;
    if sector.is_some() {
        filter_count += 1;
    }
    if concept.is_some() {
        filter_count += 1;
    }
    if all {
        filter_count += 1;
    }

    if filter_count != 1 {
        return Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        ));
    }

    match (sector, concept, all) {
        (Some(name), None, false) => Ok(LeaderFilter::Sector(name)),
        (None, Some(name), false) => Ok(LeaderFilter::Concept(name)),
        (None, None, true) => Ok(LeaderFilter::All),
        _ => Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        )),
    }
}

pub(crate) fn parse_market_date(raw: Option<&str>) -> Result<Option<NaiveDate>> {
    raw.map(|value| {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| QuantixError::Other(format!("无效日期格式: {}，请使用 YYYY-MM-DD", value)))
    })
    .transpose()
}

pub(crate) fn parse_board_sort_by(raw: Option<&str>) -> Result<BoardSortBy> {
    match raw.unwrap_or("change_pct") {
        "change" | "change_pct" => Ok(BoardSortBy::ChangePct),
        other => Err(QuantixError::Unsupported(format!(
            "不支持的 sort_by: {}，仅支持 change 或 change_pct",
            other
        ))),
    }
}
