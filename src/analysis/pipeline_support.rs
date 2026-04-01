use crate::analysis::{IndicatorCacheKey, IndicatorInput, IndicatorOutputMap, IndicatorSpec};
use crate::core::{QuantixError, Result};

pub(super) fn validate_input_range(input: &IndicatorInput) -> Result<()> {
    let (start, end) = input.range();
    if end < start || end - start != input.close().len() {
        return Err(QuantixError::Config(format!(
            "indicator input range {:?} does not match close length {}",
            input.range(),
            input.close().len()
        )));
    }

    Ok(())
}

pub(super) fn ensure_unique_instance_id(
    output: &IndicatorOutputMap,
    spec: &IndicatorSpec,
) -> Result<()> {
    if output.contains_key(spec.instance_id()) {
        return Err(QuantixError::Config(format!(
            "duplicate indicator instance_id `{}` in pipeline config",
            spec.instance_id().0
        )));
    }

    Ok(())
}

pub(super) fn ensure_minimum_lookback(
    spec: &IndicatorSpec,
    lookback: usize,
    close_len: usize,
) -> Result<()> {
    if close_len < lookback {
        return Err(QuantixError::Config(format!(
            "indicator `{}` requires at least {} bars, got {}",
            spec.name(),
            lookback,
            close_len
        )));
    }

    Ok(())
}

pub(super) fn build_cache_key(
    input: &IndicatorInput,
    spec: &IndicatorSpec,
) -> IndicatorCacheKey {
    IndicatorCacheKey::new(
        input.dataset_fingerprint(),
        spec.instance_id().clone(),
        input.range(),
    )
}
