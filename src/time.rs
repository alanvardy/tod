use chrono::offset::{TimeZone, Utc};
use chrono::DateTime;
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
pub fn format_date(date: &str) -> String {
    if *date == today() {
        String::from("Today")
    } else if date.len() == 10 {
        String::from(date)
    } else {
        datetime_from_str(date).to_string()
    }
}

/// Parse DateTime
pub fn datetime_from_str(date: &str) -> DateTime<Tz> {
    match date.len() {
        19 => Pacific
            .datetime_from_str(date, "%Y-%m-%dT%H:%M:%S")
            .expect("could not parse DateTime"),
        20 => Utc
            .datetime_from_str(date, "%Y-%m-%dT%H:%M:%SZ")
            .expect("could not parse DateTime")
            .with_timezone(&Pacific),
        _ => panic!("cannot parse datetime: {}", date),
    }
}

pub fn now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Pacific)
}
