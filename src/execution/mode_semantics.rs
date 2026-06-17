pub const PAPER_IMMEDIATE_CHANNEL: &str = "paper_immediate";
pub const PAPER_SIM_LIFECYCLE_CHANNEL: &str = "paper_sim_lifecycle";
pub const QMT_LIVE_CHANNEL: &str = "qmt_live";

pub const PAPER_IMMEDIATE_STORAGE_NAMESPACE: &str = "paper-immediate";
pub const PAPER_SIM_LIFECYCLE_STORAGE_NAMESPACE: &str = "paper-sim-lifecycle";
pub const QMT_LIVE_STORAGE_NAMESPACE: &str = "qmt-live";

pub const PAPER_IMMEDIATE_RISK_NOTICE: &str = "[paper_immediate] local ledger immediate-fill only; no broker submission, no market matching, not for liquidity or slippage validation";
pub const PAPER_SIM_LIFECYCLE_RISK_NOTICE: &str =
    "[paper_sim_lifecycle] local simulated order lifecycle; broker behavior may differ";
pub const QMT_LIVE_RISK_NOTICE: &str =
    "[qmt_live] real-money execution path; miniQMT and broker state are authoritative";

pub fn risk_notice_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        PAPER_IMMEDIATE_CHANNEL => Some(PAPER_IMMEDIATE_RISK_NOTICE),
        PAPER_SIM_LIFECYCLE_CHANNEL => Some(PAPER_SIM_LIFECYCLE_RISK_NOTICE),
        QMT_LIVE_CHANNEL => Some(QMT_LIVE_RISK_NOTICE),
        _ => None,
    }
}

pub fn storage_namespace_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        PAPER_IMMEDIATE_CHANNEL => Some(PAPER_IMMEDIATE_STORAGE_NAMESPACE),
        PAPER_SIM_LIFECYCLE_CHANNEL => Some(PAPER_SIM_LIFECYCLE_STORAGE_NAMESPACE),
        QMT_LIVE_CHANNEL => Some(QMT_LIVE_STORAGE_NAMESPACE),
        _ => None,
    }
}

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
