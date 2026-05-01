mod internals;

use self::internals::{
    bias_from_close, body_type, canonical_case, classify_against_reference, features,
    validate_input,
};
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
