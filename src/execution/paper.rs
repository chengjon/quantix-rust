use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, ExecutionCancelSemantics,
    ExecutionCapabilities, ExecutionChannel, ExecutionFillSource, ExecutionStatusSource,
    OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::mode_semantics::{PAPER_IMMEDIATE_CHANNEL, log_execution_mode_risk_notice};
use crate::execution::models::{FillDetails, OrderSide, OrderStatus};
use crate::trade::{PaperTradeStore, TradeOrderRequest, TradeRecord, TradeService};

/// Paper execution is intentionally local immediate-fill accounting only.
pub const IMMEDIATE_FILL_ONLY: bool = true;

#[derive(Debug, Clone)]
pub struct PaperExecutionAdapter<Store> {
    trade_service: TradeService<Store>,
}

impl<Store> PaperExecutionAdapter<Store>
where
    Store: PaperTradeStore,
{
    pub fn new(trade_service: TradeService<Store>) -> Self {
        Self { trade_service }
    }
}

#[async_trait]
impl<Store> ExecutionAdapter for PaperExecutionAdapter<Store>
where
    Store: PaperTradeStore,
{
    fn adapter_name(&self) -> &'static str {
        "paper"
    }

    fn capabilities(&self) -> ExecutionCapabilities {
        ExecutionCapabilities {
            channel: ExecutionChannel::PaperImmediate,
            status_source: ExecutionStatusSource::LocalImmediateAccounting,
            fill_source: ExecutionFillSource::LocalImmediateAccounting,
            relies_on_broker_api: false,
            supports_pending_order_lifecycle: false,
            supports_partial_fill: false,
            cancel_semantics: ExecutionCancelSemantics::AlreadyFilledOnly,
        }
    }

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<OrderInitialResponse, AdapterError> {
        log_execution_mode_risk_notice(PAPER_IMMEDIATE_CHANNEL);
        let trade_request = to_trade_order_request(&request)?;

        match request.side {
            OrderSide::Buy => {
                let record = self
                    .trade_service
                    .buy(trade_request, chrono::Utc::now())
                    .await
                    .map_err(|err| AdapterError::Execution(err.to_string()))?;
                Ok(OrderInitialResponse {
                    adapter_order_id: record.id.clone(),
                    latest_status: OrderStatus::Filled,
                    filled_quantity: record.volume,
                    avg_fill_price: Some(record.price),
                    fill_details: Some(fill_details_from_record(&record)),
                    rejection_reason: None,
                })
            }
            OrderSide::Sell => {
                let record = self
                    .trade_service
                    .sell(trade_request, chrono::Utc::now())
                    .await
                    .map_err(|err| AdapterError::Execution(err.to_string()))?;
                Ok(OrderInitialResponse {
                    adapter_order_id: record.id.clone(),
                    latest_status: OrderStatus::Filled,
                    filled_quantity: record.volume,
                    avg_fill_price: Some(record.price),
                    fill_details: Some(fill_details_from_record(&record)),
                    rejection_reason: None,
                })
            }
        }
    }

    async fn query_order(
        &self,
        order_id: &str,
    ) -> std::result::Result<OrderQueryResponse, AdapterError> {
        let record = self.find_trade_record(order_id).await?;
        Ok(OrderQueryResponse {
            adapter_order_id: record.id.clone(),
            latest_status: OrderStatus::Filled,
            filled_quantity: record.volume,
            avg_fill_price: Some(record.price),
            fill_details: Some(fill_details_from_record(&record)),
            rejection_reason: None,
        })
    }

    async fn cancel_order(&self, order_id: &str) -> std::result::Result<(), AdapterError> {
        let record = self.find_trade_record(order_id).await?;
        Err(AdapterError::Execution(format!(
            "paper order {} 已成交，无法撤单",
            record.id
        )))
    }
}

impl<Store> PaperExecutionAdapter<Store>
where
    Store: PaperTradeStore,
{
    async fn find_trade_record(
        &self,
        order_id: &str,
    ) -> std::result::Result<TradeRecord, AdapterError> {
        let state = self
            .trade_service
            .state_snapshot()
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?;
        state
            .trade_records
            .into_iter()
            .find(|record| record.id == order_id)
            .ok_or_else(|| AdapterError::Execution(format!("paper order {order_id} 不存在")))
    }
}

fn to_trade_order_request(
    request: &AdapterOrderRequest,
) -> std::result::Result<TradeOrderRequest, AdapterError> {
    let price = decimal_to_f64(request.price)?;
    TradeOrderRequest::new(request.symbol.clone(), price, request.quantity)
        .map_err(|err| AdapterError::Execution(err.to_string()))
}

fn decimal_to_f64(value: Decimal) -> std::result::Result<f64, AdapterError> {
    value.to_f64().ok_or_else(|| {
        AdapterError::Execution(format!(
            "phase29a paper adapter 无法将价格 {value} 转换为 f64"
        ))
    })
}

fn fill_details_from_record(record: &TradeRecord) -> FillDetails {
    FillDetails {
        fill_id: 1,
        fill_quantity: record.volume,
        fill_price: record.price,
        last_fill_price: record.price,
        last_fill_quantity: record.volume,
        total_fills: 1,
        commission: record.commission,
        fees: record.total_fee,
        venue: "paper".to_string(),
        broker_fill_id: record.id.clone(),
    }
}
