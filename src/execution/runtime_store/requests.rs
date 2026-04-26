use super::*;

impl StrategyRuntimeStore {
    pub async fn approve_signal_and_create_request(
        &self,
        signal_id: &str,
        target_mode: &str,
        target_account: &str,
        approved_by: Option<&str>,
    ) -> Result<ExecutionRequestRecord> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let signal_row = sqlx::query(
            r#"
SELECT
    signal_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    bar_end,
    signal_value,
    signal_status,
    approval_status,
    run_id,
    metadata_json,
    created_at,
    updated_at
FROM signals
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(signal_row) = signal_row else {
            return Err(QuantixError::Other(format!("signal 不可审批: {signal_id}")));
        };
        let signal = Self::row_to_signal(signal_row)?;

        let update = sqlx::query(
            r#"
UPDATE signals
SET approval_status = ?, updated_at = ?
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(ApprovalStatus::Approved.as_str())
        .bind(now.to_rfc3339())
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .execute(&mut *tx)
        .await?;

        if update.rows_affected() != 1 {
            return Err(QuantixError::Other(format!("signal 不可审批: {signal_id}")));
        }

        let record = ExecutionRequestRecord {
            request_id: Uuid::new_v4().to_string(),
            signal_id: signal_id.to_string(),
            target_mode: target_mode.to_string(),
            target_account: target_account.to_string(),
            request_status: ExecutionRequestStatus::Pending,
            approved_by: approved_by.map(str::to_string),
            created_at: now,
            updated_at: now,
            payload_json: serde_json::json!({
                "execution_snapshot": build_execution_snapshot(&signal)?,
            }),
        };

        sqlx::query(
            r#"
INSERT INTO execution_requests (
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.request_id)
        .bind(&record.signal_id)
        .bind(&record.target_mode)
        .bind(&record.target_account)
        .bind(record.request_status.as_str())
        .bind(&record.approved_by)
        .bind(record.created_at.to_rfc3339())
        .bind(record.updated_at.to_rfc3339())
        .bind(serde_json::to_string(&record.payload_json)?)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    pub async fn reject_signal(&self, signal_id: &str, reason: Option<&str>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
SELECT metadata_json
FROM signals
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            return Err(QuantixError::Other(format!("signal 不可拒绝: {signal_id}")));
        };

        let metadata_json: String = row.try_get("metadata_json")?;
        let mut metadata: serde_json::Value = serde_json::from_str(&metadata_json)?;
        if let Some(reason) = reason {
            metadata["rejection_reason"] = serde_json::Value::String(reason.to_string());
        }

        sqlx::query(
            r#"
UPDATE signals
SET approval_status = ?, metadata_json = ?, updated_at = ?
WHERE signal_id = ?
"#,
        )
        .bind(ApprovalStatus::Rejected.as_str())
        .bind(serde_json::to_string(&metadata)?)
        .bind(Utc::now().to_rfc3339())
        .bind(signal_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_execution_requests(
        &self,
        status: Option<ExecutionRequestStatus>,
    ) -> Result<Vec<ExecutionRequestRecord>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_status = ?
ORDER BY created_at ASC, request_id ASC
"#,
            )
            .bind(status.as_str())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
ORDER BY created_at ASC, request_id ASC
"#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(Self::row_to_execution_request)
            .collect()
    }

    pub async fn get_execution_request_by_signal_id(
        &self,
        signal_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE signal_id = ?
"#,
        )
        .bind(signal_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn get_execution_request(
        &self,
        request_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_id = ?
"#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn try_complete_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::InProgress,
            ExecutionRequestStatus::Completed,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn try_fail_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::InProgress,
            ExecutionRequestStatus::Failed,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn try_cancel_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::Pending,
            ExecutionRequestStatus::Canceled,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn update_execution_request_status(
        &self,
        request_id: &str,
        status: ExecutionRequestStatus,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
UPDATE execution_requests
SET request_status = ?, updated_at = ?, payload_json = ?
WHERE request_id = ?
"#,
        )
        .bind(status.as_str())
        .bind(updated_at.to_rfc3339())
        .bind(serde_json::to_string(&payload_json)?)
        .bind(request_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn try_start_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::Pending,
            ExecutionRequestStatus::InProgress,
            payload_json,
            updated_at,
        )
        .await
    }

    async fn try_update_execution_request_status(
        &self,
        request_id: &str,
        expected_status: ExecutionRequestStatus,
        target_status: ExecutionRequestStatus,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE execution_requests
SET request_status = ?, updated_at = ?, payload_json = ?
WHERE request_id = ? AND request_status = ?
"#,
        )
        .bind(target_status.as_str())
        .bind(updated_at.to_rfc3339())
        .bind(serde_json::to_string(&payload_json)?)
        .bind(request_id)
        .bind(expected_status.as_str())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn find_next_pending_execution_request(
        &self,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_status = ?
ORDER BY created_at ASC, request_id ASC
LIMIT 1
"#,
        )
        .bind(ExecutionRequestStatus::Pending.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn supersede_previous_signals_and_cancel_pending_requests(
        &self,
        strategy_instance_id: &str,
        symbol: &str,
        timeframe: &str,
        current_signal_id: &str,
        current_bar_end: DateTime<Utc>,
    ) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
        let candidate_rows = sqlx::query(
            r#"
SELECT signal_id
FROM signals
WHERE strategy_instance_id = ?
  AND symbol = ?
  AND timeframe = ?
  AND signal_id <> ?
  AND signal_status = ?
  AND bar_end < ?
"#,
        )
        .bind(strategy_instance_id)
        .bind(symbol)
        .bind(timeframe)
        .bind(current_signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(current_bar_end.to_rfc3339())
        .fetch_all(&mut *tx)
        .await?;

        let signal_ids: Vec<String> = candidate_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("signal_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for signal_id in &signal_ids {
            sqlx::query(
                r#"
UPDATE signals
SET signal_status = ?, updated_at = ?
WHERE signal_id = ?
"#,
            )
            .bind(SignalStatus::Superseded.as_str())
            .bind(Utc::now().to_rfc3339())
            .bind(signal_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
UPDATE execution_requests
SET request_status = ?, updated_at = ?
WHERE signal_id = ? AND request_status = ?
"#,
            )
            .bind(ExecutionRequestStatus::Canceled.as_str())
            .bind(Utc::now().to_rfc3339())
            .bind(signal_id)
            .bind(ExecutionRequestStatus::Pending.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(signal_ids.len())
    }
}
