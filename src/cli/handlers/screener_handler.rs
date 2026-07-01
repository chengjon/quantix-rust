use super::*;

use crate::core::{QuantixError, Result};
use crate::screener::{
    DailyKlineLoader, PresetInvocation, RuleMatchDetail, ScreenRow, ScreenRunOptions, ScreenSortBy,
    ScreenUniverse, ScreenerService, parse_preset_invocation,
};
use crate::strategy::runtime::StrategyBarLoader;
use crate::watchlist::WatchlistStorage;
use async_trait::async_trait;

pub(crate) async fn run_screener_command(cmd: ScreenerCommands) -> Result<()> {
    let output = match cmd {
        ScreenerCommands::PresetList => {
            execute_screener_command_with_loader(
                ScreenerCommands::PresetList,
                NullDailyKlineLoader,
                create_watchlist_storage(),
            )
            .await?
        }
        ScreenerCommands::Run { .. } => {
            let loader = ClickHouseDailyKlineLoader::new();
            execute_screener_command_with_loader(cmd, loader, create_watchlist_storage()).await?
        }
    };

    match output {
        ScreenerCommandOutput::PresetList(presets) => print_screener_preset_list(&presets),
        ScreenerCommandOutput::Rows(rows) => print_screener_rows(&rows),
    }

    Ok(())
}

pub(crate) struct ClickHouseDailyKlineLoader;

impl ClickHouseDailyKlineLoader {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DailyKlineLoader for ClickHouseDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        // 走统一 K 线获取入口：OpenStock /data/bars → ClickHouse day_kline fallback
        let mut rows = get_kline_for_analysis(code, None, None, None).await?;

        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }

        Ok(rows)
    }
}

#[async_trait]
impl StrategyBarLoader for ClickHouseDailyKlineLoader {
    async fn load_daily_bars(
        &self,
        code: &str,
        limit: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        // 走统一 K 线获取入口
        let mut rows = get_kline_for_analysis(code, None, None, None).await?;
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

struct NullDailyKlineLoader;

#[async_trait]
impl DailyKlineLoader for NullDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        _code: &str,
        _lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenerPresetSpec {
    pub(crate) name: &'static str,
    pub(crate) params: &'static str,
    pub(crate) description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScreenerCommandOutput {
    PresetList(Vec<ScreenerPresetSpec>),
    Rows(Vec<ScreenRow>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenerRunRequest {
    universe: ScreenUniverse,
    presets: Vec<PresetInvocation>,
    options: ScreenRunOptions,
}

pub(crate) async fn execute_screener_command_with_loader<L>(
    cmd: ScreenerCommands,
    loader: L,
    storage: WatchlistStorage,
) -> Result<ScreenerCommandOutput>
where
    L: DailyKlineLoader,
{
    match cmd {
        ScreenerCommands::PresetList => {
            Ok(ScreenerCommandOutput::PresetList(screener_preset_specs()))
        }
        ScreenerCommands::Run {
            codes,
            watchlist,
            group,
            preset,
            limit,
            sort_by,
        } => {
            let request =
                build_screener_run_request(codes, watchlist, group, preset, limit, sort_by)?;
            let service = ScreenerService::new(loader, storage);
            let rows = service
                .run(request.universe, &request.presets, request.options)
                .await?;
            Ok(ScreenerCommandOutput::Rows(rows))
        }
    }
}

fn screener_preset_specs() -> Vec<ScreenerPresetSpec> {
    vec![
        ScreenerPresetSpec {
            name: "close_above_ma",
            params: "period=<n>",
            description: "收盘价高于均线",
        },
        ScreenerPresetSpec {
            name: "close_below_ma",
            params: "period=<n>",
            description: "收盘价低于均线",
        },
        ScreenerPresetSpec {
            name: "rsi_gte",
            params: "period=<n>,value=<x>",
            description: "RSI 大于等于阈值",
        },
        ScreenerPresetSpec {
            name: "rsi_lte",
            params: "period=<n>,value=<x>",
            description: "RSI 小于等于阈值",
        },
        ScreenerPresetSpec {
            name: "volume_ratio_gte",
            params: "window=<n>,value=<x>",
            description: "量比大于等于阈值",
        },
    ]
}

fn build_screener_run_request(
    codes: Option<String>,
    watchlist: bool,
    group: Option<String>,
    preset_specs: Vec<String>,
    limit: Option<usize>,
    sort_by: Option<String>,
) -> Result<ScreenerRunRequest> {
    let universe = match (codes, watchlist) {
        (Some(_), true) => {
            return Err(QuantixError::Other(
                "--codes 与 --watchlist 不能同时使用".to_string(),
            ));
        }
        (None, false) => {
            return Err(QuantixError::Other(
                "必须指定 --codes 或 --watchlist".to_string(),
            ));
        }
        (Some(codes), false) => {
            let codes = parse_codes_csv(&codes);
            if codes.is_empty() {
                return Err(QuantixError::Other("codes 不能为空".to_string()));
            }
            if group.is_some() {
                return Err(QuantixError::Other(
                    "--group 仅可与 --watchlist 一起使用".to_string(),
                ));
            }
            ScreenUniverse::Codes(codes)
        }
        (None, true) => ScreenUniverse::Watchlist { group },
    };

    if preset_specs.is_empty() {
        return Err(QuantixError::Other("至少需要一个 --preset".to_string()));
    }

    let presets = preset_specs
        .iter()
        .map(|spec| parse_preset_invocation(spec))
        .collect::<Result<Vec<_>>>()?;

    let sort_by = match sort_by.as_deref().unwrap_or("code") {
        "code" => ScreenSortBy::Code,
        "score" => ScreenSortBy::Score,
        other => {
            return Err(QuantixError::Unsupported(format!(
                "不支持的 sort_by: {}，仅支持 code 或 score",
                other
            )));
        }
    };

    Ok(ScreenerRunRequest {
        universe,
        presets,
        options: ScreenRunOptions { limit, sort_by },
    })
}

fn parse_codes_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

fn print_screener_preset_list(presets: &[ScreenerPresetSpec]) {
    println!("{:<20} {:<24} 说明", "Preset", "参数");
    println!("{}", "-".repeat(72));

    for preset in presets {
        println!(
            "{:<20} {:<24} {}",
            preset.name, preset.params, preset.description
        );
    }
}

fn print_screener_rows(rows: &[ScreenRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的筛选结果");
        return;
    }

    println!("{:<10} {:<8} {:<12} 详情", "代码", "命中", "评分");
    println!("{}", "-".repeat(96));

    for row in rows {
        println!(
            "{:<10} {:<8} {:<12} {}",
            row.code,
            if row.matched { "yes" } else { "no" },
            row.score.round_dp(4),
            row.details
                .iter()
                .map(format_screener_rule_detail)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
}

fn format_screener_rule_detail(detail: &RuleMatchDetail) -> String {
    let status = if detail.matched { "Y" } else { "N" };

    match (
        detail.actual_value.as_ref(),
        detail.threshold_value.as_ref(),
        detail.reason.as_deref(),
    ) {
        (_, _, Some(reason)) => format!("{}:{}({})", status, detail.preset_name, reason),
        (Some(actual), Some(threshold), None) => {
            format!(
                "{}:{} {} / {}",
                status, detail.preset_name, actual, threshold
            )
        }
        _ => format!("{}:{}", status, detail.preset_name),
    }
}
