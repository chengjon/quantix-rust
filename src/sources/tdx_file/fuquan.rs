use super::*;

// ============================================================================
// 复权计算
// ============================================================================

/// 复权因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuquanFactor {
    /// 日期
    pub date: NaiveDate,
    /// 复权因子
    pub factor: f64,
    /// 前收盘价
    pub preclose: f64,
    /// 收盘价
    pub close: f64,
    /// 是否为交易日
    pub trading: bool,
    /// 是否为除权日
    pub xdxr: bool,
}

/// 复权类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuquanType {
    /// 不复权
    None,
    /// 前复权
    QFQ,
    /// 后复权
    HFQ,
}

/// 复权计算器
pub struct FuquanCalculator;

impl FuquanCalculator {
    /// 计算复权因子（使用涨跌幅算法）
    ///
    /// 算法说明:
    /// - 基于每日涨跌幅连续计算复权因子
    /// - factor = factor * (close / preclose)
    /// - 除权日需要调整前收盘价
    pub fn calculate(
        days: &[TdxDayRecord],
        gbbqs: Option<&[TdxGbbqRecord]>,
    ) -> Result<Vec<FuquanFactor>> {
        if days.is_empty() {
            return Ok(Vec::new());
        }

        let mut factors = Vec::with_capacity(days.len());
        let mut preclose = days[0].close as f64;
        let mut factor = 1.0;

        let mut gbbq_iter = gbbqs.map(|g| g.iter()).unwrap_or([].iter()).peekable();

        let mut current_xdxr = gbbq_iter.peek().copied();

        for day in days {
            let close = day.close as f64;

            // 检查是否有除权事件
            let mut xdxr = false;
            if let Some(xdxr_record) = current_xdxr {
                if day.date == xdxr_record.date {
                    // 除权日
                    let [new_preclose, _, _] =
                        xdxr_record.compute_pre_pct(day.close, preclose, true);
                    preclose = new_preclose;
                    xdxr = true;

                    // 移动到下一个除权记录
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                } else if day.date > xdxr_record.date {
                    // 跳过已经过的除权日（非交易日）
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                }
            }

            // 计算复权因子
            factor *= close / preclose;
            preclose = close;

            factors.push(FuquanFactor {
                date: day.naive_date().unwrap_or_else(fallback_date),
                factor,
                preclose: close,
                close,
                trading: true,
                xdxr,
            });
        }

        Ok(factors)
    }

    /// 获取最新的复权因子状态（用于增量更新）
    pub fn get_latest_factor(factors: &[FuquanFactor]) -> Option<(f64, f64)> {
        factors.last().map(|f| (f.close, f.factor))
    }

    /// 应用前复权
    pub fn apply_qfq(kline: &Kline, factor: f64, latest_factor: f64) -> Kline {
        let adj_factor = latest_factor / factor;
        let adj_dec = Decimal::from_f64(adj_factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::QFQ,
            ..kline.clone()
        }
    }

    /// 应用后复权
    pub fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
        let adj_dec = Decimal::from_f64(factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::HFQ,
            ..kline.clone()
        }
    }
}

// ============================================================================
// 集成类型 - 完整的日线数据（含复权）
// ============================================================================

/// 完整的日线数据（含复权信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdxDayData {
    /// 股票代码
    pub code: String,
    /// 日期
    pub date: NaiveDate,
    /// 开盘价
    pub open: Decimal,
    /// 最高价
    pub high: Decimal,
    /// 最低价
    pub low: Decimal,
    /// 收盘价
    pub close: Decimal,
    /// 成交量
    pub volume: i64,
    /// 成交额
    pub amount: Decimal,
    /// 前收盘价
    pub preclose: Decimal,
    /// 复权因子
    pub factor: Decimal,
    /// 涨跌幅
    pub change_pct: Decimal,
}

impl TdxDayData {
    /// 从 day 记录和复权因子创建
    pub fn from_record(record: &TdxDayRecord, factor: &FuquanFactor) -> Self {
        let change_pct = if factor.preclose > 0.0 {
            Decimal::from_f64((factor.close - factor.preclose) / factor.preclose * 100.0)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        Self {
            code: record.code_string(),
            date: record.naive_date().unwrap_or_else(fallback_date),
            open: Decimal::from_f32(record.open)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            high: Decimal::from_f32(record.high)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            low: Decimal::from_f32(record.low)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            close: Decimal::from_f32(record.close)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            volume: record.volume as i64,
            amount: Decimal::from_f32(record.amount)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            preclose: Decimal::from_f64(factor.preclose)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            factor: Decimal::from_f64(factor.factor)
                .map(|d| d.round_dp(6))
                .unwrap_or(Decimal::ONE),
            change_pct,
        }
    }

    /// 转换为 Kline
    pub fn to_kline(&self, adjust_type: AdjustType) -> Kline {
        Kline {
            code: self.code.clone(),
            date: self.date,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            amount: Some(self.amount),
            adjust_type,
        }
    }
}

// ============================================================================
// 批量导入
// ============================================================================

/// TDX 数据批量导入器
pub struct TdxDataImporter {
    /// 数据目录
    data_dir: String,
}

impl TdxDataImporter {
    /// 创建导入器
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// 导入单个股票的日线数据
    pub fn import_stock_day(
        &self,
        code: &str,
        gbbqs: Option<&[TdxGbbqRecord]>,
    ) -> Result<Vec<TdxDayData>> {
        let code_num = code.parse::<u32>().map_err(|_| {
            crate::core::QuantixError::DataParse(format!("无效的股票代码: {}", code))
        })?;

        let day_path = format!("{}/{}.day", self.data_dir, code);
        let records = TdxDayFile::from_file(code_num, &day_path)?;

        let factors = FuquanCalculator::calculate(&records, gbbqs)?;

        Ok(records
            .iter()
            .zip(factors.iter())
            .map(|(r, f)| TdxDayData::from_record(r, f))
            .collect())
    }

    /// 批量导入多只股票
    pub fn import_batch(
        &self,
        codes: &[String],
        gbbq_map: &HashMap<String, Vec<TdxGbbqRecord>>,
    ) -> Result<HashMap<String, Vec<TdxDayData>>> {
        let mut result = HashMap::new();

        for code in codes {
            let gbbqs = gbbq_map.get(code).map(|v| v.as_slice());
            match self.import_stock_day(code, gbbqs) {
                Ok(data) => {
                    if !data.is_empty() {
                        result.insert(code.clone(), data);
                    }
                }
                Err(e) => {
                    tracing::warn!("导入 {} 失败: {}", code, e);
                }
            }
        }

        Ok(result)
    }
}
