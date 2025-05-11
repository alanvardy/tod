// Used for testing where a fixed time is needed for the "now" and "today" functions.
// Returns a fixed time (10:28 AM UTC) for "today", using the system's local date.
// This allos us to have a consistent time for testing, regardless of the system's current time.

#![cfg(test)]

// src/test_time.rs
use crate::time::TimeProvider;
use chrono::{DateTime, NaiveDate, TimeZone};
use chrono_tz::Tz;

/// A fixed time provider for testing purposes.
/// This provider returns a fixed date and time (2025-05-10 10:28:00)
pub struct FixedTimeProvider;

impl TimeProvider for FixedTimeProvider {
    fn now(&self, tz: Tz) -> DateTime<Tz> {
        let dt_utc = chrono::NaiveDate::from_ymd_opt(2025, 5, 10)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        tz.from_utc_datetime(&dt_utc)
    }

    fn today(&self, tz: Tz) -> NaiveDate {
        self.now(tz).date_naive() // Guarantees alignment
    }
}
/// A fixed UTC timestamp string (corresponding to 2025-05-10 10:00 AM America/Vancouver)
pub fn fixed_test_utc_string() -> String {
    let tz = chrono_tz::UTC;
    let dt = tz.with_ymd_and_hms(2025, 5, 10, 10, 0, 0).unwrap();
    dt.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use crate::time::TimeProvider;

    use super::*;

    #[test]
    fn fixed_time_provider_returns_fixed_utc_time() {
        let tz = chrono_tz::UTC;
        let provider = FixedTimeProvider;
        let fixed_time = provider.now(tz);

        // This is what the fixed time is set to in fixed_test_utc_string
        let expected = tz.with_ymd_and_hms(2025, 5, 10, 10, 0, 0).unwrap();

        // Make sure the datetime is exactly as expected
        assert_eq!(fixed_time, expected);
        assert_eq!(fixed_time.date_naive(), expected.date_naive()); // This is the line that failed before
    }

    #[test]
    fn fixed_test_utc_string_outputs_expected_format() {
        let expected = "2025-05-10T10:00:00+00:00";
        assert_eq!(fixed_test_utc_string(), expected);
    }
}
