use super::*;

pub async fn run_stop_command(cmd: StopCommands) -> Result<()> {
    let watchlist_storage = create_watchlist_storage();
    let service = StopService::new(create_stop_rule_store().await?);
    let trade_store = create_trade_store();
    let output = execute_stop_command_with_context(
        cmd,
        &service,
        &watchlist_storage,
        &TdxWatchlistQuoteLookup,
        &trade_store,
    )
    .await?;
    print_stop_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StopCommandOutput {
    RuleSet(StopRule),
    RuleUpdated(StopRule),
    RuleList(Vec<StopRule>),
    StatusRows(Vec<StopStatusRow>),
    HistoryRows(Vec<StopHistoryEvent>),
    RuleRemoved { code: String, removed: bool },
}

#[cfg(test)]
pub(crate) async fn execute_stop_command_with_service<RS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
{
    let trade_store = create_trade_store();
    execute_stop_command_with_context(
        cmd,
        service,
        watchlist_storage,
        &TdxWatchlistQuoteLookup,
        &trade_store,
    )
    .await
}

pub(crate) async fn execute_stop_command_with_context<RS, Q, TS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
    quote_lookup: &Q,
    trade_store: &TS,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    match cmd {
        StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let reference_price = if loss_pct.is_some() || profit_pct.is_some() {
                Some(resolve_stop_reference_price(&code, quote_lookup, trade_store).await?)
            } else {
                None
            };
            let rule = service
                .set_rule(
                    &code,
                    loss,
                    profit,
                    loss_pct,
                    profit_pct,
                    trailing,
                    reference_price,
                    Utc::now(),
                )
                .await?;
            Ok(StopCommandOutput::RuleSet(rule))
        }
        StopCommands::Update {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
            clear_loss,
            clear_profit,
            clear_loss_pct,
            clear_profit_pct,
            clear_trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let existing = service
                .get_rule(&code)
                .await?
                .ok_or_else(|| QuantixError::Other(format!("stop update 未找到规则: {code}")))?;
            let needs_reference_price =
                (loss_pct.is_some() || profit_pct.is_some()) && existing.reference_price.is_none();
            let reference_price = if needs_reference_price {
                Some(Some(
                    resolve_stop_reference_price(&code, quote_lookup, trade_store).await?,
                ))
            } else {
                None
            };
            let rule = service
                .update_rule(
                    &code,
                    StopRuleUpdate {
                        stop_loss_price: patch_value(loss, clear_loss),
                        take_profit_price: patch_value(profit, clear_profit),
                        stop_loss_pct: patch_value(loss_pct, clear_loss_pct),
                        take_profit_pct: patch_value(profit_pct, clear_profit_pct),
                        trailing_pct: patch_value(trailing, clear_trailing),
                        reference_price,
                    },
                    Utc::now(),
                )
                .await?;
            Ok(StopCommandOutput::RuleUpdated(rule))
        }
        StopCommands::List => Ok(StopCommandOutput::RuleList(service.list_rules().await?)),
        StopCommands::Status { code } => {
            let rules = filter_stop_rules(service.list_rules().await?, code.as_deref());
            let status_rows =
                build_stop_status_rows(service, &rules, quote_lookup, trade_store, Utc::now())
                    .await?;
            Ok(StopCommandOutput::StatusRows(status_rows))
        }
        StopCommands::History {
            code,
            limit,
            date,
            event_type,
        } => Ok(StopCommandOutput::HistoryRows(
            service
                .history(
                    code.as_deref(),
                    date.as_deref().map(parse_stop_history_date).transpose()?,
                    event_type
                        .as_deref()
                        .map(parse_stop_history_event_type)
                        .transpose()?,
                    Some(limit),
                )
                .await?,
        )),
        StopCommands::Remove { code } => {
            let removed = service.remove_rule(&code, Utc::now()).await?;
            Ok(StopCommandOutput::RuleRemoved { code, removed })
        }
    }
}
