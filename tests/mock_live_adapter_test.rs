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

async fn mark_fill_applied(store: &StrategyRuntimeStore, order_id: &str, fill_id: u64) {
    let mut state = store
        .get_mock_live_order_state(order_id)
        .await
        .unwrap()
        .unwrap();
    state.last_applied_fill_id = fill_id;
    store
        .update_mock_live_order_state(order_id, Some(order_id), &state)
        .await
        .unwrap();
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
            delay_seconds: None,
            rejection_reason: None,
            timeout_seconds: None,
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

#[tokio::test]
async fn query_order_advances_through_three_partial_fill_steps() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let template = MockLiveOrderState {
        fill_plan: vec![
            MockLiveFillStep {
                quantity: 20,
                delay_secs: 0,
            },
            MockLiveFillStep {
                quantity: 30,
                delay_secs: 0,
            },
            MockLiveFillStep {
                quantity: 50,
                delay_secs: 0,
            },
        ],
        ..Default::default()
    };
    let adapter =
        MockLiveExecutionAdapter::with_state_template(store.clone(), FixedClock, template);

    adapter
        .submit_order(buy_request("mock-live-three-step", 100))
        .await
        .unwrap();

    let first = adapter.query_order("mock-live-three-step").await.unwrap();
    assert_eq!(first.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(first.filled_quantity, 20);
    assert_eq!(first.fill_details.as_ref().unwrap().fill_id, 1);

    mark_fill_applied(&store, "mock-live-three-step", 1).await;

    let second = adapter.query_order("mock-live-three-step").await.unwrap();
    assert_eq!(second.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(second.filled_quantity, 50);
    assert_eq!(second.fill_details.as_ref().unwrap().fill_id, 2);
    assert_eq!(second.fill_details.as_ref().unwrap().fill_quantity, 30);

    mark_fill_applied(&store, "mock-live-three-step", 2).await;

    let third = adapter.query_order("mock-live-three-step").await.unwrap();
    assert_eq!(third.latest_status, OrderStatus::Filled);
    assert_eq!(third.filled_quantity, 100);
    assert_eq!(third.fill_details.as_ref().unwrap().fill_id, 3);
    assert_eq!(third.fill_details.as_ref().unwrap().fill_quantity, 50);
}

#[tokio::test]
async fn query_order_can_progress_unknown_then_accepted_then_unknown_then_filled() {
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
            mode: Some("query_script:unknown,accepted,unknown".to_string()),
            delay_seconds: None,
            rejection_reason: None,
            timeout_seconds: None,
        }),
        ..Default::default()
    };
    let adapter = MockLiveExecutionAdapter::with_state_template(store, FixedClock, template);

    adapter
        .submit_order(buy_request("mock-live-query-script", 100))
        .await
        .unwrap();

    let first = adapter.query_order("mock-live-query-script").await.unwrap();
    assert_eq!(first.latest_status, OrderStatus::Unknown);
    assert_eq!(first.filled_quantity, 0);
    assert!(first.fill_details.is_none());

    let second = adapter.query_order("mock-live-query-script").await.unwrap();
    assert_eq!(second.latest_status, OrderStatus::Accepted);
    assert_eq!(second.filled_quantity, 0);
    assert!(second.fill_details.is_none());

    let third = adapter.query_order("mock-live-query-script").await.unwrap();
    assert_eq!(third.latest_status, OrderStatus::Unknown);
    assert_eq!(third.filled_quantity, 0);
    assert!(third.fill_details.is_none());

    let fourth = adapter.query_order("mock-live-query-script").await.unwrap();
    assert_eq!(fourth.latest_status, OrderStatus::Filled);
    assert_eq!(fourth.filled_quantity, 100);
    assert_eq!(fourth.fill_details.as_ref().unwrap().fill_id, 1);
}

#[tokio::test]
async fn query_fault_after_partial_fill_preserves_quantity_and_next_fill_details() {
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
        .submit_order(buy_request("mock-live-fault-after-partial", 100))
        .await
        .unwrap();

    let first = adapter
        .query_order("mock-live-fault-after-partial")
        .await
        .unwrap();
    assert_eq!(first.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(first.filled_quantity, 40);
    assert_eq!(first.fill_details.as_ref().unwrap().fill_id, 1);

    mark_fill_applied(&store, "mock-live-fault-after-partial", 1).await;

    let mut state = store
        .get_mock_live_order_state("mock-live-fault-after-partial")
        .await
        .unwrap()
        .unwrap();
    state.fault_injection = Some(MockLiveFaultInjection {
        mode: Some("network_disconnect".to_string()),
        delay_seconds: None,
        rejection_reason: None,
        timeout_seconds: None,
    });
    store
        .update_mock_live_order_state(
            "mock-live-fault-after-partial",
            Some("mock-live-fault-after-partial"),
            &state,
        )
        .await
        .unwrap();

    let err = adapter
        .query_order("mock-live-fault-after-partial")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        quantix_cli::execution::adapter::AdapterError::Network(_)
    ));

    let recovered = adapter
        .query_order("mock-live-fault-after-partial")
        .await
        .unwrap();
    assert_eq!(recovered.latest_status, OrderStatus::Filled);
    assert_eq!(recovered.filled_quantity, 100);
    let fill = recovered
        .fill_details
        .as_ref()
        .expect("final fill after fault");
    assert_eq!(fill.fill_id, 2);
    assert_eq!(fill.fill_quantity, 60);
}

#[tokio::test]
async fn cancel_after_partial_fill_keeps_filled_quantity_intact() {
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
        .submit_order(buy_request("mock-live-cancel-after-partial", 100))
        .await
        .unwrap();

    let first = adapter
        .query_order("mock-live-cancel-after-partial")
        .await
        .unwrap();
    assert_eq!(first.latest_status, OrderStatus::PartiallyFilled);
    assert_eq!(first.filled_quantity, 40);

    mark_fill_applied(&store, "mock-live-cancel-after-partial", 1).await;

    adapter
        .cancel_order("mock-live-cancel-after-partial")
        .await
        .unwrap();

    let canceled = adapter
        .query_order("mock-live-cancel-after-partial")
        .await
        .unwrap();
    assert_eq!(canceled.latest_status, OrderStatus::Canceled);
    assert_eq!(canceled.filled_quantity, 40);
    assert!(canceled.fill_details.is_none());
}
