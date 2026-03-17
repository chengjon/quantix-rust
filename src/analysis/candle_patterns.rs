use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Below,
    At,
    Above,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationTuple {
    pub open: Relation,
    pub close: Relation,
    pub high: Relation,
    pub low: Relation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalCase {
    Case01,
    Case02,
    Case03,
    Case04,
    Case05,
    Case06,
    Case07,
    Case08,
    Case09,
    Case10,
    Case11,
    Case12,
    Case13,
    Case14,
    Case15,
    Case16,
    Case17,
    Case18,
    Case19,
    Case20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceSpan {
    EntireBelow,
    Intersects,
    EntireAbove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Bull,
    Bear,
    Doji,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketBias {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternConfig {
    pub epsilon: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    InvalidEpsilon,
    InvalidOhlc,
    MissingPreviousCloseReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandleInput {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferencePricePolicy {
    Explicit(Decimal),
    PreviousClose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedPattern {
    pub reference_span: ReferenceSpan,
    pub body_type: BodyType,
    pub has_upper_shadow: bool,
    pub has_lower_shadow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleFeatures {
    pub body_size: Decimal,
    pub range_size: Decimal,
    pub upper_shadow_size: Decimal,
    pub lower_shadow_size: Decimal,
    pub body_ratio: Decimal,
    pub upper_shadow_ratio: Decimal,
    pub lower_shadow_ratio: Decimal,
    pub close_position_ratio: Decimal,
    pub gap_from_reference: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandlePattern {
    pub relation: RelationTuple,
    pub canonical_case: Option<CanonicalCase>,
    pub extended: ExtendedPattern,
    pub bias: MarketBias,
    pub features: CandleFeatures,
}

pub fn recognize_single(
    candle: &CandleInput,
    reference: Decimal,
    config: &PatternConfig,
) -> Result<CandlePattern, PatternError> {
    validate_input(candle, config)?;

    let relation = RelationTuple {
        open: classify_against_reference(candle.open, reference, config.epsilon),
        close: classify_against_reference(candle.close, reference, config.epsilon),
        high: classify_against_reference(candle.high, reference, config.epsilon),
        low: classify_against_reference(candle.low, reference, config.epsilon),
    };

    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let extended = ExtendedPattern {
        reference_span: if candle.high < reference - config.epsilon {
            ReferenceSpan::EntireBelow
        } else if candle.low > reference + config.epsilon {
            ReferenceSpan::EntireAbove
        } else {
            ReferenceSpan::Intersects
        },
        body_type: body_type(candle.open, candle.close, config.epsilon),
        has_upper_shadow: candle.high > body_top + config.epsilon,
        has_lower_shadow: candle.low < body_bottom - config.epsilon,
    };

    let canonical_case = canonical_case(candle, &relation, config.epsilon);

    Ok(CandlePattern {
        relation,
        canonical_case,
        bias: bias_from_close(candle.close, reference, config.epsilon),
        features: features(candle, reference),
        extended,
    })
}

pub fn recognize_sequence(
    candles: &[CandleInput],
    policy: &ReferencePricePolicy,
    config: &PatternConfig,
) -> Result<Vec<CandlePattern>, PatternError> {
    match policy {
        ReferencePricePolicy::Explicit(reference) => candles
            .iter()
            .map(|candle| recognize_single(candle, *reference, config))
            .collect(),
        ReferencePricePolicy::PreviousClose => {
            if candles.len() < 2 {
                return Err(PatternError::MissingPreviousCloseReference);
            }

            candles
                .windows(2)
                .map(|pair| recognize_single(&pair[1], pair[0].close, config))
                .collect()
        }
    }
}

fn validate_input(candle: &CandleInput, config: &PatternConfig) -> Result<(), PatternError> {
    if config.epsilon <= Decimal::ZERO {
        return Err(PatternError::InvalidEpsilon);
    }

    if candle.high < candle.low {
        return Err(PatternError::InvalidOhlc);
    }

    let max_body = candle.open.max(candle.close);
    let min_body = candle.open.min(candle.close);
    if candle.high < max_body || candle.low > min_body {
        return Err(PatternError::InvalidOhlc);
    }

    Ok(())
}

fn classify_against_reference(value: Decimal, reference: Decimal, epsilon: Decimal) -> Relation {
    if value > reference + epsilon {
        Relation::Above
    } else if value < reference - epsilon {
        Relation::Below
    } else {
        Relation::At
    }
}

fn body_type(open: Decimal, close: Decimal, epsilon: Decimal) -> BodyType {
    if close > open + epsilon {
        BodyType::Bull
    } else if close < open - epsilon {
        BodyType::Bear
    } else {
        BodyType::Doji
    }
}

fn bias_from_close(close: Decimal, reference: Decimal, epsilon: Decimal) -> MarketBias {
    match classify_against_reference(close, reference, epsilon) {
        Relation::Above => MarketBias::Bullish,
        Relation::Below => MarketBias::Bearish,
        Relation::At => MarketBias::Neutral,
    }
}

fn features(candle: &CandleInput, reference: Decimal) -> CandleFeatures {
    let range_size = candle.high - candle.low;
    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let body_size = body_top - body_bottom;
    let upper_shadow_size = candle.high - body_top;
    let lower_shadow_size = body_bottom - candle.low;

    CandleFeatures {
        body_size,
        range_size,
        upper_shadow_size,
        lower_shadow_size,
        body_ratio: ratio(body_size, range_size),
        upper_shadow_ratio: ratio(upper_shadow_size, range_size),
        lower_shadow_ratio: ratio(lower_shadow_size, range_size),
        close_position_ratio: ratio(candle.close - candle.low, range_size),
        gap_from_reference: candle.close - reference,
    }
}

fn ratio(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator == Decimal::ZERO {
        Decimal::ZERO
    } else {
        numerator / denominator
    }
}

fn canonical_case(
    candle: &CandleInput,
    relation: &RelationTuple,
    epsilon: Decimal,
) -> Option<CanonicalCase> {
    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let has_upper_shadow = candle.high > body_top + epsilon;
    let has_lower_shadow = candle.low < body_bottom - epsilon;

    match (relation.open, relation.close, relation.high, relation.low) {
        (Relation::At, Relation::At, Relation::At, Relation::At) => Some(CanonicalCase::Case01),
        (Relation::At, Relation::At, Relation::Above, Relation::Below)
            if has_upper_shadow && has_lower_shadow =>
        {
            Some(CanonicalCase::Case04)
        }
        (Relation::At, Relation::Below, Relation::At, Relation::Below)
            if !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case05)
        }
        (Relation::At, Relation::Above, Relation::Above, Relation::At)
            if !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case07)
        }
        _ => None,
    }
}
