use super::*;

fn create_test_series() -> OHLCVSeries {
    let mut series = OHLCVSeries::new("000001", "测试股票");
    for i in 0..60 {
        let base = 100.0 + i as f64;
        series.add(OHLCVCandle::new(
            &format!("2026-03-{:02} 10:00", i % 28 + 1),
            base,
            base + 1.0,
            base + 2.0,
            base - 1.0,
            1000000.0 + i as f64 * 10000.0,
        ));
    }
    series
}

#[test]
fn test_feature_extraction() {
    let extractor = FeatureExtractor::with_defaults();
    let series = create_test_series();

    let feature_set = extractor.extract(&series);

    assert!(feature_set.is_some());
    let fs = feature_set.unwrap();
    assert!(!fs.features.is_empty());
    assert_eq!(fs.code, "000001");
}

#[test]
fn test_feature_extraction_insufficient_data() {
    let extractor = FeatureExtractor::with_defaults();
    let mut series = OHLCVSeries::new("000002", "测试股票2");

    // Only add 10 candles (less than min_candles = 50)
    for _ in 0..10 {
        series.add(OHLCVCandle::new(
            "2026-03-01 10:00",
            100.0,
            101.0,
            102.0,
            99.0,
            1000000.0,
        ));
    }

    assert!(extractor.extract(&series).is_none());
}

#[test]
fn test_feature_extraction_no_nan() {
    let extractor = FeatureExtractor::with_defaults();
    let series = create_test_series();

    let feature_set = extractor.extract(&series).unwrap();

    // No features should be NaN
    for &val in &feature_set.features {
        assert!(!val.is_nan(), "Feature value is NaN");
    }
}

#[test]
fn test_log_returns() {
    let series = create_test_series();
    let returns = series.log_returns();

    assert!(!returns.is_empty());
    assert_eq!(returns.len(), series.len() - 1);
}

#[test]
fn test_volume_returns() {
    let series = create_test_series();
    let returns = series.volume_returns();

    assert!(!returns.is_empty());
    assert_eq!(returns.len(), series.len() - 1);
}

#[test]
fn test_volatility() {
    let series = create_test_series();
    let vol = series.volatility(20);

    assert!(vol >= 0.0);
}
