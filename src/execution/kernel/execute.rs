use super::*;

impl<A, F, R> ExecutionKernel<A, F, R>
where
    A: ExecutionAdapter,
    F: FillDeltaApplier,
    R: RiskEvaluator,
{
    pub async fn execute_request(
        &self,
        request: PreparedExecutionRequest,
    ) -> Result<KernelExecutionResult> {
        let now = Utc::now();
        self.store
            .insert_run(&StrategyRunRecord {
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                mode: request.mode.clone(),
                trigger: request.trigger.clone(),
                status: StrategyRunStatus::Running,
                symbol: request.symbol.clone(),
                timeframe: request.timeframe.clone(),
                bar_end: request.bar_end,
                started_at: now,
                finished_at: None,
                metadata_json: serde_json::json!({}),
            })
            .await?;

        self.store
            .insert_signal_event(&SignalEventRecord {
                event_id: Uuid::new_v4().to_string(),
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                symbol: request.symbol.clone(),
                signal: signal_to_str(request.signal).to_string(),
                ts: now,
                payload_json: request.signal_payload_json.clone(),
            })
            .await?;

        self.execute_prepared_order_flow(request).await
    }

    pub async fn execute_once(
        &self,
        request: ExecutionRunRequest,
        envelope: SignalEnvelope,
    ) -> Result<KernelExecutionResult> {
        if let Some(existing_run) = self
            .store
            .find_run_by_dedupe_key(
                &request.strategy_name,
                &request.mode,
                &request.symbol,
                &request.timeframe,
                request.bar_end,
            )
            .await?
        {
            let existing_order = self
                .store
                .find_first_order_for_run(&existing_run.run_id)
                .await?;
            return Ok(KernelExecutionResult {
                run_id: existing_run.run_id,
                signal: envelope.signal,
                order_status: existing_order.as_ref().map(|order| order.status),
                client_order_id: existing_order.map(|order| order.client_order_id),
            });
        }

        let now = Utc::now();
        self.store
            .insert_run(&StrategyRunRecord {
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                mode: request.mode.clone(),
                trigger: request.trigger.clone(),
                status: StrategyRunStatus::Running,
                symbol: request.symbol.clone(),
                timeframe: request.timeframe.clone(),
                bar_end: request.bar_end,
                started_at: now,
                finished_at: None,
                metadata_json: serde_json::json!({}),
            })
            .await?;

        self.store
            .insert_signal_event(&SignalEventRecord {
                event_id: Uuid::new_v4().to_string(),
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                symbol: request.symbol.clone(),
                signal: signal_to_str(envelope.signal).to_string(),
                ts: now,
                payload_json: envelope.metadata_json.clone(),
            })
            .await?;

        let maybe_intent = translate_signal(
            &envelope,
            &request.symbol,
            request.market_price,
            request.held_volume,
            &request.policy,
        )?;

        let Some(intent) = maybe_intent else {
            self.store
                .update_run_status(
                    &request.run_id,
                    StrategyRunStatus::Success,
                    Some(Utc::now()),
                )
                .await?;
            return Ok(KernelExecutionResult {
                run_id: request.run_id,
                signal: envelope.signal,
                order_status: None,
                client_order_id: None,
            });
        };

        self.execute_prepared_order_flow(PreparedExecutionRequest {
            run_id: request.run_id,
            strategy_name: request.strategy_name,
            mode: request.mode,
            trigger: request.trigger,
            symbol: request.symbol,
            timeframe: request.timeframe,
            bar_end: request.bar_end,
            signal: envelope.signal,
            signal_payload_json: envelope.metadata_json,
            intent,
            client_order_id: request.client_order_id,
        })
        .await
    }
}
