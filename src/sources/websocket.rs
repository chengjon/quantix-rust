#![allow(clippy::collapsible_if)]

/// WebSocket 实时行情客户端
///
/// 支持多数据源的 WebSocket 实时行情订阅
use crate::core::{QuantixError, Result};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};

/// WebSocket 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// 实时行情消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeQuote {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 最新价
    pub price: f64,
    /// 昨收价
    pub preclose: f64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 成交量
    pub volume: i64,
    /// 成交额
    pub amount: f64,
    /// 涨跌幅 (%)
    pub change_percent: f64,
    /// 买一价
    pub bid1: Option<f64>,
    /// 卖一价
    pub ask1: Option<f64>,
    /// 时间戳
    pub timestamp: i64,
}

/// 订阅状态
#[derive(Debug, Clone)]
pub struct Subscription {
    /// 股票代码
    pub code: String,
    /// 是否已确认
    pub confirmed: bool,
    /// 订阅时间
    pub subscribed_at: DateTime<Utc>,
}

/// WebSocket 配置
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL
    pub url: String,
    /// 心跳间隔 (秒)
    pub heartbeat_interval: u64,
    /// 重连间隔 (秒)
    pub reconnect_interval: u64,
    /// 最大重连次数
    pub max_reconnect: usize,
    /// 消息缓冲区大小
    pub buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "wss://push2.eastmoney.com/api/qt/stock/klt".to_string(),
            heartbeat_interval: 30,
            reconnect_interval: 5,
            max_reconnect: 10,
            buffer_size: 1000,
        }
    }
}

/// WebSocket 客户端
pub struct WebSocketClient {
    /// 配置
    config: WebSocketConfig,
    /// 连接状态
    state: Arc<RwLock<ConnectionState>>,
    /// 订阅列表
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    /// 消息发送器
    message_tx: Arc<Mutex<Option<mpsc::UnboundedSender<RealtimeQuote>>>>,
    /// 运行标志
    running: Arc<RwLock<bool>>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_tx: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_url(url: String) -> Self {
        Self::new(WebSocketConfig {
            url,
            ..Default::default()
        })
    }

    /// 获取连接状态
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// 获取订阅列表
    pub async fn subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.keys().cloned().collect()
    }

    /// 订阅股票
    pub async fn subscribe(&self, codes: &[String]) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        let now = Utc::now();

        for code in codes {
            subs.insert(
                code.clone(),
                Subscription {
                    code: code.clone(),
                    confirmed: false,
                    subscribed_at: now,
                },
            );
        }

        info!("订阅股票: {:?}", codes);
        Ok(())
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, codes: &[String]) -> Result<()> {
        let mut subs = self.subscriptions.write().await;

        for code in codes {
            subs.remove(code);
        }

        info!("取消订阅: {:?}", codes);
        Ok(())
    }

    /// 设置消息回调
    pub fn set_message_handler(&self, tx: mpsc::UnboundedSender<RealtimeQuote>) {
        let mut message_tx = self.message_tx.blocking_lock();
        *message_tx = Some(tx);
    }

    /// 启动连接
    pub async fn connect(&self) -> Result<()> {
        *self.running.write().await = true;
        *self.state.write().await = ConnectionState::Connecting;

        let url = self.config.url.clone();
        let state = self.state.clone();
        let subscriptions = self.subscriptions.clone();
        let message_tx = self.message_tx.clone();
        let running = self.running.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        let reconnect_interval = self.config.reconnect_interval;
        let max_reconnect = self.config.max_reconnect;

        tokio::spawn(async move {
            let mut reconnect_count = 0;

            while *running.read().await {
                // 连接 WebSocket
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        *state.write().await = ConnectionState::Connected;
                        info!("WebSocket 连接成功");
                        reconnect_count = 0;

                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        // 发送订阅消息
                        let subs = subscriptions.read().await;
                        let codes: Vec<_> = subs.keys().cloned().collect();
                        drop(subs);

                        if !codes.is_empty() {
                            if let Err(e) = Self::send_subscribe(&mut ws_sender, &codes).await {
                                error!("发送订阅消息失败: {}", e);
                            }
                        }

                        // 心跳定时器通道
                        let (ping_tx, mut ping_rx) = mpsc::unbounded_channel::<()>();
                        let running_heart = running.clone();
                        tokio::spawn(async move {
                            while *running_heart.read().await {
                                tokio::time::sleep(tokio::time::Duration::from_secs(
                                    heartbeat_interval,
                                ))
                                .await;
                                if ping_tx.send(()).is_err() {
                                    break;
                                }
                            }
                        });

                        // 消息处理循环 (接收 + 心跳发送)
                        loop {
                            tokio::select! {
                                // 接收 WebSocket 消息
                                msg = ws_receiver.next() => {
                                    match msg {
                                        Some(Ok(msg)) => {
                                            match msg {
                                                Message::Text(text) => {
                                                    if let Some(quote) = Self::parse_message(&text) {
                                                        if let Some(tx) = message_tx.lock().await.as_ref() {
                                                            let _ = tx.send(quote);
                                                        }
                                                    }
                                                }
                                                Message::Ping(data) => {
                                                    if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                                                        error!("响应 Pong 失败: {}", e);
                                                        break;
                                                    }
                                                }
                                                Message::Pong(_) => {
                                                    debug!("收到 Pong");
                                                }
                                                Message::Close(_) => {
                                                    warn!("服务器关闭连接");
                                                    break;
                                                }
                                                _ => {}
                                            }
                                        }
                                        Some(Err(e)) => {
                                            error!("WebSocket 错误: {}", e);
                                            break;
                                        }
                                        None => {
                                            warn!("WebSocket 流结束");
                                            break;
                                        }
                                    }
                                }
                                // 发送心跳
                                _ = ping_rx.recv() => {
                                    if let Err(e) = ws_sender.send(Message::Ping(vec![1])).await {
                                        error!("发送心跳失败: {}", e);
                                        break;
                                    }
                                }
                                // 检查运行状态
                                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                                    if !*running.read().await {
                                        break;
                                    }
                                }
                            }
                        }

                        *state.write().await = ConnectionState::Disconnected;
                    }
                    Err(e) => {
                        error!("WebSocket 连接失败: {}", e);
                        *state.write().await = ConnectionState::Disconnected;

                        reconnect_count += 1;
                        if reconnect_count >= max_reconnect {
                            error!("超过最大重连次数");
                            *running.write().await = false;
                            break;
                        }
                    }
                }

                // 等待重连
                if *running.read().await {
                    *state.write().await = ConnectionState::Reconnecting;
                    info!("等待 {} 秒后重连...", reconnect_interval);
                    tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_interval)).await;
                }
            }

            info!("WebSocket 连接已关闭");
        });

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Result<()> {
        *self.running.write().await = false;
        *self.state.write().await = ConnectionState::Disconnected;
        info!("WebSocket 连接已断开");
        Ok(())
    }

    /// 发送订阅消息
    async fn send_subscribe(
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        codes: &[String],
    ) -> Result<()> {
        // 构建订阅消息 (东方财富格式)
        let subscribe_msg = serde_json::json!({
            "cmd": "sub",
            "data": codes
        });

        ws_sender
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .map_err(|e| QuantixError::Other(format!("发送订阅消息失败: {}", e)))?;

        debug!("发送订阅: {:?}", codes);
        Ok(())
    }

    /// 解析行情消息
    fn parse_message(text: &str) -> Option<RealtimeQuote> {
        // 简化实现，实际需要根据具体数据源格式解析
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(obj) = data.as_object() {
                // 东方财富格式示例
                if let Some(code) = obj.get("code").and_then(|v| v.as_str()) {
                    return Some(RealtimeQuote {
                        code: code.to_string(),
                        name: obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        price: obj.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        preclose: obj.get("preclose").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        open: obj.get("open").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        high: obj.get("high").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        low: obj.get("low").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        volume: obj.get("volume").and_then(|v| v.as_i64()).unwrap_or(0),
                        amount: obj.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        change_percent: obj
                            .get("change_percent")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        bid1: obj.get("bid1").and_then(|v| v.as_f64()),
                        ask1: obj.get("ask1").and_then(|v| v.as_f64()),
                        timestamp: chrono::Utc::now().timestamp(),
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.reconnect_interval, 5);
        assert_eq!(config.max_reconnect, 10);
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_connection_state() {
        let state1 = ConnectionState::Disconnected;
        let state2 = ConnectionState::Connected;
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_realtime_quote() {
        let quote = RealtimeQuote {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 10.50,
            preclose: 10.00,
            open: 10.20,
            high: 10.60,
            low: 10.10,
            volume: 1000000,
            amount: 10500000.0,
            change_percent: 5.0,
            bid1: Some(10.50),
            ask1: Some(10.51),
            timestamp: 1640000000,
        };

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.price, 10.50);
        assert_eq!(quote.change_percent, 5.0);
    }

    #[test]
    fn test_subscription() {
        let sub = Subscription {
            code: "000001".to_string(),
            confirmed: true,
            subscribed_at: Utc::now(),
        };

        assert_eq!(sub.code, "000001");
        assert!(sub.confirmed);
    }

    #[tokio::test]
    async fn test_websocket_client_create() {
        let client = WebSocketClient::with_default_url("ws://localhost:8080".to_string());
        assert_eq!(client.state().await, ConnectionState::Disconnected);
        assert!(client.subscriptions().await.is_empty());
    }

    #[tokio::test]
    async fn test_websocket_subscribe() {
        let client = WebSocketClient::with_default_url("ws://localhost:8080".to_string());
        client
            .subscribe(&["000001".to_string(), "000002".to_string()])
            .await
            .unwrap();

        let subs = client.subscriptions().await;
        assert_eq!(subs.len(), 2);
        assert!(subs.contains(&"000001".to_string()));
    }

    #[tokio::test]
    async fn test_websocket_unsubscribe() {
        let client = WebSocketClient::with_default_url("ws://localhost:8080".to_string());
        client
            .subscribe(&["000001".to_string(), "000002".to_string()])
            .await
            .unwrap();
        client.unsubscribe(&["000001".to_string()]).await.unwrap();

        let subs = client.subscriptions().await;
        assert_eq!(subs.len(), 1);
        assert!(!subs.contains(&"000001".to_string()));
    }
}
