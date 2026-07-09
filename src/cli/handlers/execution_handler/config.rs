//! Execution config + daemon command handlers.

use super::*;
use std::time::Duration;

fn create_execution_config_store() -> JsonExecutionConfigStore {
    let runtime = CliRuntime::load();
    JsonExecutionConfigStore::new(runtime.execution_config_path)
}

pub fn create_bridge_client() -> Result<BridgeHttpClient> {
    let runtime = CliRuntime::load();
    BridgeHttpClient::new(runtime.bridge.base_url, runtime.bridge.api_key)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

pub(crate) async fn execute_execution_config_init() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_config_show() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_daemon_run(once: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let config_store = JsonExecutionConfigStore::new(runtime.execution_config_path);
    let config = config_store.load_or_create()?;
    let trade_store = create_trade_store();
    let risk_store = create_risk_store();

    if once {
        let summary =
            consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
                .await?;
        print_execution_daemon_summary(&summary);
        return Ok(());
    }

    loop {
        let summary = consume_next_pending_request_with_components(
            &runtime_store,
            trade_store.clone(),
            risk_store.clone(),
        )
        .await?;
        print_execution_daemon_summary(&summary);
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}
