use super::*;
use crate::cli::{RiskImportCommands, RiskRebuildCommands, RiskSyncCommands};
use crate::risk::{
    BuyLockState, IndustrySyncSummary, JsonRiskStore, MySqlIndustrySyncSource, PositionRiskRow,
    RiskAccountSnapshot, RiskLockStateSource, RiskLogEvent, RiskLogEventType, RiskRule,
    RiskService, RiskStatus, sync_industry_reference_data_at,
};
use chrono::{DateTime, Utc};
use std::path::Path;

mod output;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::trade::{JsonPaperTradeStore, PaperTradeAccount, PaperTradeStore};

pub async fn run_risk_command(cmd: RiskCommands) -> Result<()> {
    let runtime = CliRuntime::load();
    let risk_path = runtime.risk_path.clone();
    let service = RiskService::new(JsonRiskStore::new(risk_path.clone()));
    let output = match &cmd {
        RiskCommands::Sync(_) => {
            let sync_source = MySqlIndustrySyncSource::new(runtime.upstream_mysql.clone());
            execute_risk_command_with_service_and_sync_at(
                cmd,
                &service,
                &risk_path,
                &sync_source,
                Utc::now(),
            )
            .await?
        }
        _ => execute_risk_command_with_service_at(cmd, &service, Utc::now()).await?,
    };
    output::print_risk_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RiskCommandOutput {
    RuleSet(RiskRule),
    RuleList(Vec<RiskRule>),
    RuleToggled(RiskRule),
    ImportSummary(crate::risk::LiveImportBatchSummary),
    RebuildSummary(crate::risk::LiveImportMirrorAccount),
    IndustrySync(IndustrySyncSummary),
    Log(Vec<RiskLogEvent>),
    LockReleased(BuyLockState),
    Status(RiskStatus),
    Pnl(RiskStatus),
    Position(RiskStatus),
}

pub(super) async fn execute_risk_command_with_service_at<Store>(
    cmd: RiskCommands,
    service: &RiskService<Store>,
    now: chrono::DateTime<Utc>,
) -> Result<RiskCommandOutput>
where
    Store: crate::risk::RiskStore,
{
    match cmd {
        RiskCommands::Import(import_cmd) => match import_cmd {
            RiskImportCommands::LiveTrades { account, input } => {
                let store = create_live_import_store().await?;
                let contents = std::fs::read_to_string(&input)?;
                let records = parse_live_import_by_path(&input, &contents)?;
                Ok(RiskCommandOutput::ImportSummary(
                    store
                        .import_records(&account, &input, &records, now)
                        .await?,
                ))
            }
        },
        RiskCommands::Rebuild(rebuild_cmd) => match rebuild_cmd {
            RiskRebuildCommands::LiveAccount { account } => {
                let store = create_live_import_store().await?;
                let engine = crate::risk::SqliteLiveMirrorRebuildEngine::new(store);
                Ok(RiskCommandOutput::RebuildSummary(
                    engine.rebuild_account(&account, now).await?,
                ))
            }
        },
        RiskCommands::Rule(rule_cmd) => match rule_cmd {
            RiskRuleCommands::Set { rule_type, value } => Ok(RiskCommandOutput::RuleSet(
                service.set_rule(&rule_type, &value, now).await?,
            )),
            RiskRuleCommands::List => Ok(RiskCommandOutput::RuleList(service.list_rules().await?)),
            RiskRuleCommands::Enable { rule_type } => Ok(RiskCommandOutput::RuleToggled(
                service.enable_rule(&rule_type, now).await?,
            )),
            RiskRuleCommands::Disable { rule_type } => Ok(RiskCommandOutput::RuleToggled(
                service.disable_rule(&rule_type, now).await?,
            )),
        },
        RiskCommands::Log {
            limit,
            date,
            event_type,
        } => Ok(RiskCommandOutput::Log(
            service
                .list_log(
                    Some(limit),
                    parse_risk_log_date(date.as_deref())?,
                    parse_risk_log_type(event_type.as_deref())?,
                )
                .await?,
        )),
        RiskCommands::Lock(lock_cmd) => match lock_cmd {
            RiskLockCommands::Release => Ok(RiskCommandOutput::LockReleased(
                service.release_buy_lock(now).await?,
            )),
        },
        RiskCommands::Status { source, account } => Ok(RiskCommandOutput::Status(
            load_risk_status_for_source(service, source.as_deref(), account.as_deref(), now)
                .await?,
        )),
        RiskCommands::Pnl { source, account } => Ok(RiskCommandOutput::Pnl(
            load_risk_status_for_source(service, source.as_deref(), account.as_deref(), now)
                .await?,
        )),
        RiskCommands::Position { source, account } => Ok(RiskCommandOutput::Position(
            load_risk_status_for_source(service, source.as_deref(), account.as_deref(), now)
                .await?,
        )),
        RiskCommands::Sync(_) => Err(QuantixError::Unsupported(
            "risk sync 需要通过显式同步入口执行".to_string(),
        )),
    }
}

async fn execute_risk_command_with_service_and_sync_at<Store, Source>(
    cmd: RiskCommands,
    service: &RiskService<Store>,
    risk_state_path: &Path,
    sync_source: &Source,
    now: chrono::DateTime<Utc>,
) -> Result<RiskCommandOutput>
where
    Store: crate::risk::RiskStore,
    Source: crate::risk::IndustrySyncSource,
{
    match cmd {
        RiskCommands::Sync(sync_cmd) => match sync_cmd {
            RiskSyncCommands::Industry { standard } => {
                let standard = crate::risk::ClassificationStandard::parse(&standard)?;
                Ok(RiskCommandOutput::IndustrySync(
                    sync_industry_reference_data_at(risk_state_path, standard, sync_source, now)
                        .await?,
                ))
            }
        },
        other => execute_risk_command_with_service_at(other, service, now).await,
    }
}

fn parse_risk_log_date(raw: Option<&str>) -> Result<Option<chrono::NaiveDate>> {
    raw.map(|value| {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| QuantixError::Other(format!("risk log date 无法解析: {value}")))
    })
    .transpose()
}

fn parse_risk_log_type(raw: Option<&str>) -> Result<Option<RiskLogEventType>> {
    raw.map(RiskLogEventType::parse).transpose()
}

fn print_risk_status(status: &RiskStatus) {
    println!("[账户摘要]");
    println!("账户: {}", status.account_id);
    println!("交易日: {}", status.trading_date);
    println!("日初资产: {}", status.starting_total_assets);
    println!("当前资产: {}", status.current_total_assets);
    println!("当日盈亏: {}", status.daily_pnl);
    println!("当日盈亏比: {}%", status.daily_pnl_pct);
    println!();

    println!("[锁状态]");
    println!(
        "买入状态: {}",
        if status.buy_locked { "locked" } else { "open" }
    );
    println!(
        "状态来源: {}",
        match status.lock_state_source {
            RiskLockStateSource::Open => "open",
            RiskLockStateSource::DailyLossLocked => "daily_loss_locked",
            RiskLockStateSource::ManualReleaseActive => "manual_release_active",
        }
    );

    if let Some(trading_date) = status.lock_effective_trading_date {
        println!("作用交易日: {}", trading_date);
    }
    if let Some(reason) = &status.lock_trigger_reason {
        println!("触发原因: {}", reason);
    }
    if let Some(triggered_at) = status.lock_triggered_at {
        println!("触发时间: {}", triggered_at.to_rfc3339());
    }

    for line in build_auto_reduce_lines(status) {
        println!("{line}");
    }

    if !status.position_ratios.is_empty() {
        println!();
        println!("[持仓风险]");
        println!("{:<10} {:<14} 仓位占比", "代码", "市值");
        println!("{}", "-".repeat(42));
        for row in &status.position_ratios {
            println!(
                "{:<10} {:<14} {}%",
                row.code, row.market_value, row.ratio_pct
            );
        }
    }

    if !status.rules.is_empty() {
        println!();
        println!("[规则]");
        println!("{:<20} {:<12} 状态", "规则", "值");
        println!("{}", "-".repeat(48));
        for rule in &status.rules {
            println!(
                "{:<20} {:<12} {}",
                rule.rule_type.as_cli_str(),
                rule.value.display(),
                if rule.enabled { "enabled" } else { "disabled" }
            );
        }
    }
}

fn print_risk_pnl(status: &RiskStatus) {
    for line in build_risk_pnl_lines(status) {
        println!("{line}");
    }
}

fn print_risk_positions(status: &RiskStatus) {
    for line in build_risk_position_lines(status) {
        println!("{line}");
    }
}

pub(super) fn build_risk_pnl_lines(status: &RiskStatus) -> Vec<String> {
    let mut lines = build_risk_summary_lines(status);
    if status.auto_reduce_recommendation.is_some() {
        lines.push(String::new());
        lines.extend(build_auto_reduce_lines(status));
    }
    lines.push(String::new());
    lines.extend(build_risk_lock_lines(status));
    lines
}

pub(super) fn build_risk_position_lines(status: &RiskStatus) -> Vec<String> {
    let mut lines = vec![
        "[账户摘要]".to_string(),
        format!("账户: {}", status.account_id),
        format!("交易日: {}", status.trading_date),
        format!("当前资产: {}", status.current_total_assets),
    ];

    if status.auto_reduce_recommendation.is_some() {
        lines.push(String::new());
        lines.extend(build_auto_reduce_lines(status));
    }

    lines.push(String::new());

    if status.position_ratios.is_empty() {
        lines.push("[持仓风险]".to_string());
        lines.push("📭 暂无持仓风险视图".to_string());
        return lines;
    }

    lines.push("[持仓风险]".to_string());
    lines.push(format!("{:<10} {:<14} {}", "代码", "市值", "仓位占比"));
    lines.push("-".repeat(42));
    lines.extend(status.position_ratios.iter().map(format_position_row));
    lines
}

fn build_risk_summary_lines(status: &RiskStatus) -> Vec<String> {
    vec![
        "[账户摘要]".to_string(),
        format!("账户: {}", status.account_id),
        format!("交易日: {}", status.trading_date),
        format!("日初资产: {}", status.starting_total_assets),
        format!("当前资产: {}", status.current_total_assets),
        format!("当日盈亏: {}", status.daily_pnl),
        format!("当日盈亏比: {}%", status.daily_pnl_pct),
    ]
}

fn build_risk_lock_lines(status: &RiskStatus) -> Vec<String> {
    let mut lines = vec![
        "[锁状态]".to_string(),
        format!(
            "买入状态: {}",
            if status.buy_locked { "locked" } else { "open" }
        ),
        format!(
            "状态来源: {}",
            match status.lock_state_source {
                RiskLockStateSource::Open => "open",
                RiskLockStateSource::DailyLossLocked => "daily_loss_locked",
                RiskLockStateSource::ManualReleaseActive => "manual_release_active",
            }
        ),
    ];

    if let Some(trading_date) = status.lock_effective_trading_date {
        lines.push(format!("作用交易日: {}", trading_date));
    }
    if let Some(reason) = &status.lock_trigger_reason {
        lines.push(format!("触发原因: {}", reason));
    }
    if let Some(triggered_at) = status.lock_triggered_at {
        lines.push(format!("触发时间: {}", triggered_at.to_rfc3339()));
    }

    lines
}

fn build_auto_reduce_lines(status: &RiskStatus) -> Vec<String> {
    let Some(recommendation) = &status.auto_reduce_recommendation else {
        return Vec::new();
    };

    let targets = if recommendation.position_codes.is_empty() {
        "无可减仓持仓".to_string()
    } else {
        recommendation.position_codes.join(",")
    };

    vec![
        String::new(),
        "[自动减仓建议]".to_string(),
        format!("当前亏损比: {}%", recommendation.current_loss_pct),
        format!(
            "建议动作: 当前仅提供人工执行建议，不会自动卖出；如需处理，可按 {}% 减仓以下持仓 {}",
            recommendation.reduce_ratio, targets
        ),
        format!("触发时间: {}", recommendation.triggered_at.to_rfc3339()),
    ]
}

fn format_position_row(row: &PositionRiskRow) -> String {
    format!(
        "{:<10} {:<14} {}%",
        row.code, row.market_value, row.ratio_pct
    )
}

async fn create_live_import_store() -> Result<crate::risk::SqliteLiveImportStore> {
    let runtime = CliRuntime::load();
    let live_import_path = runtime.risk_path.with_file_name("live_import.db");
    crate::risk::SqliteLiveImportStore::new(live_import_path).await
}

async fn load_risk_status_for_source<Store>(
    service: &RiskService<Store>,
    source: Option<&str>,
    account: Option<&str>,
    now: DateTime<Utc>,
) -> Result<RiskStatus>
where
    Store: crate::risk::RiskStore,
{
    match parse_risk_source(source)? {
        crate::risk::RiskAccountSource::Paper => {
            service
                .status(&load_paper_risk_account_snapshot().await?, now)
                .await
        }
        crate::risk::RiskAccountSource::LiveImport => {
            let account = account.ok_or_else(|| {
                QuantixError::Other("risk --source live_import 需要显式指定 --account".to_string())
            })?;
            let store = create_live_import_store().await?;
            let mirror = store
                .get_latest_mirror_account(account)
                .await?
                .ok_or_else(|| {
                    QuantixError::Other(format!("live_import mirror 不存在: {account}"))
                })?;
            let snapshot = build_risk_account_snapshot_from_live_import(&mirror);
            service.status(&snapshot, now).await
        }
    }
}

async fn load_paper_risk_account_snapshot() -> Result<RiskAccountSnapshot> {
    let runtime = CliRuntime::load();
    let trade_store = JsonPaperTradeStore::new(runtime.trade_path);
    let account = trade_store
        .load_state()
        .await?
        .and_then(|state| state.account)
        .ok_or_else(|| {
            QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
        })?;

    Ok(build_risk_account_snapshot(&account))
}

fn build_risk_account_snapshot_from_live_import(
    mirror: &crate::risk::LiveImportMirrorAccount,
) -> RiskAccountSnapshot {
    let positions: Vec<(String, rust_decimal::Decimal)> = mirror
        .positions
        .iter()
        .map(|position| {
            (
                position.code.clone(),
                rust_decimal::Decimal::from(position.volume) * position.avg_cost,
            )
        })
        .collect();
    RiskAccountSnapshot::new(
        mirror.account_id.clone(),
        mirror.current_total_assets,
        positions,
    )
}

fn parse_risk_source(raw: Option<&str>) -> Result<crate::risk::RiskAccountSource> {
    match raw {
        None => Ok(crate::risk::RiskAccountSource::Paper),
        Some(value) => crate::risk::RiskAccountSource::from_str(value)
            .ok_or_else(|| QuantixError::Unsupported(format!("risk --source 不支持的值: {value}"))),
    }
}

fn parse_live_import_by_path(
    path: &str,
    contents: &str,
) -> Result<Vec<crate::risk::LiveImportRecord>> {
    match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("csv") => crate::risk::parse_live_import_csv(contents),
        Some("json") => crate::risk::parse_live_import_json(contents),
        other => Err(QuantixError::Unsupported(format!(
            "risk import 暂不支持的文件扩展: {}",
            other.unwrap_or("<none>")
        ))),
    }
}

fn build_risk_account_snapshot(account: &PaperTradeAccount) -> RiskAccountSnapshot {
    let positions: Vec<(String, rust_decimal::Decimal)> = account
        .positions
        .values()
        .map(|position| {
            (
                position.code.clone(),
                rust_decimal::Decimal::from(position.volume) * position.last_trade_price,
            )
        })
        .collect();
    let position_value = positions
        .iter()
        .fold(rust_decimal::Decimal::ZERO, |acc, (_, value)| acc + *value);

    RiskAccountSnapshot::new(
        account.account_id.clone(),
        account.available_cash + position_value,
        positions,
    )
}

#[allow(dead_code)]
fn build_position_rows(
    total_assets: rust_decimal::Decimal,
    positions: &[(String, rust_decimal::Decimal)],
) -> Vec<PositionRiskRow> {
    positions
        .iter()
        .map(|(code, market_value)| PositionRiskRow {
            code: code.clone(),
            market_value: *market_value,
            ratio_pct: if total_assets.is_zero() {
                rust_decimal::Decimal::ZERO
            } else {
                *market_value / total_assets * rust_decimal::Decimal::from(100)
            },
        })
        .collect()
}

#[cfg(test)]
mod tests;
