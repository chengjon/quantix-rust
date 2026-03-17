use quantix_cli::analysis::candle_patterns::{
    BodyType, CandleInput, CanonicalCase, MarketBias, PatternConfig, PatternError,
    ReferencePricePolicy, ReferenceSpan, recognize_sequence, recognize_single,
};
use rust_decimal_macros::dec;

fn default_config() -> PatternConfig {
    PatternConfig { epsilon: dec!(0.0001) }
}

#[test]
fn recognizes_case01_flat_line_at_reference() {
    let candle = CandleInput {
        open: dec!(10),
        high: dec!(10),
        low: dec!(10),
        close: dec!(10),
    };

    let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();

    assert_eq!(pattern.canonical_case, Some(CanonicalCase::Case01));
    assert_eq!(pattern.bias, MarketBias::Neutral);
    assert_eq!(pattern.extended.reference_span, ReferenceSpan::Intersects);
    assert_eq!(pattern.extended.body_type, BodyType::Doji);
}

#[test]
fn recognizes_legal_non_canonical_pattern_below_reference() {
    let candle = CandleInput {
        open: dec!(8),
        high: dec!(9),
        low: dec!(6),
        close: dec!(6),
    };

    let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();

    assert_eq!(pattern.canonical_case, None);
    assert_eq!(pattern.bias, MarketBias::Bearish);
    assert_eq!(pattern.extended.reference_span, ReferenceSpan::EntireBelow);
    assert_eq!(pattern.extended.body_type, BodyType::Bear);
    assert!(pattern.extended.has_upper_shadow);
    assert!(!pattern.extended.has_lower_shadow);
}

#[test]
fn rejects_invalid_ohlc_input() {
    let candle = CandleInput {
        open: dec!(10),
        high: dec!(9),
        low: dec!(8),
        close: dec!(9),
    };

    let error = recognize_single(&candle, dec!(10), &default_config()).unwrap_err();

    assert_eq!(error, PatternError::InvalidOhlc);
}

#[test]
fn recognizes_sequence_using_previous_close_reference() {
    let candles = vec![
        CandleInput {
            open: dec!(10),
            high: dec!(11),
            low: dec!(9),
            close: dec!(10),
        },
        CandleInput {
            open: dec!(10),
            high: dec!(12),
            low: dec!(10),
            close: dec!(12),
        },
    ];

    let patterns =
        recognize_sequence(&candles, &ReferencePricePolicy::PreviousClose, &default_config())
            .unwrap();

    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].bias, MarketBias::Bullish);
}

#[test]
fn rejects_previous_close_sequence_without_prior_candle() {
    let candles = vec![CandleInput {
        open: dec!(10),
        high: dec!(10),
        low: dec!(10),
        close: dec!(10),
    }];

    let error =
        recognize_sequence(&candles, &ReferencePricePolicy::PreviousClose, &default_config())
            .unwrap_err();

    assert_eq!(error, PatternError::MissingPreviousCloseReference);
}

#[test]
fn recognizes_case04_cross_star() {
    let candle = CandleInput {
        open: dec!(10),
        high: dec!(12),
        low: dec!(8),
        close: dec!(10),
    };

    let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();

    assert_eq!(pattern.canonical_case, Some(CanonicalCase::Case04));
    assert_eq!(pattern.bias, MarketBias::Neutral);
    assert_eq!(pattern.extended.reference_span, ReferenceSpan::Intersects);
}

#[test]
fn recognizes_case05_full_bearish_body_from_reference() {
    let candle = CandleInput {
        open: dec!(10),
        high: dec!(10),
        low: dec!(8),
        close: dec!(8),
    };

    let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();

    assert_eq!(pattern.canonical_case, Some(CanonicalCase::Case05));
    assert_eq!(pattern.bias, MarketBias::Bearish);
    assert_eq!(pattern.extended.body_type, BodyType::Bear);
}

#[test]
fn recognizes_case07_full_bullish_body_from_reference() {
    let candle = CandleInput {
        open: dec!(10),
        high: dec!(12),
        low: dec!(10),
        close: dec!(12),
    };

    let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();

    assert_eq!(pattern.canonical_case, Some(CanonicalCase::Case07));
    assert_eq!(pattern.bias, MarketBias::Bullish);
    assert_eq!(pattern.extended.body_type, BodyType::Bull);
}

#[test]
fn recognizes_all_documented_canonical_cases() {
    let cases = [
        (
            CandleInput {
                open: dec!(10),
                high: dec!(10),
                low: dec!(10),
                close: dec!(10),
            },
            CanonicalCase::Case01,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(10),
                low: dec!(8),
                close: dec!(10),
            },
            CanonicalCase::Case02,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(12),
                low: dec!(10),
                close: dec!(10),
            },
            CanonicalCase::Case03,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(12),
                low: dec!(8),
                close: dec!(10),
            },
            CanonicalCase::Case04,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(10),
                low: dec!(8),
                close: dec!(8),
            },
            CanonicalCase::Case05,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(12),
                low: dec!(8),
                close: dec!(8),
            },
            CanonicalCase::Case06,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(12),
                low: dec!(10),
                close: dec!(12),
            },
            CanonicalCase::Case07,
        ),
        (
            CandleInput {
                open: dec!(10),
                high: dec!(14),
                low: dec!(10),
                close: dec!(12),
            },
            CanonicalCase::Case08,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(10),
                low: dec!(8),
                close: dec!(10),
            },
            CanonicalCase::Case09,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(12),
                low: dec!(8),
                close: dec!(10),
            },
            CanonicalCase::Case10,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(8),
                low: dec!(6),
                close: dec!(6),
            },
            CanonicalCase::Case11,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(10),
                low: dec!(6),
                close: dec!(6),
            },
            CanonicalCase::Case12,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(8),
                low: dec!(5),
                close: dec!(6),
            },
            CanonicalCase::Case13,
        ),
        (
            CandleInput {
                open: dec!(8),
                high: dec!(12),
                low: dec!(8),
                close: dec!(12),
            },
            CanonicalCase::Case14,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(12),
                low: dec!(10),
                close: dec!(10),
            },
            CanonicalCase::Case15,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(12),
                low: dec!(8),
                close: dec!(10),
            },
            CanonicalCase::Case16,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(12),
                low: dec!(8),
                close: dec!(8),
            },
            CanonicalCase::Case17,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(14),
                low: dec!(12),
                close: dec!(14),
            },
            CanonicalCase::Case18,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(16),
                low: dec!(12),
                close: dec!(14),
            },
            CanonicalCase::Case19,
        ),
        (
            CandleInput {
                open: dec!(12),
                high: dec!(14),
                low: dec!(11),
                close: dec!(14),
            },
            CanonicalCase::Case20,
        ),
    ];

    for (candle, expected) in cases {
        let pattern = recognize_single(&candle, dec!(10), &default_config()).unwrap();
        assert_eq!(pattern.canonical_case, Some(expected), "failed canonical case {expected:?}");
    }
}

#[test]
fn exposes_stable_metadata_for_canonical_cases() {
    let cases = [
        (CanonicalCase::Case01, "Case01", "一字线"),
        (CanonicalCase::Case02, "Case02", "T字线"),
        (CanonicalCase::Case03, "Case03", "倒T字线"),
        (CanonicalCase::Case04, "Case04", "十字星"),
        (CanonicalCase::Case05, "Case05", "光头光脚阴线"),
        (CanonicalCase::Case06, "Case06", "光脚阴线"),
        (CanonicalCase::Case07, "Case07", "光头光脚阳线"),
        (CanonicalCase::Case08, "Case08", "光头阳线"),
        (CanonicalCase::Case09, "Case09", "光头光脚阳线"),
        (CanonicalCase::Case10, "Case10", "光脚阳线"),
        (CanonicalCase::Case11, "Case11", "光头光脚阴线"),
        (CanonicalCase::Case12, "Case12", "光脚阴线"),
        (CanonicalCase::Case13, "Case13", "光头阴线"),
        (CanonicalCase::Case14, "Case14", "光头光脚阳线"),
        (CanonicalCase::Case15, "Case15", "光头光脚阴线"),
        (CanonicalCase::Case16, "Case16", "光头阴线"),
        (CanonicalCase::Case17, "Case17", "光头光脚阴线"),
        (CanonicalCase::Case18, "Case18", "光头光脚阳线"),
        (CanonicalCase::Case19, "Case19", "光脚阳线"),
        (CanonicalCase::Case20, "Case20", "光头阳线"),
    ];

    for (case_id, expected_id, expected_name) in cases {
        assert_eq!(case_id.id(), expected_id);
        assert_eq!(case_id.display_name(), expected_name);
    }
}
