use super::*;

pub(super) fn print_trade_command_output(output: &TradeCommandOutput) {
    match output {
        TradeCommandOutput::AccountInitialized(account) => {
            println!("✅ 已初始化模拟账户 {}", account.account_id);
            print_trade_account_summary(account);
        }
        TradeCommandOutput::AccountReset(account) => {
            println!("✅ 已重置模拟账户 {}", account.account_id);
            print_trade_account_summary(account);
        }
        TradeCommandOutput::TradeExecuted(record) => print_trade_record(record),
        TradeCommandOutput::HistoryRows(rows) => print_trade_history_rows(rows),
        TradeCommandOutput::FeeRows(rows) => print_trade_fee_rows(rows),
        TradeCommandOutput::Overview(overview) => print_trade_overview(overview),
        TradeCommandOutput::PositionList(positions) => print_trade_positions(positions),
        TradeCommandOutput::PositionCurrentList(rows) => print_trade_current_positions(rows),
        TradeCommandOutput::Cash(snapshot) => print_trade_cash(snapshot),
    }
}

pub(super) fn print_trade_account_summary(account: &PaperTradeAccount) {
    println!("初始资金: {}", account.initial_capital);
    println!("可用资金: {}", account.available_cash);
    println!("佣金费率: {}", account.fee_config.commission_rate);
    println!("最低佣金: {}", account.fee_config.commission_min);
    println!("印花税率: {}", account.fee_config.stamp_duty_rate);
    println!("过户费率: {}", account.fee_config.transfer_fee_rate);
}

pub(super) fn print_trade_record(record: &TradeRecord) {
    println!(
        "✅ 已{} {} {} 股 @ {}",
        format_trade_side(record),
        record.code,
        record.volume,
        record.price
    );
    println!("成交额: {}", record.amount);
    println!("总费用: {}", record.total_fee);
}

pub(super) fn format_trade_side(record: &TradeRecord) -> &'static str {
    match record.side {
        crate::trade::TradeSide::Buy => "买入",
        crate::trade::TradeSide::Sell => "卖出",
    }
}

pub(super) fn format_trade_side_label(side: crate::trade::TradeSide) -> &'static str {
    match side {
        crate::trade::TradeSide::Buy => "买入",
        crate::trade::TradeSide::Sell => "卖出",
    }
}

pub(super) fn print_trade_positions(positions: &[TradePosition]) {
    if positions.is_empty() {
        println!("📭 暂无持仓");
        return;
    }

    println!(
        "{:<10} {:<10} {:<14} 最新成交价",
        "代码", "数量", "持仓成本"
    );
    println!("{}", "-".repeat(56));

    for position in positions {
        println!(
            "{:<10} {:<10} {:<14} {}",
            position.code, position.volume, position.avg_cost, position.last_trade_price
        );
    }
}

pub(super) fn print_trade_cash(snapshot: &CashSnapshot) {
    println!("初始资金: {}", snapshot.initial_capital);
    println!("可用现金: {}", snapshot.available_cash);
    println!("持仓估值: {}", snapshot.estimated_position_value);
    println!("总资产估算: {}", snapshot.estimated_total_assets);
}

pub(super) fn print_trade_history_rows(rows: &[TradeHistoryRow]) {
    if rows.is_empty() {
        println!("📭 暂无成交历史");
        return;
    }

    println!(
        "{:<20} {:<10} {:<6} {:<10} {:<8} {:<12} {:<10} 净现金影响",
        "时间", "代码", "方向", "价格", "数量", "成交额", "费用"
    );
    println!("{}", "-".repeat(100));

    for row in rows {
        println!(
            "{:<20} {:<10} {:<6} {:<10} {:<8} {:<12} {:<10} {}",
            row.executed_at.format("%Y-%m-%d %H:%M:%S"),
            row.code,
            format_trade_side_label(row.side),
            row.price,
            row.volume,
            row.amount,
            row.total_fee,
            row.net_cash_impact
        );
    }
}

pub(super) fn print_trade_fee_rows(rows: &[TradeFeeRow]) {
    if rows.is_empty() {
        println!("📭 暂无费用明细");
        return;
    }

    println!(
        "{:<20} {:<10} {:<6} {:<10} {:<10} {:<10} 总费用",
        "时间", "代码", "方向", "佣金", "印花税", "过户费"
    );
    println!("{}", "-".repeat(90));

    for row in rows {
        println!(
            "{:<20} {:<10} {:<6} {:<10} {:<10} {:<10} {}",
            row.executed_at.format("%Y-%m-%d %H:%M:%S"),
            row.code,
            format_trade_side_label(row.side),
            row.commission,
            row.stamp_duty,
            row.transfer_fee,
            row.total_fee
        );
    }
}

pub(super) fn print_trade_overview(overview: &TradeOverview) {
    println!("初始资金: {}", overview.initial_capital);
    println!("可用现金: {}", overview.available_cash);
    println!("账面持仓估值: {}", overview.booked_position_value);
    println!("账面总资产: {}", overview.booked_total_assets);
    println!("成交笔数: {}", overview.trade_count);
    println!("持仓数: {}", overview.holding_count);
    println!("累计买入额: {}", overview.total_buy_amount);
    println!("累计卖出额: {}", overview.total_sell_amount);
    println!("累计费用: {}", overview.total_fee);

    if let Some((resolved, total)) = overview.quote_coverage {
        println!("实时价格覆盖: {resolved}/{total}");
    }
    if let Some(value) = overview.live_position_value {
        println!("实时持仓估值: {}", value);
    }
    if let Some(value) = overview.live_total_assets {
        println!("实时总资产: {}", value);
    }
}

pub(super) fn print_trade_current_positions(rows: &[TradePositionCurrentRow]) {
    if rows.is_empty() {
        println!("📭 暂无持仓");
        return;
    }

    println!(
        "{:<10} {:<10} {:<14} {:<12} {:<12} {:<12} {:<12} 价格状态",
        "代码", "数量", "持仓成本", "最新成交价", "当前价", "当前市值", "浮盈亏"
    );
    println!("{}", "-".repeat(112));

    for row in rows {
        println!(
            "{:<10} {:<10} {:<14} {:<12} {:<12} {:<12} {:<12} {}",
            row.code,
            row.volume,
            row.avg_cost,
            row.last_trade_price,
            row.current_price
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.current_market_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.unrealized_pnl
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            format_trade_quote_status(row.quote_status)
        );
    }
}

pub(super) fn format_trade_quote_status(status: TradeQuoteStatus) -> &'static str {
    match status {
        TradeQuoteStatus::BookOnly => "book",
        TradeQuoteStatus::Live => "live",
        TradeQuoteStatus::Missing => "missing",
    }
}
