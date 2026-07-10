use chrono::NaiveDate;
use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};

/// 因子分类：Technical 技术因子、Fundamental 基本面因子、Composite 复合因子、Experimental 实验性因子（仅供研究）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorCategory {
    Technical,
    Fundamental,
    Composite,
    Experimental,
}

/// 缺失值处理策略：KeepNull 保留 null、ForwardFill 前向填充、DropRow 删除该行、DropLeadingWindow 删除 warmup 期前的不稳定行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingPolicy {
    KeepNull,
    ForwardFill,
    DropRow,
    DropLeadingWindow,
}

/// 因子元数据：id 因子唯一键、category 分类、description 文档、author/source/refresh_frequency 审计字段、required_fields 依赖列、missing_policy 缺失策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorMeta {
    pub id: String,
    pub category: FactorCategory,
    pub description: String,
    pub author: Option<String>,
    pub source: Option<String>,
    pub refresh_frequency: Option<String>,
    pub required_fields: Vec<String>,
    pub missing_policy: MissingPolicy,
}

/// 因子数据加载请求：symbols 标的列表、start/end 日期区间、required_fields 必备字段（loader 需校验）。
#[derive(Debug, Clone)]
pub struct FactorLoadRequest {
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub required_fields: Vec<String>,
}

/// 因子计算请求：factors 待计算的因子 id 列表、symbols/start/end 数据范围、run_checks 是否执行完整性检查（NaN 比例、时间对齐等）。
#[derive(Debug, Clone)]
pub struct FactorComputeRequest {
    pub factors: Vec<String>,
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub run_checks: bool,
}

/// 单因子计算结果：factor_id 因子 id、frame 计算结果 DataFrame（含 symbol/date/factor 列）。
#[derive(Debug, Clone)]
pub struct FactorComputeResult {
    pub factor_id: String,
    pub frame: DataFrame,
}
