use super::*;

#[test]
fn test_tdx_day_record_size() {
    // TdxDayRecord 应该是 32 字节 (与原始 day 文件记录大小一致)
    assert_eq!(std::mem::size_of::<TdxDayRecord>(), 32);
}

#[test]
fn test_date_string_conversion() {
    assert_eq!(date_string(20210801), "2021-08-01");
    assert_eq!(date_string(19900101), "1990-01-01");
}

#[test]
fn test_code_string_conversion() {
    let record = TdxDayRecord {
        code: 600000,
        date: 20210801,
        open: 100.0,
        high: 110.0,
        low: 95.0,
        close: 105.0,
        amount: 1000000.0,
        volume: 10000,
    };
    assert_eq!(record.code_string(), "600000");
}

#[test]
fn test_fuquan_calculator_empty() {
    let result = FuquanCalculator::calculate(&[], None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_fuquan_calculator_no_gbbq() {
    let days = vec![
        TdxDayRecord {
            code: 600000,
            date: 20210801,
            open: 100.0,
            high: 105.0,
            low: 99.0,
            close: 104.0,
            amount: 1000000.0,
            volume: 10000,
        },
        TdxDayRecord {
            code: 600000,
            date: 20210802,
            open: 104.5,
            high: 108.0,
            low: 103.0,
            close: 107.0,
            amount: 1000000.0,
            volume: 10000,
        },
    ];

    let result = FuquanCalculator::calculate(&days, None).unwrap();
    assert_eq!(result.len(), 2);

    // 第一天因子应该约为 1.0 * (104/104) = 1.0
    assert!((result[0].factor - 1.0).abs() < 0.01);

    // 第二天因子应该约为 1.0 * (107/104) ≈ 1.029
    assert!((result[1].factor - 1.029).abs() < 0.01);
}
