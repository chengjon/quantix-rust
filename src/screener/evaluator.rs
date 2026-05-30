use crate::analysis::indicators::{ma, rsi};
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::screener::{PresetInvocation, PresetKind, RuleMatchDetail};
use rust_decimal::Decimal;

pub fn required_lookback(invocation: &PresetInvocation) -> Result<usize> {
    match invocation.kind {
        PresetKind::CloseAboveMa | PresetKind::CloseBelowMa => {
            get_usize_param(invocation, "period")
        }
        PresetKind::RsiGte | PresetKind::RsiLte => get_usize_param(invocation, "period")?
            .checked_add(1)
            .ok_or_else(|| QuantixError::Other("参数 period 过大".to_string())),
        PresetKind::VolumeRatioGte => get_usize_param(invocation, "window"),
    }
}

pub fn evaluate_preset(invocation: &PresetInvocation, klines: &[Kline]) -> Result<RuleMatchDetail> {
    let required = required_lookback(invocation)?;
    if klines.len() < required {
        return Ok(RuleMatchDetail {
            preset_name: preset_name(&invocation.kind).to_string(),
            params: invocation.params.clone(),
            actual_value: None,
            threshold_value: None,
            matched: false,
            reason: Some(format!("数据不足: 至少需要 {} 条日线", required)),
        });
    }

    let closes: Vec<Decimal> = klines.iter().map(|item| item.close).collect();
    let latest_close = *closes
        .last()
        .ok_or_else(|| QuantixError::Other("缺少最新收盘价".to_string()))?;

    match invocation.kind {
        PresetKind::CloseAboveMa => {
            let period = get_usize_param(invocation, "period")?;
            let latest_ma = ma(&closes, period)
                .last()
                .and_then(|value| *value)
                .ok_or_else(|| QuantixError::Other("无法计算 MA".to_string()))?;
            Ok(RuleMatchDetail {
                preset_name: preset_name(&invocation.kind).to_string(),
                params: invocation.params.clone(),
                actual_value: Some(latest_close),
                threshold_value: Some(latest_ma),
                matched: latest_close > latest_ma,
                reason: None,
            })
        }
        PresetKind::CloseBelowMa => {
            let period = get_usize_param(invocation, "period")?;
            let latest_ma = ma(&closes, period)
                .last()
                .and_then(|value| *value)
                .ok_or_else(|| QuantixError::Other("无法计算 MA".to_string()))?;
            Ok(RuleMatchDetail {
                preset_name: preset_name(&invocation.kind).to_string(),
                params: invocation.params.clone(),
                actual_value: Some(latest_close),
                threshold_value: Some(latest_ma),
                matched: latest_close < latest_ma,
                reason: None,
            })
        }
        PresetKind::RsiGte => {
            let period = get_usize_param(invocation, "period")?;
            let threshold = get_decimal_param(invocation, "value")?;
            let latest_rsi = rsi(&closes, period)
                .last()
                .and_then(|value| *value)
                .ok_or_else(|| QuantixError::Other("无法计算 RSI".to_string()))?;
            Ok(RuleMatchDetail {
                preset_name: preset_name(&invocation.kind).to_string(),
                params: invocation.params.clone(),
                actual_value: Some(latest_rsi),
                threshold_value: Some(threshold),
                matched: latest_rsi >= threshold,
                reason: None,
            })
        }
        PresetKind::RsiLte => {
            let period = get_usize_param(invocation, "period")?;
            let threshold = get_decimal_param(invocation, "value")?;
            let latest_rsi = rsi(&closes, period)
                .last()
                .and_then(|value| *value)
                .ok_or_else(|| QuantixError::Other("无法计算 RSI".to_string()))?;
            Ok(RuleMatchDetail {
                preset_name: preset_name(&invocation.kind).to_string(),
                params: invocation.params.clone(),
                actual_value: Some(latest_rsi),
                threshold_value: Some(threshold),
                matched: latest_rsi <= threshold,
                reason: None,
            })
        }
        PresetKind::VolumeRatioGte => {
            let window = get_usize_param(invocation, "window")?;
            let threshold = get_decimal_param(invocation, "value")?;
            let latest_volume = Decimal::from(
                klines
                    .last()
                    .ok_or_else(|| QuantixError::Other("缺少最新成交量".to_string()))?
                    .volume,
            );
            let avg_volume = klines[klines.len() - window..]
                .iter()
                .map(|item| Decimal::from(item.volume))
                .sum::<Decimal>()
                / Decimal::from(window as i64);
            let ratio = if avg_volume == Decimal::ZERO {
                Decimal::ZERO
            } else {
                latest_volume / avg_volume
            };

            Ok(RuleMatchDetail {
                preset_name: preset_name(&invocation.kind).to_string(),
                params: invocation.params.clone(),
                actual_value: Some(ratio),
                threshold_value: Some(threshold),
                matched: ratio >= threshold,
                reason: None,
            })
        }
    }
}

fn preset_name(kind: &PresetKind) -> &'static str {
    match kind {
        PresetKind::CloseAboveMa => "close_above_ma",
        PresetKind::CloseBelowMa => "close_below_ma",
        PresetKind::RsiGte => "rsi_gte",
        PresetKind::RsiLte => "rsi_lte",
        PresetKind::VolumeRatioGte => "volume_ratio_gte",
    }
}

fn get_usize_param(invocation: &PresetInvocation, key: &str) -> Result<usize> {
    let value = invocation.params[key]
        .parse::<usize>()
        .map_err(|_| QuantixError::Other(format!("参数 {} 必须是正整数", key)))?;

    if value == 0 {
        return Err(QuantixError::Other(format!("参数 {} 必须是正整数", key)));
    }

    Ok(value)
}

fn get_decimal_param(invocation: &PresetInvocation, key: &str) -> Result<Decimal> {
    invocation.params[key]
        .parse::<Decimal>()
        .map_err(|_| QuantixError::Other(format!("参数 {} 必须是数字", key)))
}
