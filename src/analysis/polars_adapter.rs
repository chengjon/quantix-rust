/// Polars 统一数据管理层
///
/// 使用 Polars 进行所有数据处理，无论数据量大小
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

type IndicatorBatchResult = HashMap<String, HashMap<String, Vec<Option<Decimal>>>>;

/// 全局 Polars 配置
pub fn init_polars() -> Result<()> {
    // 设置全局线程数（等于 CPU 核心数）
    let num_threads = std::thread::available_parallelism()
        .map_err(|e| QuantixError::Other(format!("获取 CPU 核心数失败: {}", e)))?;
    // Polars 0.43 通过环境变量控制线程数
    unsafe { std::env::set_var("POLARS_MAX_THREADS", num_threads.to_string()) };
    tracing::info!("Polars 初始化完成，使用 {} 线程", num_threads);
    Ok(())
}

/// K线数据批量结构
#[derive(Debug, Clone)]
pub struct BatchKlineData {
    /// 股票代码
    pub code: String,
    /// 时间戳
    pub timestamps: Vec<i64>,
    /// 开盘价
    pub open: Vec<f64>,
    /// 最高价
    pub high: Vec<f64>,
    /// 最低价
    pub low: Vec<f64>,
    /// 收盘价
    pub close: Vec<f64>,
    /// 成交量
    pub volume: Vec<i64>,
    /// 成交额
    pub amount: Vec<f64>,
}

impl BatchKlineData {
    /// 获取数据行数
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }

    /// 转换为 Decimal 数组
    pub fn close_as_decimal(&self) -> Vec<Decimal> {
        self.close
            .iter()
            .map(|&v| Decimal::from_str(&format!("{}", v)).unwrap_or_default())
            .collect()
    }

    /// 获取指定列
    pub fn get_column(&self, col: &str) -> Vec<f64> {
        match col {
            "open" => self.open.clone(),
            "high" => self.high.clone(),
            "low" => self.low.clone(),
            "close" => self.close.clone(),
            "volume" => self.volume.iter().map(|&v| v as f64).collect(),
            "amount" => self.amount.clone(),
            _ => vec![],
        }
    }

    /// 获取列名列表
    pub fn columns(&self) -> Vec<&'static str> {
        vec!["open", "high", "low", "close", "volume", "amount"]
    }
}

/// 统一指标计算器 (基于 Polars)
pub struct PolarsCalculator {
    _private: (),
}

impl PolarsCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// 计算移动平均线
    pub fn ma(&self, data: &BatchKlineData, period: usize) -> Vec<Option<Decimal>> {
        use polars::prelude::*;

        // 使用 PlSmallStr 创建 Series 名称
        let name = PlSmallStr::from("close");
        let s = Series::new(name, &data.close);

        // 使用 rolling_mean (Polars 0.43 - 只接受 1 个参数)
        let opts = RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            center: false,
            ..Default::default()
        };

        match s.rolling_mean(opts) {
            Ok(result) => {
                // 使用 DataType::Float64 进行 cast
                match result.cast(&DataType::Float64) {
                    Ok(casted) => {
                        // 使用 iter() 转换为 Vec
                        let values: Vec<Option<f64>> =
                            casted.iter().map(|av| av.extract::<f64>()).collect();
                        values
                            .into_iter()
                            .map(|v| v.and_then(|f| Decimal::from_str(&format!("{}", f)).ok()))
                            .collect()
                    }
                    Err(_) => vec![None; data.len()],
                }
            }
            Err(_) => vec![None; data.len()],
        }
    }

    /// 计算指数移动平均线
    pub fn ema(&self, data: &BatchKlineData, period: usize) -> Vec<Option<Decimal>> {
        use polars::prelude::*;

        let name = PlSmallStr::from("close");
        let _s = Series::new(name, &data.close);
        // EMA 需要 custom rolling window
        let alpha = Decimal::from(2) / (Decimal::from(period as i64) + Decimal::ONE);
        let alpha_f64 = alpha.to_f64().unwrap_or(2.0 / (period as f64 + 1.0));

        let mut result = vec![None; data.len()];
        if data.len() < period {
            return result;
        }

        // 初始 SMA
        let mut sum = 0.0;
        for i in 0..period {
            sum += data.close[i];
        }
        let mut ema_val = sum / period as f64;
        result[period - 1] = Some(Decimal::from_str(&format!("{}", ema_val)).unwrap_or_default());

        for (slot, close) in result.iter_mut().skip(period).zip(data.close.iter().skip(period)) {
            ema_val = *close * alpha_f64 + ema_val * (1.0 - alpha_f64);
            *slot = Some(Decimal::from_str(&format!("{}", ema_val)).unwrap_or_default());
        }

        result
    }

    /// 计算 RSI
    pub fn rsi(&self, data: &BatchKlineData, period: usize) -> Vec<Option<Decimal>> {
        // RSI 计算较复杂，保持当前实现
        crate::analysis::indicators::rsi(&data.close_as_decimal(), period)
    }

    /// 计算 MACD
    pub fn macd(
        &self,
        data: &BatchKlineData,
        fast: usize,
        slow: usize,
        signal: usize,
    ) -> Vec<Option<crate::analysis::indicators::Macd>> {
        crate::analysis::indicators::macd(&data.close_as_decimal(), fast, slow, signal)
    }

    /// 计算 KDJ
    pub fn kdj(
        &self,
        data: &BatchKlineData,
        n: usize,
        m1: usize,
        m2: usize,
    ) -> Vec<Option<crate::analysis::indicators::Kdj>> {
        crate::analysis::indicators::kdj(
            &data
                .high
                .iter()
                .map(|&v| Decimal::from_str(&format!("{}", v)).unwrap_or_default())
                .collect::<Vec<_>>(),
            &data
                .low
                .iter()
                .map(|&v| Decimal::from_str(&format!("{}", v)).unwrap_or_default())
                .collect::<Vec<_>>(),
            &data.close_as_decimal(),
            n,
            m1,
            m2,
        )
    }

    /// 计算布林带
    pub fn bollinger_bands(
        &self,
        data: &BatchKlineData,
        period: usize,
        std_dev: usize,
    ) -> Vec<Option<crate::analysis::indicators::BollingerBands>> {
        crate::analysis::indicators::bollinger_bands(&data.close_as_decimal(), period, std_dev)
    }

    /// 计算多个指标 (批量优化)
    pub fn calculate_batch(
        &self,
        data: &BatchKlineData,
        indicators: &[&str],
    ) -> HashMap<String, Vec<Option<Decimal>>> {
        use polars::prelude::*;

        // 构建 DataFrame (Polars 0.43)
        let df_result: PolarsResult<DataFrame> = df!(
            PlSmallStr::from("open") => &data.open,
            PlSmallStr::from("high") => &data.high,
            PlSmallStr::from("low") => &data.low,
            PlSmallStr::from("close") => &data.close,
            PlSmallStr::from("volume") => &data.volume.iter().map(|&v| v as f64).collect::<Vec<_>>(),
        );

        let mut result = HashMap::new();

        // 批量计算所有 MA 指标
        let ma_periods: Vec<usize> = indicators
            .iter()
            .filter_map(|s| s.strip_prefix("ma").and_then(|p| p.parse().ok()))
            .collect();

        if !ma_periods.is_empty() && let Ok(df) = df_result {
            let close_col = PlSmallStr::from("close");
            match df.column(&close_col) {
                Ok(close_series) => {
                    // 为每个周期计算 MA
                    for &period in &ma_periods {
                        if data.len() >= period {
                            let opts = RollingOptionsFixedWindow {
                                window_size: period,
                                min_periods: period,
                                center: false,
                                ..Default::default()
                            };

                            let ma_result = close_series.rolling_mean(opts);

                            if let Ok(ma_series) = ma_result {
                                let values: Vec<Option<Decimal>> = match ma_series
                                    .cast(&DataType::Float64)
                                {
                                    Ok(casted) => {
                                        let float_values: Vec<Option<f64>> = casted
                                            .iter()
                                            .map(|av| av.extract::<f64>())
                                            .collect();
                                        float_values
                                            .into_iter()
                                            .map(|v| {
                                                v.and_then(|f| {
                                                    Decimal::from_str(&format!("{}", f)).ok()
                                                })
                                            })
                                            .collect()
                                    }
                                    Err(_) => vec![None; data.len()],
                                };
                                result.insert(format!("ma{}", period), values);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Fallback: 使用 simple Series
                    let name = PlSmallStr::from("close");
                    let close_series = Series::new(name, &data.close);

                    for &period in &ma_periods {
                        if data.len() >= period {
                            let opts = RollingOptionsFixedWindow {
                                window_size: period,
                                min_periods: period,
                                center: false,
                                ..Default::default()
                            };

                            if let Ok(ma_series) = close_series.rolling_mean(opts)
                                && let Ok(casted) = ma_series.cast(&DataType::Float64)
                            {
                                let float_values: Vec<Option<f64>> =
                                    casted.iter().map(|av| av.extract::<f64>()).collect();
                                let values: Vec<Option<Decimal>> = float_values
                                    .into_iter()
                                    .map(|v| {
                                        v.and_then(|f| {
                                            Decimal::from_str(&format!("{}", f)).ok()
                                        })
                                    })
                                    .collect();
                                result.insert(format!("ma{}", period), values);
                            }
                        }
                    }
                }
            }
        }

        // 其他指标使用当前实现
        if indicators.contains(&"rsi14") {
            result.insert("rsi14".to_string(), self.rsi(data, 14));
        }

        result
    }
}

impl Default for PolarsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// 多股票批量数据
#[derive(Debug, Clone)]
pub struct MultiStockData {
    pub stocks: HashMap<String, BatchKlineData>,
}

impl MultiStockData {
    /// 创建新的多股票数据
    pub fn new() -> Self {
        Self {
            stocks: HashMap::new(),
        }
    }

    /// 添加股票数据
    pub fn add_stock(&mut self, code: String, data: BatchKlineData) {
        self.stocks.insert(code, data);
    }

    /// 批量计算指标 (使用 Polars group_by)
    pub fn calculate_batch_indicators(
        &self,
        indicators: &[&str],
    ) -> Result<IndicatorBatchResult> {
        use polars::prelude::*;

        // 构建合并的 DataFrame
        let mut all_codes = Vec::new();
        let mut all_timestamps = Vec::new();
        let mut all_close = Vec::new();

        for (code, data) in &self.stocks {
            for (&ts, &close) in data.timestamps.iter().zip(data.close.iter()) {
                all_codes.push(code.as_str());
                all_timestamps.push(ts);
                all_close.push(close);
            }
        }

        let _df = df!(
            PlSmallStr::from("code") => &all_codes,
            PlSmallStr::from("timestamp") => &all_timestamps,
            PlSmallStr::from("close") => &all_close,
        )
        .map_err(|e| QuantixError::Other(format!("创建 DataFrame 失败: {}", e)))?;

        let mut result = HashMap::new();

        // 按 code 分组并计算指标
        for code in self.stocks.keys() {
            let stock_result = self.calculate_single_stock(code, indicators)?;
            result.insert(code.clone(), stock_result);
        }

        Ok(result)
    }

    /// 计算单个股票的指标
    fn calculate_single_stock(
        &self,
        code: &str,
        indicators: &[&str],
    ) -> Result<HashMap<String, Vec<Option<Decimal>>>> {
        if let Some(data) = self.stocks.get(code) {
            let calc = PolarsCalculator::new();
            Ok(calc.calculate_batch(data, indicators))
        } else {
            Ok(HashMap::new())
        }
    }
}

impl Default for MultiStockData {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 Vec<Kline> 构建批量数据
pub fn from_kline_vec(klines: &[crate::data::models::Kline]) -> BatchKlineData {
    if klines.is_empty() {
        return BatchKlineData {
            code: String::new(),
            timestamps: vec![],
            open: vec![],
            high: vec![],
            low: vec![],
            close: vec![],
            volume: vec![],
            amount: vec![],
        };
    }

    let code = klines[0].code.clone();
    let mut timestamps = Vec::with_capacity(klines.len());
    let mut open = Vec::with_capacity(klines.len());
    let mut high = Vec::with_capacity(klines.len());
    let mut low = Vec::with_capacity(klines.len());
    let mut close = Vec::with_capacity(klines.len());
    let mut volume = Vec::with_capacity(klines.len());
    let mut amount = Vec::with_capacity(klines.len());

    for kline in klines {
        timestamps.push(
            kline
                .date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp(),
        );
        open.push(kline.open.to_f64().unwrap_or(0.0));
        high.push(kline.high.to_f64().unwrap_or(0.0));
        low.push(kline.low.to_f64().unwrap_or(0.0));
        close.push(kline.close.to_f64().unwrap_or(0.0));
        volume.push(kline.volume);
        amount.push(kline.amount.unwrap_or_default().to_f64().unwrap_or(0.0));
    }

    BatchKlineData {
        code,
        timestamps,
        open,
        high,
        low,
        close,
        volume,
        amount,
    }
}
