use crate::config::Config;
use chrono::offset::{TimeZone, Utc};
use chrono::{DateTime, NaiveDate};
use chrono_tz::{Tz, TZ_VARIANTS};

pub fn now(config: &Config) -> DateTime<Tz> {
    let tz = timezone_from_str(&config.timezone);
    Utc::now().with_timezone(&tz)
}

/// Return today's date in format 2021-09-16
pub fn today_string(config: &Config) -> String {
    now(config).format("%Y-%m-%d").to_string()
}

/// Return today's date in Utc
pub fn today_date(config: &Config) -> NaiveDate {
    now(config).date_naive()
}

pub fn datetime_is_today(datetime: DateTime<Tz>, config: &Config) -> bool {
    date_is_today(datetime.date_naive(), config)
}

pub fn date_is_today(date: NaiveDate, config: &Config) -> bool {
    date.format("%Y-%m-%d").to_string() == today_string(config)
}

pub fn is_date_in_past(date: NaiveDate, config: &Config) -> bool {
    date.signed_duration_since(today_date(config)).num_days() < 0
}

pub fn format_date(date: &NaiveDate, config: &Config) -> String {
    if date_is_today(*date, config) {
        String::from("Today")
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

pub fn format_datetime(datetime: &DateTime<Tz>, config: &Config) -> String {
    let tz = timezone_from_str(&config.timezone);
    if datetime_is_today(*datetime, config) {
        datetime.with_timezone(&tz).format("%H:%M").to_string()
    } else {
        datetime.with_timezone(&tz).to_string()
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
        Some(string) => string.parse::<Tz>().unwrap(),
    }
}

/// Parse Date
pub fn date_from_str(str: &str, timezone: Tz) -> Result<NaiveDate, String> {
    let date = match str.len() {
        10 => NaiveDate::parse_from_str(str, "%Y-%m-%d").or(Err("could not parse Date"))?,
        19 => timezone
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .or(Err("could not parse DateTime"))?
            .date_naive(),

        20 => timezone
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .or(Err("could not parse DateTime"))?
            .date_naive(),
        _ => return Err(format!("cannot parse NaiveDate, unknown length: {}", str)),
    };

    Ok(date)
}

pub fn list_timezones() {
    println!("Timezones:");
    for (num, tz) in TZ_VARIANTS.iter().enumerate() {
        println!("{}: {}", num, tz);
    }
}

pub fn get_timezone(num: usize) -> String {
    TZ_VARIANTS[num].to_string()
}
