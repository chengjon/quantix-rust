use std::collections::HashMap;

use quantix_cli::analysis::{IndicatorPipelineConfig, IndicatorSpec};
use quantix_cli::strategy::ConfiguredStrategyInstance;
use serde_json::json;

#[test]
fn config_maps_ma_cross_to_two_sma_instances() {
    let cfg = ConfiguredStrategyInstance {
        id: "ma_fast_5_slow_20".into(),
        name: "ma_cross".into(),
        enabled: true,
        params: json!({"fast": 5, "slow": 20}),
    };

    let pipeline = IndicatorPipelineConfig::try_from(&cfg).unwrap();

    assert_eq!(pipeline.indicators.len(), 2);
    assert_eq!(pipeline.indicators[0].name(), "sma");
    assert_eq!(pipeline.indicators[0].params().get("period"), Some(&json!(5)));
    assert_eq!(pipeline.indicators[0].instance_id().0, "sma:{\"period\":5}");
    assert_eq!(pipeline.indicators[1].name(), "sma");
    assert_eq!(pipeline.indicators[1].params().get("period"), Some(&json!(20)));
    assert_eq!(pipeline.indicators[1].instance_id().0, "sma:{\"period\":20}");
}

#[test]
fn config_rejects_non_ma_cross_first_slice() {
    let cfg = ConfiguredStrategyInstance {
        id: "unknown".into(),
        name: "momentum".into(),
        enabled: true,
        params: json!({}),
    };

    assert!(IndicatorPipelineConfig::try_from(&cfg).is_err());
}

#[test]
fn config_generates_stable_instance_id_with_sorted_param_keys() {
    let mut params = HashMap::new();
    params.insert("zeta".to_string(), json!(3));
    params.insert("period".to_string(), json!(5));
    params.insert("alpha".to_string(), json!(1));

    let spec = IndicatorSpec::new("sma", params);

    assert_eq!(spec.instance_id().0, "sma:{\"alpha\":1,\"period\":5,\"zeta\":3}");
}

#[test]
fn config_instance_id_is_unambiguous_for_string_and_numeric_values() {
    let mut numeric_params = HashMap::new();
    numeric_params.insert("period".to_string(), json!(5));
    let numeric = IndicatorSpec::new("sma", numeric_params);

    let mut string_params = HashMap::new();
    string_params.insert("period".to_string(), json!("5"));
    let stringy = IndicatorSpec::new("sma", string_params);

    assert_ne!(numeric.instance_id(), stringy.instance_id());
}

#[test]
fn config_instance_id_handles_delimiters_inside_string_values() {
    let mut params = HashMap::new();
    params.insert("label".to_string(), json!("a,b:c=d"));

    let spec = IndicatorSpec::new("sma", params);

    assert_eq!(spec.instance_id().0, "sma:{\"label\":\"a,b:c=d\"}");
}
