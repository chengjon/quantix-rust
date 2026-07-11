//! chrono ↔ time conversion helpers.
//!
//! clickhouse-rs 0.12 has no chrono feature; CH row structs use
//! `time::OffsetDateTime`. Domain code (OpenStock, kline aggregator) uses
//! `chrono::{DateTime<Utc>, NaiveDateTime, NaiveDate}`. These helpers
//! bridge the two at the db layer boundary.
//!
//! Wall-clock semantics preserved: a Beijing naive datetime becomes an
//! OffsetDateTime with the same Y/M/D/H/M/S fields (offset tag is Utc,
//! matching the existing "naive_is Beijing wall-clock; tag as Utc"
//! convention from kline.rs pre-refactor).

use chrono::Utc;

pub(crate) fn naive_to_offsetdatetime(naive: chrono::NaiveDateTime) -> time::OffsetDateTime {
    use chrono::{Datelike, Timelike};
    // Construction via Y/M/D/H/M/S to avoid timezone-conversion surprises —
    // a Beijing wall-clock naive dt becomes an OffsetDateTime with the same
    // Y/M/D/H/M/S, tagged UTC (matching the pre-refactor kline.rs convention).
    let month: time::Month = (naive.month() as u8)
        .try_into()
        .unwrap_or(time::Month::January);
    let date = time::Date::from_calendar_date(naive.year(), month, naive.day() as u8)
        .unwrap_or(time::Date::MIN);
    let t = time::Time::from_hms(
        naive.hour() as u8,
        naive.minute() as u8,
        naive.second() as u8,
    )
    .unwrap_or(time::Time::MIDNIGHT);
    time::PrimitiveDateTime::new(date, t).assume_utc()
}

pub(crate) fn offsetdatetime_to_naivedate(dt: time::OffsetDateTime) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(dt.year(), dt.month() as u32, dt.day() as u32)
        .unwrap_or_default()
}

pub(crate) fn datetime_utc_to_offsetdatetime(dt: chrono::DateTime<Utc>) -> time::OffsetDateTime {
    naive_to_offsetdatetime(dt.naive_utc())
}
