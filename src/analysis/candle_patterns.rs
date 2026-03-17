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

impl CanonicalCase {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Case01 => "Case01",
            Self::Case02 => "Case02",
            Self::Case03 => "Case03",
            Self::Case04 => "Case04",
            Self::Case05 => "Case05",
            Self::Case06 => "Case06",
            Self::Case07 => "Case07",
            Self::Case08 => "Case08",
            Self::Case09 => "Case09",
            Self::Case10 => "Case10",
            Self::Case11 => "Case11",
            Self::Case12 => "Case12",
            Self::Case13 => "Case13",
            Self::Case14 => "Case14",
            Self::Case15 => "Case15",
            Self::Case16 => "Case16",
            Self::Case17 => "Case17",
            Self::Case18 => "Case18",
            Self::Case19 => "Case19",
            Self::Case20 => "Case20",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Case01 => "一字线",
            Self::Case02 => "T字线",
            Self::Case03 => "倒T字线",
            Self::Case04 => "十字星",
            Self::Case05 => "光头光脚阴线",
            Self::Case06 => "光脚阴线",
            Self::Case07 => "光头光脚阳线",
            Self::Case08 => "光头阳线",
            Self::Case09 => "光头光脚阳线",
            Self::Case10 => "光脚阳线",
            Self::Case11 => "光头光脚阴线",
            Self::Case12 => "光脚阴线",
            Self::Case13 => "光头阴线",
            Self::Case14 => "光头光脚阳线",
            Self::Case15 => "光头光脚阴线",
            Self::Case16 => "光头阴线",
            Self::Case17 => "光头光脚阴线",
            Self::Case18 => "光头光脚阳线",
            Self::Case19 => "光脚阳线",
            Self::Case20 => "光头阳线",
        }
    }
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

fn canonical_case(
    candle: &CandleInput,
    relation: &RelationTuple,
    epsilon: Decimal,
) -> Option<CanonicalCase> {
    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let has_upper_shadow = candle.high > body_top + epsilon;
    let has_lower_shadow = candle.low < body_bottom - epsilon;
    let body_type = body_type(candle.open, candle.close, epsilon);

    match (relation.open, relation.close, relation.high, relation.low) {
        (Relation::At, Relation::At, Relation::At, Relation::At) => Some(CanonicalCase::Case01),
        (Relation::At, Relation::At, Relation::At, Relation::Below)
            if !has_upper_shadow && has_lower_shadow =>
        {
            Some(CanonicalCase::Case02)
        }
        (Relation::At, Relation::At, Relation::Above, Relation::At)
            if has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case03)
        }
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
        (Relation::At, Relation::Below, Relation::Above, Relation::Below)
            if has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case06)
        }
        (Relation::At, Relation::Above, Relation::Above, Relation::At)
            if !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case07)
        }
        (Relation::At, Relation::Above, Relation::Above, Relation::At)
            if has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case08)
        }
        (Relation::Below, Relation::At, Relation::At, Relation::Below)
            if body_type == BodyType::Bull && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case09)
        }
        (Relation::Below, Relation::At, Relation::Above, Relation::Below)
            if body_type == BodyType::Bull && has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case10)
        }
        (Relation::Below, Relation::Below, Relation::Below, Relation::Below)
            if body_type == BodyType::Bear && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case11)
        }
        (Relation::Below, Relation::Below, Relation::At, Relation::Below)
            if body_type == BodyType::Bear && has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case12)
        }
        (Relation::Below, Relation::Below, Relation::Below, Relation::Below)
            if body_type == BodyType::Bear && !has_upper_shadow && has_lower_shadow =>
        {
            Some(CanonicalCase::Case13)
        }
        (Relation::Below, Relation::Above, Relation::Above, Relation::Below)
            if body_type == BodyType::Bull && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case14)
        }
        (Relation::Above, Relation::At, Relation::Above, Relation::At)
            if body_type == BodyType::Bear && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case15)
        }
        (Relation::Above, Relation::At, Relation::Above, Relation::Below)
            if body_type == BodyType::Bear && !has_upper_shadow && has_lower_shadow =>
        {
            Some(CanonicalCase::Case16)
        }
        (Relation::Above, Relation::Below, Relation::Above, Relation::Below)
            if body_type == BodyType::Bear && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case17)
        }
        (Relation::Above, Relation::Above, Relation::Above, Relation::Above)
            if body_type == BodyType::Bull && !has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case18)
        }
        (Relation::Above, Relation::Above, Relation::Above, Relation::Above)
            if body_type == BodyType::Bull && has_upper_shadow && !has_lower_shadow =>
        {
            Some(CanonicalCase::Case19)
        }
        (Relation::Above, Relation::Above, Relation::Above, Relation::Above)
            if body_type == BodyType::Bull && !has_upper_shadow && has_lower_shadow =>
        {
            Some(CanonicalCase::Case20)
        }
        _ => None,
    }
}

fn features(candle: &CandleInput, reference: Decimal) -> CandleFeatures {
    let range_size = candle.high - candle.low;
    let body_size = (candle.close - candle.open).abs();
    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let upper_shadow_size = candle.high - body_top;
    let lower_shadow_size = body_bottom - candle.low;

    let ratio = |value: Decimal| {
        if range_size.is_zero() {
            Decimal::ZERO
        } else {
            value / range_size
        }
    };

    let close_position_ratio = if range_size.is_zero() {
        Decimal::ZERO
    } else {
        (candle.close - candle.low) / range_size
    };

    CandleFeatures {
        body_size,
        range_size,
        upper_shadow_size,
        lower_shadow_size,
        body_ratio: ratio(body_size),
        upper_shadow_ratio: ratio(upper_shadow_size),
        lower_shadow_ratio: ratio(lower_shadow_size),
        close_position_ratio,
        gap_from_reference: candle.open - reference,
    }
}
