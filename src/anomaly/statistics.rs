//! Statistical functions for anomaly detection
//!
//! Provides utility functions for calculating statistical measures
//! used in feature extraction and anomaly scoring.

use serde::{Deserialize, Serialize};

/// Result of linear regression calculation
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RegressionStats {
    /// Slope of the regression line
    pub slope: f64,
    /// Intercept of the regression line
    pub intercept: f64,
    /// R-squared value (coefficient of determination)
    pub r_squared: f64,
    /// P-value for slope significance
    pub p_value: f64,
    /// Standard error of the slope
    pub std_err: f64,
}

/// Calculate linear regression statistics
///
/// Performs linear regression with x-axis as [0, 1, 2, ..., n-1]
/// and returns slope, r-squared, p-value, and standard error.
///
/// # Arguments
/// * `data` - Y-values for regression
///
/// # Returns
/// * `Option<RegressionStats>` - Regression statistics, or None if insufficient data
pub fn linear_regression(data: &[f64]) -> Option<RegressionStats> {
    let n = data.len();

    if n < 2 {
        return None;
    }

    // Filter out NaN values
    let clean_data: Vec<f64> = data.iter().filter(|v| !v.is_nan()).cloned().collect();

    if clean_data.len() < 2 {
        return None;
    }

    let n = clean_data.len();

    // Calculate means
    let x_mean = (n - 1) as f64 / 2.0;
    let y_mean = clean_data.iter().sum::<f64>() / n as f64;

    // Calculate slope using least squares
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y_val) in clean_data.iter().enumerate() {
        let x_val = i as f64;
        numerator += (x_val - x_mean) * (y_val - y_mean);
        denominator += (x_val - x_mean).powi(2);
    }

    if denominator.abs() < f64::EPSILON {
        return Some(RegressionStats {
            slope: 0.0,
            intercept: y_mean,
            r_squared: 0.0,
            p_value: 1.0,
            std_err: 0.0,
        });
    }

    let slope = numerator / denominator;
    let intercept = y_mean - slope * x_mean;

    // Calculate R-squared
    let mut ss_tot = 0.0;
    let mut ss_res = 0.0;

    for (i, &y_val) in clean_data.iter().enumerate() {
        let y_pred = slope * i as f64 + intercept;
        ss_tot += (y_val - y_mean).powi(2);
        ss_res += (y_val - y_pred).powi(2);
    }

    let r_squared = if ss_tot > f64::EPSILON {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    // Calculate p-value and standard error
    let (p_value, std_err) = calculate_p_value_and_se(slope, &clean_data, intercept, n);

    Some(RegressionStats {
        slope,
        intercept,
        r_squared: r_squared.abs(),
        p_value,
        std_err,
    })
}

/// Calculate p-value and standard error for slope coefficient
fn calculate_p_value_and_se(slope: f64, y: &[f64], intercept: f64, n: usize) -> (f64, f64) {
    if n < 3 {
        return (1.0, 0.0);
    }

    let x_mean = (n - 1) as f64 / 2.0;
    let mut x_var = 0.0;

    for i in 0..n {
        x_var += (i as f64 - x_mean).powi(2);
    }

    if x_var < f64::EPSILON {
        return (if slope.abs() < f64::EPSILON { 1.0 } else { 0.0 }, 0.0);
    }

    // Calculate residual sum of squares
    let mut ss_res = 0.0;
    for (i, &y_val) in y.iter().enumerate() {
        let y_pred = slope * i as f64 + intercept;
        ss_res += (y_val - y_pred).powi(2);
    }

    let mse = ss_res / (n - 2) as f64;
    let se_slope = (mse / x_var).sqrt();

    if se_slope < f64::EPSILON {
        return (if slope.abs() < f64::EPSILON { 1.0 } else { 0.0 }, se_slope);
    }

    // t-statistic
    let t_stat = slope / se_slope;

    // Two-tailed p-value using normal approximation (good for n > 30)
    let p_value = 2.0 * (1.0 - normal_cdf(t_stat.abs()));

    (p_value, se_slope)
}

/// Standard normal cumulative distribution function
fn normal_cdf(x: f64) -> f64 {
    // Approximation using error function
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Error function approximation
fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Calculate standard deviation
pub fn std_dev(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;

    variance.sqrt()
}

/// Calculate mean
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    data.iter().sum::<f64>() / data.len() as f64
}

/// Calculate volatility (standard deviation of returns)
pub fn volatility(returns: &[f64]) -> f64 {
    std_dev(returns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression_perfect_positive() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = linear_regression(&data).unwrap();

        assert!((stats.slope - 1.0).abs() < 1e-10);
        assert!((stats.r_squared - 1.0).abs() < 1e-10);
        assert!((stats.p_value - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_perfect_negative() {
        let data = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let stats = linear_regression(&data).unwrap();

        assert!((stats.slope - (-1.0)).abs() < 1e-10);
        assert!((stats.r_squared - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_flat() {
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let stats = linear_regression(&data).unwrap();

        assert!((stats.slope - 0.0).abs() < 1e-10);
        assert!((stats.r_squared - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_insufficient_data() {
        let data = vec![1.0];
        assert!(linear_regression(&data).is_none());
    }

    #[test]
    fn test_std_dev() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&data);

        // Expected std dev is 2.0
        assert!((sd - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&data) - 3.0).abs() < 1e-10);
    }
}
