use super::*;

impl ClickHouseClient {
    /// 批量写入市场基础面快照。
    ///
    /// 该表作为本地 ETL 落盘层，为后续强弱板块内个股排序提供稳定数据源。
    pub async fn insert_market_fundamental_snapshots(
        &self,
        snapshots: &[MarketFundamentalSnapshotCH],
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
}
