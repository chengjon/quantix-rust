use crate::bridge::client::BridgeHttpClient;
use crate::core::{QuantixError, Result};

pub const QMT_LIVE_BRIDGE_MODE_REQUIREMENT: &str = "bridge qmt.mode=live";
pub const QMT_LIVE_BRIDGE_COMMAND: &str = "execution bridge qmt-live";
pub const QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT: &str = "bridge qmt.supports includes order_submit";

pub async fn ensure_bridge_qmt_live_mode(client: &BridgeHttpClient) -> Result<()> {
    let capabilities = client
        .capabilities()
        .await
        .map_err(|err| QuantixError::Other(format!("QMT 实盘能力检查失败: {err}")))?;

    if !capabilities.qmt.enabled {
        return Err(QuantixError::Other(
            "QMT 实盘下单被拒绝: bridge qmt capability 未启用".to_string(),
        ));
    }

    if capabilities.qmt.mode != "live" {
        return Err(QuantixError::Other(format!(
            "QMT 实盘下单被拒绝: bridge qmt.mode={}，要求 bridge qmt.mode=live",
            capabilities.qmt.mode
        )));
    }

    if !capabilities
        .qmt
        .supports
        .iter()
        .any(|support| support == "order_submit")
    {
        return Err(QuantixError::Other(format!(
            "QMT 实盘下单被拒绝: 缺少 order_submit 能力，要求 {QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT}"
        )));
    }

    Ok(())
}
