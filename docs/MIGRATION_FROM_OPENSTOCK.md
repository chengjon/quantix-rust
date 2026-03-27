# OpenStock 功能移植计划

> 从 OpenStock (Next.js/TypeScript) 移植核心功能到 Quantix-Rust

## 概述

本文档规划将 OpenStock 的三个核心模块移植到 Quantix-Rust：
1. **情绪分析模块** - 多源情绪聚合与信号生成
2. **通知服务** - 邮件/消息推送系统
3. **AI 多 Provider** - 多 LLM 提供商抽象层

---

## 1. 情绪分析模块 (Sentiment Analysis)

### 1.1 OpenStock 实现分析

**数据源配置**:
```typescript
// 4 个数据源，每个有独立的 API 端点
SOURCE_CONFIG = {
  reddit:     { path: '/reddit/stocks/v1/compare', metric: 'mentions' },
  x:         { path: '/x/stocks/v1/compare',       metric: 'mentions' },
  news:      { path: '/news/stocks/v1/compare',    metric: 'mentions' },
  polymarket:{ path: '/polymarket/stocks/v1/compare', metric: 'trade_count' },
}
```

**核心数据结构**:
```typescript
interface SentimentSourceInsight {
  source: 'reddit' | 'x' | 'news' | 'polymarket';
  buzzScore: number;        // 热度分数 (0-100)
  bullishPct: number | null; // 看涨百分比
  trend: 'rising' | 'falling' | 'stable' | null;
  metricValue: number;      // 提及数/交易数
}

interface StockSentimentInsights {
  symbol: string;
  averageBuzz: number;      // 平均热度
  bullishAverage: number;   // 平均看涨比例
  sourceAlignment: string;  // "Bullish alignment" | "Bearish alignment" | "Mixed" 等
  sources: SentimentSourceInsight[];
}
```

**关键算法 - 来源一致性判断**:
```typescript
function getSourceAlignment(bullishValues: number[]): string {
  const spread = max - min;
  const avg = average(bullishValues);

  if (spread <= 12 && avg >= 60) return 'Bullish alignment';
  if (spread <= 12 && avg <= 40) return 'Bearish alignment';
  if (spread <= 12) return 'Tight alignment';
  if (spread >= 25) return 'Wide divergence';
  return 'Mixed';
}
```

### 1.2 Rust 实现设计

**模块位置**: `src/sentiment/`

**目录结构**:
```
src/sentiment/
├── mod.rs              # 模块导出
├── types.rs            # 数据结构定义
├── client.rs           # HTTP 客户端
├── sources/
│   ├── mod.rs
│   ├── adanos.rs       # Adanos API 客户端
│   └── traits.rs       # Source trait 定义
├── aggregator.rs       # 多源聚合逻辑
└── signal.rs           # 信号生成 (用于策略)
```

**核心类型定义** (`types.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentimentSource {
    Reddit,
    X, // Twitter
    News,
    Polymarket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentimentTrend {
    Rising,
    Falling,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInsight {
    pub source: SentimentSource,
    pub label: String,
    pub company_name: Option<String>,
    pub buzz_score: f64,
    pub bullish_pct: Option<f64>,
    pub trend: Option<SentimentTrend>,
    pub metric_label: String,
    pub metric_value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSentiment {
    pub symbol: String,
    pub company_name: Option<String>,
    pub average_buzz: f64,
    pub bullish_average: Option<f64>,
    pub source_alignment: SourceAlignment,
    pub available_sources: usize,
    pub sources: Vec<SourceInsight>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceAlignment {
    BullishAlignment,
    BearishAlignment,
    TightAlignment,
    WideDivergence,
    Mixed,
    SingleSourceView,
    NoSentimentMix,
}

impl SourceAlignment {
    pub fn from_bullish_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return SourceAlignment::NoSentimentMix;
        }
        if values.len() == 1 {
            return SourceAlignment::SingleSourceView;
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let spread = max - min;
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        if spread <= 12.0 && avg >= 60.0 {
            SourceAlignment::BullishAlignment
        } else if spread <= 12.0 && avg <= 40.0 {
            SourceAlignment::BearishAlignment
        } else if spread <= 12.0 {
            SourceAlignment::TightAlignment
        } else if spread >= 25.0 {
            SourceAlignment::WideDivergence
        } else {
            SourceAlignment::Mixed
        }
    }
}
```

**API 客户端** (`sources/adanos.rs`):
```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.adanos.org";
const DEFAULT_TIMEOUT_MS: u64 = 5000;

pub struct AdanosClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl AdanosClient {
    pub fn new(api_key: Option<String>, base_url: Option<String>) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            api_key,
        })
    }

    pub async fn fetch_sentiment(
        &self,
        symbol: &str,
        source: SentimentSource,
        days: u8,
    ) -> anyhow::Result<Option<SourceInsight>> {
        let path = match source {
            SentimentSource::Reddit => "/reddit/stocks/v1/compare",
            SentimentSource::X => "/x/stocks/v1/compare",
            SentimentSource::News => "/news/stocks/v1/compare",
            SentimentSource::Polymarket => "/polymarket/stocks/v1/compare",
        };

        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client
            .get(&url)
            .query(&[("tickers", symbol.to_uppercase().as_str())])
            .query(&[("days", &days.to_string())]);

        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        let response = request.send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let payload: ComparePayload = response.json().await?;
        // ... 解析逻辑
    }
}
```

**聚合器** (`aggregator.rs`):
```rust
use futures::future::join_all;

pub struct SentimentAggregator {
    client: AdanosClient,
    default_lookback_days: u8,
}

impl SentimentAggregator {
    pub async fn get_stock_sentiment(
        &self,
        symbol: &str,
        days: Option<u8>,
    ) -> anyhow::Result<Option<StockSentiment>> {
        let days = days.unwrap_or(self.default_lookback_days).min(30).max(1);
        let sources = [SentimentSource::Reddit, SentimentSource::X,
                       SentimentSource::News, SentimentSource::Polymarket];

        // 并行请求所有数据源
        let futures: Vec<_> = sources
            .iter()
            .map(|&source| self.client.fetch_sentiment(symbol, source, days))
            .collect();

        let results = join_all(futures).await;

        // 过滤有效结果并聚合
        let insights: Vec<SourceInsight> = results
            .into_iter()
            .filter_map(|r| r.ok().flatten())
            .collect();

        if insights.is_empty() {
            return Ok(None);
        }

        Ok(Some(self.build_sentiment(symbol, insights)))
    }

    fn build_sentiment(&self, symbol: &str, insights: Vec<SourceInsight>) -> StockSentiment {
        let buzz_values: Vec<f64> = insights.iter().map(|i| i.buzz_score).collect();
        let bullish_values: Vec<f64> = insights.iter()
            .filter_map(|i| i.bullish_pct)
            .collect();

        let average_buzz = buzz_values.iter().sum::<f64>() / buzz_values.len() as f64;
        let bullish_average = if bullish_values.is_empty() {
            None
        } else {
            Some(bullish_values.iter().sum::<f64>() / bullish_values.len() as f64)
        };

        let source_alignment = SourceAlignment::from_bullish_values(&bullish_values);

        StockSentiment {
            symbol: symbol.to_uppercase(),
            company_name: insights.first().and_then(|i| i.company_name.clone()),
            average_buzz: (average_buzz * 10.0).round() / 10.0,
            bullish_average: bullish_average.map(|v| (v * 10.0).round() / 10.0),
            source_alignment,
            available_sources: insights.len(),
            sources: insights,
            fetched_at: chrono::Utc::now(),
        }
    }
}
```

### 1.3 与 Quantix 策略集成

**信号生成** (`signal.rs`):
```rust
use crate::strategy::{Signal, SignalStrength};

impl StockSentiment {
    /// 将情绪数据转换为策略信号
    pub fn to_signal(&self) -> Option<Signal> {
        match &self.source_alignment {
            SourceAlignment::BullishAlignment if self.bullish_average? >= 65.0 => {
                Some(Signal::Long {
                    strength: SignalStrength::Strong,
                    reason: format!(
                        "Bullish sentiment alignment: {:.1}% across {} sources",
                        self.bullish_average?, self.available_sources
                    ),
                })
            }
            SourceAlignment::BearishAlignment if self.bullish_average? <= 35.0 => {
                Some(Signal::Short {
                    strength: SignalStrength::Moderate,
                    reason: format!(
                        "Bearish sentiment alignment: {:.1}% across {} sources",
                        self.bullish_average?, self.available_sources
                    ),
                })
            }
            SourceAlignment::WideDivergence => {
                // 分歧大时可能有波动机会
                Some(Signal::Watch {
                    reason: "High sentiment divergence - potential volatility".to_string(),
                })
            }
            _ => None,
        }
    }
}
```

### 1.4 CLI 命令

```rust
// 在 src/cli/handlers.rs 添加
pub async fn handle_sentiment(symbol: &str, days: Option<u8>) -> anyhow::Result<()> {
    let aggregator = SentimentAggregator::from_env()?;
    let sentiment = aggregator.get_stock_sentiment(symbol, days).await?;

    match sentiment {
        Some(s) => {
            println!("📊 Sentiment Analysis for {}", s.symbol);
            println!("   Average Buzz: {:.1}", s.average_buzz);
            println!("   Alignment: {:?}", s.source_alignment);
            if let Some(bullish) = s.bullish_average {
                println!("   Bullish %: {:.1}%", bullish);
            }
            println!("\n   Sources:");
            for src in &s.sources {
                println!("   - {:?}: buzz={:.1}, trend={:?}",
                    src.source, src.buzz_score, src.trend);
            }
        }
        None => println!("No sentiment data available for {}", symbol),
    }
    Ok(())
}
```

---

## 2. 通知服务 (Notification Service)

### 2.1 OpenStock 实现分析

**Nodemailer 配置** (Gmail SMTP):
```typescript
transporter = nodemailer.createTransport({
    service: 'gmail',
    auth: { user: process.env.NODEMAILER_EMAIL, pass: process.env.NODEMAILER_PASSWORD },
    pool: true,
    maxConnections: 1,
    maxMessages: 3,
});
```

**邮件类型**:
- Welcome Email (欢迎邮件)
- News Summary (新闻摘要)
- Price Alert Upper (价格上涨提醒)
- Price Alert Lower (价格下跌提醒)
- Volume Alert (成交量异常)
- Inactive User Reminder (用户召回)

### 2.2 Rust 实现设计

**模块位置**: `src/notification/`

**目录结构**:
```
src/notification/
├── mod.rs
├── types.rs           # 通知类型定义
├── email/
│   ├── mod.rs
│   ├── smtp.rs        # SMTP 发送
│   ├── templates.rs   # HTML 模板
│   └── webhook.rs     # Webhook 发送 (可选)
└── channel.rs         # 多渠道分发
```

**核心类型** (`types.rs`):
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    PriceAlert {
        symbol: String,
        company: String,
        current_price: f64,
        target_price: f64,
        condition: AlertCondition,
        timestamp: DateTime<Utc>,
    },
    VolumeAlert {
        symbol: String,
        company: String,
        current_volume: u64,
        average_volume: u64,
        spike_percent: f64,
        current_price: f64,
        change_percent: f64,
        timestamp: DateTime<Utc>,
    },
    StrategySignal {
        strategy_name: String,
        symbol: String,
        signal_type: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    DailySummary {
        date: String,
        top_gainers: Vec<StockSummary>,
        top_losers: Vec<StockSummary>,
        market_breadth: MarketBreadth,
    },
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    Above,
    Below,
}

#[derive(Debug, Clone)]
pub struct StockSummary {
    pub symbol: String,
    pub name: String,
    pub change_percent: f64,
    pub volume: u64,
}

#[derive(Debug, Clone)]
pub struct MarketBreadth {
    pub advancing: u32,
    pub declining: u32,
    pub unchanged: u32,
}

pub struct NotificationConfig {
    pub email: Option<EmailConfig>,
    pub webhook_url: Option<String>,
}

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_name: String,
    pub from_email: String,
}
```

**SMTP 发送** (`email/smtp.rs`):
```rust
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

pub struct EmailSender {
    transport: SmtpTransport,
    from_name: String,
    from_email: String,
}

impl EmailSender {
    pub fn new(config: &EmailConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(
            config.smtp_user.clone(),
            config.smtp_pass.clone(),
        );

        let transport = SmtpTransport::relay(&config.smtp_host)?
            .credentials(creds)
            .port(config.smtp_port)
            .build();

        Ok(Self {
            transport,
            from_name: config.from_name.clone(),
            from_email: config.from_email.clone(),
        })
    }

    pub fn send(&self, to: &str, subject: &str, html: &str) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML_UTF_8)
                            .body(html.to_string())
                    )
            )?;

        self.transport.send(&email)?;
        Ok(())
    }
}
```

**HTML 模板** (`email/templates.rs`):
```rust
use handlebars::Handlebars;

lazy_static! {
    static ref HANDLEBARS: Handlebars<'static> = {
        let mut hb = Handlebars::new();
        hb.register_template_string("price_alert_upper", include_str!("templates/price_alert_upper.html"))
            .unwrap();
        hb.register_template_string("price_alert_lower", include_str!("templates/price_alert_lower.html"))
            .unwrap();
        hb.register_template_string("volume_alert", include_str!("templates/volume_alert.html"))
            .unwrap();
        hb.register_template_string("strategy_signal", include_str!("templates/strategy_signal.html"))
            .unwrap();
        hb
    };
}

pub fn render_price_alert_upper(
    symbol: &str,
    company: &str,
    current_price: f64,
    target_price: f64,
    timestamp: &str,
) -> String {
    let data = serde_json::json!({
        "symbol": symbol,
        "company": company,
        "currentPrice": format!("{:.2}", current_price),
        "targetPrice": format!("{:.2}", target_price),
        "timestamp": timestamp,
    });

    HANDLEBARS.render("price_alert_upper", &data).unwrap()
}
```

**多渠道分发** (`channel.rs`):
```rust
pub struct NotificationDispatcher {
    email: Option<EmailSender>,
    webhook: Option<WebhookSender>,
}

impl NotificationDispatcher {
    pub async fn dispatch(&self, event: &NotificationEvent, recipients: &[String]) -> anyhow::Result<()> {
        let subject = event.subject();
        let html = event.to_html();

        // 并行发送
        let mut tasks = vec![];

        if let Some(ref email) = self.email {
            for recipient in recipients {
                let email = email.clone();
                let to = recipient.clone();
                let subject = subject.clone();
                let html = html.clone();
                tasks.push(tokio::spawn(async move {
                    email.send(&to, &subject, &html)
                }));
            }
        }

        if let Some(ref webhook) = self.webhook {
            let webhook = webhook.clone();
            let payload = event.to_json();
            tasks.push(tokio::spawn(async move {
                webhook.send(&payload).await
            }));
        }

        join_all(tasks).await;
        Ok(())
    }
}
```

### 2.3 与 Quantix Monitor 集成

```rust
// 在 src/monitor/alert_handler.rs 中
use crate::notification::{NotificationDispatcher, NotificationEvent};

pub async fn check_and_notify_alerts(
    monitor: &MonitorService,
    notifier: &NotificationDispatcher,
    recipients: &[String],
) -> anyhow::Result<()> {
    let triggered = monitor.check_alerts().await?;

    for alert in triggered {
        let event = NotificationEvent::PriceAlert {
            symbol: alert.symbol.clone(),
            company: alert.company_name.clone().unwrap_or_default(),
            current_price: alert.current_price,
            target_price: alert.target_price,
            condition: alert.condition.into(),
            timestamp: chrono::Utc::now(),
        };

        notifier.dispatch(&event, recipients).await?;
    }

    Ok(())
}
```

---

## 3. AI 多 Provider (Multi-LLM Provider)

### 3.1 OpenStock 实现分析

**Provider 配置**:
```typescript
type AIProviderName = "gemini" | "minimax" | "siray";

// Gemini: 原生 REST API
// MiniMax: OpenAI 兼容 API
// Siray: OpenAI 兼容 API
```

**自动降级逻辑**:
```typescript
async function callAIProviderWithFallback(prompt: string): Promise<string> {
    const primary = process.env.AI_PROVIDER || "gemini";
    const fallback = getFallbackProviderName(primary);

    try {
        return await callAIProvider(prompt, primary);
    } catch (error) {
        console.error(`${primary} failed, switching to ${fallback}`);
        return await callAIProvider(prompt, fallback);
    }
}
```

### 3.2 Rust 实现设计

**模块位置**: `src/ai/`

**目录结构**:
```
src/ai/
├── mod.rs
├── types.rs           # 通用类型
├── provider/
│   ├── mod.rs
│   ├── traits.rs      # Provider trait
│   ├── gemini.rs      # Gemini 实现
│   ├── openai_compat.rs # OpenAI 兼容实现
│   └── config.rs      # 配置加载
├── client.rs          # 统一客户端
└── prompts/           # 预定义提示词
    ├── mod.rs
    └── market_analysis.rs
```

**Provider Trait** (`provider/traits.rs`):
```rust
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
    async fn generate_with_options(
        &self,
        prompt: &str,
        options: &GenerateOptions,
    ) -> anyhow::Result<GenerateResponse>;
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct GenerateResponse {
    pub text: String,
    pub tokens_used: Option<TokenUsage>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

**Gemini 实现** (`provider/gemini.rs`):
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            model: model.unwrap_or_else(|| "gemini-2.0-flash".to_string()),
        }
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    fn name(&self) -> &str { "gemini" }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key);

        let body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": prompt }]
            }]
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty response from Gemini"))
    }
}
```

**OpenAI 兼容实现** (`provider/openai_compat.rs`):
```rust
pub struct OpenAICompatibleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    name: &'static str,
}

impl OpenAICompatibleProvider {
    pub fn minimax(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.minimax.io/v1".to_string(),
            model: model.unwrap_or_else(|| "MiniMax-M2.7".to_string()),
            name: "minimax",
        }
    }

    pub fn siray(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.siray.ai/v1".to_string(),
            model: "siray-1.0-ultra".to_string(),
            name: "siray",
        }
    }
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    fn name(&self) -> &str { self.name }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "temperature": 0.7
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;

        response.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty response from {}", self.name))
    }
}
```

**统一客户端** (`client.rs`):
```rust
use std::sync::Arc;

pub struct AIClient {
    primary: Arc<dyn AIProvider>,
    fallback: Option<Arc<dyn AIProvider>>,
}

impl AIClient {
    pub fn from_env() -> anyhow::Result<Self> {
        let provider_name = std::env::var("AI_PROVIDER")
            .unwrap_or_else(|_| "gemini".to_string());

        let primary: Arc<dyn AIProvider> = match provider_name.as_str() {
            "gemini" => Arc::new(GeminiProvider::new(
                std::env::var("GEMINI_API_KEY")?,
                std::env::var("GEMINI_MODEL").ok(),
            )),
            "minimax" => Arc::new(OpenAICompatibleProvider::minimax(
                std::env::var("MINIMAX_API_KEY")?,
                std::env::var("MINIMAX_MODEL").ok(),
            )),
            "siray" => Arc::new(OpenAICompatibleProvider::siray(
                std::env::var("SIRAY_API_KEY")?,
            )),
            _ => return Err(anyhow::anyhow!("Unknown AI provider: {}", provider_name)),
        };

        let fallback = Self::get_fallback(&provider_name)?;

        Ok(Self { primary, fallback })
    }

    fn get_fallback(primary: &str) -> anyhow::Result<Option<Arc<dyn AIProvider>>> {
        if primary == "gemini" {
            if let Ok(key) = std::env::var("MINIMAX_API_KEY") {
                return Ok(Some(Arc::new(OpenAICompatibleProvider::minimax(key, None))));
            }
            if let Ok(key) = std::env::var("SIRAY_API_KEY") {
                return Ok(Some(Arc::new(OpenAICompatibleProvider::siray(key))));
            }
        } else if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            return Ok(Some(Arc::new(GeminiProvider::new(key, None))));
        }
        Ok(None)
    }

    pub async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        match self.primary.generate(prompt).await {
            Ok(response) => Ok(response),
            Err(e) => {
                if let Some(ref fallback) = self.fallback {
                    tracing::warn!("{} failed, using fallback: {}", self.primary.name(), e);
                    fallback.generate(prompt).await
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

### 3.3 预定义提示词 (`prompts/market_analysis.rs`)

```rust
pub fn stock_analysis_prompt(
    symbol: &str,
    kline_summary: &str,
    sentiment_summary: &str,
) -> String {
    format!(
        r#"You are a quantitative trading analyst. Analyze the following stock data and provide a brief assessment.

Stock: {}
Recent Price Action:
{}

Market Sentiment:
{}

Please provide:
1. Technical analysis summary (2-3 sentences)
2. Sentiment interpretation (1-2 sentences)
3. Risk level: Low/Medium/High
4. Suggested action: Watch/Buy/Avoid (with brief reason)

Keep response under 200 words."#,
        symbol, kline_summary, sentiment_summary
    )
}

pub fn news_summary_prompt(articles: &[NewsArticle]) -> String {
    let articles_text = articles
        .iter()
        .take(10)
        .map(|a| format!("- {}: {}", a.source, a.headline))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Summarize today's market news in Chinese (A股市场 focused).

Today's Top Headlines:
{}

Provide:
1. 市场整体趋势 (2-3句话)
2. 重点板块/概念 (列举3-5个)
3. 风险提示 (1句话)

Keep under 300 characters."#,
        articles_text
    )
}
```

---

## 4. Cargo 依赖

```toml
# Cargo.toml 添加
[dependencies]
# 现有依赖...

# 情绪分析
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 通知服务
lettre = "0.11"
handlebars = "5.1"

# AI Provider
async-trait = "0.1"

# 工具
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## 5. 环境变量配置

```bash
# .env 添加
# === 情绪分析 ===
ADANOS_API_KEY=your_key
ADANOS_BASE_URL=https://api.adanos.org  # 可选

# === 通知服务 ===
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
EMAIL_FROM_NAME=Quantix
EMAIL_FROM_ADDRESS=noreply@quantix.local
NOTIFICATION_RECIPIENTS=user1@example.com,user2@example.com

# Webhook (可选)
NOTIFICATION_WEBHOOK_URL=https://hooks.slack.com/services/xxx

# === AI Provider ===
AI_PROVIDER=gemini  # gemini | minimax | siray
GEMINI_API_KEY=your_key
GEMINI_MODEL=gemini-2.0-flash  # 可选
MINIMAX_API_KEY=your_key  # 可选
SIRAY_API_KEY=your_key  # 可选
```

---

## 6. 实施计划

### Phase 1: AI 多 Provider (1周)
- [ ] 创建 `src/ai/` 模块结构
- [ ] 实现 Provider trait
- [ ] 实现 Gemini provider
- [ ] 实现 OpenAI 兼容 provider
- [ ] 添加自动降级逻辑
- [ ] 添加预定义提示词
- [ ] 集成测试

### Phase 2: 情绪分析 (1.5周)
- [ ] 创建 `src/sentiment/` 模块
- [ ] 实现数据类型
- [ ] 实现 Adanos 客户端
- [ ] 实现聚合逻辑
- [ ] 添加信号生成
- [ ] CLI 命令 `quantix sentiment <symbol>`
- [ ] 与策略系统集成

### Phase 3: 通知服务 (1周)
- [ ] 创建 `src/notification/` 模块
- [ ] 实现 SMTP 发送器
- [ ] 移植 HTML 模板
- [ ] 实现事件类型
- [ ] 多渠道分发
- [ ] 与 Monitor 集成
- [ ] systemd timer 配置

### Phase 4: 集成测试 (0.5周)
- [ ] 端到端测试
- [ ] 文档更新
- [ ] CI/CD 更新

---

## 7. 验收标准

1. **情绪分析**
   - `quantix sentiment AAPL` 返回格式化输出
   - 支持 1-30 天回看
   - 信号可被策略使用

2. **通知服务**
   - 价格触发时自动发送邮件
   - 支持自定义收件人
   - HTML 邮件正确渲染

3. **AI Provider**
   - `quantix analyze <symbol>` 使用 AI 分析
   - Provider 失败时自动降级
   - 超时处理正常

---

## 8. 后续扩展

- **情绪数据源扩展**: 雪球、东方财富股吧、同花顺社区
- **通知渠道扩展**: Telegram Bot、企业微信、钉钉
- **AI Provider 扩展**: DeepSeek、Qwen、本地 LLM
- **模板定制**: 用户自定义邮件模板
