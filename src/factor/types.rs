use chrono::NaiveDate;
use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorCategory {
    Technical,
    Fundamental,
    Composite,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingPolicy {
    KeepNull,
    ForwardFill,
    DropRow,
    DropLeadingWindow,
}

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

#[derive(Debug, Clone)]
pub struct FactorLoadRequest {
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FactorComputeRequest {
    pub factors: Vec<String>,
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub run_checks: bool,
}

#[derive(Debug, Clone)]
pub struct FactorComputeResult {
    pub factor_id: String,
    pub frame: DataFrame,
}
