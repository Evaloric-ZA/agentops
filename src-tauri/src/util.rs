//! Small shared helpers.

use chrono::{DateTime, SecondsFormat, Utc};

/// The current instant as an RFC 3339 UTC string.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Format a timestamp as RFC 3339 UTC, truncating to whole seconds.
pub fn to_rfc3339(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Parse an RFC 3339 timestamp. Returns `None` when the input is not a
/// valid timestamp, so a malformed record never crashes a read path.
pub fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Seconds between two RFC 3339 timestamps, or `None` if either is invalid.
pub fn seconds_between(start: &str, end: &str) -> Option<i64> {
    let a = parse_rfc3339(start)?;
    let b = parse_rfc3339(end)?;
    Some((b - a).num_seconds())
}

/// Round a duration to whole seconds, never negative.
pub fn duration_seconds(start: &str, end: &str) -> i64 {
    seconds_between(start, end).unwrap_or(0).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_timestamp_round_trips() {
        let raw = "2026-01-01T00:00:00Z";
        let parsed = parse_rfc3339(raw).expect("valid");
        assert_eq!(to_rfc3339(parsed), raw);
    }

    #[test]
    fn a_malformed_timestamp_is_none_not_a_panic() {
        assert!(parse_rfc3339("not-a-time").is_none());
        assert!(parse_rfc3339("").is_none());
    }

    #[test]
    fn a_negative_duration_clamps_to_zero() {
        assert_eq!(
            duration_seconds("2026-01-01T00:01:00Z", "2026-01-01T00:00:00Z"),
            0
        );
    }
}
