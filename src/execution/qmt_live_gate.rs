use crate::bridge::client::BridgeHttpClient;
use crate::core::{QuantixError, Result};

pub const QMT_LIVE_BRIDGE_MODE_REQUIREMENT: &str = "bridge qmt.mode=live";
pub const QMT_LIVE_BRIDGE_COMMAND: &str = "execution bridge qmt-live";
pub const QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT: &str = "bridge qmt.supports includes order_submit";

/// qmt_live gate 失败原因：CapabilityCheckFailed capability 检查异常（如 503/超时）、CapabilityDisabled bridge qmt capability 未启用、ModeNotLive bridge qmt.mode 不是 live、MissingOrderSubmitSupport 缺少 order_submit 能力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QmtLiveGateFailure {
    CapabilityCheckFailed(String),
    CapabilityDisabled,
    ModeNotLive { observed_mode: String },
    MissingOrderSubmitSupport,
}

/// ModeNotLive 的细分：NonLive 明确非 live（如 preview_only）、Ambiguous 模糊（空串、首尾空白、unknown/unsupported/unavailable 等占位值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QmtLiveModeFailureKind {
    NonLive,
    Ambiguous,
}

impl QmtLiveGateFailure {
    /// 把 gate failure 翻译为面向用户的 QuantixError::Other，错误信息明确指出被拒原因（capability 未启用 / mode 不是 live / 缺少 order_submit 支持 / capability 检查异常），并附上期望值与观察值。
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

    /// 仅对 ModeNotLive 失败返回 QmtLiveModeFailureKind（ambiguous / non_live），其他失败返回 None。ambiguous 判定：观察到的 mode 为空、首尾含空白、或属于 unknown/unsupported/unavailable 这些占位字符串。
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

/// 向 bridge 拉取 capabilities，校验 qmt.enabled=true、qmt.mode="live"、qmt.supports 含 "order_submit" 三项；任一不满足返回带原因的 QmtLiveGateFailure。capabilities 调用失败归入 CapabilityCheckFailed。
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

/// 等价于 check_bridge_qmt_live_mode，但把 QmtLiveGateFailure 翻译为 QuantixError 后返回；适合实盘下单前的硬性 gate，调用方拿到 Err 直接终止流程即可。
pub async fn ensure_bridge_qmt_live_mode(client: &BridgeHttpClient) -> Result<()> {
    check_bridge_qmt_live_mode(client)
        .await
        .map_err(|err| err.to_quantix_error())
}
