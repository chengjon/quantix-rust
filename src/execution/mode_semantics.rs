use crate::execution::adapter::ExecutionChannel;

pub const PAPER_IMMEDIATE_CHANNEL: &str = "paper_immediate";
pub const PAPER_SIM_LIFECYCLE_CHANNEL: &str = "paper_sim_lifecycle";
pub const QMT_LIVE_CHANNEL: &str = "qmt_live";

pub const PAPER_IMMEDIATE_STORAGE_NAMESPACE: &str = "paper-immediate";
pub const PAPER_SIM_LIFECYCLE_STORAGE_NAMESPACE: &str = "paper-sim-lifecycle";
pub const QMT_LIVE_STORAGE_NAMESPACE: &str = "qmt-live";

pub const PAPER_CONFIGURED_MODE: &str = "paper";
pub const EXECUTION_MODE_RUNTIME_SWITCHING_ALLOWED: bool = false;

pub const PAPER_IMMEDIATE_RISK_NOTICE: &str = "[paper_immediate] local ledger immediate-fill only; no broker submission, no market matching, not for liquidity or slippage validation";
pub const PAPER_SIM_LIFECYCLE_RISK_NOTICE: &str =
    "[paper_sim_lifecycle] local simulated order lifecycle; broker behavior may differ";
pub const QMT_LIVE_RISK_NOTICE: &str =
    "[qmt_live] real-money execution path; miniQMT and broker state are authoritative";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionModeStorageBinding {
    pub configured_mode: &'static str,
    pub channel: &'static str,
    pub storage_namespace: &'static str,
    pub runtime_switching_allowed: bool,
}

/// 按 channel 字符串返回对应的 risk notice（paper_immediate / paper_sim_lifecycle / qmt_live），未知返回 `None`。
pub fn risk_notice_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        PAPER_IMMEDIATE_CHANNEL => Some(PAPER_IMMEDIATE_RISK_NOTICE),
        PAPER_SIM_LIFECYCLE_CHANNEL => Some(PAPER_SIM_LIFECYCLE_RISK_NOTICE),
        QMT_LIVE_CHANNEL => Some(QMT_LIVE_RISK_NOTICE),
        _ => None,
    }
}

/// `risk_notice_for_channel` 的 ExecutionChannel 入参适配，按 channel.as_str() 转发。
pub fn risk_notice_for_execution_channel(channel: ExecutionChannel) -> Option<&'static str> {
    risk_notice_for_channel(channel.as_str())
}

/// 按 channel 字符串返回对应的 storage namespace（如 "qmt-live"），未知返回 `None`。
pub fn storage_namespace_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        PAPER_IMMEDIATE_CHANNEL => Some(PAPER_IMMEDIATE_STORAGE_NAMESPACE),
        PAPER_SIM_LIFECYCLE_CHANNEL => Some(PAPER_SIM_LIFECYCLE_STORAGE_NAMESPACE),
        QMT_LIVE_CHANNEL => Some(QMT_LIVE_STORAGE_NAMESPACE),
        _ => None,
    }
}

/// `storage_namespace_for_channel` 的 ExecutionChannel 入参适配，按 channel.as_str() 转发。
pub fn storage_namespace_for_execution_channel(channel: ExecutionChannel) -> Option<&'static str> {
    storage_namespace_for_channel(channel.as_str())
}

/// 按 configured_mode 字符串返回 storage 绑定（configured_mode + channel + namespace + 是否允许运行时切换）；未知返回 `None`。
pub fn storage_binding_for_configured_execution_mode(
    configured_mode: &str,
) -> Option<ExecutionModeStorageBinding> {
    match configured_mode {
        PAPER_CONFIGURED_MODE | PAPER_IMMEDIATE_CHANNEL => Some(ExecutionModeStorageBinding {
            configured_mode: PAPER_CONFIGURED_MODE,
            channel: PAPER_IMMEDIATE_CHANNEL,
            storage_namespace: PAPER_IMMEDIATE_STORAGE_NAMESPACE,
            runtime_switching_allowed: EXECUTION_MODE_RUNTIME_SWITCHING_ALLOWED,
        }),
        PAPER_SIM_LIFECYCLE_CHANNEL => Some(ExecutionModeStorageBinding {
            configured_mode: PAPER_SIM_LIFECYCLE_CHANNEL,
            channel: PAPER_SIM_LIFECYCLE_CHANNEL,
            storage_namespace: PAPER_SIM_LIFECYCLE_STORAGE_NAMESPACE,
            runtime_switching_allowed: EXECUTION_MODE_RUNTIME_SWITCHING_ALLOWED,
        }),
        QMT_LIVE_CHANNEL => Some(ExecutionModeStorageBinding {
            configured_mode: QMT_LIVE_CHANNEL,
            channel: QMT_LIVE_CHANNEL,
            storage_namespace: QMT_LIVE_STORAGE_NAMESPACE,
            runtime_switching_allowed: EXECUTION_MODE_RUNTIME_SWITCHING_ALLOWED,
        }),
        _ => None,
    }
}

/// 以 tracing::warn! 输出指定 channel 的 risk notice；未知 channel 静默无输出。
pub fn log_execution_mode_risk_notice(channel: &'static str) {
    if let Some(notice) = risk_notice_for_channel(channel) {
        tracing::warn!(
            target: "quantix::execution_mode",
            execution_channel = channel,
            notice = notice,
            "{notice}"
        );
    }
}
