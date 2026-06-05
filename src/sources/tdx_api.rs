/// TDX-API Docker 服务桥接
///
/// 通过 HTTP 调用 tdx-api Docker 服务获取行情数据，
/// 无需在 Rust 端重新实现通达信协议。
use std::sync::RwLock;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use urlencoding::encode as url_encode;

use crate::core::{QuantixError, Result};
use crate::data::fetcher::Fetcher;
use crate::data::models::{AdjustType, Kline, Market, StockInfo};
use crate::sources::tdx::StockQuote;

/// serde helper: JSON `null` → `T::default()` (用于 Go 端返回 null 而非空数组的字段)
fn deserialize_null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const DEFAULT_BASE_URL: &str = "http://tdx-api:8080";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_BATCH_QUOTE_SIZE: usize = 50;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 500;
const CACHE_TTL_SECS: u64 = 3600; // 1 hour for codes / workday

/// tdx-api 客户端配置
#[derive(Debug, Clone)]
pub struct TdxApiConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub enabled: bool,
    pub max_batch_quote_size: usize,
    pub health_timeout: Duration,
}

impl Default for TdxApiConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: MAX_RETRIES,
            enabled: true,
            max_batch_quote_size: DEFAULT_MAX_BATCH_QUOTE_SIZE,
            health_timeout: Duration::from_secs(DEFAULT_HEALTH_TIMEOUT_SECS),
        }
    }
}

impl TdxApiConfig {
    pub fn from_env() -> Self {
        let base_url =
            std::env::var("TDX_API_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let timeout_secs = std::env::var("TDX_API_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let enabled = std::env::var("TDX_API_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);
        let max_batch_quote_size = std::env::var("TDX_API_MAX_BATCH_QUOTE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_BATCH_QUOTE_SIZE);
        let health_timeout_secs = std::env::var("TDX_API_HEALTH_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_HEALTH_TIMEOUT_SECS);
        Self {
            base_url,
            timeout: Duration::from_secs(timeout_secs),
            max_retries: MAX_RETRIES,
            enabled,
            max_batch_quote_size,
            health_timeout: Duration::from_secs(health_timeout_secs),
        }
    }

    pub fn from_app_config(cfg: &crate::core::config::TdxApiConfig) -> Self {
        Self {
            base_url: cfg.base_url.clone(),
            timeout: Duration::from_secs(cfg.timeout_secs),
            max_retries: cfg.max_retries,
            enabled: cfg.enabled,
            max_batch_quote_size: cfg.max_batch_quote_size,
            health_timeout: Duration::from_secs(cfg.health_timeout_secs),
        }
    }
}

// ---------------------------------------------------------------------------
// Response envelope & protocol types
// ---------------------------------------------------------------------------

/// tdx-api 通用响应包装
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TdxApiHealthStatus {
    pub status: Option<String>,
    pub code: Option<i64>,
    pub message: Option<String>,
}

impl TdxApiHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.status.as_deref() == Some("healthy") || self.code == Some(0)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TdxApiServerStatus {
    pub status: String,
    pub connected: bool,
    pub version: Option<String>,
    pub uptime: Option<String>,
}

/// K线响应 (PascalCase — Go 无 json tag)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KlineResp {
    #[serde(default)]
    pub count: i64,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub list: Vec<KlineItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KlineItem {
    #[serde(rename = "Last")]
    pub last: i64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume: i64,
    pub amount: i64,
    pub time: String,
}

/// 五档行情响应 (PascalCase)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct QuoteItem {
    exchange: u8,
    code: String,
    #[serde(rename = "K")]
    k: PriceInfo,
    total_hand: i64,
    amount: f64,
    inside_dish: i64,
    outer_disc: i64,
    buy_level: Vec<PriceLevel>,
    sell_level: Vec<PriceLevel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PriceInfo {
    last: i64,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PriceLevel {
    buy: bool,
    price: i64,
    number: i32,
}

/// 逐笔成交响应 (PascalCase)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TradeResp {
    #[serde(default)]
    count: i64,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    list: Vec<TradeItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TradeItem {
    time: String,
    price: i64,
    volume: i32,
    status: i32,
    number: i32,
}

/// 分时数据响应
#[derive(Debug, Deserialize)]
pub struct MinuteResp {
    pub date: String,
    #[serde(rename = "Count")]
    pub count: i32,
    #[serde(
        rename = "List",
        default,
        deserialize_with = "deserialize_null_default"
    )]
    pub list: Vec<MinuteItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinuteItem {
    pub time: String,
    pub price: i64,
    pub number: i32,
}

// ---------------------------------------------------------------------------
// Extended API types (snake_case)
// ---------------------------------------------------------------------------

/// 代码查询响应
#[derive(Debug, Clone, Deserialize)]
pub struct CodesResponse {
    pub total: usize,
    pub codes: Vec<CodeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeEntry {
    pub code: String,
    pub name: String,
    pub exchange: String,
}

/// 搜索结果
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub code: String,
    pub name: String,
    pub exchange: String,
}

/// 交易日响应
#[derive(Debug, Clone, Deserialize)]
pub struct WorkdayResponse {
    pub date: WorkdayDate,
    pub is_workday: bool,
    pub next: Vec<WorkdayDate>,
    pub previous: Vec<WorkdayDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkdayDate {
    pub iso: String,
    pub numeric: String,
}

/// 交易日范围响应
#[derive(Debug, Clone, Deserialize)]
pub struct WorkdayRangeResponse {
    pub count: usize,
    pub list: Vec<WorkdayDate>,
}

/// 收益计算响应
#[derive(Debug, Deserialize)]
pub struct IncomeResponse {
    pub count: usize,
    pub list: Vec<IncomeItem>,
}

#[derive(Debug, Deserialize)]
pub struct IncomeItem {
    pub offset: i32,
    pub time: String,
    pub rise: f64,
    pub rise_rate: f64,
    pub source: IncomeOhlcv,
    pub current: IncomeOhlcv,
}

#[derive(Debug, Deserialize)]
pub struct IncomeOhlcv {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// 市场统计响应
#[derive(Debug, Deserialize)]
pub struct MarketStatsResponse {
    pub sh: MarketStatItem,
    pub sz: MarketStatItem,
    pub bj: MarketStatItem,
    pub update_time: String,
}

#[derive(Debug, Deserialize)]
pub struct MarketStatItem {
    pub total: usize,
    pub up: usize,
    pub down: usize,
    pub flat: usize,
}

/// K线完整数据响应 (kline-all)
#[derive(Debug, Deserialize)]
struct KlineAllResp {
    count: i64,
    list: Vec<KlineItem>,
    meta: Option<KlineAllMeta>,
}

#[derive(Debug, Deserialize)]
struct KlineAllMeta {
    source: String,
    #[serde(rename = "type")]
    kline_type: String,
}

// ---------------------------------------------------------------------------
// Task management types
// ---------------------------------------------------------------------------

/// 异步任务信息
#[derive(Debug, Clone, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub ended_at: Option<String>,
}

/// 创建任务请求 (K线拉取)
#[derive(Debug, Clone, Serialize)]
pub struct PullKlineRequest {
    pub codes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tables: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub dir: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub start_date: String,
}

/// 创建任务请求 (成交拉取)
#[derive(Debug, Clone, Serialize)]
pub struct PullTradeRequest {
    pub code: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub dir: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub start_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub end_year: Option<i32>,
}

/// 创建任务响应
#[derive(Debug, Deserialize)]
struct CreateTaskResp {
    task_id: String,
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Cached<T> {
    data: T,
    expires_at: Instant,
}

impl<T> Cached<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

#[derive(Debug)]
struct TdxApiCache {
    codes: Option<Cached<CodesResponse>>,
    workday_range: Option<Cached<Vec<NaiveDate>>>,
}

impl Default for TdxApiCache {
    fn default() -> Self {
        Self {
            codes: None,
            workday_range: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Kline period type (module-level)
// ---------------------------------------------------------------------------

/// K线周期类型
#[derive(Debug, Clone, Copy)]
pub enum KlineType {
    Min1,
    Min5,
    Min15,
    Min30,
    Hour,
    Day,
    Week,
    Month,
}

impl KlineType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Min1 => "minute1",
            Self::Min5 => "minute5",
            Self::Min15 => "minute15",
            Self::Min30 => "minute30",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// tdx-api HTTP 客户端
///
/// 通过 REST API 调用 tdx-api Docker 服务，获取通达信行情数据。
/// 内置重试机制和本地缓存。
#[derive(Debug)]
pub struct TdxApiClient {
    client: reqwest::Client,
    config: TdxApiConfig,
    cache: RwLock<TdxApiCache>,
}

impl TdxApiClient {
    pub fn new(config: TdxApiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(4)
            .build()
            .map_err(QuantixError::Http)?;
        Ok(Self {
            client,
            config,
            cache: RwLock::new(TdxApiCache::default()),
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::new(TdxApiConfig::from_env())
    }

    /// 从应用配置文件创建
    pub fn from_app_config(cfg: &crate::core::config::TdxApiConfig) -> Result<Self> {
        let runtime = TdxApiConfig::from_app_config(cfg);
        if !runtime.enabled {
            return Err(QuantixError::Unsupported(
                "tdx-api source is disabled".to_string(),
            ));
        }
        Self::new(runtime)
    }

    // -----------------------------------------------------------------------
    // Core: retry + request
    // -----------------------------------------------------------------------

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        self.request_with_retry(|| {
            let client = &self.client;
            let url = url.clone();
            async move { client.get(&url).send().await }
        })
        .await
    }

    async fn get_with_query<T: DeserializeOwned, Q: Serialize + Clone>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        let query = query.clone();
        self.request_with_retry(|| {
            let client = &self.client;
            let url = url.clone();
            let query = query.clone();
            async move { client.get(&url).query(&query).send().await }
        })
        .await
    }

    async fn post_json<T: DeserializeOwned, B: Serialize + Clone>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        let body = body.clone();
        self.request_with_retry(|| {
            let client = &self.client;
            let url = url.clone();
            let body = body.clone();
            async move { client.post(&url).json(&body).send().await }
        })
        .await
    }

    /// 带指数退避的重试请求
    async fn request_with_retry<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<reqwest::Response, reqwest::Error>>,
        T: DeserializeOwned,
    {
        let mut last_err = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                sleep(Duration::from_millis(delay)).await;
            }
            match f().await {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        last_err = Some(QuantixError::DataSource(format!(
                            "tdx-api HTTP {}: {}",
                            status, body
                        )));
                        if status.is_server_error() {
                            continue; // retry on 5xx
                        }
                        return Err(last_err.unwrap()); // no retry on 4xx
                    }
                    let api_resp: ApiResponse<T> = resp.json().await.map_err(|e| {
                        QuantixError::DataParse(format!("tdx-api 响应解析失败: {e}"))
                    })?;
                    if api_resp.code != 0 {
                        return Err(QuantixError::DataSource(format!(
                            "tdx-api 业务错误 [{}]: {}",
                            api_resp.code, api_resp.message
                        )));
                    }
                    return api_resp.data.ok_or_else(|| {
                        QuantixError::DataSource("tdx-api 响应 data 为空".to_string())
                    });
                }
                Err(e) => {
                    last_err = Some(QuantixError::Http(e));
                    continue;
                }
            }
        }
        Err(last_err.unwrap_or_else(|| QuantixError::Timeout("tdx-api 重试耗尽".to_string())))
    }

    // -----------------------------------------------------------------------
    // Price conversion helpers
    // -----------------------------------------------------------------------

    /// tdx-api Price (int64, 厘) → f64 (元)
    fn price_to_f64(raw: i64) -> f64 {
        raw as f64 / 1000.0
    }

    /// tdx-api Price (int64, 厘) → Decimal (元)
    fn price_to_decimal(raw: i64, _field: &str) -> Result<Decimal> {
        Ok(Decimal::new(raw, 3))
    }

    // -----------------------------------------------------------------------
    // Symbol format helpers
    // -----------------------------------------------------------------------

    /// code → tdx-api symbol (sh600000 / sz000001 / bj430047)
    fn to_symbol(code: &str) -> String {
        let code = code.trim();
        if code.starts_with("sh")
            || code.starts_with("sz")
            || code.starts_with("bj")
            || code.contains('.')
        {
            return code.to_string();
        }
        let prefix = if code.starts_with('6') || code.starts_with("510") || code.starts_with("51") {
            "sh"
        } else if code.starts_with('0') || code.starts_with('3') || code.starts_with('1') {
            "sz"
        } else {
            "bj"
        };
        format!("{prefix}{code}")
    }

    /// symbol → (code, Market)
    fn from_symbol(symbol: &str) -> (&str, Market) {
        if let Some(code) = symbol.strip_prefix("sh") {
            (code, Market::SH)
        } else if let Some(code) = symbol.strip_prefix("sz") {
            (code, Market::SZ)
        } else if let Some(code) = symbol.strip_prefix("bj") {
            (code, Market::BJ)
        } else {
            (symbol, Market::SH)
        }
    }

    // -----------------------------------------------------------------------
    // Public API: Quote
    // -----------------------------------------------------------------------

    /// 获取实时五档行情
    pub async fn get_quote(&self, code: &str) -> Result<StockQuote> {
        let symbol = Self::to_symbol(code);
        let quotes: Vec<QuoteItem> = self.get(&format!("/api/quote?code={symbol}")).await?;
        let q = quotes
            .into_iter()
            .next()
            .ok_or_else(|| QuantixError::DataSource(format!("tdx-api 行情无数据: {code}")))?;
        let price = Self::price_to_f64(q.k.close);
        let preclose = Self::price_to_f64(q.k.last);
        Ok(StockQuote::from_tdx(
            q.code,
            String::new(),
            price,
            preclose,
            Self::price_to_f64(q.k.open),
            Self::price_to_f64(q.k.high),
            Self::price_to_f64(q.k.low),
            q.total_hand as f64,
            q.amount,
            q.exchange,
        ))
    }

    /// 批量获取行情 (最多50个)
    pub async fn batch_quote(&self, codes: &[&str]) -> Result<Vec<StockQuote>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        if codes.len() > self.config.max_batch_quote_size {
            return Err(QuantixError::DataSource(format!(
                "tdx-api batch quote requested {} codes, exceeding max_batch_quote_size {}",
                codes.len(),
                self.config.max_batch_quote_size
            )));
        }
        let symbols: Vec<String> = codes.iter().map(|c| Self::to_symbol(c)).collect();
        #[derive(Debug, Clone, Serialize)]
        struct BatchReq {
            codes: Vec<String>,
        }
        let quotes: Vec<QuoteItem> = self
            .post_json("/api/batch-quote", &BatchReq { codes: symbols })
            .await?;
        quotes
            .into_iter()
            .map(|q| {
                let price = Self::price_to_f64(q.k.close);
                let preclose = Self::price_to_f64(q.k.last);
                Ok(StockQuote::from_tdx(
                    q.code,
                    String::new(),
                    price,
                    preclose,
                    Self::price_to_f64(q.k.open),
                    Self::price_to_f64(q.k.high),
                    Self::price_to_f64(q.k.low),
                    q.total_hand as f64,
                    q.amount,
                    q.exchange,
                ))
            })
            .collect()
    }

    /// 兼容 CollectScheduler 的批量采集，按配置上限分批调用 batch_quote
    pub async fn collect_all_quotes(&self, codes: &[String]) -> Result<Vec<StockQuote>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let mut all = Vec::new();
        let batch_size = self.config.max_batch_quote_size.max(1);
        for chunk in codes.chunks(batch_size) {
            let refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
            match self.batch_quote(&refs).await {
                Ok(q) => all.extend(q),
                Err(e) => {
                    tracing::warn!("tdx-api 批量行情采集失败: {e}, 跳过");
                }
            }
            if chunk.len() == batch_size {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        Ok(all)
    }

    /// 获取 K 线数据 (原始协议格式，价格单位: 厘)
    pub async fn get_kline_raw(
        &self,
        code: &str,
        kline_type: KlineType,
        limit: Option<u32>,
    ) -> Result<KlineResp> {
        let symbol = Self::to_symbol(code);
        let mut path = format!("/api/kline?code={symbol}&type={}", kline_type.as_str());
        if let Some(n) = limit {
            path = format!("{path}&limit={n}");
        }
        self.get(&path).await
    }

    /// 获取日线 K 线并转为标准 Kline 模型
    pub async fn get_daily_kline(
        &self,
        code: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Kline>> {
        let resp = self.get_kline_raw(code, KlineType::Day, None).await?;
        Self::kline_resp_to_klines(resp, code, start, end, AdjustType::None)
    }

    /// 获取同花顺前复权完整日线
    pub async fn get_kline_ths_qfq(&self, code: &str) -> Result<Vec<Kline>> {
        let symbol = Self::to_symbol(code);
        let resp: KlineAllResp = self
            .get(&format!("/api/kline-all/ths?code={symbol}&type=day"))
            .await?;
        let start = NaiveDate::from_ymd_opt(1990, 12, 19).unwrap_or_default();
        let end = chrono::Local::now().date_naive();
        Self::kline_resp_to_klines(
            KlineResp {
                count: resp.count,
                list: resp.list,
            },
            code,
            start,
            end,
            AdjustType::QFQ,
        )
    }

    fn kline_resp_to_klines(
        resp: KlineResp,
        code: &str,
        start: NaiveDate,
        end: NaiveDate,
        adjust: AdjustType,
    ) -> Result<Vec<Kline>> {
        resp.list
            .into_iter()
            .filter_map(|item| {
                // Parse date from "2025-01-15T00:00:00+08:00" or "2025-01-15"
                let date_str = item.time.split('T').next()?;
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| {
                        if d >= start && d <= end {
                            Some((d, item))
                        } else {
                            None
                        }
                    })
            })
            .map(|(date, item)| {
                Ok(Kline {
                    code: code.to_string(),
                    date,
                    open: Self::price_to_decimal(item.open, "open")?,
                    high: Self::price_to_decimal(item.high, "high")?,
                    low: Self::price_to_decimal(item.low, "low")?,
                    close: Self::price_to_decimal(item.close, "close")?,
                    volume: item.volume,
                    amount: Some(Self::price_to_decimal(item.amount, "amount")?),
                    adjust_type: adjust,
                })
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Public API: Minute / Trade
    // -----------------------------------------------------------------------

    /// 获取分时数据
    pub async fn get_minute(&self, code: &str, date: Option<&str>) -> Result<MinuteResp> {
        let symbol = Self::to_symbol(code);
        let mut path = format!("/api/minute?code={symbol}");
        if let Some(d) = date {
            path = format!("{path}&date={d}");
        }
        self.get(&path).await
    }

    /// 获取逐笔成交
    pub async fn get_trades(&self, code: &str, date: Option<&str>) -> Result<TradeResp> {
        let symbol = Self::to_symbol(code);
        let mut path = format!("/api/trade?code={symbol}");
        if let Some(d) = date {
            path = format!("{path}&date={d}");
        }
        self.get(&path).await
    }

    // -----------------------------------------------------------------------
    // Public API: Search / Codes
    // -----------------------------------------------------------------------

    /// 搜索股票代码/名称
    pub async fn search_codes(&self, keyword: &str) -> Result<Vec<SearchResult>> {
        let encoded = url_encode(keyword);
        self.get(&format!("/api/search?keyword={encoded}")).await
    }

    /// 获取全部股票代码列表 (带缓存)
    pub async fn get_codes(&self, exchange: Option<&str>) -> Result<CodesResponse> {
        {
            let cache = self.cache.read().unwrap_or_else(|e| e.into_inner());
            if let Some(ref cached) = cache.codes {
                if cached.is_valid() {
                    return Ok(cached.data.clone());
                }
            }
        }

        let mut path = "/api/codes".to_string();
        if let Some(ex) = exchange {
            path = format!("{path}?exchange={ex}");
        }
        let resp: CodesResponse = self.get(&path).await?;

        let mut cache = self.cache.write().unwrap_or_else(|e| e.into_inner());
        cache.codes = Some(Cached::new(
            resp.clone(),
            Duration::from_secs(CACHE_TTL_SECS),
        ));
        Ok(resp)
    }

    // -----------------------------------------------------------------------
    // Public API: Workday
    // -----------------------------------------------------------------------

    /// 查询交易日 (单个日期)
    pub async fn get_workday(&self, date: &str, count: u32) -> Result<WorkdayResponse> {
        self.get(&format!("/api/workday?date={date}&count={count}"))
            .await
    }

    /// 查询交易日范围 (带缓存)
    pub async fn get_workday_range(&self, start: &str, end: &str) -> Result<Vec<NaiveDate>> {
        {
            let cache = self.cache.read().unwrap_or_else(|e| e.into_inner());
            if let Some(ref cached) = cache.workday_range {
                if cached.is_valid() {
                    return Ok(cached.data.clone());
                }
            }
        }

        let resp: WorkdayRangeResponse = self
            .get(&format!("/api/workday/range?start={start}&end={end}"))
            .await?;
        let dates: Vec<NaiveDate> = resp
            .list
            .iter()
            .filter_map(|d| NaiveDate::parse_from_str(&d.iso, "%Y-%m-%d").ok())
            .collect();

        let mut cache = self.cache.write().unwrap_or_else(|e| e.into_inner());
        cache.workday_range = Some(Cached::new(
            dates.clone(),
            Duration::from_secs(CACHE_TTL_SECS),
        ));
        Ok(dates)
    }

    /// 判断是否为交易日
    pub async fn is_trading_day(&self, date: NaiveDate) -> Result<bool> {
        let ds = date.format("%Y%m%d").to_string();
        let resp = self.get_workday(&ds, 1).await?;
        Ok(resp.is_workday)
    }

    // -----------------------------------------------------------------------
    // Public API: Income
    // -----------------------------------------------------------------------

    /// N日收益计算
    pub async fn get_income(
        &self,
        code: &str,
        start_date: &str,
        days: &[i32],
    ) -> Result<IncomeResponse> {
        let symbol = Self::to_symbol(code);
        let days_str = days
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.get(&format!(
            "/api/income?code={symbol}&start_date={start_date}&days={days_str}"
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Public API: Market Stats
    // -----------------------------------------------------------------------

    /// 获取市场涨跌统计
    pub async fn get_market_stats(&self) -> Result<MarketStatsResponse> {
        self.get("/api/market-stats").await
    }

    /// 获取市场股票数量
    pub async fn get_market_count(&self) -> Result<serde_json::Value> {
        self.get("/api/market-count").await
    }

    // -----------------------------------------------------------------------
    // Public API: Index
    // -----------------------------------------------------------------------

    /// 获取指数 K 线
    pub async fn get_index_kline(
        &self,
        code: &str,
        kline_type: KlineType,
        limit: Option<u32>,
    ) -> Result<KlineResp> {
        let symbol = Self::to_symbol(code);
        let mut path = format!("/api/index?code={symbol}&type={}", kline_type.as_str());
        if let Some(n) = limit {
            path = format!("{path}&limit={n}");
        }
        self.get(&path).await
    }

    // -----------------------------------------------------------------------
    // Public API: Full Kline History
    // -----------------------------------------------------------------------

    /// 获取完整K线 (TDX源)
    pub async fn get_kline_all_tdx(
        &self,
        code: &str,
        kline_type: KlineType,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>> {
        let symbol = Self::to_symbol(code);
        let mut path = format!(
            "/api/kline-all/tdx?code={symbol}&type={}",
            kline_type.as_str()
        );
        if let Some(n) = limit {
            path = format!("{path}&limit={n}");
        }
        let resp: KlineAllResp = self.get(&path).await?;
        let start = NaiveDate::from_ymd_opt(1990, 12, 19).unwrap_or_default();
        let end = chrono::Local::now().date_naive();
        Self::kline_resp_to_klines(
            KlineResp {
                count: resp.count,
                list: resp.list,
            },
            code,
            start,
            end,
            AdjustType::None,
        )
    }

    /// 获取完整K线 (同花顺源, 仅 day/week/month)
    pub async fn get_kline_all_ths(&self, code: &str, kline_type: KlineType) -> Result<Vec<Kline>> {
        let symbol = Self::to_symbol(code);
        let resp: KlineAllResp = self
            .get(&format!(
                "/api/kline-all/ths?code={symbol}&type={}",
                kline_type.as_str()
            ))
            .await?;
        let start = NaiveDate::from_ymd_opt(1990, 12, 19).unwrap_or_default();
        let end = chrono::Local::now().date_naive();
        Self::kline_resp_to_klines(
            KlineResp {
                count: resp.count,
                list: resp.list,
            },
            code,
            start,
            end,
            AdjustType::QFQ,
        )
    }

    /// 获取分页K线历史
    pub async fn get_kline_history(
        &self,
        code: &str,
        kline_type: KlineType,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>> {
        let symbol = Self::to_symbol(code);
        let mut path = format!(
            "/api/kline-history?code={symbol}&type={}",
            kline_type.as_str()
        );
        if let Some(n) = limit {
            path = format!("{path}&limit={n}");
        }
        let resp: KlineResp = self.get(&path).await?;
        let start = NaiveDate::from_ymd_opt(1990, 12, 19).unwrap_or_default();
        let end = chrono::Local::now().date_naive();
        Self::kline_resp_to_klines(resp, code, start, end, AdjustType::None)
    }

    // -----------------------------------------------------------------------
    // Public API: Task Management
    // -----------------------------------------------------------------------

    /// 创建K线拉取异步任务
    pub async fn create_pull_kline_task(&self, req: &PullKlineRequest) -> Result<String> {
        let resp: CreateTaskResp = self.post_json("/api/tasks/pull-kline", req).await?;
        Ok(resp.task_id)
    }

    /// 创建成交拉取异步任务
    pub async fn create_pull_trade_task(&self, req: &PullTradeRequest) -> Result<String> {
        let resp: CreateTaskResp = self.post_json("/api/tasks/pull-trade", req).await?;
        Ok(resp.task_id)
    }

    /// 列出所有任务
    pub async fn list_tasks(&self) -> Result<Vec<TaskInfo>> {
        self.get("/api/tasks").await
    }

    /// 获取单个任务状态
    pub async fn get_task(&self, task_id: &str) -> Result<TaskInfo> {
        self.get(&format!("/api/tasks/{task_id}")).await
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<serde_json::Value> {
        self.get(&format!("/api/tasks/{task_id}/cancel")).await
    }

    // -----------------------------------------------------------------------
    // Health
    // -----------------------------------------------------------------------

    /// 健康检查 (不走通用 envelope，直接返回原始 JSON)
    pub async fn health(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/health", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(QuantixError::Http)?;
        resp.json()
            .await
            .map_err(|e| QuantixError::DataParse(format!("tdx-api health 解析失败: {e}")))
    }

    pub async fn health_status(&self) -> Result<TdxApiHealthStatus> {
        let url = format!("{}/api/health", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .timeout(self.config.health_timeout)
            .send()
            .await
            .map_err(QuantixError::Http)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(QuantixError::DataSource(format!(
                "tdx-api health HTTP {}: {}",
                status, body
            )));
        }

        let health: TdxApiHealthStatus = resp
            .json()
            .await
            .map_err(|e| QuantixError::DataParse(format!("tdx-api health 解析失败: {e}")))?;

        if !health.is_healthy() {
            return Err(QuantixError::DataSource(format!(
                "tdx-api health unhealthy: {:?}",
                health
            )));
        }

        Ok(health)
    }

    pub async fn server_status(&self) -> Result<TdxApiServerStatus> {
        self.get("/api/server-status").await
    }

    /// 清除缓存
    pub fn invalidate_cache(&self) {
        let mut cache = self.cache.write().unwrap_or_else(|e| e.into_inner());
        cache.codes = None;
        cache.workday_range = None;
    }
}

// ---------------------------------------------------------------------------
// Fetcher trait implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl Fetcher for TdxApiClient {
    async fn get_stock_info(&self, code: &str) -> Result<Option<StockInfo>> {
        let codes = self.get_codes(None).await?;
        let found = codes.codes.iter().find(|c| c.code == code);
        match found {
            Some(entry) => {
                let market = match entry.exchange.as_str() {
                    "sh" => Market::SH,
                    "sz" => Market::SZ,
                    "bj" => Market::BJ,
                    _ => Market::SH,
                };
                Ok(Some(StockInfo {
                    code: entry.code.clone(),
                    name: entry.name.clone(),
                    market,
                    list_date: None,
                    delist_date: None,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_kline(&self, code: &str, start: NaiveDate, end: NaiveDate) -> Result<Vec<Kline>> {
        self.get_daily_kline(code, start, end).await
    }

    async fn check_connection(&self) -> Result<()> {
        self.health_status().await?;
        let status = self.server_status().await?;
        if !status.connected {
            return Err(QuantixError::DataSource(
                "tdx-api server is running but not connected to TDX upstream".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn default_config_uses_tdx_api_container_endpoint() {
        let config = TdxApiConfig::default();

        assert_eq!(config.base_url, "http://tdx-api:8080");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.health_timeout, Duration::from_secs(5));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_batch_quote_size, 50);
        assert!(config.enabled);
    }

    #[test]
    fn runtime_config_from_app_config_maps_extended_fields() {
        let app_config = crate::core::config::TdxApiConfig {
            base_url: "http://127.0.0.1:8089".to_string(),
            timeout_secs: 11,
            max_retries: 2,
            enabled: false,
            max_batch_quote_size: 25,
            health_timeout_secs: 3,
        };

        let runtime = TdxApiConfig::from_app_config(&app_config);

        assert_eq!(runtime.base_url, "http://127.0.0.1:8089");
        assert_eq!(runtime.timeout, Duration::from_secs(11));
        assert_eq!(runtime.health_timeout, Duration::from_secs(3));
        assert_eq!(runtime.max_retries, 2);
        assert_eq!(runtime.max_batch_quote_size, 25);
        assert!(!runtime.enabled);
    }

    #[tokio::test]
    async fn health_status_accepts_observed_status_shape() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "healthy",
                "time": "2026-06-05T00:00:00Z"
            })))
            .mount(&server)
            .await;

        let client = TdxApiClient::new(TdxApiConfig {
            base_url: server.uri(),
            timeout: Duration::from_secs(1),
            max_retries: 0,
            enabled: true,
            max_batch_quote_size: 50,
            health_timeout: Duration::from_secs(1),
        })
        .unwrap();

        let health = client.health_status().await.unwrap();
        assert!(health.is_healthy());
    }

    #[tokio::test]
    async fn server_status_reads_connected_flag() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/server-status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0,
                "message": "success",
                "data": {
                    "status": "running",
                    "connected": true,
                    "version": "1.0.0",
                    "uptime": "unknown"
                }
            })))
            .mount(&server)
            .await;

        let client = TdxApiClient::new(TdxApiConfig {
            base_url: server.uri(),
            timeout: Duration::from_secs(1),
            max_retries: 0,
            enabled: true,
            max_batch_quote_size: 50,
            health_timeout: Duration::from_secs(1),
        })
        .unwrap();

        let status = client.server_status().await.unwrap();
        assert!(status.connected);
        assert_eq!(status.status, "running");
        assert_eq!(status.version.as_deref(), Some("1.0.0"));
    }

    #[tokio::test]
    async fn batch_quote_rejects_requests_over_configured_limit() {
        let client = TdxApiClient::new(TdxApiConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            timeout: Duration::from_secs(1),
            max_retries: 0,
            enabled: true,
            max_batch_quote_size: 1,
            health_timeout: Duration::from_secs(1),
        })
        .unwrap();

        let err = client.batch_quote(&["000001", "600519"]).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("tdx-api batch quote"));
        assert!(msg.contains("2"));
        assert!(msg.contains("1"));
    }

    #[test]
    fn symbol_helpers_map_common_a_share_markets() {
        assert_eq!(TdxApiClient::to_symbol("600000"), "sh600000");
        assert_eq!(TdxApiClient::to_symbol("510300"), "sh510300");
        assert_eq!(TdxApiClient::to_symbol("000001"), "sz000001");
        assert_eq!(TdxApiClient::to_symbol("300750"), "sz300750");
        assert_eq!(TdxApiClient::to_symbol("430047"), "bj430047");
        assert_eq!(TdxApiClient::to_symbol("sh600000"), "sh600000");

        let (code, market) = TdxApiClient::from_symbol("sz000001");
        assert_eq!(code, "000001");
        assert!(matches!(market, Market::SZ));
    }

    #[test]
    fn price_helpers_convert_milli_yuan_units() {
        assert_eq!(TdxApiClient::price_to_f64(12345), 12.345);
        assert_eq!(
            TdxApiClient::price_to_decimal(12345, "close")
                .unwrap()
                .to_string(),
            "12.345"
        );
    }
}
