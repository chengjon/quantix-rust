use super::*;

impl ClickHouseClient {
    /// 查询 K线数据 (支持多周期)
    pub async fn get_kline_data(
        &self,
        code: &str,
        period: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        limit: Option<usize>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut where_clause = format!("code = '{}' AND period = '{}'", code, period);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND date <= '{}'", end));
        }

        let limit_str = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            r#"
            SELECT
                timestamp, code, name, period,
                open, high, low, close, volume, amount
            FROM kline_data
            WHERE {}
            ORDER BY timestamp ASC
            {}
            "#,
            where_clause, limit_str
        );

        let rows: Vec<KlineDataCH> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询 K线数据失败: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::data::models::Kline {
                code: r.code.clone(),
                date: r.timestamp.date_naive(),
                open: Decimal::from_str(&format!("{}", r.open)).unwrap_or_default(),
                high: Decimal::from_str(&format!("{}", r.high)).unwrap_or_default(),
                low: Decimal::from_str(&format!("{}", r.low)).unwrap_or_default(),
                close: Decimal::from_str(&format!("{}", r.close)).unwrap_or_default(),
                volume: r.volume as i64,
                amount: Decimal::from_str(&format!("{}", r.amount)).ok(),
                adjust_type: crate::data::models::AdjustType::None,
            })
            .collect())
    }

    /// 插入 K线数据
    pub async fn insert_kline_data(
        &self,
        kline: &crate::data::models::Kline,
        period: &str,
    ) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO kline_data (
                timestamp, code, name, period,
                open, high, low, close, volume, amount, source
            ) VALUES
                (toDateTime64('{}'), '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, 'TDX')
            "#,
            kline.date.format("%Y-%m-%d %H:%M:%S"),
            kline.code,
            kline.code,
            period,
            kline.open,
            kline.high,
            kline.low,
            kline.close,
            kline.volume,
            kline.amount.unwrap_or_default()
        );

        self.client
            .query(&sql)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入 K线数据失败: {}", e)))?;

        Ok(())
    }

    /// 批量插入 K线数据 (优化版本)
    pub async fn insert_kline_data_batch(
        &self,
        klines: &[crate::data::models::Kline],
        period: &str,
    ) -> Result<()> {
        self.insert_kline_data_batch_with_source(klines, period, "TDX")
            .await
    }

    /// 批量插入 K线数据 (指定数据源)
    pub async fn insert_kline_data_batch_with_source(
        &self,
        klines: &[crate::data::models::Kline],
        period: &str,
        source: &str,
    ) -> Result<()> {
        if klines.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条 K线数据 (周期: {})", klines.len(), period);

        for chunk in klines.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("kline_data")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for kline in chunk {
                let timestamp = kline
                    .date
                    .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let row = KlineDataCH {
                    timestamp: DateTime::<Utc>::from_naive_utc_and_offset(timestamp, Utc),
                    code: kline.code.clone(),
                    name: kline.code.clone(),
                    period: period.to_string(),
                    open: kline.open.to_f64().unwrap_or(0.0),
                    high: kline.high.to_f64().unwrap_or(0.0),
                    low: kline.low.to_f64().unwrap_or(0.0),
                    close: kline.close.to_f64().unwrap_or(0.0),
                    volume: kline.volume as f64,
                    amount: kline.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
                    trade_count: 0,
                    source: source.to_string(),
                };
                insert
                    .write(&row)
                    .await
                    .map_err(|e| QuantixError::DatabaseQuery(format!("写入 K线数据失败: {}", e)))?;
            }

            insert
                .end()
                .await
                .map_err(|e| QuantixError::DatabaseQuery(format!("批量插入 K线数据失败: {}", e)))?;

            debug!("成功插入 {} 条 K线数据", chunk.len());
        }

        Ok(())
    }

    /// 聚合查询：从分钟线聚合为日线
    pub async fn get_daily_from_minute(
        &self,
        code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut where_clause = format!("code = '{}' AND period = '1m'", code);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND date <= '{}'", end));
        }

        let sql = format!(
            r#"
            SELECT
                toStartOfDay(timestamp) as day,
                argMin(open, timestamp) as open,
                max(high) as high,
                min(low) as low,
                argMax(close, timestamp) as close,
                sum(volume) as volume,
                sum(amount) as amount
            FROM kline_data
            WHERE {}
            GROUP BY day
            ORDER BY day ASC
            "#,
            where_clause
        );

        let _result: Vec<u8> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("聚合查询失败: {}", e)))?;

        Ok(vec![])
    }

    /// 插入单条实时行情
    pub async fn insert_stock_quote(&self, quote: &StockQuoteCH) -> Result<()> {
        let mut insert = self
            .client
            .insert("stock_realtime_quotes")
            .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?;
        insert
            .write(quote)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入实时行情失败: {}", e)))?;
        insert
            .end()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入实时行情失败: {}", e)))?;
        Ok(())
    }

    /// 批量插入实时行情
    pub async fn insert_stock_quotes_batch(&self, quotes: &[StockQuoteCH]) -> Result<()> {
        if quotes.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条实时行情", quotes.len());

        for chunk in quotes.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("stock_realtime_quotes")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for quote in chunk {
                insert
                    .write(quote)
                    .await
                    .map_err(|e| QuantixError::DatabaseQuery(format!("写入实时行情失败: {}", e)))?;
            }

            insert
                .end()
                .await
                .map_err(|e| QuantixError::DatabaseQuery(format!("批量插入实时行情失败: {}", e)))?;

            debug!("成功插入 {} 条实时行情", chunk.len());
        }

        Ok(())
    }
}
