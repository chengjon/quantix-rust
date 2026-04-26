use super::*;
use crate::cli::{RiskImportCommands, RiskRebuildCommands, RiskSyncCommands};
use crate::risk::service::check_auto_reduce_trigger;
use crate::risk::{
    DailyRiskBaseline, IndustrySyncSummary, MySqlIndustrySyncSource, RiskRuleType, RiskState,
    RuleValue, sync_industry_reference_data_at,
};
use std::path::Path;

mod output;

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
            let rules = service.list_rules().await?;
            Ok(build_risk_status_from_live_import(&mirror, &rules)?)
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

fn build_risk_status_from_live_import(
    mirror: &crate::risk::LiveImportMirrorAccount,
    rules: &[RiskRule],
) -> Result<RiskStatus> {
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
    let daily_pnl = mirror.current_total_assets - mirror.starting_total_assets;
    let daily_pnl_pct = if mirror.starting_total_assets.is_zero() {
        rust_decimal::Decimal::ZERO
    } else {
        daily_pnl / mirror.starting_total_assets * rust_decimal::Decimal::from(100)
    };

    let daily_loss_limit = rules
        .iter()
        .find(|rule| rule.enabled && rule.rule_type == RiskRuleType::DailyLossLimit);
    let buy_locked = match daily_loss_limit {
        Some(rule) => evaluate_daily_loss_rule_triggered(rule, daily_pnl, daily_pnl_pct)?,
        None => false,
    };
    let lock_reason = if buy_locked {
        daily_loss_limit.map(|rule| format!("daily-loss-limit {} 已触发", rule.value.display()))
    } else {
        None
    };
    let snapshot = RiskAccountSnapshot::new(
        mirror.account_id.clone(),
        mirror.current_total_assets,
        positions.clone(),
    );
    let auto_reduce_recommendation = check_auto_reduce_trigger(
        &RiskState {
            account_id: mirror.account_id.clone(),
            daily_baseline: Some(DailyRiskBaseline {
                trading_date: mirror.trading_date,
                starting_total_assets: mirror.starting_total_assets,
            }),
            rules: rules.to_vec(),
            ..RiskState::default()
        },
        &snapshot,
        mirror.last_rebuild_at,
    )
    .map(|decision| {
        let mut position_codes = decision
            .positions_to_reduce
            .iter()
            .map(|position| position.code.clone())
            .collect::<Vec<_>>();
        position_codes.sort();
        position_codes.dedup();

        crate::risk::AutoReduceRecommendation {
            current_loss_pct: decision.current_loss_pct,
            reduce_ratio: decision.reduce_ratio,
            position_codes,
            triggered_at: decision.triggered_at,
        }
    });

    Ok(RiskStatus {
        account_id: mirror.account_id.clone(),
        trading_date: mirror.trading_date,
        starting_total_assets: mirror.starting_total_assets,
        current_total_assets: mirror.current_total_assets,
        daily_pnl,
        daily_pnl_pct,
        buy_locked,
        manual_release_active: false,
        lock_state_source: if buy_locked {
            RiskLockStateSource::DailyLossLocked
        } else {
            RiskLockStateSource::Open
        },
        lock_reason: lock_reason.clone(),
        lock_trigger_reason: lock_reason,
        lock_triggered_at: if buy_locked {
            Some(mirror.last_rebuild_at)
        } else {
            None
        },
        lock_effective_trading_date: if buy_locked {
            Some(mirror.trading_date)
        } else {
            None
        },
        position_ratios: build_position_rows(mirror.current_total_assets, &positions),
        rules: rules
            .iter()
            .map(|rule| crate::risk::RiskRuleSnapshot {
                rule_type: rule.rule_type,
                value: rule.value.clone(),
                enabled: rule.enabled,
            })
            .collect(),
        auto_reduce_recommendation,
    })
}

fn evaluate_daily_loss_rule_triggered(
    rule: &RiskRule,
    daily_pnl: rust_decimal::Decimal,
    daily_pnl_pct: rust_decimal::Decimal,
) -> Result<bool> {
    match &rule.value {
        RuleValue::Amount(limit) => Ok(daily_pnl <= -*limit),
        RuleValue::Percentage(limit_pct) => Ok(daily_pnl_pct <= -*limit_pct),
        RuleValue::TextList(_) => Err(QuantixError::Other(
            "risk rule daily-loss-limit 配置无效".to_string(),
        )),
    }
}

fn parse_risk_source(raw: Option<&str>) -> Result<crate::risk::RiskAccountSource> {
    match raw {
        None => Ok(crate::risk::RiskAccountSource::Paper),
        Some(value) => crate::risk::RiskAccountSource::from_str(value)
            .ok_or_else(|| QuantixError::Other(format!("risk --source 不支持的值: {value}"))),
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
        other => Err(QuantixError::Other(format!(
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
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;
    use tempfile::tempdir;

    use crate::risk::{
        ClassificationStandard, IndustryClassificationLevel, IndustrySyncSource, JsonRiskStore,
        ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow, SqliteIndustryStore,
    };

    fn fixed_ts() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 25, 10, 0, 0).unwrap()
    }

    #[derive(Debug)]
    struct FakeIndustrySyncSource {
        current_rows: Vec<ShenwanCurrentSeedRow>,
        historical_rows: Vec<ShenwanHistoricalSeedRow>,
    }

    #[async_trait]
    impl IndustrySyncSource for FakeIndustrySyncSource {
        async fn fetch_shenwan_current_rows(&self) -> Result<Vec<ShenwanCurrentSeedRow>> {
            Ok(self.current_rows.clone())
        }

        async fn fetch_shenwan_history_rows(&self) -> Result<Vec<ShenwanHistoricalSeedRow>> {
            Ok(self.historical_rows.clone())
        }
    }

    #[tokio::test]
    async fn execute_risk_sync_industry_command_returns_summary_and_persists_rows() {
        let dir = tempdir().unwrap();
        let risk_state_path = dir.path().join("risk").join("risk_state.json");
        let service = RiskService::new(JsonRiskStore::new(&risk_state_path));
        let source = FakeIndustrySyncSource {
            current_rows: vec![ShenwanCurrentSeedRow {
                security_code: "000001.SZ".to_string(),
                industry_name: "银行".to_string(),
                source: "fake_current_sync".to_string(),
            }],
            historical_rows: vec![ShenwanHistoricalSeedRow {
                security_code: "000001".to_string(),
                industry_name: "银行".to_string(),
                effective_from: chrono::NaiveDate::from_ymd_opt(2014, 1, 1).unwrap(),
                effective_to: None,
                source: "fake_history_sync".to_string(),
            }],
        };

        let output = execute_risk_command_with_service_and_sync_at(
            RiskCommands::Sync(RiskSyncCommands::Industry {
                standard: "shenwan".to_string(),
            }),
            &service,
            &risk_state_path,
            &source,
            fixed_ts(),
        )
        .await
        .unwrap();

        match output {
            RiskCommandOutput::IndustrySync(summary) => {
                assert_eq!(summary.standard, ClassificationStandard::Shenwan);
                assert_eq!(summary.current_rows, 1);
                assert_eq!(summary.history_rows, 1);
            }
            other => panic!("unexpected output: {other:?}"),
        }

        let store = SqliteIndustryStore::from_risk_state_path(&risk_state_path)
            .await
            .unwrap();
        let current = store
            .lookup_current(
                ClassificationStandard::Shenwan,
                IndustryClassificationLevel::FirstLevel,
                "000001",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(current.industry_name, "银行");
    }

    #[tokio::test]
    async fn execute_risk_rule_set_industry_limit_command_returns_percentage_rule() {
        let dir = tempdir().unwrap();
        let risk_state_path = dir.path().join("risk").join("risk_state.json");
        let service = RiskService::new(JsonRiskStore::new(&risk_state_path));

        let output = execute_risk_command_with_service_at(
            RiskCommands::Rule(RiskRuleCommands::Set {
                rule_type: "industry-limit".to_string(),
                value: "30%".to_string(),
            }),
            &service,
            fixed_ts(),
        )
        .await
        .unwrap();

        match output {
            RiskCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.rule_type, RiskRuleType::IndustryLimit);
                assert_eq!(rule.value, RuleValue::Percentage(dec!(30)));
                assert!(rule.enabled);
            }
            other => panic!("unexpected output: {other:?}"),
        }
    }

    #[test]
    fn build_risk_pnl_lines_surfaces_auto_reduce_as_manual_recommendation() {
        let status = RiskStatus {
            account_id: "paper".to_string(),
            trading_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 25).unwrap(),
            starting_total_assets: dec!(1000000),
            current_total_assets: dec!(930000),
            daily_pnl: dec!(-70000),
            daily_pnl_pct: dec!(-7),
            buy_locked: false,
            manual_release_active: false,
            lock_state_source: RiskLockStateSource::Open,
            lock_reason: None,
            lock_trigger_reason: None,
            lock_triggered_at: None,
            lock_effective_trading_date: None,
            position_ratios: vec![PositionRiskRow {
                code: "000001".to_string(),
                market_value: dec!(120000),
                ratio_pct: dec!(12.9),
            }],
            rules: vec![],
            auto_reduce_recommendation: Some(crate::risk::AutoReduceRecommendation {
                current_loss_pct: dec!(-7),
                reduce_ratio: dec!(50),
                position_codes: vec!["000001".to_string()],
                triggered_at: fixed_ts(),
            }),
        };

        let lines = build_risk_pnl_lines(&status).join("\n");
        assert!(lines.contains("[自动减仓建议]"));
        assert!(lines.contains("不会自动卖出"));
        assert!(lines.contains("000001"));
    }
}
