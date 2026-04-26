use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_sma() {
    let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let result = sma(&data, 3);

    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    assert_eq!(result[2], Some(dec!(2)));
    assert_eq!(result[3], Some(dec!(3)));
    assert_eq!(result[4], Some(dec!(4)));
}

#[test]
fn test_ema() {
    let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let result = ema(&data, 3);

    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    assert!(result[2].is_some());
    assert!(result[3].is_some());
    assert!(result[4].is_some());
}

#[test]
fn test_rsi() {
    let data = vec![
        dec!(10),
        dec!(12),
        dec!(11),
        dec!(13),
        dec!(15),
        dec!(14),
        dec!(16),
        dec!(15),
        dec!(17),
        dec!(19),
        dec!(18),
        dec!(20),
        dec!(19),
        dec!(21),
    ];
    let result = rsi(&data, 6);

    // RSI 应该在 0-100 之间
    for i in 6..data.len() {
        if let Some(val) = result[i] {
            assert!(val >= Decimal::ZERO);
            assert!(val <= Decimal::from(100));
        }
    }
}

#[test]
fn test_macd() {
    let data: Vec<Decimal> = (1..=50).map(|x| Decimal::from(x)).collect();
    let result = macd(&data, 12, 26, 9);

    // 后期应该有值
    assert!(result[40].is_some());
    assert!(result[49].is_some());
}

#[test]
fn test_kdj() {
    let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
    let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
    let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

    let result = kdj(&high, &low, &close, 9, 3, 3);

    // KDJ 应该有值
    assert!(result[9].is_some());
    if let Some(kdj) = result[9] {
        assert!(kdj.k >= Decimal::ZERO && kdj.k <= Decimal::from(100));
    }
}

#[test]
fn test_bollinger_bands() {
    let data: Vec<Decimal> = (1..=30).map(|x| Decimal::from(x)).collect();
    let result = bollinger_bands(&data, 20, 2);

    // 后期应该有值
    assert!(result[19].is_some());
    if let Some(boll) = result[19] {
        assert!(boll.upper > boll.middle);
        assert!(boll.lower < boll.middle);
    }
}

#[test]
fn test_atr() {
    let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
    let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
    let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

    let result = atr(&high, &low, &close, 14);

    // ATR 应该是正值
    assert!(result[14].is_some());
    if let Some(atr_val) = result[14] {
        assert!(atr_val > Decimal::ZERO);
    }
}

#[test]
fn test_obv() {
    let close = vec![dec!(10), dec!(11), dec!(10), dec!(12), dec!(11)];
    let volume = vec![1000, 2000, 1500, 3000, 2500];

    let result = obv(&close, &volume);

    assert_eq!(result[0], Some(1000)); // 初始值
    assert_eq!(result[1], Some(3000)); // 10→11 上涨: 1000 + 2000 = 3000
    assert_eq!(result[2], Some(1500)); // 11→10 下跌: 3000 - 1500 = 1500
    assert_eq!(result[3], Some(4500)); // 10→12 上涨: 1500 + 3000 = 4500
    assert_eq!(result[4], Some(2000)); // 12→11 下跌: 4500 - 2500 = 2000
}

#[test]
fn test_cci() {
    let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
    let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
    let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

    let result = cci(&high, &low, &close, 14);

    // CCI 应该有值
    assert!(result[13].is_some());
}

#[test]
fn test_williams_r() {
    let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
    let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
    let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

    let result = williams_r(&high, &low, &close, 9);

    // WR 应该在 -100 到 0 之间
    assert!(result[9].is_some());
    if let Some(wr_val) = result[9] {
        assert!(wr_val >= Decimal::from(-100));
        assert!(wr_val <= Decimal::ZERO);
    }
}

#[test]
fn test_wma() {
    let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let result = wma(&data, 3);

    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    assert!(result[2].is_some());
    // WMA 应该比 SMA 大（给近期数据更高权重）
    let sma_result = sma(&data, 3);
    if let (Some(w), Some(s)) = (result[4], sma_result[4]) {
        assert!(w >= s);
    }
}
