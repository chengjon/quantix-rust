use crate::bridge::client::BridgeHttpClient;
use crate::core::{QuantixError, Result};

pub const QMT_LIVE_BRIDGE_MODE_REQUIREMENT: &str = "bridge qmt.mode=live";
pub const QMT_LIVE_BRIDGE_COMMAND: &str = "execution bridge qmt-live";
pub const QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT: &str = "bridge qmt.supports includes order_submit";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QmtLiveGateFailure {
    CapabilityCheckFailed(String),
    CapabilityDisabled,
    ModeNotLive { observed_mode: String },
    MissingOrderSubmitSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QmtLiveModeFailureKind {
    NonLive,
    Ambiguous,
}

impl QmtLiveGateFailure {
    pub fn to_quantix_error(&self) -> QuantixError {
        match self {
            Self::CapabilityCheckFailed(message) => {
                QuantixError::Other(format!("QMT 实盘能力检查失败: {message}"))
            }
            Self::CapabilityDisabled => {
                QuantixError::Other("QMT 实盘下单被拒绝: bridge qmt capability 未启用".to_string())
            }
            Self::ModeNotLive { observed_mode } => QuantixError::Other(format!(
                "QMT 实盘下单被拒绝: bridge qmt.mode={}，要求 bridge qmt.mode=live",
                observed_mode
            )),
            Self::MissingOrderSubmitSupport => QuantixError::Other(format!(
                "QMT 实盘下单被拒绝: 缺少 order_submit 能力，要求 {}",
                QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT
            )),
        }
    }

    pub fn mode_failure_kind(&self) -> Option<QmtLiveModeFailureKind> {
        match self {
            Self::ModeNotLive { observed_mode } if is_ambiguous_qmt_mode(observed_mode) => {
                Some(QmtLiveModeFailureKind::Ambiguous)
            }
            Self::ModeNotLive { .. } => Some(QmtLiveModeFailureKind::NonLive),
            _ => None,
        }
    }
}

fn is_ambiguous_qmt_mode(mode: &str) -> bool {
    let normalized = mode.trim();
    normalized.is_empty()
        || normalized != mode
        || matches!(normalized, "unknown" | "unsupported" | "unavailable")
}

pub async fn check_bridge_qmt_live_mode(
    client: &BridgeHttpClient,
) -> std::result::Result<(), QmtLiveGateFailure> {
    let capabilities = client
        .capabilities()
        .await
        .map_err(|err| QmtLiveGateFailure::CapabilityCheckFailed(err.to_string()))?;

    if !capabilities.qmt.enabled {
        return Err(QmtLiveGateFailure::CapabilityDisabled);
    }

    if capabilities.qmt.mode != "live" {
        return Err(QmtLiveGateFailure::ModeNotLive {
            observed_mode: capabilities.qmt.mode,
        });
    }

    if !capabilities
        .qmt
        .supports
        .iter()
        .any(|support| support == "order_submit")
    {
        return Err(QmtLiveGateFailure::MissingOrderSubmitSupport);
    }

    Ok(())
}

pub async fn ensure_bridge_qmt_live_mode(client: &BridgeHttpClient) -> Result<()> {
    check_bridge_qmt_live_mode(client)
        .await
        .map_err(|err| err.to_quantix_error())
}
