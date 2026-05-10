use crate::core::{QuantixError, Result};
use crate::factor::dataset::FactorDataset;
use crate::factor::operators::{cs_rank, ts_delay, ts_delta};
use crate::factor::types::{FactorCategory, FactorComputeResult, FactorMeta, MissingPolicy};

pub struct FactorCatalog {
    metas: Vec<FactorMeta>,
}

pub fn builtin_factor_catalog() -> FactorCatalog {
    FactorCatalog {
        metas: vec![
            FactorMeta {
                id: "rank_close".to_string(),
                category: FactorCategory::Technical,
                description: "Cross-sectional rank of close within each date".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delay_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol delayed close".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delta_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol close delta".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
        ],
    }
}

impl FactorCatalog {
    pub fn list(&self) -> &[FactorMeta] {
        &self.metas
    }

    pub fn compute(&self, factor_id: &str, dataset: &FactorDataset) -> Result<FactorComputeResult> {
        let values = match factor_id {
            "rank_close" => cs_rank(dataset.frame(), "close"),
            "delay_close_1" => ts_delay(dataset.frame(), "close", 1),
            "delta_close_1" => ts_delta(dataset.frame(), "close", 1),
            other => {
                return Err(QuantixError::Unsupported(format!(
                    "unknown factor `{}`",
                    other
                )));
            }
        }
        .map_err(|e| {
            QuantixError::DataParse(format!("factor `{}` compute failed: {}", factor_id, e))
        })?;

        let mut frame = dataset
            .frame()
            .select(["date", "symbol"])
            .map_err(|e| QuantixError::DataParse(format!("factor output select failed: {}", e)))?;
        let mut values = values.clone();
        values.rename("value".into());
        frame.with_column(values).map_err(|e| {
            QuantixError::DataParse(format!("factor output value attach failed: {}", e))
        })?;

        Ok(FactorComputeResult {
            factor_id: factor_id.to_string(),
            frame,
        })
    }
}
