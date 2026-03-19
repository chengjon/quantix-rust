/// 数据验证模块
///
/// 提供数据完整性校验功能
use crate::core::Result;
use crate::data::models::Kline;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

/// 验证配置
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// 启用价格验证
    pub enable_price_validation: bool,
    /// 启用成交量验证
    pub enable_volume_validation: bool,
    /// 启用日期验证
    pub enable_date_validation: bool,
    /// 价格范围检查（可选）
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    /// 最小成交量
    pub min_volume: Option<i64>,
    /// 日期范围检查（可选）
    pub min_date: Option<NaiveDate>,
    pub max_date: Option<NaiveDate>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_price_validation: true,
            enable_volume_validation: true,
            enable_date_validation: true,
            min_price: Some(Decimal::ZERO),
            max_price: Some(Decimal::from(1_000_000)),
            min_volume: Some(0),
            min_date: None,
            max_date: None,
        }
    }
}

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 错误类型
    pub error_type: String,
    /// 字段名
    pub field: String,
    /// 错误消息
    pub message: String,
    /// 行号
    pub row_number: Option<usize>,
}

impl ValidationError {
    /// 创建新的验证错误
    pub fn new(error_type: String, field: String, message: String) -> Self {
        Self {
            error_type,
            field,
            message,
            row_number: None,
        }
    }

    /// 设置行号
    pub fn with_row_number(mut self, row: usize) -> Self {
        self.row_number = Some(row);
        self
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否验证通过
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
    /// 警告列表
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// 创建通过的验证结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// 合并多个验证结果
    pub fn merge(mut self, other: ValidationResult) -> Self {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }
}

/// 数据验证器
pub struct DataValidator {
    config: ValidationConfig,
}

impl DataValidator {
    /// 创建新的验证器
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// 验证 K线数据
    pub fn validate_kline(&self, kline: &Kline, row_number: usize) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证价格
        if self.config.enable_price_validation {
            if let Some(e) = self.validate_price(kline, "open") {
                errors.push(e.with_row_number(row_number));
            }
            if let Some(e) = self.validate_price(kline, "high") {
                errors.push(e.with_row_number(row_number));
            }
            if let Some(e) = self.validate_price(kline, "low") {
                errors.push(e.with_row_number(row_number));
            }
            if let Some(e) = self.validate_price(kline, "close") {
                errors.push(e.with_row_number(row_number));
            }

            // 价格逻辑验证
            if let Some(e) = self.validate_price_logic(kline) {
                errors.push(e.with_row_number(row_number));
            }
        }

        // 验证成交量
        if self.config.enable_volume_validation {
            if let Some(e) = self.validate_volume(kline) {
                errors.push(e.with_row_number(row_number));
            }
        }

        // 验证日期
        if self.config.enable_date_validation {
            if let Some(e) = self.validate_date(kline) {
                errors.push(e.with_row_number(row_number));
            }
        }

        if errors.is_empty() {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid(errors)
        }
    }

    /// 验证价格值
    fn validate_price(&self, kline: &Kline, field: &str) -> Option<ValidationError> {
        let price = match field {
            "open" => kline.open,
            "high" => kline.high,
            "low" => kline.low,
            "close" => kline.close,
            _ => return None,
        };

        // 检查价格范围
        if let Some(min) = self.config.min_price {
            if price < min {
                return Some(ValidationError::new(
                    "RANGE_ERROR".to_string(),
                    field.to_string(),
                    format!("价格 {} 小于最小值 {}", price, min),
                ));
            }
        }

        if let Some(max) = self.config.max_price {
            if price > max {
                return Some(ValidationError::new(
                    "RANGE_ERROR".to_string(),
                    field.to_string(),
                    format!("价格 {} 大于最大值 {}", price, max),
                ));
            }
        }

        // 检查价格为正数
        if price < Decimal::ZERO {
            return Some(ValidationError::new(
                "INVALID_VALUE".to_string(),
                field.to_string(),
                format!("价格必须为正数，实际值: {}", price),
            ));
        }

        None
    }

    /// 验证价格逻辑
    fn validate_price_logic(&self, kline: &Kline) -> Option<ValidationError> {
        // high >= low
        if kline.high < kline.low {
            return Some(ValidationError::new(
                "LOGIC_ERROR".to_string(),
                "price_relation".to_string(),
                format!("最高价 {} 不能低于最低价 {}", kline.high, kline.low),
            ));
        }

        // close 应该在 high 和 low 之间
        if kline.close > kline.high {
            return Some(ValidationError::new(
                "LOGIC_ERROR".to_string(),
                "close_vs_high".to_string(),
                format!("收盘价 {} 不能高于最高价 {}", kline.close, kline.high),
            ));
        }

        if kline.close < kline.low {
            return Some(ValidationError::new(
                "LOGIC_ERROR".to_string(),
                "close_vs_low".to_string(),
                format!("收盘价 {} 不能低于最低价 {}", kline.close, kline.low),
            ));
        }

        None
    }

    /// 验证成交量
    fn validate_volume(&self, kline: &Kline) -> Option<ValidationError> {
        if let Some(min) = self.config.min_volume {
            if kline.volume < min {
                return Some(ValidationError::new(
                    "RANGE_ERROR".to_string(),
                    "volume".to_string(),
                    format!("成交量 {} 小于最小值 {}", kline.volume, min),
                ));
            }
        }

        if kline.volume < 0 {
            return Some(ValidationError::new(
                "INVALID_VALUE".to_string(),
                "volume".to_string(),
                format!("成交量不能为负数: {}", kline.volume),
            ));
        }

        None
    }

    /// 验证日期
    fn validate_date(&self, kline: &Kline) -> Option<ValidationError> {
        if let Some(min) = self.config.min_date {
            if kline.date < min {
                return Some(ValidationError::new(
                    "RANGE_ERROR".to_string(),
                    "date".to_string(),
                    format!("日期 {} 早于最小日期 {}", kline.date, min),
                ));
            }
        }

        if let Some(max) = self.config.max_date {
            if kline.date > max {
                return Some(ValidationError::new(
                    "RANGE_ERROR".to_string(),
                    "date".to_string(),
                    format!("日期 {} 晚于最大日期 {}", kline.date, max),
                ));
            }
        }

        None
    }

    /// 批量验证 K线数据
    pub fn validate_klines(&self, klines: &[Kline]) -> ValidationResult {
        let mut overall_result = ValidationResult::valid();

        for (i, kline) in klines.iter().enumerate() {
            let result = self.validate_kline(kline, i + 1);
            overall_result = overall_result.merge(result);
        }

        overall_result
    }

    /// 获取数据质量报告
    pub fn quality_report(&self, klines: &[Kline]) -> DataQualityReport {
        let mut report = DataQualityReport::default();

        for kline in klines {
            report.total_records += 1;

            // 检查缺失值
            if kline.amount.is_none() {
                report.missing_amount += 1;
            }

            // 检查异常价格
            if kline.high < kline.low || kline.close < kline.low || kline.close > kline.high {
                report.invalid_price_relation += 1;
            }

            // 检查零成交量
            if kline.volume == 0 {
                report.zero_volume += 1;
            }
        }

        report
    }
}

/// 数据质量报告
#[derive(Debug, Clone, Default)]
pub struct DataQualityReport {
    /// 总记录数
    pub total_records: usize,
    /// 缺失 amount 字段的数量
    pub missing_amount: usize,
    /// 无效价格关系的数量
    pub invalid_price_relation: usize,
    /// 零成交量的数量
    pub zero_volume: usize,
    /// 数据质量评分（0-100）
    pub quality_score: u8,
}

impl DataQualityReport {
    /// 计算质量评分
    pub fn calculate_score(&mut self) {
        if self.total_records == 0 {
            self.quality_score = 0;
            return;
        }

        // 扣分项
        let missing_amount_penalty =
            (self.missing_amount as f64 / self.total_records as f64) * 20.0;
        let invalid_price_penalty =
            (self.invalid_price_relation as f64 / self.total_records as f64) * 40.0;
        let zero_volume_penalty = (self.zero_volume as f64 / self.total_records as f64) * 10.0;

        let total_penalty = missing_amount_penalty + invalid_price_penalty + zero_volume_penalty;
        let score = (100.0 - total_penalty).max(0.0) as u8;

        self.quality_score = score;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::AdjustType;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn create_test_kline() -> Kline {
        Kline {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            open: dec!(10.0),
            high: dec!(11.0),
            low: dec!(9.5),
            close: dec!(10.5),
            volume: 1000000,
            amount: Some(dec!(10500000)),
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = DataValidator::with_defaults();
        assert!(validator.config.enable_price_validation);
    }

    #[test]
    fn test_valid_kline() {
        let validator = DataValidator::with_defaults();
        let kline = create_test_kline();
        let result = validator.validate_kline(&kline, 1);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_price_relation() {
        let validator = DataValidator::with_defaults();
        let mut kline = create_test_kline();
        kline.high = dec!(9.0); // 高价低于低价

        let result = validator.validate_kline(&kline, 1);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1); // only high < low error (close is 10.5, which is not > 9.0)
    }

    #[test]
    fn test_negative_price() {
        let config = ValidationConfig {
            enable_price_validation: true,
            ..Default::default()
        };
        let validator = DataValidator::new(config);
        let mut kline = create_test_kline();
        kline.open = dec!(-1.0);

        let result = validator.validate_kline(&kline, 1);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_zero_volume() {
        let config = ValidationConfig {
            enable_volume_validation: true,
            min_volume: Some(1),
            ..Default::default()
        };
        let validator = DataValidator::new(config);
        let mut kline = create_test_kline();
        kline.volume = 0;

        let result = validator.validate_kline(&kline, 1);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_batch_validation() {
        let validator = DataValidator::with_defaults();
        let klines = vec![create_test_kline(), create_test_kline()];

        let result = validator.validate_klines(&klines);
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_quality_report() {
        let validator = DataValidator::with_defaults();
        let mut klines = vec![create_test_kline()];
        klines[0].amount = None; // 缺失 amount

        let mut report = validator.quality_report(&klines);
        report.calculate_score();

        assert_eq!(report.total_records, 1);
        assert_eq!(report.missing_amount, 1);
        assert_eq!(report.quality_score, 80); // 扣20分
    }
}
