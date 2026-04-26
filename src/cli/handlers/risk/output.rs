use super::*;

pub(super) fn print_risk_command_output(output: &RiskCommandOutput) {
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
        RiskCommandOutput::IndustrySync(summary) => {
            println!(
                "✅ 已同步 {} 行业引用表: current={} history={} sqlite={} at={}",
                summary.standard.as_str(),
                summary.current_rows,
                summary.history_rows,
                summary.store_path.display(),
                summary.synced_at.to_rfc3339()
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

    println!("{:<20} {:<12} 状态", "规则", "值");
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
