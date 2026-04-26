use super::*;

pub(crate) fn build_trade_init_request(
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

pub(crate) fn build_trade_order_request(
    command_name: &str,
    code: String,
    price: f64,
    volume: i64,
) -> Result<TradeOrderRequest> {
    TradeOrderRequest::new(code, price, volume)
        .map_err(|err| remap_trade_request_error(err, command_name))
}

pub(crate) fn decimal_to_f64(value: Decimal, command_name: &str) -> Result<f64> {
    value
        .to_f64()
        .ok_or_else(|| QuantixError::Other(format!("{command_name} 无法将价格 {value} 转换为 f64")))
}

pub(crate) fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(
            message
                .replace("trade init", command_name)
                .replace("trade order", command_name),
        ),
        other => other,
    }
}

pub(crate) fn patch_value(value: Option<f64>, clear: bool) -> Option<Option<f64>> {
    if clear { Some(None) } else { value.map(Some) }
}

pub(crate) fn parse_stop_history_event_type(value: &str) -> Result<StopHistoryEventType> {
    StopHistoryEventType::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 stop history event_type: {value}")))
}

pub(crate) fn parse_stop_history_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| QuantixError::Other(format!("stop history --date 无效: {value}")))
}

pub(crate) fn filter_stop_rules(rules: Vec<StopRule>, code: Option<&str>) -> Vec<StopRule> {
    match code {
        Some(code) => rules.into_iter().filter(|rule| rule.code == code).collect(),
        None => rules,
    }
}

pub(crate) async fn build_avg_cost_map_from_trade_store<Store>(
    trade_store: &Store,
) -> Result<HashMap<String, f64>>
where
    Store: PaperTradeStore,
{
    let Some(state) = trade_store.load_state().await? else {
        return Ok(HashMap::new());
    };
    let Some(account) = state.account else {
        return Ok(HashMap::new());
    };

    Ok(account
        .positions
        .into_iter()
        .filter_map(|(code, position)| position.avg_cost.to_f64().map(|avg_cost| (code, avg_cost)))
        .collect())
}

pub(crate) async fn resolve_stop_reference_price<Q, TS>(
    code: &str,
    quote_lookup: &Q,
    trade_store: &TS,
) -> Result<f64>
where
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let quote_price = quote_lookup
        .lookup_quotes(&[code.to_string()])
        .await
        .ok()
        .and_then(|quotes| {
            quotes
                .get(code)
                .and_then(|snapshot| snapshot.latest_price.to_f64())
        });
    if let Some(price) = quote_price {
        return Ok(price);
    }

    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    if let Some(avg_cost) = avg_cost_by_code.get(code).copied() {
        return Ok(avg_cost);
    }

    Err(QuantixError::Other(format!(
        "stop percent 规则缺少参考价，且当前无法从行情或持仓解析 {} 的 reference_price",
        code
    )))
}

pub(crate) async fn build_stop_status_rows<RS, Q, TS>(
    service: &StopService<RS>,
    rules: &[StopRule],
    quote_lookup: &Q,
    trade_store: &TS,
    observed_at: DateTime<Utc>,
) -> Result<Vec<StopStatusRow>>
where
    RS: StopRuleStore,
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let codes: Vec<String> = rules.iter().map(|rule| rule.code.clone()).collect();
    let quote_rows = quote_lookup
        .lookup_quotes(&codes)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(code, snapshot)| MonitorQuoteRow {
            code,
            group: String::new(),
            tags: Vec::new(),
            last_price: snapshot.latest_price.to_f64(),
            change_pct: snapshot.price_change_pct.and_then(|value| value.to_f64()),
            quote_time: None,
            note: None,
        })
        .collect::<Vec<_>>();
    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    Ok(service.status_rows(rules, &quote_rows, &avg_cost_by_code, observed_at))
}

pub(crate) fn ensure_watchlist_contains_code(storage: &WatchlistStorage, code: &str) -> Result<()> {
    let store = load_watchlist_store_for_read(storage)?;
    if store.entries.contains_key(code) {
        Ok(())
    } else {
        Err(QuantixError::Other(format!("股票不在自选池: {}", code)))
    }
}

pub(crate) fn format_stop_eval_state(state: crate::stop::StopEvalState) -> &'static str {
    match state {
        crate::stop::StopEvalState::Armed => "armed",
        crate::stop::StopEvalState::Triggered => "triggered",
        crate::stop::StopEvalState::AnchorMissing => "anchor_missing",
        crate::stop::StopEvalState::QuoteMissing => "quote_missing",
    }
}
