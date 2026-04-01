use std::collections::HashMap;

use crate::analysis::{
    IndicatorCache, IndicatorInstanceId, IndicatorInput, IndicatorPipelineConfig, IndicatorRegistry,
    IndicatorSeries,
};
use crate::core::Result;

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
        super::pipeline_support::validate_input_range(input)?;

        let mut output = HashMap::new();

        for spec in &config.indicators {
            super::pipeline_support::ensure_unique_instance_id(&output, spec)?;

            let descriptor = self.registry.descriptor(spec)?;
            super::pipeline_support::ensure_minimum_lookback(
                spec,
                descriptor.meta.lookback,
                input.close().len(),
            )?;

            let key = super::pipeline_support::build_cache_key(input, spec);
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
