use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::core::Result;
use quantix_cli::execution::adapter::{
    ExecutionAdapter, ExecutionCancelSemantics, ExecutionChannel, ExecutionFillSource,
    ExecutionStatusSource,
};
use quantix_cli::execution::mock_live::{MockLiveClock, MockLiveExecutionAdapter};
use quantix_cli::execution::paper::PaperExecutionAdapter;
use quantix_cli::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::trade::{PaperTradeState, PaperTradeStore, TradeService};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[derive(Clone, Default)]
struct FakePaperTradeStore {
    state: Arc<Mutex<Option<PaperTradeState>>>,
}

#[async_trait]
impl PaperTradeStore for FakePaperTradeStore {
    async fn load_state(&self) -> Result<Option<PaperTradeState>> {
        Ok(self.state.lock().unwrap().clone())
    }

    async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct FixedClock;

impl MockLiveClock for FixedClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 19, 10, 0, 0).unwrap()
    }
}

#[test]
fn paper_immediate_reports_local_immediate_fill_capabilities() {
    let adapter = PaperExecutionAdapter::new(TradeService::new(FakePaperTradeStore::default()));

    let capabilities = adapter.capabilities();

    assert_eq!(capabilities.channel, ExecutionChannel::PaperImmediate);
    assert_eq!(
        capabilities.status_source,
        ExecutionStatusSource::LocalImmediateAccounting
    );
    assert_eq!(
        capabilities.fill_source,
        ExecutionFillSource::LocalImmediateAccounting
    );
    assert!(!capabilities.relies_on_broker_api);
    assert!(!capabilities.supports_pending_order_lifecycle);
    assert!(!capabilities.supports_partial_fill);
    assert_eq!(
        capabilities.cancel_semantics,
        ExecutionCancelSemantics::AlreadyFilledOnly
    );
}

#[tokio::test]
async fn mock_live_reports_local_lifecycle_capabilities() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = MockLiveExecutionAdapter::new(store, FixedClock);

    let capabilities = adapter.capabilities();

    assert_eq!(capabilities.channel, ExecutionChannel::MockLive);
    assert_eq!(
        capabilities.status_source,
        ExecutionStatusSource::LocalSimulatedLifecycle
    );
    assert_eq!(
        capabilities.fill_source,
        ExecutionFillSource::LocalSimulatedMatcher
    );
    assert!(!capabilities.relies_on_broker_api);
    assert!(capabilities.supports_pending_order_lifecycle);
    assert!(capabilities.supports_partial_fill);
    assert_eq!(
        capabilities.cancel_semantics,
        ExecutionCancelSemantics::LocalLifecycle
    );
}

#[test]
fn qmt_live_reports_broker_backed_capabilities() {
    let client = BridgeHttpClient::new_with_contract(
        "http://127.0.0.1:17580".to_string(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();
    let adapter = QmtLiveExecutionAdapter::with_polling(client, 1, 10);

    let capabilities = adapter.capabilities();

    assert_eq!(capabilities.channel, ExecutionChannel::QmtLive);
    assert_eq!(capabilities.status_source, ExecutionStatusSource::Broker);
    assert_eq!(capabilities.fill_source, ExecutionFillSource::Broker);
    assert!(capabilities.relies_on_broker_api);
    assert!(capabilities.supports_pending_order_lifecycle);
    assert!(capabilities.supports_partial_fill);
    assert_eq!(
        capabilities.cancel_semantics,
        ExecutionCancelSemantics::Broker
    );
}
