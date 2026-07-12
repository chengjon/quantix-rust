use super::*;

#[test]
fn test_avg_path_length() {
    assert!((avg_path_length(1.0) - 0.0).abs() < 1e-10);
    assert!((avg_path_length(2.0) - 1.0).abs() < 1e-10);
    assert!(avg_path_length(100.0) > 0.0);
}

#[test]
fn test_isolation_forest_basic() {
    let mut forest = IsolationForest::new()
        .n_estimators(50)
        .max_samples(50)
        .random_state(42);

    // Create diverse data
    let mut features = Vec::new();
    for i in 0..100 {
        features.push(vec![
            (i as f64 * 0.1).sin(),
            (i as f64 * 0.2).cos(),
            (i as f64 * 0.05).tan(),
        ]);
    }

    forest.fit(&features).unwrap();
    let scores = forest.decision_function(&features);

    // Verify we get scores for all samples
    assert_eq!(scores.len(), features.len());

    // Verify scores are in expected range [-0.5, 0.5] approximately
    for score in &scores {
        assert!(
            score.abs() < 1.0,
            "Score {} is out of expected range",
            score
        );
    }
}

#[test]
fn test_find_anomalies() {
    let mut forest = IsolationForest::new()
        .n_estimators(50)
        .max_samples(50)
        .random_state(42);

    // Create normal data
    let mut features = Vec::new();
    for i in 0..50 {
        features.push(vec![i as f64 * 0.1, (i as f64 * 0.1).sin()]);
    }
    // Add some anomalies
    features.push(vec![100.0, 100.0]);
    features.push(vec![-50.0, -50.0]);

    forest.fit(&features).unwrap();

    let codes: Vec<String> = (0..52).map(|i| format!("{:06}", i)).collect();
    let names: Vec<String> = codes.clone();

    let anomalies = forest.find_anomalies(&features, &codes, &names, 5);

    assert_eq!(anomalies.len(), 5);
    // All returned should have negative scores (anomalous)
    for a in &anomalies {
        assert!(a.score < 0.0);
    }
}

#[test]
fn threshold_orders_nan_scores_after_finite_scores() {
    let forest = IsolationForest::new().contamination(0.1);

    let threshold = forest.get_threshold(&[-0.25, f64::NAN, 0.25]);

    assert_eq!(threshold, 0.25);
}
