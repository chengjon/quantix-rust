use chrono::{DateTime, Timelike, Utc};

use super::kline_aggregator::KlinePeriod;

pub(super) fn quote_timestamp_or(timestamp: u64, fallback: DateTime<Utc>) -> DateTime<Utc> {
    i64::try_from(timestamp)
        .ok()
        .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
        .unwrap_or(fallback)
}

pub(super) fn resolved_high_low(open: f64, high: f64, low: f64) -> (f64, f64) {
    let resolved_high = if high > f64::MIN { high } else { open };
    let resolved_low = if low < f64::MAX { low } else { open };
    (resolved_high, resolved_low)
}

pub(super) fn make_window_key(
    code: &str,
    period: KlinePeriod,
    time: &DateTime<Utc>,
) -> String {
    let date_str = time.format("%Y-%m-%d").to_string();
    format!("{}:{}:{}", code, period.as_str(), date_str)
}

pub(super) fn calculate_window_start(
    time: DateTime<Utc>,
    period: KlinePeriod,
) -> DateTime<Utc> {
    match period {
        KlinePeriod::OneMinute => time
            .with_second(0)
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(time),
        KlinePeriod::FiveMinutes => {
            let window_min = (time.minute() / 5) * 5;
            time.with_minute(window_min)
                .and_then(|t| t.with_second(0))
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(time)
        }
        KlinePeriod::FifteenMinutes => {
            let window_min = (time.minute() / 15) * 15;
            time.with_minute(window_min)
                .and_then(|t| t.with_second(0))
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(time)
        }
        KlinePeriod::ThirtyMinutes => {
            let window_min = (time.minute() / 30) * 30;
            time.with_minute(window_min)
                .and_then(|t| t.with_second(0))
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(time)
        }
        KlinePeriod::OneHour => {
            let window_min = (time.minute() / 60) * 60;
            time.with_minute(window_min)
                .and_then(|t| t.with_second(0))
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(time)
        }
        KlinePeriod::Daily => time
            .with_hour(9)
            .and_then(|t| t.with_minute(30))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(time),
    }
}

pub(super) fn should_retain_window(
    current_time: DateTime<Utc>,
    last_update: DateTime<Utc>,
) -> bool {
    let elapsed = current_time.signed_duration_since(last_update).num_seconds();
    elapsed < 7200
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_timestamp_or_uses_parsed_time_or_fallback() {
        let fallback = DateTime::parse_from_rfc3339("2026-01-02T10:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let valid_timestamp = u64::try_from(fallback.timestamp()).unwrap();

        let valid = quote_timestamp_or(valid_timestamp, fallback);
        let invalid = quote_timestamp_or(u64::MAX, fallback);

        assert_eq!(valid, fallback);
        assert_eq!(invalid, fallback);
    }

    #[test]
    fn resolved_high_low_falls_back_to_open_for_uninitialized_bounds() {
        assert_eq!(resolved_high_low(10.5, f64::MIN, f64::MAX), (10.5, 10.5));
        assert_eq!(resolved_high_low(10.5, 11.2, 9.8), (11.2, 9.8));
    }
}
