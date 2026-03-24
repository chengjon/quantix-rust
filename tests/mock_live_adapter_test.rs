use chrono::{TimeZone, Utc};
use quantix_cli::execution::adapter::{AdapterOrderRequest, ExecutionAdapter};
use quantix_cli::execution::mock_live::{MockLiveClock, MockLiveExecutionAdapter};
use quantix_cli::execution::models::{
    MockLiveFaultInjection, MockLiveFillStep, MockLiveOrderState, OrderSide, OrderStatus,
};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use rust_decimal_macros::dec;
use tempfile::tempdir;

#[derive(Clone, Copy)]
struct FixedClock;

impl MockLiveClock for FixedClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 22, 10, 0, 0).unwrap()
    }
}

fn buy_request(client_order_id: &str, quantity: i64) -> AdapterOrderRequest {
    AdapterOrderRequest {
        client_order_id: client_order_id.to_string(),
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        quantity,
        price: dec!(12.34),
    }
}

#[tokio::test]
async fn submit_order_defaults_to_accepted_and_persists_private_state() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = MockLiveExecutionAdapter::new(store.clone(), FixedClock);

    assert_eq!(adapter.adapter_name(), "mock_live");

    let response = adapter
        .submit_order(buy_request("mock-live-1", 100))
        .await
        .unwrap();

    assert_eq!(response.latest_status, OrderStatus::Accepted);
    assert_eq!(response.filled_quantity, 0);
    assert!(response.fill_details.is_none());

    let saved = store
        .get_mock_live_order_state("mock-live-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.next_step_index, 0);
}

#[tokio::test]
async fn query_order_advances_to_partial_then_fill() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let template = MockLiveOrderState {
        fill_plan: vec![
            MockLiveFillStep {
                quantity: 40,
                delay_secs: 0,
            },
            MockLiveFillStep {
                quantity: 60,
                delay_secs: 0,
            },
        ],
        ..Default::default()
    };
    let adapter =
        MockLiveExecutionAdapter::with_state_template(store.clone(), FixedClock, template);

    adapter
        .submit_order(buy_request("mock-live-2", 100))
        .await
        .unwrap();

    let first = adapter.query_order("mock-live-2").await.unwrap();
    assert_eq!(first.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(first.filled_quantity, 40);
    let first_fill = first.fill_details.as_ref().expect("partial fill details");
    assert_eq!(first_fill.fill_id, 1);
    assert_eq!(first_fill.fill_quantity, 40);
    assert_eq!(first_fill.fill_price, dec!(12.34));

    let mut state = store
        .get_mock_live_order_state("mock-live-2")
        .await
        .unwrap()
        .unwrap();
    state.last_applied_fill_id = 1;
    store
        .update_mock_live_order_state("mock-live-2", Some("mock-live-2"), &state)
        .await
        .unwrap();

    let second = adapter.query_order("mock-live-2").await.unwrap();
    assert_eq!(second.latest_status, OrderStatus::Filled);
    assert_eq!(second.filled_quantity, 100);
    let second_fill = second.fill_details.as_ref().expect("final fill details");
    assert_eq!(second_fill.fill_id, 2);
    assert_eq!(second_fill.fill_quantity, 60);
    assert_eq!(second_fill.fill_price, dec!(12.34));
}

#[tokio::test]
async fn query_order_without_fill_plan_returns_no_fill_details() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = MockLiveExecutionAdapter::new(store, FixedClock);

    adapter
        .submit_order(buy_request("mock-live-accepted", 100))
        .await
        .unwrap();

    let response = adapter.query_order("mock-live-accepted").await.unwrap();
    assert_eq!(response.latest_status, OrderStatus::Accepted);
    assert_eq!(response.filled_quantity, 0);
    assert!(response.fill_details.is_none());
}

#[tokio::test]
async fn cancel_order_resolves_to_canceled_on_next_query() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = MockLiveExecutionAdapter::new(store, FixedClock);

    adapter
        .submit_order(buy_request("mock-live-3", 100))
        .await
        .unwrap();
    adapter.cancel_order("mock-live-3").await.unwrap();

    let response = adapter.query_order("mock-live-3").await.unwrap();
    assert_eq!(response.latest_status, OrderStatus::Canceled);
}

#[tokio::test]
async fn unknown_once_fault_recovers_on_follow_up_query() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let template = MockLiveOrderState {
        fill_plan: vec![MockLiveFillStep {
            quantity: 100,
            delay_secs: 0,
        }],
        fault_injection: Some(MockLiveFaultInjection {
            mode: Some("unknown_once".to_string()),
        }),
        ..Default::default()
    };
    let adapter = MockLiveExecutionAdapter::with_state_template(store, FixedClock, template);

    adapter
        .submit_order(buy_request("mock-live-4", 100))
        .await
        .unwrap();

    let first = adapter.query_order("mock-live-4").await.unwrap();
    assert_eq!(first.latest_status, OrderStatus::Unknown);

    let second = adapter.query_order("mock-live-4").await.unwrap();
    assert_eq!(second.latest_status, OrderStatus::Filled);
    assert_eq!(second.filled_quantity, 100);
}

#[tokio::test]
async fn query_order_repeats_pending_fill_until_last_applied_fill_id_advances() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let template = MockLiveOrderState {
        fill_plan: vec![
            MockLiveFillStep {
                quantity: 40,
                delay_secs: 0,
            },
            MockLiveFillStep {
                quantity: 60,
                delay_secs: 0,
            },
        ],
        ..Default::default()
    };
    let adapter =
        MockLiveExecutionAdapter::with_state_template(store.clone(), FixedClock, template);

    adapter
        .submit_order(buy_request("mock-live-repeat", 100))
        .await
        .unwrap();

    let first = adapter.query_order("mock-live-repeat").await.unwrap();
    let first_fill = first.fill_details.as_ref().expect("first pending fill");
    assert_eq!(first.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(first_fill.fill_id, 1);
    assert_eq!(first.filled_quantity, 40);

    let repeated = adapter.query_order("mock-live-repeat").await.unwrap();
    let repeated_fill = repeated
        .fill_details
        .as_ref()
        .expect("same pending fill should repeat");
    assert_eq!(repeated.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(repeated.filled_quantity, 40);
    assert_eq!(repeated_fill.fill_id, 1);
    assert_eq!(repeated_fill.fill_quantity, 40);

    let mut state = store
        .get_mock_live_order_state("mock-live-repeat")
        .await
        .unwrap()
        .unwrap();
    state.last_applied_fill_id = 1;
    store
        .update_mock_live_order_state("mock-live-repeat", Some("mock-live-repeat"), &state)
        .await
        .unwrap();

    let next = adapter.query_order("mock-live-repeat").await.unwrap();
    let next_fill = next.fill_details.as_ref().expect("second fill after ack");
    assert_eq!(next.latest_status, OrderStatus::Filled);
    assert_eq!(next.filled_quantity, 100);
    assert_eq!(next_fill.fill_id, 2);
    assert_eq!(next_fill.fill_quantity, 60);
}
