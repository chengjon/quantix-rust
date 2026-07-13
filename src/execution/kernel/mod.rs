//! 执行内核：信号翻译、风控、下单、成交增量应用、状态机持久化、幂等去重与崩溃恢复。
//!
//! 从原生 `kernel.rs` 拆出。`mod.rs` 持有所有类型 / trait / enum / struct 定义、风控契约以及
//! 子模块共用的私有辅助函数（`signal_to_str`、`fill_details_json`）；大块
//! `ExecutionKernel` 实现拆入 [`core`](crate::execution::kernel::core) 子模块，
//! 崩溃恢复拆入 [`recovery`](crate::execution::kernel::recovery)。

mod core;
mod recovery;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::core::Result;
use crate::core::signal::Signal;
use crate::execution::models::{
    ExecutionPolicy, FillDeltaContext, FillDeltaResult, FillDetails, OrderIntent, OrderStatus,
};
use crate::execution::runtime_store::StrategyRuntimeStore;

/// 风控决策：Allow 放行、Reject 拒绝（含原因）。由 RiskEvaluator 返回，决定 kernel 是否提交订单到 adapter。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskDecision {
    Allow,
    Reject { reason: String },
}

/// 风控评估器 trait：evaluate 在下单前评估 intent 决定是否放行；sync_after_fill 在成交后同步风控内部状态（如持仓/当日基线）。
#[async_trait]
pub trait RiskEvaluator: Send + Sync {
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision>;

    async fn sync_after_fill(&self) -> Result<()>;
}

/// 成交增量应用 trait：apply_fill_delta 根据 new/old filled_quantity 差值计算本轮成交增量，并产生可选 trade_record（用于 paper/回测的持仓与账务联动）。
#[async_trait]
pub trait FillDeltaApplier: Send + Sync {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult>;
}

/// 空实现的 FillDeltaApplier：计算 delta 但不产生 trade_record_id；用于不需要 broker 价差模拟的执行通道（如 qmt_live 直接走真实 broker）。
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopFillDeltaApplier;

#[async_trait]
impl FillDeltaApplier for NoopFillDeltaApplier {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        let delta_quantity = (ctx.new_filled_quantity - ctx.old_filled_quantity).max(0);
        Ok(FillDeltaResult {
            applied: delta_quantity > 0,
            delta_quantity,
            trade_record_id: None,
        })
    }
}

/// 执行请求（来自策略信号）：run_id 策略运行 ID、strategy_name/mode/trigger 策略与触发上下文、symbol/timeframe 标的与周期、bar_end/bar 截止时间、market_price 信号价、held_volume 持仓、policy 执行策略、client_order_id 幂等键。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRunRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub market_price: rust_decimal::Decimal,
    pub held_volume: Option<i64>,
    pub policy: ExecutionPolicy,
    pub client_order_id: String,
}

/// 已解析出的执行请求（含 intent）：由 ExecutionRunRequest 经信号翻译 + risk gate 前构造；execute_request 接收此结构走完整下单事务。
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedExecutionRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub signal: Signal,
    pub signal_payload_json: serde_json::Value,
    pub intent: OrderIntent,
    pub client_order_id: String,
}

/// 执行内核结果：run_id、signal 信号、order_status 可选订单终态、client_order_id 可选幂等键。order_status 为 None 表示信号被 hold/无 intent，未产生订单。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelExecutionResult {
    pub run_id: String,
    pub signal: Signal,
    pub order_status: Option<OrderStatus>,
    pub client_order_id: Option<String>,
}

/// 崩溃恢复扫描汇总：scanned 扫描数、recovered 恢复数、unchanged 未变数、failed 失败数、skipped 跳过数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// 执行内核：组合 StrategyRuntimeStore + ExecutionAdapter + FillDeltaApplier + RiskEvaluator，提供 execute_once/execute_request 入口，承担信号翻译、风控、下单、成交增量应用、状态机持久化、幂等去重等完整事务。
#[derive(Debug, Clone)]
pub struct ExecutionKernel<A, F, R> {
    store: StrategyRuntimeStore,
    adapter: A,
    fill_delta: F,
    risk: R,
}

fn signal_to_str(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

fn fill_details_json(fill_details: Option<&FillDetails>) -> Value {
    match fill_details {
        Some(fill) => serde_json::json!({
            "fill_id": fill.fill_id,
            "fill_quantity": fill.fill_quantity,
            "fill_price": fill.fill_price,
        }),
        None => Value::Null,
    }
}
