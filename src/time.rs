use chrono::offset::Utc;
use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use chrono_tz::US::Pacific;

/// Return today's date in format 2021-09-16
pub fn today() -> String {
    Utc::now()
        .with_timezone(&Pacific)
        .format("%Y-%m-%d")
        .to_string()
}

/// Returns today or date
pub fn maybe_today(date: &str) -> String {
    if *date == today() {
        String::from("Today")
    } else {
        String::from(date)
    }
}

/// Parse DateTime
pub fn datetime_from_str(date: &str) -> DateTime<Tz> {
    let format = match date.len() {
        19 => "%Y-%m-%dT%H:%M:%S",
        _ => "%Y-%m-%dT%H:%M:%SZ",
    };
    Pacific
        .datetime_from_str(date, format)
        .expect("could not parse DateTime")
}

pub fn now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Pacific)
}
