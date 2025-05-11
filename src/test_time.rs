// Used for testing where a fixed time is needed for the "now" and "today" functions.
// Returns a fixed time (10:28 AM UTC) for "today", using the system's local date.
// This allos us to have a consistent time for testing, regardless of the system's current time.

#![cfg(test)]

use crate::time;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

pub struct FixedTimeProvider;

impl time::TimeProvider for FixedTimeProvider {
    fn now(&self, tz: Tz) -> DateTime<Tz> {
        let today = time::today_date_from_tz(tz).unwrap_or_else(|_| Utc::now().date_naive());
        today
            .and_hms_opt(10, 28, 0)
            .unwrap()
            .and_local_timezone(tz)
            .unwrap()
    }

    fn today(&self, tz: Tz) -> NaiveDate {
        time::today_date_from_tz(tz).unwrap_or_else(|_| Utc::now().date_naive())
    }
}
#[cfg(test)]
mod tests {
    use crate::time::TimeProvider;

    use super::*;
    use chrono::Timelike;

    #[test]
    fn fixed_time_provider_returns_fixed_today() {
        let tz = chrono_tz::UTC;
        let fixed = FixedTimeProvider;

        let now = fixed.now(tz);
        let today = fixed.today(tz);

        assert_eq!(now.date_naive(), today);
        assert_eq!(now.time().hour(), 10);
        assert_eq!(now.time().minute(), 28);
        assert_eq!(now.time().second(), 0);
    }
}
