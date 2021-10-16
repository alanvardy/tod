use chrono::offset::{TimeZone, Utc};
use chrono::{Date, DateTime, NaiveDate};
use chrono_tz::Tz;

pub fn now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Tz::UTC)
}

/// Return today's date in format 2021-09-16
pub fn today_string() -> String {
    now().format("%Y-%m-%d").to_string()
}

/// Return today's date in Utc
pub fn today_date() -> Date<Tz> {
    now().date()
}

pub fn datetime_is_today(datetime: DateTime<Tz>) -> bool {
    date_is_today(datetime.date())
}

pub fn date_is_today(date: Date<Tz>) -> bool {
    date.format("%Y-%m-%d").to_string() == today_string()
}

pub fn is_date_in_past(date: Date<Tz>) -> bool {
    date.signed_duration_since(today_date()).num_days() < 0
}

pub fn format_date(date: &Date<Tz>) -> String {
    if date_is_today(*date) {
        String::from("Today")
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

pub fn format_datetime(datetime: &DateTime<Tz>) -> String {
    if datetime_is_today(*datetime) {
        datetime.format("%H:%M").to_string()
    } else {
        datetime.to_string()
    }
}

/// Parse DateTime
pub fn datetime_from_str(str: &str, timezone: Tz) -> Result<DateTime<Tz>, String> {
    let datetime = match str.len() {
        19 => timezone
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .expect("could not parse DateTime"),
        20 => Utc
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .expect("could not parse DateTime")
            .with_timezone(&Tz::UTC),
        _ => return Err(format!("cannot parse DateTime: {}", str)),
    };

    Ok(datetime)
}

pub fn timezone_from_str(timezone_string: &Option<String>) -> Tz {
    match timezone_string {
        None => Tz::UTC,
        Some(string) => {
            let tz: Tz = string.parse().unwrap();
            tz
        }
    }
}

/// Parse Date
pub fn date_from_str(str: &str, timezone: Tz) -> Result<Date<Tz>, String> {
    let date = match str.len() {
        10 => {
            let date =
                NaiveDate::parse_from_str(str, "%Y-%m-%d").or(Err("could not parse Date"))?;
            timezone.from_local_date(&date).unwrap()
        }
        19 => timezone
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .or(Err("could not parse DateTime"))?
            .date(),

        20 => timezone
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .or(Err("could not parse DateTime"))?
            .date(),
        _ => return Err(format!("cannot parse NaiveDate, unknown length: {}", str)),
    };

    Ok(date)
}
