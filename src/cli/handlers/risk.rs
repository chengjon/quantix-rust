use super::*;
use crate::cli::{RiskImportCommands, RiskRebuildCommands};
use crate::risk::{RiskRuleType, RuleValue};
use std::path::Path;

pub async fn run_risk_command(cmd: RiskCommands) -> Result<()> {
    let service = RiskService::new(create_risk_store());
    let output = execute_risk_command_with_service_at(cmd, &service, Utc::now()).await?;
    print_risk_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RiskCommandOutput {
    RuleSet(RiskRule),
    RuleList(Vec<RiskRule>),
    RuleToggled(RiskRule),
    ImportSummary(crate::risk::LiveImportBatchSummary),
    RebuildSummary(crate::risk::LiveImportMirrorAccount),
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
                    store.import_records(&account, &input, &records, now).await?,
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
    }
}

fn print_risk_command_output(output: &RiskCommandOutput) {
    match output {
        RiskCommandOutput::RuleSet(rule) => {
            println!(
                "✅ 已设置风控规则 {} = {}",
                rule.rule_type.as_cli_str(),
                rule.value.display()
            );
        }
        RiskCommandOutput::RuleList(rules) => print_risk_rules(rules),
        RiskCommandOutput::RuleToggled(rule) => {
            let status = if rule.enabled { "启用" } else { "禁用" };
            println!("✅ 已{}风控规则 {}", status, rule.rule_type.as_cli_str());
        }
        RiskCommandOutput::ImportSummary(summary) => {
            println!(
                "✅ 已导入 {}: total={} inserted={} skipped={} conflicts={}",
                summary.account_id,
                summary.total_rows,
                summary.inserted,
                summary.skipped_duplicates,
                summary.conflicts
            );
        }
        RiskCommandOutput::RebuildSummary(summary) => {
            println!(
                "✅ 已重建 {} as_of={} cash={} positions={} realized_pnl={}",
                summary.account_id,
                summary.as_of.to_rfc3339(),
                summary.cash_balance,
                summary.positions.len(),
                summary.realized_pnl
            );
        }
        RiskCommandOutput::Log(events) => print_risk_log(events),
        RiskCommandOutput::LockReleased(lock_state) => {
            if let Some(trading_date) = lock_state.released_for_date {
                println!("✅ 已释放买入锁，{} 当日内不再自动重新锁定", trading_date);
            } else {
                println!("✅ 已释放买入锁");
            }
        }
        RiskCommandOutput::Status(status) => print_risk_status(status),
        RiskCommandOutput::Pnl(status) => print_risk_pnl(status),
        RiskCommandOutput::Position(status) => print_risk_positions(status),
    }
}

fn print_risk_rules(rules: &[RiskRule]) {
    if rules.is_empty() {
        println!("📭 暂无风控规则");
        return;
    }

    println!("{:<20} {:<12} {}", "规则", "值", "状态");
    println!("{}", "-".repeat(48));

    for rule in rules {
        println!(
            "{:<20} {:<12} {}",
            rule.rule_type.as_cli_str(),
            rule.value.display(),
            if rule.enabled { "enabled" } else { "disabled" }
        );
    }
}

fn print_risk_log(events: &[RiskLogEvent]) {
    for line in build_risk_log_lines(events) {
        println!("{line}");
    }
}

pub(super) fn build_risk_log_lines(events: &[RiskLogEvent]) -> Vec<String> {
    if events.is_empty() {
        return vec!["🕘 暂无风控事件日志".to_string()];
    }

    let mut lines = Vec::new();
    let mut index = 0;

    while index < events.len() {
        let event_date = events[index].ts.date_naive();
        let group_end = events[index..]
            .iter()
            .position(|event| event.ts.date_naive() != event_date)
            .map(|offset| index + offset)
            .unwrap_or(events.len());
        let group = &events[index..group_end];

        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("[{event_date}] · {} 条", group.len()));
        lines.push(build_risk_log_group_summary(group));
        lines.push(format!(
            "{:<10} {:<18} {:<12} {}",
            "时间", "事件", "交易日", "说明"
        ));
        lines.push("-".repeat(76));

        lines.extend(group.iter().map(|event| {
            format!(
                "{:<10} {:<18} {:<12} {}",
                event.ts.format("%H:%M:%S"),
                event.event_type.display_label(),
                event
                    .trading_date
                    .map(|date| date.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                display_risk_log_detail(event)
            )
        }));

        index = group_end;
    }

    lines
}

fn build_risk_log_group_summary(events: &[RiskLogEvent]) -> String {
    let mut rule_changes = 0;
    let mut lock_triggered = 0;
    let mut lock_released = 0;
    let mut lock_cleared = 0;
    let mut industry_triggered = 0;
    let mut auto_reduce = 0;

    for event in events {
        match event.event_type {
            RiskLogEventType::RuleSet
            | RiskLogEventType::RuleEnabled
            | RiskLogEventType::RuleDisabled => rule_changes += 1,
            RiskLogEventType::DailyLossLockTriggered => lock_triggered += 1,
            RiskLogEventType::BuyLockReleased => lock_released += 1,
            RiskLogEventType::BuyLockCleared => lock_cleared += 1,
            RiskLogEventType::IndustryLimitTriggered => industry_triggered += 1,
            RiskLogEventType::AutoReduceTriggered | RiskLogEventType::AutoReduceExecuted => {
                auto_reduce += 1
            }
        }
    }

    format!(
        "摘要: 规则变更 {} / 锁触发 {} / 手动释放 {} / 锁清除 {} / 行业超限 {} / 自动减仓 {}",
        rule_changes, lock_triggered, lock_released, lock_cleared, industry_triggered, auto_reduce
    )
}

fn display_risk_log_detail(event: &RiskLogEvent) -> String {
    match event.event_type {
        RiskLogEventType::RuleSet => {
            if let Some((left, right)) = event.detail.split_once(" = ") {
                format!("{left}={right}")
            } else {
                event.detail.clone()
            }
        }
        RiskLogEventType::RuleEnabled => format!("启用 {}", event.detail),
        RiskLogEventType::RuleDisabled => format!("禁用 {}", event.detail),
        RiskLogEventType::DailyLossLockTriggered => {
            if let Some(detail) = event.detail.strip_suffix(" 已触发") {
                format!("阈值触发: {detail}")
            } else {
                event.detail.clone()
            }
        }
        RiskLogEventType::BuyLockReleased => {
            if let Some(detail) = event.detail.strip_suffix(" 已触发") {
                format!("手动释放: {detail}")
            } else {
                format!("手动释放: {}", event.detail)
            }
        }
        RiskLogEventType::BuyLockCleared => match event.detail.as_str() {
            "day rollover" => "跨日清除".to_string(),
            "trade init/reset" => "账户重置清除".to_string(),
            _ => event.detail.clone(),
        },
        RiskLogEventType::IndustryLimitTriggered => {
            format!("行业超限: {}", event.detail)
        }
        RiskLogEventType::AutoReduceTriggered => {
            format!("减仓触发: {}", event.detail)
        }
        RiskLogEventType::AutoReduceExecuted => {
            format!("减仓执行: {}", event.detail)
        }
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

    if !status.position_ratios.is_empty() {
        println!();
        println!("[持仓风险]");
        println!("{:<10} {:<14} {}", "代码", "市值", "仓位占比");
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
        println!("{:<20} {:<12} {}", "规则", "值", "状态");
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
        String::new(),
    ];

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

fn format_position_row(row: &PositionRiskRow) -> String {
    format!(
        "{:<10} {:<14} {}%",
        row.code, row.market_value, row.ratio_pct
    )
}

pub(super) fn create_risk_store() -> JsonRiskStore {
    let runtime = CliRuntime::load();
    JsonRiskStore::new(runtime.risk_path)
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
            service.status(&load_paper_risk_account_snapshot().await?, now).await
        }
        crate::risk::RiskAccountSource::LiveImport => {
            let account = account.ok_or_else(|| {
                QuantixError::Other("risk --source live_import 需要显式指定 --account".to_string())
            })?;
            let store = create_live_import_store().await?;
            let mirror = store
                .get_latest_mirror_account(account)
                .await?
                .ok_or_else(|| QuantixError::Other(format!("live_import mirror 不存在: {account}")))?;
            let rules = service.list_rules().await?;
            Ok(build_risk_status_from_live_import(&mirror, &rules))
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
) -> RiskStatus {
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
    let buy_locked = daily_loss_limit
        .map(|rule| match rule.value {
            RuleValue::Amount(limit) => daily_pnl <= -limit,
            RuleValue::Percentage(limit_pct) => daily_pnl_pct <= -limit_pct,
        })
        .unwrap_or(false);
    let lock_reason = if buy_locked {
        daily_loss_limit
            .map(|rule| format!("daily-loss-limit {} 已触发", rule.value.display()))
    } else {
        None
    };

    RiskStatus {
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
    }
}

fn parse_risk_source(raw: Option<&str>) -> Result<crate::risk::RiskAccountSource> {
    match raw {
        None => Ok(crate::risk::RiskAccountSource::Paper),
        Some(value) => crate::risk::RiskAccountSource::from_str(value).ok_or_else(|| {
            QuantixError::Other(format!("risk --source 不支持的值: {value}"))
        }),
    }
}

fn parse_live_import_by_path(path: &str, contents: &str) -> Result<Vec<crate::risk::LiveImportRecord>> {
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
