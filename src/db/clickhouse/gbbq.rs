use super::*;

impl ClickHouseClient {
    /// 插入 GBBQ 事件
    pub async fn insert_gbbq_event(&self, event: &crate::data::models::GbbqEvent) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO gbbq_events (
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date, market
            ) VALUES
            ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {})
            "#,
            event.event_date,
            event.code,
            event.category,
            event.dividend,
            event.bonus_price,
            event.bonus_share,
            event.rights_share,
            event
                .ex_price
                .map(|v| v.to_string())
                .unwrap_or("NULL".to_string()),
            event
                .record_date
                .map(|d| d.to_string())
                .unwrap_or("NULL".to_string()),
            if event.code.starts_with('6') || event.code.starts_with('5') {
                1u8
            } else {
                0u8
            }
        );

        self.client
            .query(&sql)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入 GBBQ 事件失败: {}", e)))?;

        Ok(())
    }

    /// 批量插入 GBBQ 事件 (优化版本)
    pub async fn insert_gbbq_events(
        &self,
        events: &[crate::data::models::GbbqEvent],
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条 GBBQ 事件", events.len());

        for chunk in events.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("gbbq_events")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for event in chunk {
                let row = GbbqEventCH {
                    event_date: event.event_date,
                    code: event.code.clone(),
                    category: event.category,
                    dividend: event.dividend,
                    bonus_price: event.bonus_price,
                    bonus_share: event.bonus_share,
                    rights_share: event.rights_share,
                    ex_price: event.ex_price,
                    record_date: event.record_date,
                };
                insert.write(&row).await.map_err(|e| {
                    QuantixError::DatabaseQuery(format!("写入 GBBQ 事件失败: {}", e))
                })?;
            }

            insert.end().await.map_err(|e| {
                QuantixError::DatabaseQuery(format!("批量插入 GBBQ 事件失败: {}", e))
            })?;

            debug!("成功插入 {} 条 GBBQ 事件", chunk.len());
        }

        Ok(())
    }

    /// 查询股票的 GBBQ 事件
    pub async fn get_gbbq_events(
        &self,
        code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<crate::data::models::GbbqEvent>> {
        let mut where_clause = format!("code = '{}'", code);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND event_date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND event_date <= '{}'", end));
        }

        let sql = format!(
            r#"
            SELECT
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date
            FROM gbbq_events
            WHERE {}
            ORDER BY event_date ASC
            "#,
            where_clause
        );

        let rows: Vec<GbbqEventCH> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询 GBBQ 事件失败: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::data::models::GbbqEvent {
                code: r.code,
                event_date: r.event_date,
                category: r.category,
                dividend: r.dividend,
                bonus_price: r.bonus_price,
                bonus_share: r.bonus_share,
                rights_share: r.rights_share,
                ex_price: r.ex_price,
                record_date: r.record_date,
            })
            .collect())
    }

    /// 获取股票的最新除权除息事件
    pub async fn get_latest_gbbq_event(
        &self,
        code: &str,
    ) -> Result<Option<crate::data::models::GbbqEvent>> {
        let sql = format!(
            r#"
            SELECT
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date
            FROM gbbq_events
            WHERE code = '{}' AND category = 1
            ORDER BY event_date DESC
            LIMIT 1
            "#,
            code
        );

        let rows: Vec<GbbqEventCH> =
            self.client.query(&sql).fetch_all().await.map_err(|e| {
                QuantixError::DatabaseQuery(format!("查询最新 GBBQ 事件失败: {}", e))
            })?;

        Ok(rows
            .into_iter()
            .next()
            .map(|r| crate::data::models::GbbqEvent {
                code: r.code,
                event_date: r.event_date,
                category: r.category,
                dividend: r.dividend,
                bonus_price: r.bonus_price,
                bonus_share: r.bonus_share,
                rights_share: r.rights_share,
                ex_price: r.ex_price,
                record_date: r.record_date,
            }))
    }
}
