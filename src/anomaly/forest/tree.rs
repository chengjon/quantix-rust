//! Isolation-tree internals: [`TreeNode`], [`IsolationTree`].

use tracing::warn;

/// A single isolation tree node
#[derive(Debug, Clone)]
pub(crate) struct TreeNode {
    /// Split feature index
    pub(crate) split_feature: Option<usize>,
    /// Split value
    pub(crate) split_value: Option<f64>,
    /// Left child (values < split_value)
    pub(crate) left: Option<Box<TreeNode>>,
    /// Right child (values >= split_value)
    pub(crate) right: Option<Box<TreeNode>>,
    /// Path length if this is a leaf
    pub(crate) leaf_path_length: Option<f64>,
}

impl TreeNode {
    pub(crate) fn new_leaf(path_length: f64) -> Self {
        Self {
            split_feature: None,
            split_value: None,
            left: None,
            right: None,
            leaf_path_length: Some(path_length),
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.split_feature.is_none()
    }

    /// Build an isolation tree node from data
    pub(crate) fn build<R: rand::Rng>(
        data: &[Vec<f64>],
        height_limit: usize,
        current_height: usize,
        rng: &mut R,
    ) -> Self {
        use rand::seq::SliceRandom;

        // Stop conditions: height limit reached or single sample
        if current_height >= height_limit || data.len() <= 1 {
            return Self::new_leaf(
                current_height as f64 + super::model::avg_path_length(data.len() as f64),
            );
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
            return Self::new_leaf(
                current_height as f64 + super::model::avg_path_length(data.len() as f64),
            );
        }

        // Randomly select a feature
        let Some(split_feature) = valid_features.choose(rng).copied() else {
            return Self::new_leaf(
                current_height as f64 + super::model::avg_path_length(data.len() as f64),
            );
        };

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
            return Self::new_leaf(
                current_height as f64 + super::model::avg_path_length(data.len() as f64),
            );
        }

        Self {
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
    pub(crate) fn path_length(&self, sample: &[f64], current_height: usize) -> f64 {
        if self.is_leaf() {
            return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
        }

        // is_leaf() == false 保证 split_feature / split_value / 子节点都存在；
        // 若不变量被破坏（数据竞争或反序列化畸形），降级为按 leaf 路径长度返回，
        // 而不是 panic —— 调用方拿到的是一个保守估计值，仍可在生产环境继续运行。
        let Some(feature) = self.split_feature else {
            warn!("anomaly forest internal invariant broken: missing split_feature");
            return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
        };
        let Some(value) = self.split_value else {
            warn!("anomaly forest internal invariant broken: missing split_value");
            return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
        };
        let next = if sample[feature] < value {
            match self.left.as_ref() {
                Some(node) => node,
                None => {
                    warn!("anomaly forest internal invariant broken: missing left child");
                    return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
                }
            }
        } else {
            match self.right.as_ref() {
                Some(node) => node,
                None => {
                    warn!("anomaly forest internal invariant broken: missing right child");
                    return current_height as f64 + self.leaf_path_length.unwrap_or(0.0);
                }
            }
        };

        next.path_length(sample, current_height + 1)
    }
}

/// A single isolation tree
#[derive(Debug, Clone)]
pub(crate) struct IsolationTree {
    pub(crate) root: TreeNode,
}

impl IsolationTree {
    /// Path length for a single sample
    pub(crate) fn path_length(&self, sample: &[f64]) -> f64 {
        self.root.path_length(sample, 0)
    }
}
