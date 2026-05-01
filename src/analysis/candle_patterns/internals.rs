use super::*;

pub(super) fn validate_input(
    candle: &CandleInput,
    config: &PatternConfig,
) -> Result<(), PatternError> {
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

pub(super) fn classify_against_reference(
    value: Decimal,
    reference: Decimal,
    epsilon: Decimal,
) -> Relation {
    if value > reference + epsilon {
        Relation::Above
    } else if value < reference - epsilon {
        Relation::Below
    } else {
        Relation::At
    }
}

pub(super) fn body_type(open: Decimal, close: Decimal, epsilon: Decimal) -> BodyType {
    if close > open + epsilon {
        BodyType::Bull
    } else if close < open - epsilon {
        BodyType::Bear
    } else {
        BodyType::Doji
    }
}

pub(super) fn bias_from_close(close: Decimal, reference: Decimal, epsilon: Decimal) -> MarketBias {
    match classify_against_reference(close, reference, epsilon) {
        Relation::Above => MarketBias::Bullish,
        Relation::Below => MarketBias::Bearish,
        Relation::At => MarketBias::Neutral,
    }
}

pub(super) fn features(candle: &CandleInput, reference: Decimal) -> CandleFeatures {
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

pub(super) fn ratio(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator == Decimal::ZERO {
        Decimal::ZERO
    } else {
        numerator / denominator
    }
}

pub(super) fn canonical_case(
    candle: &CandleInput,
    relation: &RelationTuple,
    epsilon: Decimal,
) -> Option<CanonicalCase> {
    let body_top = candle.open.max(candle.close);
    let body_bottom = candle.open.min(candle.close);
    let has_upper_shadow = candle.high > body_top + epsilon;
    let has_lower_shadow = candle.low < body_bottom - epsilon;
    let body_type = body_type(candle.open, candle.close, epsilon);

    canonical_rules()
        .iter()
        .find(|rule| {
            rule.relation == *relation
                && rule.body_type == body_type
                && rule.has_upper_shadow == has_upper_shadow
                && rule.has_lower_shadow == has_lower_shadow
        })
        .map(|rule| rule.case)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalCaseRule {
    case: CanonicalCase,
    relation: RelationTuple,
    body_type: BodyType,
    has_upper_shadow: bool,
    has_lower_shadow: bool,
}

fn canonical_rules() -> &'static [CanonicalCaseRule] {
    use BodyType::{Bear, Bull, Doji};
    use CanonicalCase::*;
    use Relation::{Above, At, Below};

    &[
        CanonicalCaseRule {
            case: Case01,
            relation: RelationTuple {
                open: At,
                close: At,
                high: At,
                low: At,
            },
            body_type: Doji,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case02,
            relation: RelationTuple {
                open: At,
                close: At,
                high: At,
                low: Below,
            },
            body_type: Doji,
            has_upper_shadow: false,
            has_lower_shadow: true,
        },
        CanonicalCaseRule {
            case: Case03,
            relation: RelationTuple {
                open: At,
                close: At,
                high: Above,
                low: At,
            },
            body_type: Doji,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case04,
            relation: RelationTuple {
                open: At,
                close: At,
                high: Above,
                low: Below,
            },
            body_type: Doji,
            has_upper_shadow: true,
            has_lower_shadow: true,
        },
        CanonicalCaseRule {
            case: Case05,
            relation: RelationTuple {
                open: At,
                close: Below,
                high: At,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case06,
            relation: RelationTuple {
                open: At,
                close: Below,
                high: Above,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case07,
            relation: RelationTuple {
                open: At,
                close: Above,
                high: Above,
                low: At,
            },
            body_type: Bull,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case08,
            relation: RelationTuple {
                open: At,
                close: Above,
                high: Above,
                low: At,
            },
            body_type: Bull,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case09,
            relation: RelationTuple {
                open: Below,
                close: At,
                high: At,
                low: Below,
            },
            body_type: Bull,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case10,
            relation: RelationTuple {
                open: Below,
                close: At,
                high: Above,
                low: Below,
            },
            body_type: Bull,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case11,
            relation: RelationTuple {
                open: Below,
                close: Below,
                high: Below,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case12,
            relation: RelationTuple {
                open: Below,
                close: Below,
                high: At,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case13,
            relation: RelationTuple {
                open: Below,
                close: Below,
                high: Below,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: true,
        },
        CanonicalCaseRule {
            case: Case14,
            relation: RelationTuple {
                open: Below,
                close: Above,
                high: Above,
                low: Below,
            },
            body_type: Bull,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case15,
            relation: RelationTuple {
                open: Above,
                close: At,
                high: Above,
                low: At,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case16,
            relation: RelationTuple {
                open: Above,
                close: At,
                high: Above,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: true,
        },
        CanonicalCaseRule {
            case: Case17,
            relation: RelationTuple {
                open: Above,
                close: Below,
                high: Above,
                low: Below,
            },
            body_type: Bear,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case18,
            relation: RelationTuple {
                open: Above,
                close: Above,
                high: Above,
                low: Above,
            },
            body_type: Bull,
            has_upper_shadow: false,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case19,
            relation: RelationTuple {
                open: Above,
                close: Above,
                high: Above,
                low: Above,
            },
            body_type: Bull,
            has_upper_shadow: true,
            has_lower_shadow: false,
        },
        CanonicalCaseRule {
            case: Case20,
            relation: RelationTuple {
                open: Above,
                close: Above,
                high: Above,
                low: Above,
            },
            body_type: Bull,
            has_upper_shadow: false,
            has_lower_shadow: true,
        },
    ]
}
