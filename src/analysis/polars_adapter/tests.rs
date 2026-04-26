use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_init_polars() {
    // 测试 Polars 初始化
    let result = init_polars();
    assert!(result.is_ok());
}

#[test]
fn test_batch_kline_data() {
    let data = BatchKlineData {
        code: "000001".to_string(),
        timestamps: vec![1, 2, 3],
        open: vec![10.0, 11.0, 12.0],
        high: vec![10.5, 11.5, 12.5],
        low: vec![9.5, 10.5, 11.5],
        close: vec![10.0, 11.0, 12.0],
        volume: vec![1000, 2000, 3000],
        amount: vec![10000.0, 20000.0, 30000.0],
    };

    assert_eq!(data.len(), 3);
    assert!(!data.is_empty());
    assert_eq!(data.get_column("close"), vec![10.0, 11.0, 12.0]);
}

#[test]
fn test_calculator_ma() {
    let calc = PolarsCalculator::new();
    let data = BatchKlineData {
        code: "000001".to_string(),
        timestamps: vec![1, 2, 3, 4, 5],
        open: vec![10.0, 11.0, 12.0, 13.0, 14.0],
        high: vec![10.5, 11.5, 12.5, 13.5, 14.5],
        low: vec![9.5, 10.5, 11.5, 12.5, 13.5],
        close: vec![10.0, 11.0, 12.0, 13.0, 14.0],
        volume: vec![1000, 2000, 3000, 4000, 5000],
        amount: vec![10000.0, 20000.0, 30000.0, 40000.0, 50000.0],
    };

    let result = calc.ma(&data, 3);
    // 前 2 个应该是 None (窗口不足)
    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    // 第 3 个应该是 (10+11+12)/3 = 11
    assert_eq!(result[2], Some(dec!(11)));
}

#[test]
fn test_calculator_batch() {
    let calc = PolarsCalculator::new();
    let data = BatchKlineData {
        code: "000001".to_string(),
        timestamps: vec![1, 2, 3, 4, 5],
        open: vec![10.0, 11.0, 12.0, 13.0, 14.0],
        high: vec![10.5, 11.5, 12.5, 13.5, 14.5],
        low: vec![9.5, 10.5, 11.5, 12.5, 13.5],
        close: vec![10.0, 11.0, 12.0, 13.0, 14.0],
        volume: vec![1000, 2000, 3000, 4000, 5000],
        amount: vec![10000.0, 20000.0, 30000.0, 40000.0, 50000.0],
    };

    let result = calc.calculate_batch(&data, &["ma3", "ma5"]);
    assert!(result.contains_key("ma3"));
    assert!(result.contains_key("ma5"));
}

#[test]
fn test_multi_stock_data() {
    let mut multi = MultiStockData::new();

    multi.add_stock(
        "000001".to_string(),
        BatchKlineData {
            code: "000001".to_string(),
            timestamps: vec![1, 2],
            close: vec![10.0, 11.0],
            open: vec![10.0, 11.0],
            high: vec![10.5, 11.5],
            low: vec![9.5, 10.5],
            volume: vec![1000, 2000],
            amount: vec![10000.0, 20000.0],
        },
    );

    multi.add_stock(
        "000002".to_string(),
        BatchKlineData {
            code: "000002".to_string(),
            timestamps: vec![1, 2],
            close: vec![20.0, 21.0],
            open: vec![20.0, 21.0],
            high: vec![20.5, 21.5],
            low: vec![19.5, 20.5],
            volume: vec![3000, 4000],
            amount: vec![60000.0, 70000.0],
        },
    );

    assert_eq!(multi.stocks.len(), 2);
    assert!(multi.stocks.contains_key("000001"));
    assert!(multi.stocks.contains_key("000002"));
}

#[test]
fn test_from_kline_vec() {
    use crate::data::models::Kline;

    let klines = vec![Kline {
        code: "000001".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        open: dec!(10.0),
        high: dec!(10.5),
        low: dec!(9.5),
        close: dec!(10.0),
        volume: 1000,
        amount: Some(dec!(10000.0)),
        adjust_type: crate::data::models::AdjustType::None,
    }];

    let batch = from_kline_vec(&klines);
    assert_eq!(batch.code, "000001");
    assert_eq!(batch.len(), 1);
}
