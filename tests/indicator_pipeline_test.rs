use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use quantix_cli::analysis::{
    IndicatorCache, IndicatorCacheKey, IndicatorInput, IndicatorInstanceId, IndicatorPipeline,
    IndicatorPipelineConfig, IndicatorRegistry, IndicatorSeries, IndicatorSeriesKind, IndicatorSpec,
};
use rust_decimal::Decimal;
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

#[test]
fn registry_reports_sma_metadata() {
    let registry = IndicatorRegistry::register_builtin();
    let spec = spec("sma", &[("period", 5)]);

    let descriptor = registry.descriptor(&spec).unwrap();
    assert_eq!(descriptor.meta.lookback, 5);
    assert_eq!(descriptor.meta.warmup_len, 4);
    assert_eq!(descriptor.series_kind, IndicatorSeriesKind::Scalar);
}

#[test]
fn registry_computes_scalar_series_for_sma() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let spec = spec("sma", &[("period", 3)]);

    let output = registry.compute(&spec, &input).unwrap();
    assert!(matches!(output, IndicatorSeries::ScalarSeries(_)));
}

#[test]
fn registry_sma_scalar_series_has_expected_length_and_warmup_shape() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let spec = spec("sma", &[("period", 3)]);

    let output = registry.compute(&spec, &input).unwrap();
    let IndicatorSeries::ScalarSeries(values) = output else {
        panic!("expected scalar series");
    };

    assert_eq!(values.len(), 5);
    assert_eq!(values[0], None);
    assert_eq!(values[1], None);
    assert_eq!(values[2], Some(Decimal::from(2)));
    assert_eq!(values[3], Some(Decimal::from(3)));
    assert_eq!(values[4], Some(Decimal::from(4)));
}

#[test]
fn registry_rejects_unsupported_indicator() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let spec = spec("wma", &[("period", 3)]);

    let err = registry.compute(&spec, &input).unwrap_err();
    assert!(err.to_string().contains("first slice only supports sma/ema/rsi"));
}

#[test]
fn registry_routes_ema_to_ema_calculation() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 10, 1, 10]);
    let spec = spec("ema", &[("period", 2)]);

    let output = registry.compute(&spec, &input).unwrap();
    let IndicatorSeries::ScalarSeries(values) = output else {
        panic!("expected scalar series");
    };

    assert_eq!(values.len(), 4);
    assert_eq!(values[0], None);
    assert_eq!(values[1], Some(Decimal::new(55, 1))); // 5.5
    assert_ne!(values[2], Some(Decimal::new(55, 1))); // prove not SMA-flat 5.5
    assert_eq!(values[2].unwrap().round_dp(1), Decimal::new(25, 1)); // ~2.5
    assert_eq!(values[3].unwrap().round_dp(1), Decimal::new(75, 1)); // ~7.5
}

#[test]
fn registry_normalizes_indicator_name_case() {
    let registry = IndicatorRegistry::new();
    let input = close_input(&[1, 10, 1, 10]);
    let spec = spec("EMA", &[("period", 2)]);

    let output = registry.compute(&spec, &input).unwrap();
    assert!(matches!(output, IndicatorSeries::ScalarSeries(_)));
}

#[test]
fn registry_reports_rsi_metadata_and_routes_to_rsi_calculation() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 2, 3, 4]);
    let spec = spec("rsi", &[("period", 3)]);

    let descriptor = registry.descriptor(&spec).unwrap();
    assert_eq!(descriptor.meta.lookback, 4);
    assert_eq!(descriptor.meta.warmup_len, 3);
    assert_eq!(descriptor.series_kind, IndicatorSeriesKind::Scalar);

    let output = registry.compute(&spec, &input).unwrap();
    let IndicatorSeries::ScalarSeries(values) = output else {
        panic!("expected scalar series");
    };

    assert_eq!(values.len(), 6);
    assert_eq!(values[0], None);
    assert_eq!(values[1], None);
    assert_eq!(values[2], None);
    assert!(values[3].is_some());
}

#[test]
fn registry_rejects_missing_period() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let spec = IndicatorSpec::new("sma", HashMap::new());

    let err = registry.compute(&spec, &input).unwrap_err();
    assert!(err.to_string().contains("`period` is required"));
}

#[test]
fn registry_rejects_non_integer_period() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let mut params = HashMap::new();
    params.insert("period".to_string(), json!("3"));
    let spec = IndicatorSpec::new("sma", params);

    let err = registry.compute(&spec, &input).unwrap_err();
    assert!(err.to_string().contains("positive integer"));
}

#[test]
fn registry_rejects_zero_period() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let mut params = HashMap::new();
    params.insert("period".to_string(), json!(0));
    let spec = IndicatorSpec::new("sma", params);

    let err = registry.compute(&spec, &input).unwrap_err();
    assert!(err.to_string().contains("greater than zero"));
}

#[test]
fn registry_rejects_rsi_period_one() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input(&[1, 2, 3, 4, 5]);
    let mut params = HashMap::new();
    params.insert("period".to_string(), json!(1));
    let spec = IndicatorSpec::new("rsi", params);

    let err = registry.compute(&spec, &input).unwrap_err();
    assert!(err.to_string().contains("requires `period` >= 2"));
}

#[test]
fn cache_key_keeps_sma_instances_separate() {
    let k1 = IndicatorCacheKey::new("000001:1d", IndicatorInstanceId("sma:{\"period\":5}".into()), (0, 20));
    let k2 = IndicatorCacheKey::new(
        "000001:1d",
        IndicatorInstanceId("sma:{\"period\":20}".into()),
        (0, 20),
    );

    assert_ne!(k1, k2);
}

#[test]
fn cache_get_or_compute_reuses_cached_value() {
    let mut cache = IndicatorCache::new();
    let key = IndicatorCacheKey::new(
        "000001:1d",
        IndicatorInstanceId("sma:{\"period\":5}".into()),
        (0, 5),
    );
    let calls = AtomicUsize::new(0);

    let first = cache
        .get_or_compute(key.clone(), || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(IndicatorSeries::ScalarSeries(vec![None, Some(Decimal::from(3))]))
        })
        .unwrap();
    let second = cache
        .get_or_compute(key, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(IndicatorSeries::ScalarSeries(vec![None, Some(Decimal::from(9))]))
        })
        .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let IndicatorSeries::ScalarSeries(first_values) = first else {
        panic!("expected scalar series");
    };
    let IndicatorSeries::ScalarSeries(second_values) = second else {
        panic!("expected scalar series");
    };
    assert_eq!(first_values, vec![None, Some(Decimal::from(3))]);
    assert_eq!(second_values, vec![None, Some(Decimal::from(3))]);
}

#[test]
fn indicator_input_new_derives_distinct_dataset_fingerprints() {
    let first = close_input(&[1, 2, 3]);
    let second = close_input(&[1, 2, 4]);

    assert_ne!(first.dataset_fingerprint(), second.dataset_fingerprint());
}

#[test]
fn pipeline_returns_both_sma_instances_without_overwrite() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = close_input_with_dataset(
        "000001:1d",
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
    );
    let config = pipeline_config(&[("sma", 5), ("sma", 20)]);

    let output = pipeline.run(&config, &input).unwrap();

    assert!(output.contains_key(&IndicatorInstanceId("sma:{\"period\":5}".into())));
    assert!(output.contains_key(&IndicatorInstanceId("sma:{\"period\":20}".into())));
}

#[test]
fn pipeline_preserves_warmup_none_values() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = close_input_with_dataset(
        "000001:1d",
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
    );
    let config = pipeline_config(&[("sma", 20)]);

    let output = pipeline.run(&config, &input).unwrap();
    let slow = output
        .get(&IndicatorInstanceId("sma:{\"period\":20}".into()))
        .unwrap();
    let IndicatorSeries::ScalarSeries(values) = slow else {
        panic!("expected scalar series");
    };

    assert_eq!(values.len(), 20);
    assert!(values.iter().take(19).all(Option::is_none));
    assert_eq!(values[19], Some(Decimal::new(105, 1)));
}

#[test]
fn pipeline_rejects_duplicate_instance_id() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = close_input_with_dataset("000001:1d", &[1, 2, 3, 4, 5, 6]);
    let config = IndicatorPipelineConfig {
        indicators: vec![spec("sma", &[("period", 3)]), spec("sma", &[("period", 3)])],
    };

    let err = pipeline.run(&config, &input).unwrap_err();
    assert!(err
        .to_string()
        .contains("duplicate indicator instance_id"));
}

#[test]
fn pipeline_reuses_cached_entry_across_runs() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = close_input_with_dataset("000001:1d", &[1, 2, 3, 4, 5, 6]);
    let config = pipeline_config(&[("sma", 3)]);

    pipeline.run(&config, &input).unwrap();
    let cache_len_after_first = pipeline.cache_len();
    pipeline.run(&config, &input).unwrap();
    let cache_len_after_second = pipeline.cache_len();

    assert_eq!(cache_len_after_first, 1);
    assert_eq!(cache_len_after_second, 1);
}

#[test]
fn pipeline_rejects_range_length_mismatch() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = IndicatorInput::with_context(
        "000001:1d",
        (0, 5),
        vec![Decimal::from(1), Decimal::from(2), Decimal::from(3)],
    );
    let config = pipeline_config(&[("sma", 3)]);

    let err = pipeline.run(&config, &input).unwrap_err();
    assert!(err.to_string().contains("does not match close length"));
}

fn spec(name: &str, params: &[(&str, i64)]) -> IndicatorSpec {
    let map = params
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(*v)))
        .collect::<HashMap<_, _>>();
    IndicatorSpec::new(name, map)
}

fn close_input(close: &[i64]) -> IndicatorInput {
    IndicatorInput::new(close.iter().copied().map(Decimal::from).collect())
}

fn close_input_with_dataset(dataset_fingerprint: &str, close: &[i64]) -> IndicatorInput {
    IndicatorInput::with_dataset_fingerprint(
        dataset_fingerprint,
        close.iter().copied().map(Decimal::from).collect(),
    )
}

fn pipeline_config(specs: &[(&str, i64)]) -> IndicatorPipelineConfig {
    IndicatorPipelineConfig {
        indicators: specs
            .iter()
            .map(|(name, period)| spec(name, &[("period", *period)]))
            .collect(),
    }
}
