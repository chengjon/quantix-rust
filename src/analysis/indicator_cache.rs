use std::collections::HashMap;

use crate::analysis::{IndicatorInstanceId, IndicatorSeries};
use crate::core::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorCacheKey {
    pub dataset_fingerprint: String,
    pub instance_id: IndicatorInstanceId,
    pub range: (usize, usize),
}

impl IndicatorCacheKey {
    pub fn new(
        dataset_fingerprint: impl Into<String>,
        instance_id: IndicatorInstanceId,
        range: (usize, usize),
    ) -> Self {
        Self {
            dataset_fingerprint: dataset_fingerprint.into(),
            instance_id,
            range,
        }
    }
}

#[derive(Debug, Default)]
pub struct IndicatorCache {
    entries: HashMap<IndicatorCacheKey, IndicatorSeries>,
}

impl IndicatorCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_compute<F>(
        &mut self,
        key: IndicatorCacheKey,
        compute: F,
    ) -> Result<IndicatorSeries>
    where
        F: FnOnce() -> Result<IndicatorSeries>,
    {
        if let Some(cached) = self.entries.get(&key) {
            return Ok(cached.clone());
        }

        let computed = compute()?;
        self.entries.insert(key, computed.clone());
        Ok(computed)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
