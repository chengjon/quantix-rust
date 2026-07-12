//! Isolation Forest model: [`AnomalyScore`], [`IsolationForest`], helpers.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use super::tree::{IsolationTree, TreeNode};

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
                let avg_path: f64 = self
                    .trees
                    .iter()
                    .map(|tree| tree.path_length(sample))
                    .sum::<f64>()
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
        results.sort_by(|a, b| compare_scores_ascending(a.score, b.score));
        results.into_iter().take(top_n).collect()
    }

    /// Predict whether each sample is an anomaly
    pub fn predict(&self, features: &[Vec<f64>]) -> Vec<bool> {
        let scores = self.decision_function(features);
        let threshold = self.get_threshold(&scores);
        scores.into_iter().map(|s| s < threshold).collect()
    }

    /// Get the anomaly threshold based on contamination
    pub(crate) fn get_threshold(&self, scores: &[f64]) -> f64 {
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| compare_scores_ascending(*a, *b));
        let idx = ((self.contamination * sorted.len() as f64).ceil() as usize).min(sorted.len());
        sorted[idx.min(sorted.len().saturating_sub(1))]
    }
}

pub(crate) fn compare_scores_ascending(left: f64, right: f64) -> Ordering {
    match (left.is_nan(), right.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => left.total_cmp(&right),
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
