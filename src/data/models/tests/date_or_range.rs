use chrono::NaiveDate;

use crate::data::models::{DateOrRange, chunk_range_weekly, iter_dates_inclusive};

#[test]
fn u1_date_only_returns_date_variant() {
    let r = DateOrRange::from_cli(Some("2026-06-30"), None, None).unwrap();
    assert!(matches!(r, DateOrRange::Date(_)));
    if let DateOrRange::Date(actual) = r {
        assert_eq!(actual, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }
}

#[test]
fn u2_start_and_end_returns_range_variant() {
    let r = DateOrRange::from_cli(None, Some("2026-06-01"), Some("2026-06-30")).unwrap();
    if let DateOrRange::Range { start, end } = r {
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    } else {
        panic!("expected Range, got {:?}", r);
    }
}

#[test]
fn u3_start_only_errors() {
    let r = DateOrRange::from_cli(None, Some("2026-06-01"), None);
    assert!(r.is_err());
    let msg = format!("{}", r.unwrap_err());
    assert!(
        msg.contains("--start") && msg.contains("--end"),
        "error should name both flags: {}",
        msg
    );
}

#[test]
fn u4_end_only_errors() {
    let r = DateOrRange::from_cli(None, None, Some("2026-06-30"));
    assert!(r.is_err());
}

#[test]
fn u5_date_and_start_conflict_errors() {
    let r = DateOrRange::from_cli(Some("2026-06-30"), Some("2026-06-01"), None);
    assert!(r.is_err());
    let msg = format!("{}", r.unwrap_err());
    assert!(
        msg.contains("--date") && msg.contains("--start"),
        "msg: {}",
        msg
    );
}

#[test]
fn u6_start_after_end_errors() {
    let r = DateOrRange::from_cli(None, Some("2026-06-30"), Some("2026-06-01"));
    assert!(r.is_err());
}

#[test]
fn u7_all_none_errors() {
    let r = DateOrRange::from_cli(None, None, None);
    assert!(r.is_err());
    let msg = format!("{}", r.unwrap_err());
    assert!(
        msg.contains("--date") || msg.contains("--start"),
        "msg: {}",
        msg
    );
}

#[test]
fn iter_dates_inclusive_yields_all_days_in_order() {
    let days: Vec<NaiveDate> = iter_dates_inclusive(
        NaiveDate::from_ymd_opt(2026, 6, 28).unwrap(),
        NaiveDate::from_ymd_opt(2026, 7, 2).unwrap(),
    )
    .collect();
    assert_eq!(days.len(), 5);
    assert_eq!(days[0], NaiveDate::from_ymd_opt(2026, 6, 28).unwrap());
    assert_eq!(days[4], NaiveDate::from_ymd_opt(2026, 7, 2).unwrap());
    // 跨月验证
    assert_eq!(days[2], NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    assert_eq!(days[3], NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
}

#[test]
fn iter_dates_inclusive_single_day_yields_one() {
    let days: Vec<NaiveDate> = iter_dates_inclusive(
        NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
    )
    .collect();
    assert_eq!(days, vec![NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()]);
}

#[test]
fn chunk_range_weekly_single_day_returns_one_chunk() {
    let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    let chunks = chunk_range_weekly(ymd(2026, 6, 1), ymd(2026, 6, 1));
    assert_eq!(chunks, vec![(ymd(2026, 6, 1), ymd(2026, 6, 1))]);
}

#[test]
fn chunk_range_weekly_exact_7_day_returns_one_chunk() {
    let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 7 days inclusive: 2026-06-01..=2026-06-07
    let chunks = chunk_range_weekly(ymd(2026, 6, 1), ymd(2026, 6, 7));
    assert_eq!(chunks, vec![(ymd(2026, 6, 1), ymd(2026, 6, 7))]);
}

#[test]
fn chunk_range_weekly_8_day_returns_two_chunks() {
    let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 8 days inclusive: 2026-06-01..=2026-06-08
    // First chunk: 06-01..=06-07 (7 days); second chunk: 06-08..=06-08 (1 day)
    let chunks = chunk_range_weekly(ymd(2026, 6, 1), ymd(2026, 6, 8));
    assert_eq!(
        chunks,
        vec![
            (ymd(2026, 6, 1), ymd(2026, 6, 7)),
            (ymd(2026, 6, 8), ymd(2026, 6, 8)),
        ]
    );
}

#[test]
fn chunk_range_weekly_15_day_returns_three_chunks() {
    let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 15 days inclusive: 2026-06-01..=2026-06-15
    // Boundary: 7-day chunks at exactly 7 inclusive days remaining are taken
    // whole. (end - cursor).num_days() at cursor=06-01 vs end=06-15 is 14
    // (>= 7) → cut at 06-07; at cursor=06-08 vs end=06-15 is 7 (>= 7) → cut
    // at 06-14; at cursor=06-15 vs end=06-15 is 0 (< 7) → seg_end=end.
    let chunks = chunk_range_weekly(ymd(2026, 6, 1), ymd(2026, 6, 15));
    assert_eq!(
        chunks,
        vec![
            (ymd(2026, 6, 1), ymd(2026, 6, 7)),
            (ymd(2026, 6, 8), ymd(2026, 6, 14)),
            (ymd(2026, 6, 15), ymd(2026, 6, 15)),
        ]
    );
    // INV-1B: contiguous coverage
    assert_eq!(chunks.first().unwrap().0, ymd(2026, 6, 1));
    assert_eq!(chunks.last().unwrap().1, ymd(2026, 6, 15));
    for window in chunks.windows(2) {
        assert_eq!(window[0].1.succ_opt().unwrap(), window[1].0);
    }
}

#[test]
fn chunk_range_weekly_long_range_covers_full_window() {
    let ymd = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 30 days inclusive: 2026-06-01..=2026-06-30
    let chunks = chunk_range_weekly(ymd(2026, 6, 1), ymd(2026, 6, 30));
    // Expected: 06-01..=06-07, 06-08..=06-14, 06-15..=06-21, 06-22..=06-28, 06-29..=06-30
    assert_eq!(
        chunks,
        vec![
            (ymd(2026, 6, 1), ymd(2026, 6, 7)),
            (ymd(2026, 6, 8), ymd(2026, 6, 14)),
            (ymd(2026, 6, 15), ymd(2026, 6, 21)),
            (ymd(2026, 6, 22), ymd(2026, 6, 28)),
            (ymd(2026, 6, 29), ymd(2026, 6, 30)),
        ]
    );
    // INV-1B: contiguous coverage
    assert_eq!(chunks.first().unwrap().0, ymd(2026, 6, 1));
    assert_eq!(chunks.last().unwrap().1, ymd(2026, 6, 30));
    for window in chunks.windows(2) {
        // chunks[i].1 + 1 day == chunks[i+1].0
        assert_eq!(
            window[0].1.succ_opt().unwrap(),
            window[1].0,
            "gap between {:?} and {:?}",
            window[0],
            window[1]
        );
    }
    // INV-1C: each chunk ≤ 7 days inclusive
    for (s, e) in &chunks {
        let n = (*e - *s).num_days() + 1;
        assert!(n <= 7, "chunk {:?}-{:?} is {} days, > 7", s, e, n);
    }
}
