use crate::core::{QuantixError, Result};
use crate::screener::{PresetInvocation, PresetKind};
use std::collections::BTreeMap;

pub fn parse_preset_invocation(spec: &str) -> Result<PresetInvocation> {
    let (name, raw_params) = spec
        .split_once(':')
        .ok_or_else(|| QuantixError::Other(format!("无效的 preset 格式: {}", spec)))?;
    let kind = parse_preset_kind(name)?;
    let params = parse_params(raw_params)?;

    validate_params(&kind, &params)?;

    Ok(PresetInvocation { kind, params })
}

fn parse_preset_kind(name: &str) -> Result<PresetKind> {
    match name {
        "close_above_ma" => Ok(PresetKind::CloseAboveMa),
        "close_below_ma" => Ok(PresetKind::CloseBelowMa),
        "rsi_gte" => Ok(PresetKind::RsiGte),
        "rsi_lte" => Ok(PresetKind::RsiLte),
        "volume_ratio_gte" => Ok(PresetKind::VolumeRatioGte),
        other => Err(QuantixError::Other(format!("未知的 preset: {}", other))),
    }
}

fn parse_params(raw_params: &str) -> Result<BTreeMap<String, String>> {
    let mut params = BTreeMap::new();

    for item in raw_params.split(',') {
        if item.trim().is_empty() {
            continue;
        }

        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| QuantixError::Other(format!("无效的参数格式: {}", item)))?;

        if key.trim().is_empty() || value.trim().is_empty() {
            return Err(QuantixError::Other(format!("参数不能为空: {}", item)));
        }

        params.insert(key.trim().to_string(), value.trim().to_string());
    }

    if params.is_empty() {
        return Err(QuantixError::Other("preset 参数不能为空".to_string()));
    }

    Ok(params)
}

fn validate_params(kind: &PresetKind, params: &BTreeMap<String, String>) -> Result<()> {
    match kind {
        PresetKind::CloseAboveMa | PresetKind::CloseBelowMa => {
            validate_exact_keys(params, &["period"])?;
            parse_usize_param(params, "period")?;
        }
        PresetKind::RsiGte | PresetKind::RsiLte => {
            validate_exact_keys(params, &["period", "value"])?;
            parse_usize_param(params, "period")?;
            parse_f64_param(params, "value")?;
        }
        PresetKind::VolumeRatioGte => {
            validate_exact_keys(params, &["window", "value"])?;
            parse_usize_param(params, "window")?;
            parse_f64_param(params, "value")?;
        }
    }

    Ok(())
}

fn validate_exact_keys(params: &BTreeMap<String, String>, allowed: &[&str]) -> Result<()> {
    for key in params.keys() {
        if !allowed.iter().any(|item| item == &key.as_str()) {
            return Err(QuantixError::Other(format!("不支持的参数: {}", key)));
        }
    }

    for key in allowed {
        if !params.contains_key(*key) {
            return Err(QuantixError::Other(format!("缺少必需参数: {}", key)));
        }
    }

    Ok(())
}

fn parse_usize_param(params: &BTreeMap<String, String>, key: &str) -> Result<usize> {
    params[key]
        .parse::<usize>()
        .map_err(|_| QuantixError::Other(format!("参数 {} 必须是正整数", key)))
}

fn parse_f64_param(params: &BTreeMap<String, String>, key: &str) -> Result<f64> {
    params[key]
        .parse::<f64>()
        .map_err(|_| QuantixError::Other(format!("参数 {} 必须是数字", key)))
}
