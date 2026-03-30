use std::collections::HashMap;

use crate::analysis::{
    IndicatorCache, IndicatorCacheKey, IndicatorInstanceId, IndicatorInput, IndicatorPipelineConfig,
    IndicatorRegistry, IndicatorSeries,
};
use crate::core::{QuantixError, Result};

pub type IndicatorOutputMap = HashMap<IndicatorInstanceId, IndicatorSeries>;

pub struct IndicatorPipeline {
    registry: IndicatorRegistry,
    cache: IndicatorCache,
}

impl IndicatorPipeline {
    pub fn with_builtin_registry() -> Self {
        Self {
            registry: IndicatorRegistry::register_builtin(),
            cache: IndicatorCache::new(),
        }
    }

    pub fn run(
        &mut self,
        config: &IndicatorPipelineConfig,
        input: &IndicatorInput,
    ) -> Result<IndicatorOutputMap> {
        let (start, end) = input.range();
        if end < start || end - start != input.close().len() {
            return Err(QuantixError::Config(format!(
                "indicator input range {:?} does not match close length {}",
                input.range(),
                input.close().len()
            )));
        }

        let mut output = HashMap::new();

        for spec in &config.indicators {
            if output.contains_key(spec.instance_id()) {
                return Err(QuantixError::Config(format!(
                    "duplicate indicator instance_id `{}` in pipeline config",
                    spec.instance_id().0
                )));
            }

            let descriptor = self.registry.descriptor(spec)?;
            if input.close().len() < descriptor.meta.lookback {
                return Err(QuantixError::Config(format!(
                    "indicator `{}` requires at least {} bars, got {}",
                    spec.name(),
                    descriptor.meta.lookback,
                    input.close().len()
                )));
            }

            let key = IndicatorCacheKey::new(
                input.dataset_fingerprint(),
                spec.instance_id().clone(),
                input.range(),
            );
            let series = self
                .cache
                .get_or_compute(key, || self.registry.compute(spec, input))?;

            output.insert(spec.instance_id().clone(), series);
        }

        Ok(output)
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}
