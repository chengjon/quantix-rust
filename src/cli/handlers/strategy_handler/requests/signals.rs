use super::*;
use crate::safety::{
    JsonKillSwitchStore, format_execution_kill_switch_block_message,
    load_blocking_kill_switch_state,
};

pub(crate) async fn execute_strategy_signal_list(
    strategy_instance: Option<&str>,
    strategy: Option<&str>,
    code: Option<&str>,
    approval_status: Option<&str>,
    signal_status: Option<&str>,
    limit: usize,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows = execute_strategy_signal_list_with_store(
        &runtime_store,
        StrategySignalListFilters {
            strategy_instance,
            strategy,
            code,
            approval_status,
            signal_status,
            limit: Some(limit),
        },
    )
    .await?;

    for row in rows {
        println!("{}", format_strategy_signal_row(&row));
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StrategySignalListFilters<'a> {
    pub(crate) strategy_instance: Option<&'a str>,
    pub(crate) strategy: Option<&'a str>,
    pub(crate) code: Option<&'a str>,
    pub(crate) approval_status: Option<&'a str>,
    pub(crate) signal_status: Option<&'a str>,
    pub(crate) limit: Option<usize>,
}

pub(crate) fn format_strategy_signal_row(row: &StrategySignalRecord) -> String {
    let source_id = row
        .metadata_json
        .get("bar_source_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let fallback = row
        .metadata_json
        .get("bar_source_fallback")
        .and_then(|value| value.as_bool())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "{} {} {} {} {} {} bar_end={} source={} fallback={}",
        row.signal_id,
        row.strategy_instance_id,
        row.symbol,
        row.signal_value,
        row.signal_status.as_str(),
        row.approval_status.as_str(),
        row.bar_end.format("%Y-%m-%dT%H:%M:%SZ"),
        source_id,
        fallback
    )
}

pub(crate) async fn execute_strategy_signal_list_with_store(
    store: &StrategyRuntimeStore,
    filters: StrategySignalListFilters<'_>,
) -> Result<Vec<StrategySignalRecord>> {
    let approval_filter = filters
        .approval_status
        .map(parse_approval_status)
        .transpose()?;
    let signal_filter = filters.signal_status.map(parse_signal_status).transpose()?;

    let rows = store.list_signals().await?;
    let mut filtered = rows
        .into_iter()
        .filter(|row| {
            filters
                .strategy_instance
                .is_none_or(|id| row.strategy_instance_id == id)
        })
        .filter(|row| {
            filters
                .strategy
                .is_none_or(|name| row.strategy_name == name)
        })
        .filter(|row| filters.code.is_none_or(|code| row.symbol == code))
        .filter(|row| approval_filter.is_none_or(|status| row.approval_status == status))
        .filter(|row| signal_filter.is_none_or(|status| row.signal_status == status))
        .collect::<Vec<_>>();

    if let Some(limit) = filters.limit {
        filtered.truncate(limit);
    }

    Ok(filtered)
}

pub(crate) async fn execute_strategy_signal_approve(
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        signal_id,
        target_mode,
        target_account,
    )
    .await?;
    println!("{}", format_strategy_approval_result(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_signal_approve_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<ExecutionRequestRecord> {
    let kill_switch_store = JsonKillSwitchStore::with_default_path()?;
    execute_strategy_signal_approve_with_store_and_kill_switch(
        store,
        &kill_switch_store,
        signal_id,
        target_mode,
        target_account,
    )
    .await
}

pub(crate) async fn execute_strategy_signal_approve_with_store_and_kill_switch(
    store: &StrategyRuntimeStore,
    kill_switch_store: &JsonKillSwitchStore,
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<ExecutionRequestRecord> {
    validate_strategy_signal_approval_target_mode(target_mode)?;
    guard_strategy_signal_approval_kill_switch(kill_switch_store, target_mode)?;

    store
        .approve_signal_and_create_request(signal_id, target_mode, target_account, Some("cli"))
        .await
}

fn validate_strategy_signal_approval_target_mode(target_mode: &str) -> Result<()> {
    match target_mode {
        "paper" | "mock_live" | "qmt_live" => Ok(()),
        "live" => Err(QuantixError::Unsupported(format!(
            "strategy signal approve 暂不支持 target_mode=live；如需真实 QMT 提交，请改用 target_mode=qmt_live，然后走 {QMT_LIVE_BRIDGE_COMMAND} 路径，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}"
        ))),
        other => Err(QuantixError::Unsupported(format!(
            "strategy signal approve 不支持 target_mode={other}"
        ))),
    }
}

fn guard_strategy_signal_approval_kill_switch(
    kill_switch_store: &JsonKillSwitchStore,
    target_mode: &str,
) -> Result<()> {
    if !matches!(target_mode, "mock_live" | "qmt_live") {
        return Ok(());
    }

    if let Some(state) = load_blocking_kill_switch_state(kill_switch_store, target_mode)? {
        return Err(QuantixError::Other(
            format_execution_kill_switch_block_message(target_mode, &state),
        ));
    }

    Ok(())
}

pub(crate) async fn execute_strategy_signal_reject(
    signal_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let signal =
        execute_strategy_signal_reject_with_store(&runtime_store, signal_id, reason).await?;
    println!("{}", format_strategy_rejection_result(&signal));
    Ok(())
}

pub(crate) async fn execute_strategy_signal_reject_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    reason: Option<&str>,
) -> Result<StrategySignalRecord> {
    store.reject_signal(signal_id, reason).await?;
    store
        .get_signal(signal_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {signal_id}")))
}

pub(crate) fn format_strategy_approval_result(request: &ExecutionRequestRecord) -> String {
    format!(
        "{} signal={} target={}/{} status={}",
        request.request_id,
        request.signal_id,
        request.target_mode,
        request.target_account,
        request.request_status.as_str()
    )
}

pub(crate) fn format_strategy_rejection_result(signal: &StrategySignalRecord) -> String {
    let reason = signal
        .metadata_json
        .get("rejection_reason")
        .and_then(|value| value.as_str())
        .unwrap_or("-");

    format!(
        "{} signal_status={} approval_status={} reason={}",
        signal.signal_id,
        signal.signal_status.as_str(),
        signal.approval_status.as_str(),
        reason
    )
}

pub(crate) fn parse_approval_status(value: &str) -> Result<ApprovalStatus> {
    ApprovalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 approval_status: {value}")))
}

pub(crate) fn parse_signal_status(value: &str) -> Result<SignalStatus> {
    SignalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 signal_status: {value}")))
}
