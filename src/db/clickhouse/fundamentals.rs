use super::*;
use chrono::NaiveDate;

impl ClickHouseClient {
    /// 批量写入市场基础面快照。
    ///
    /// 该表作为本地 ETL 落盘层，为后续强弱板块内个股排序提供稳定数据源。
    pub async fn insert_market_fundamental_snapshots(
        &self,
        snapshots: &[MarketFundamentalSnapshotInsertCH],
    ) -> Result<()> {
        if snapshots.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条市场基础面快照", snapshots.len());

        for chunk in snapshots.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("market_fundamentals_daily")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for snapshot in chunk {
                insert.write(snapshot).await.map_err(|e| {
                    QuantixError::DatabaseQuery(format!("写入市场基础面快照失败: {}", e))
                })?;
            }

            insert.end().await.map_err(|e| {
                QuantixError::DatabaseQuery(format!("批量插入市场基础面快照失败: {}", e))
            })?;

            debug!("成功插入 {} 条市场基础面快照", chunk.len());
        }

        Ok(())
    }

    /// 查询指定股票代码在某日之前最近可用的市场基础面快照。
    pub async fn get_latest_market_fundamental_snapshots(
        &self,
        codes: &[String],
        as_of: Option<NaiveDate>,
    ) -> Result<Vec<MarketFundamentalSnapshotCH>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        let sql = build_latest_market_fundamental_snapshots_sql(codes, as_of);

        self.query_json(&sql).await
    }
}

fn build_latest_market_fundamental_snapshots_sql(
    codes: &[String],
    as_of: Option<NaiveDate>,
) -> String {
    let code_list = codes
        .iter()
        .map(|code| format!("'{}'", code.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");

    let date_clause = as_of
        .map(|date| format!(" AND snapshot_date <= '{date}'"))
        .unwrap_or_default();

    format!(
        r#"
            SELECT
                code,
                max(snapshot_date) AS latest_snapshot_date,
                argMax(market_cap, snapshot_date) AS market_cap,
                argMax(latest_report_profit, snapshot_date) AS latest_report_profit,
                argMax(profit_source, snapshot_date) AS profit_source,
                argMax(pe_dynamic, snapshot_date) AS pe_dynamic,
                formatDateTime(max(updated_at), '%F %T') AS updated_at
            FROM market_fundamentals_daily
            WHERE code IN ({code_list}){date_clause}
            GROUP BY code
            "#
    )
}

#[cfg(test)]
mod tests {
    use super::build_latest_market_fundamental_snapshots_sql;
    use chrono::NaiveDate;

    #[test]
    fn latest_market_fundamentals_sql_avoids_snapshot_date_alias_collision() {
        let sql = build_latest_market_fundamental_snapshots_sql(
            &["600519".to_string(), "601398".to_string()],
            Some(NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()),
        );

        assert!(sql.contains("max(snapshot_date) AS latest_snapshot_date"));
        assert!(sql.contains("argMax(market_cap, snapshot_date) AS market_cap"));
        assert!(
            sql.contains("WHERE code IN ('600519', '601398') AND snapshot_date <= '2026-03-14'")
        );
        assert!(!sql.contains("max(snapshot_date) AS snapshot_date"));
    }
}
