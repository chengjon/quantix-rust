use quantix_cli::screener::{PresetKind, parse_preset_invocation};

#[test]
fn parses_close_above_ma_preset_with_period() {
    let invocation = parse_preset_invocation("close_above_ma:period=20").unwrap();

    assert_eq!(invocation.kind, PresetKind::CloseAboveMa);
    assert_eq!(
        invocation.params.get("period").map(String::as_str),
        Some("20")
    );
}

#[test]
fn parses_rsi_gte_preset_with_period_and_value() {
    let invocation = parse_preset_invocation("rsi_gte:period=14,value=55").unwrap();

    assert_eq!(invocation.kind, PresetKind::RsiGte);
    assert_eq!(
        invocation.params.get("period").map(String::as_str),
        Some("14")
    );
    assert_eq!(
        invocation.params.get("value").map(String::as_str),
        Some("55")
    );
}

#[test]
fn rejects_unknown_preset_name() {
    let err = parse_preset_invocation("unknown_rule:period=20").unwrap_err();

    assert!(err.to_string().contains("unknown_rule"));
}

#[test]
fn rejects_invalid_param_key_for_preset() {
    let err = parse_preset_invocation("close_above_ma:value=20").unwrap_err();

    assert!(err.to_string().contains("value"));
}

#[test]
fn rejects_invalid_numeric_param_value() {
    let err = parse_preset_invocation("rsi_gte:period=abc,value=55").unwrap_err();

    assert!(err.to_string().contains("period"));
}

#[test]
fn rejects_zero_usize_params() {
    let period_err = parse_preset_invocation("close_above_ma:period=0").unwrap_err();
    let window_err = parse_preset_invocation("volume_ratio_gte:window=0,value=1.5").unwrap_err();

    assert!(period_err.to_string().contains("period"));
    assert!(window_err.to_string().contains("window"));
}

#[test]
fn parses_volume_ratio_gte_with_decimal_value() {
    let invocation = parse_preset_invocation("volume_ratio_gte:window=5,value=1.5").unwrap();

    assert_eq!(invocation.kind, PresetKind::VolumeRatioGte);
    assert_eq!(
        invocation.params.get("window").map(String::as_str),
        Some("5")
    );
    assert_eq!(
        invocation.params.get("value").map(String::as_str),
        Some("1.5")
    );
}
