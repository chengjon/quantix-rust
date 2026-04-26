use super::*;

pub(super) fn print_stop_command_output(output: &StopCommandOutput) {
    match output {
        StopCommandOutput::RuleSet(rule) => {
            println!("✅ 已设置 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleUpdated(rule) => {
            println!("✅ 已更新 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleList(rules) => print_stop_rules(rules),
        StopCommandOutput::StatusRows(rows) => print_stop_status_rows(rows),
        StopCommandOutput::HistoryRows(rows) => print_stop_history_rows(rows),
        StopCommandOutput::RuleRemoved { code, removed } => {
            if *removed {
                println!("✅ 已移除 {} 的止盈止损规则", code);
            } else {
                println!("⚠️  未找到 {} 的止盈止损规则", code);
            }
        }
    }
}

pub(super) fn print_stop_status_rows(rows: &[StopStatusRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的止盈止损状态");
        return;
    }

    for row in rows {
        println!(
            "{} last_price={:?} anchor_price={:?} anchor_source={} loss_threshold={:?} profit_threshold={:?} trailing_pct={:?} highest_price={:?} eval_state={}",
            row.code,
            row.last_price,
            row.anchor_price,
            row.anchor_source
                .map(|source| source.as_str())
                .unwrap_or("-"),
            row.loss_threshold,
            row.profit_threshold,
            row.trailing_pct,
            row.highest_price,
            format_stop_eval_state(row.eval_state),
        );
    }
}

pub(super) fn print_stop_history_rows(rows: &[StopHistoryEvent]) {
    if rows.is_empty() {
        println!("📭 没有可展示的止盈止损历史");
        return;
    }

    for row in rows {
        println!(
            "{} type={} trigger={:?} price={:?} anchor_price={:?} anchor_source={} ts={}",
            row.code,
            row.event_type.as_str(),
            row.trigger_kind.map(|kind| kind.as_str()),
            row.trigger_price,
            row.anchor_price,
            row.anchor_source.as_deref().unwrap_or("-"),
            row.created_at.to_rfc3339(),
        );
    }
}

pub(super) fn print_stop_rules(rules: &[StopRule]) {
    if rules.is_empty() {
        println!("📭 暂无止盈止损规则");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<10} {:<12} 最近触发",
        "代码", "止损价", "止盈价", "追踪%", "最高价"
    );
    println!("{}", "-".repeat(80));

    for rule in rules {
        println!(
            "{:<10} {:<12} {:<12} {:<10} {:<12} {}",
            rule.code,
            format_optional_price(rule.stop_loss_price),
            format_optional_price(rule.take_profit_price),
            format_optional_price(rule.trailing_pct),
            format_optional_price(rule.highest_price),
            format_optional_timestamp(rule.last_triggered_at),
        );
    }
}

pub(super) fn format_optional_price(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}", value))
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn format_optional_timestamp(value: Option<chrono::DateTime<Utc>>) -> String {
    value
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn format_triggered_stop_message(triggered_stop: &TriggeredStop) -> String {
    match triggered_stop.kind {
        StopTriggerKind::Loss => format!(
            "{} 当前价 {:.2} 触发 stop-loss {:.2}",
            triggered_stop.code, triggered_stop.current_price, triggered_stop.threshold_price
        ),
        StopTriggerKind::Profit => format!(
            "{} 当前价 {:.2} 触发 take-profit {:.2}",
            triggered_stop.code, triggered_stop.current_price, triggered_stop.threshold_price
        ),
        StopTriggerKind::TrailingLoss => {
            let trailing_pct = triggered_stop
                .highest_price
                .map(|highest| (1.0 - triggered_stop.threshold_price / highest) * 100.0)
                .unwrap_or_default();
            match triggered_stop.highest_price {
                Some(highest_price) => format!(
                    "{} 当前价 {:.2} 触发 trailing-stop {:.2}% (highest {:.2})",
                    triggered_stop.code, triggered_stop.current_price, trailing_pct, highest_price
                ),
                None => format!(
                    "{} 当前价 {:.2} 触发 trailing-stop {:.2}",
                    triggered_stop.code,
                    triggered_stop.current_price,
                    triggered_stop.threshold_price
                ),
            }
        }
    }
}
