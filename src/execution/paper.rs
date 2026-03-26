use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::models::{FillDetails, OrderSide, OrderStatus};
use crate::trade::{PaperTradeStore, TradeOrderRequest, TradeService};

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

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<OrderInitialResponse, AdapterError> {
        let trade_request = to_trade_order_request(&request)?;

        match request.side {
            OrderSide::Buy => {
                let record = self
                    .trade_service
                    .buy(trade_request, chrono::Utc::now())
                    .await
                    .map_err(|err| AdapterError::Execution(err.to_string()))?;
                Ok(OrderInitialResponse {
                    adapter_order_id: request.client_order_id,
                    latest_status: OrderStatus::Filled,
                    filled_quantity: record.volume,
                    avg_fill_price: Some(record.price),
                    fill_details: Some(FillDetails {
                        fill_id: 1,
                        fill_quantity: record.volume,
                        fill_price: record.price,
                    }),
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
                    adapter_order_id: request.client_order_id,
                    latest_status: OrderStatus::Filled,
                    filled_quantity: record.volume,
                    avg_fill_price: Some(record.price),
                    fill_details: Some(FillDetails {
                        fill_id: 1,
                        fill_quantity: record.volume,
                        fill_price: record.price,
                    }),
                    rejection_reason: None,
                })
            }
        }
    }

    async fn query_order(
        &self,
        _order_id: &str,
    ) -> std::result::Result<OrderQueryResponse, AdapterError> {
        Err(AdapterError::Unsupported(
            "phase29a paper adapter 不支持 query_order".to_string(),
        ))
    }

    async fn cancel_order(&self, _order_id: &str) -> std::result::Result<(), AdapterError> {
        Err(AdapterError::Unsupported(
            "phase29a paper adapter 不支持 cancel_order".to_string(),
        ))
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
