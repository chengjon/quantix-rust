// 监控模块
//
/// Phase 16: 实时监控系统
/// - 信号监控 (signal_monitor) - 策略信号实时追踪
/// - 持仓监控 (position_monitor) - 持仓状态实时更新
/// - 性能监控 (performance_monitor) - 实时性能指标计算
/// - 告警系统 (alert) - 阈值告警通知
/// - 通知系统 (notification) - 多渠道通知推送
pub mod alert;
pub mod health;
pub mod metrics;
pub mod notification;
pub mod performance_monitor;
pub mod position_monitor;
pub mod signal_monitor;

// Phase 16 导出
pub use alert::{Alert, AlertConfig, AlertLevel, AlertManager, AlertThreshold, AlertType};
pub use health::{ComponentHealth, HealthCheck, HealthRegistry, HealthStatus, SystemHealth};
pub use metrics::{MetricsCollector, MetricsExporter, MetricsFormat};
pub use notification::{
    DesktopSender, LogSender, Notification, NotificationChannel, NotificationConfig,
    NotificationService, QuietHours, WebhookSender,
};
pub use performance_monitor::{PerformanceMonitor, PerformanceMonitorConfig, RealtimeMetrics};
pub use position_monitor::{PositionMonitor, PositionMonitorConfig, PositionSnapshot};
pub use signal_monitor::{SignalEvent, SignalMonitor, SignalMonitorConfig, SignalStats};
