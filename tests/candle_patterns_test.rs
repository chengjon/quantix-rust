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
