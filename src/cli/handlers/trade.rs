use super::*;

pub async fn run_trade_command(cmd: TradeCommands) -> Result<()> {
    let service = TradeService::new(create_trade_store());
    let output = execute_trade_command_with_service(cmd, &service).await?;
    print_trade_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TradeCommandOutput {
    AccountInitialized(PaperTradeAccount),
    AccountReset(PaperTradeAccount),
    TradeExecuted(TradeRecord),
    PositionList(Vec<TradePosition>),
    Cash(CashSnapshot),
}

pub(super) async fn execute_trade_command_with_service<Store>(
    cmd: TradeCommands,
    service: &TradeService<Store>,
) -> Result<TradeCommandOutput>
where
    Store: PaperTradeStore,
{
    match cmd {
        TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade init",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountInitialized(
                service.init_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Reset {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade reset",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountReset(
                service.reset_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Buy {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade buy", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.buy(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Sell {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade sell", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.sell(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Position => Ok(TradeCommandOutput::PositionList(service.positions().await?)),
        TradeCommands::Cash => Ok(TradeCommandOutput::Cash(service.cash_snapshot().await?)),
    }
}

fn build_trade_init_request(
    command_name: &str,
    capital: Option<f64>,
    commission_rate: Option<f64>,
    commission_min: Option<f64>,
    stamp_duty_rate: Option<f64>,
    transfer_fee_rate: Option<f64>,
) -> Result<InitAccountRequest> {
    InitAccountRequest::new(
        capital,
        commission_rate,
        commission_min,
        stamp_duty_rate,
        transfer_fee_rate,
    )
    .map_err(|err| remap_trade_request_error(err, command_name))
}

fn build_trade_order_request(
    command_name: &str,
    code: String,
    price: f64,
    volume: i64,
) -> Result<TradeOrderRequest> {
    TradeOrderRequest::new(code, price, volume)
        .map_err(|err| remap_trade_request_error(err, command_name))
}

fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(
            message
                .replace("trade init", command_name)
                .replace("trade order", command_name),
        ),
        other => other,
    }
}

fn print_trade_command_output(output: &TradeCommandOutput) {
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
        TradeCommandOutput::PositionList(positions) => print_trade_positions(positions),
        TradeCommandOutput::Cash(snapshot) => print_trade_cash(snapshot),
    }
}

fn print_trade_account_summary(account: &PaperTradeAccount) {
    println!("初始资金: {}", account.initial_capital);
    println!("可用资金: {}", account.available_cash);
    println!("佣金费率: {}", account.fee_config.commission_rate);
    println!("最低佣金: {}", account.fee_config.commission_min);
    println!("印花税率: {}", account.fee_config.stamp_duty_rate);
    println!("过户费率: {}", account.fee_config.transfer_fee_rate);
}

fn print_trade_record(record: &TradeRecord) {
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

fn format_trade_side(record: &TradeRecord) -> &'static str {
    match record.side {
        crate::trade::TradeSide::Buy => "买入",
        crate::trade::TradeSide::Sell => "卖出",
    }
}

fn print_trade_positions(positions: &[TradePosition]) {
    if positions.is_empty() {
        println!("📭 暂无持仓");
        return;
    }

    println!("{:<10} {:<10} {:<14} {}", "代码", "数量", "持仓成本", "最新成交价");
    println!("{}", "-".repeat(56));

    for position in positions {
        println!(
            "{:<10} {:<10} {:<14} {}",
            position.code, position.volume, position.avg_cost, position.last_trade_price
        );
    }
}

fn print_trade_cash(snapshot: &CashSnapshot) {
    println!("初始资金: {}", snapshot.initial_capital);
    println!("可用现金: {}", snapshot.available_cash);
    println!("持仓估值: {}", snapshot.estimated_position_value);
    println!("总资产估算: {}", snapshot.estimated_total_assets);
}

fn create_trade_store() -> JsonPaperTradeStore {
    let runtime = CliRuntime::load();
    JsonPaperTradeStore::new(runtime.trade_path)
}
