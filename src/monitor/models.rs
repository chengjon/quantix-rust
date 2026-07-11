use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 监控自选股行情展示行：code 标的、group 分组、tags 自定义标签、可选 last_price/change_pct/quote_time 实时行情、可选 note 备注。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorQuoteRow {
    pub code: String,
    pub group: String,
    pub tags: Vec<String>,
    pub last_price: Option<f64>,
    pub change_pct: Option<f64>,
    pub quote_time: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

/// 价格预警方向：Above 价格上穿阈值、Below 价格下穿阈值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceAlertKind {
    Above,
    Below,
}

/// 单条价格预警：id 主键、code 标的、kind 预警方向、target_price 阈值价、created_at 创建时间、last_triggered_at 最近触发时间。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: i64,
    pub code: String,
    pub kind: PriceAlertKind,
    pub target_price: f64,
    pub created_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

/// 监控事件类型：PriceAlert 价格预警、StopLoss 止损、StopProfit 止盈、TrailingStop 移动止损。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorEventType {
    PriceAlert,
    StopLoss,
    StopProfit,
    TrailingStop,
}

/// 监控运行模式：Foreground 前台运行（CLI）、Daemon 守护进程模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorRunMode {
    Foreground,
    Daemon,
}

/// 新产生的监控事件（待入库）：event_time/event_type/code、可选 price、message 文本、source_type/source_key 来源标识、可选 observed_at、run_mode 运行模式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewMonitorEvent {
    pub event_time: DateTime<Utc>,
    pub event_type: MonitorEventType,
    pub code: String,
    pub price: Option<f64>,
    pub message: String,
    pub source_type: String,
    pub source_key: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub run_mode: MonitorRunMode,
}

/// 监控事件入库记录：id 主键 + NewMonitorEvent 全字段，由 monitor_events 表返回。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorEventRow {
    pub id: i64,
    pub event_time: DateTime<Utc>,
    pub event_type: MonitorEventType,
    pub code: String,
    pub price: Option<f64>,
    pub message: String,
    pub source_type: String,
    pub source_key: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub run_mode: MonitorRunMode,
}

/// 监控事件查询过滤器：limit 行数上限、可选 code 标的过滤、可选 event_type 类型过滤。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorEventFilter {
    pub limit: usize,
    pub code: Option<String>,
    pub event_type: Option<MonitorEventType>,
}

/// 已触发的价格预警快照：alert_id、code、kind、target_price 阈值、current_price 当前价、triggered_at 触发时间。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggeredAlert {
    pub alert_id: i64,
    pub code: String,
    pub kind: PriceAlertKind,
    pub target_price: f64,
    pub current_price: f64,
    pub triggered_at: Option<DateTime<Utc>>,
}

/// 监控自选股快照：rows 行情行列表、triggered_alerts 本次扫描触发的预警、warnings 扫描过程中的告警信息（缺行情等）。Default 为全空。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MonitorWatchlistSnapshot {
    pub rows: Vec<MonitorQuoteRow>,
    pub triggered_alerts: Vec<TriggeredAlert>,
    pub warnings: Vec<String>,
}
