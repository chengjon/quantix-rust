//! Isolation Forest implementation for anomaly detection
//!
//! The Isolation Forest algorithm isolates anomalies by randomly selecting
//! a feature and a split value. Anomalies are isolated faster (shorter path
//! length) than normal points.
//!
//! # Algorithm
//!
//! 1. Build `n_estimators` isolation trees
//! 2. Each tree randomly selects features and split values
//! 3. Anomalies have shorter average path lengths
//! 4. Score = 2^(-E[h(x)] / c(n)) - 0.5
//!
//! # References
//!
//! - Liu, Fei Tony, Kai Ming Ting, and Zhi-Hua Zhou. "Isolation forest."
//!   2008 Eighth IEEE International Conference on Data Mining.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Anomaly score result for a single sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScore {
    /// Stock code
    pub code: String,
    /// Stock name
    pub name: String,
    /// Anomaly score (negative = anomalous, positive = normal)
    /// Range: approximately [-0.5, 0.5]
    pub score: f64,
    /// Whether this is considered an anomaly
    pub is_anomaly: bool,
    /// Index in the original feature matrix
    #[serde(skip)]
    pub index: usize,
    /// Volume ratio (today / 5-day avg)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_ratio: Option<f64>,
    /// 5-period volatility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_5: Option<f64>,
    /// 20-period volatility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_20: Option<f64>,
    /// Latest timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_time: Option<String>,
}

/// A single isolation tree node
#[derive(Debug, Clone)]
struct TreeNode {
    /// Split feature index
    split_feature: Option<usize>,
    /// Split value
    split_value: Option<f64>,
    /// Left child (values < split_value)
    left: Option<Box<TreeNode>>,
    /// Right child (values >= split_value)
    right: Option<Box<TreeNode>>,
    /// Path length if this is a leaf
    leaf_path_length: Option<f64>,
}

impl TreeNode {
    fn new_leaf(path_length: f64) -> Self {
        TreeNode {
            split_feature: None,
            split_value: None,
            left: None,
            right: None,
            leaf_path_length: Some(path_length),
        }
    }

    fn is_leaf(&self) -> bool {
        self.split_feature.is_none()
    }

    /// Build an isolation tree node from data
    fn build<R: rand::Rng>(
        data: &[Vec<f64>],
        height_limit: usize,
        current_height: usize,
        rng: &mut R,
    ) -> TreeNode {
        use rand::seq::SliceRandom;

        // Stop conditions: height limit reached or single sample
        if current_height >= height_limit || data.len() <= 1 {
            return TreeNode::new_leaf(current_height as f64 + avg_path_length(data.len() as f64));
        }

        let n_features = data[0].len();

        // Find features with variance
        let valid_features: Vec<usize> = (0..n_features)
            .filter(|&f| {
                let min = data.iter().map(|s| s[f]).fold(f64::INFINITY, f64::min);
                let max = data.iter().map(|s| s[f]).fold(f64::NEG_INFINITY, f64::max);
                (max - min).abs() > f64::EPSILON
            })
            .collect();

        if valid_features.is_empty() {
            return TreeNode::new_leaf(current_height as f64 + avg_path_length(data.len() as f64));
        }

        // Randomly select a feature
        let split_feature = valid_features.choose(rng).copied().unwrap();

        // Find min and max for the selected feature
        let min_val = data
            .iter()
            .map(|s| s[split_feature])
            .fold(f64::INFINITY, f64::min);
        let max_val = data
            .iter()
            .map(|s| s[split_feature])
            .fold(f64::NEG_INFINITY, f64::max);

        // Random split value between min and max
        let split_value = rng.gen_range(min_val..=max_val);

        // Split data
        let left_data: Vec<Vec<f64>> = data
            .iter()
            .filter(|s| s[split_feature] < split_value)
            .cloned()
            .collect();
        let right_data: Vec<Vec<f64>> = data
            .iter()
            .filter(|s| s[split_feature] >= split_value)
            .cloned()
            .collect();

        // Handle edge cases where split doesn't divide data
        if left_data.is_empty() || right_data.is_empty() {
            return TreeNode::new_leaf(current_height as f64 + avg_path_length(data.len() as f64));
        }

        TreeNode {
            split_feature: Some(split_feature),
            split_value: Some(split_value),
            left: Some(Box::new(Self::build(
                &left_data,
                height_limit,
                current_height + 1,
                rng,
            ))),
            right: Some(Box::new(Self::build(
                &right_data,
                height_limit,
                current_height + 1,
                rng,
            ))),
            leaf_path_length: None,
        }
    }

    /// Path length for a single sample
    fn path_length(&self, sample: &[f64], current_height: usize) -> f64 {
        if self.is_leaf() {
            return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
        }

        let feature = self.split_feature.unwrap();
        let value = self.split_value.unwrap();

        if sample[feature] < value {
            self.left
                .as_ref()
                .unwrap()
                .path_length(sample, current_height + 1)
        } else {
            self.right
                .as_ref()
                .unwrap()
                .path_length(sample, current_height + 1)
        }
    }
}

/// A single isolation tree
#[derive(Debug, Clone)]
struct IsolationTree {
    root: TreeNode,
}

impl IsolationTree {
    /// Path length for a single sample
    fn path_length(&self, sample: &[f64]) -> f64 {
        self.root.path_length(sample, 0)
    }
}

/// Isolation Forest model
#[derive(Debug)]
pub struct IsolationForest {
    /// Number of trees
    n_estimators: usize,
    /// Maximum samples per tree
    max_samples: usize,
    /// Random seed for reproducibility
    random_state: u64,
    /// Trained trees
    trees: Vec<IsolationTree>,
    /// Number of training samples (for normalization)
    n_samples: usize,
    /// Contamination rate (expected proportion of anomalies)
    contamination: f64,
}

impl Default for IsolationForest {
    fn default() -> Self {
        Self::new()
    }
}

impl IsolationForest {
    /// Create a new Isolation Forest with default parameters
    pub fn new() -> Self {
        Self {
            n_estimators: 100,
            max_samples: 256,
            random_state: 42,
            trees: Vec::new(),
            n_samples: 0,
            contamination: 0.1,
        }
    }

    /// Set the number of trees
    pub fn n_estimators(mut self, n: usize) -> Self {
        self.n_estimators = n;
        self
    }

    /// Set the maximum samples per tree
    pub fn max_samples(mut self, n: usize) -> Self {
        self.max_samples = n;
        self
    }

    /// Set the random seed
    pub fn random_state(mut self, seed: u64) -> Self {
        self.random_state = seed;
        self
    }

    /// Set the contamination rate
    pub fn contamination(mut self, rate: f64) -> Self {
        self.contamination = rate;
        self
    }

    /// Train the isolation forest on the given features
    pub fn fit(&mut self, features: &[Vec<f64>]) -> Result<(), String> {
        if features.is_empty() {
            return Err("No features provided".to_string());
        }

        self.n_samples = features.len();
        let actual_samples = self.max_samples.min(features.len());
        let height_limit = (actual_samples as f64).log2().ceil() as usize;

        // Use parallel tree building
        self.trees = (0..self.n_estimators)
            .into_par_iter()
            .map(|i| {
                use rand::prelude::*;
                use rand_chacha::ChaCha8Rng;

                let mut rng = ChaCha8Rng::seed_from_u64(self.random_state + i as u64);

                let subset = if features.len() <= actual_samples {
                    features.to_vec()
                } else {
                    let mut indices: Vec<usize> = (0..features.len()).collect();
                    indices.shuffle(&mut rng);
                    indices[..actual_samples]
                        .iter()
                        .map(|&idx| features[idx].clone())
                        .collect()
                };

                IsolationTree {
                    root: TreeNode::build(&subset, height_limit, 0, &mut rng),
                }
            })
            .collect();

        Ok(())
    }

    /// Compute anomaly scores for all samples
    ///
    /// Returns scores in range [-0.5, 0.5] approximately
    /// - Negative scores indicate anomalies
    /// - Positive scores indicate normal points
    pub fn decision_function(&self, features: &[Vec<f64>]) -> Vec<f64> {
        if self.trees.is_empty() || features.is_empty() {
            return vec![0.0; features.len()];
        }

        let c_n = avg_path_length(self.n_samples as f64);

        features
            .par_iter()
            .map(|sample| {
                let avg_path: f64 =
                    self.trees.iter().map(|tree| tree.path_length(sample)).sum::<f64>()
                        / self.n_estimators as f64;

                if c_n > 0.0 {
                    let score = (-avg_path / c_n).exp2();
                    score - 0.5
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Find the most anomalous samples
    ///
    /// # Arguments
    /// * `features` - Feature matrix
    /// * `codes` - Stock codes corresponding to each sample
    /// * `names` - Stock names corresponding to each sample
    /// * `top_n` - Number of top anomalies to return
    pub fn find_anomalies(
        &self,
        features: &[Vec<f64>],
        codes: &[String],
        names: &[String],
        top_n: usize,
    ) -> Vec<AnomalyScore> {
        let scores = self.decision_function(features);

        let mut results: Vec<AnomalyScore> = scores
            .iter()
            .enumerate()
            .map(|(i, &score)| AnomalyScore {
                code: codes.get(i).cloned().unwrap_or_default(),
                name: names.get(i).cloned().unwrap_or_default(),
                score,
                is_anomaly: score < 0.0,
                index: i,
                volume_ratio: None,
                volatility_5: None,
                volatility_20: None,
                latest_time: None,
            })
            .collect();

        // Sort by score (ascending - most anomalous first)
        results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        results.into_iter().take(top_n).collect()
    }

    /// Predict whether each sample is an anomaly
    pub fn predict(&self, features: &[Vec<f64>]) -> Vec<bool> {
        let scores = self.decision_function(features);
        let threshold = self.get_threshold(&scores);
        scores.into_iter().map(|s| s < threshold).collect()
    }

    /// Get the anomaly threshold based on contamination
    fn get_threshold(&self, scores: &[f64]) -> f64 {
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((self.contamination * sorted.len() as f64).ceil() as usize).min(sorted.len());
        sorted[idx.min(sorted.len().saturating_sub(1))]
    }
}

/// Calculate the average path length in a binary search tree with n nodes
///
/// This is used for normalizing path lengths in Isolation Forest
pub fn avg_path_length(n: f64) -> f64 {
    if n <= 1.0 {
        return 0.0;
    }
    if n == 2.0 {
        return 1.0;
    }

    // Euler-Mascheroni constant
    const EULER_MASCHERONI: f64 = 0.5772156649;

    2.0 * (n.ln() + EULER_MASCHERONI) - 2.0 * (n - 1.0) / n
}

#[cfg(test)]
mod tests {
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
            assert!(score.abs() < 1.0, "Score {} is out of expected range", score);
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
}
