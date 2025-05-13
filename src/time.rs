use crate::config::Config;
use crate::errors::{self, Error};

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Tz;
use regex::Regex;
use std::str::FromStr;

pub const FORMAT_DATE: &str = "%Y-%m-%d";
const FORMAT_TIME: &str = "%H:%M";
const FORMAT_DATETIME: &str = "%Y-%m-%dT%H:%M:%S";
const FORMAT_DATETIME_ZULU: &str = "%Y-%m-%dT%H:%M:%SZ";
const FORMAT_DATETIME_LONG: &str = "%Y-%m-%dT%H:%M:%S%.fZ";

pub const FORMAT_DATE_AND_TIME: &str = "%Y-%m-%d %H:%M";

#[cfg(test)] //Fixed Time Provider for Testing
use crate::test_time::FixedTimeProvider;

/// Enum for selecting Time Provider
#[derive(Clone, Debug)]
pub enum TimeProviderEnum {
    // Default to System Time Provider
    System(SystemTimeProvider),
    /// Fixed time provider for testing
    #[cfg(test)]
    Fixed(FixedTimeProvider),
}
//Default to SystemTimeProvider for TimeProviderEnum
impl Default for TimeProviderEnum {
    fn default() -> Self {
        TimeProviderEnum::System(SystemTimeProvider)
    }
}

impl TimeProvider for TimeProviderEnum {
    fn now(&self, tz: Tz) -> DateTime<Tz> {
        match self {
            TimeProviderEnum::System(provider) => provider.now(tz),
            #[cfg(test)]
            TimeProviderEnum::Fixed(provider) => provider.now(tz),
        }
    }
    fn today(&self, tz: Tz) -> NaiveDate {
        match self {
            TimeProviderEnum::System(provider) => provider.today(tz),
            #[cfg(test)]
            TimeProviderEnum::Fixed(provider) => provider.today(tz),
        }
    }
}

/// Trait for Time Provider
pub trait TimeProvider: Send + Sync + Clone {
    fn now(&self, tz: Tz) -> DateTime<Tz>;
    fn today(&self, tz: Tz) -> NaiveDate {
        self.now(tz).date_naive()
    }
    //Only currently used for testing
    #[cfg(test)]
    fn now_string(&self, tz: Tz) -> String {
        self.now(tz).to_rfc3339()
    }
}

#[derive(Clone, Debug)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now(&self, tz: Tz) -> DateTime<Tz> {
        Utc::now().with_timezone(&tz)
    }
}
/// Returns the current time in the given timezone
/// If no timezone is given, it defaults to UTC
pub fn now(config: &Config) -> Result<DateTime<Tz>, Error> {
    let tz = timezone_from_str(&config.timezone)?;

    let provider = config.time_provider.clone();

    Ok(provider.now(tz))
}

/// Return today's date in format 2021-09-16
pub fn today_string(config: &Config) -> Result<String, Error> {
    let tz = timezone_from_str(&config.timezone)?;
    let today = config.time_provider.today(tz);

    Ok(today.format(FORMAT_DATE).to_string())
}

/// Return today's date in Utc
pub fn today_date(config: &Config) -> Result<NaiveDate, Error> {
    let tz = timezone_from_str(&config.timezone)?;
    Ok(config.time_provider.today(tz))
}

/// Returns today's date in given timezone for testing. Only used in tests currently but included for completeness.
#[allow(dead_code)]
pub fn today_date_from_tz(tz: Tz) -> Result<NaiveDate, Error> {
    Ok(chrono::Utc::now().with_timezone(&tz).date_naive())
}
// Checks if datetime is today
pub fn datetime_is_today(datetime: DateTime<Tz>, config: &Config) -> Result<bool, Error> {
    date_is_today(datetime.date_naive(), config)
}
// Check if date is today
pub fn date_is_today(date: NaiveDate, config: &Config) -> Result<bool, Error> {
    let date_string = date.format(FORMAT_DATE).to_string();
    let today_string = today_string(config)?;
    Ok(date_string == today_string)
}
// Converts a date string to a NaiveDate
pub fn date_string_to_naive_date(date_string: &str) -> Result<NaiveDate, Error> {
    let date = NaiveDate::from_str(date_string)?;
    Ok(date)
}
// / Check if date is in the past
pub fn is_date_in_past(date: NaiveDate, config: &Config) -> Result<bool, Error> {
    Ok(num_days_from_today(date, config)? < 0)
}

/// Returns 0 if today, negative if date given is in the past
pub fn num_days_from_today(date: NaiveDate, config: &Config) -> Result<i64, Error> {
    let duration = date.signed_duration_since(today_date(config)?);
    Ok(duration.num_days())
}
// Formats a date to a string
pub fn format_date(date: &NaiveDate, config: &Config) -> Result<String, Error> {
    if date_is_today(*date, config)? {
        Ok(String::from("Today"))
    } else {
        Ok(date.format(FORMAT_DATE).to_string())
    }
}
// Formats a datetime to a string
pub fn format_datetime(datetime: &DateTime<Tz>, config: &Config) -> Result<String, Error> {
    let tz = timezone_from_str(&config.timezone)?;
    if datetime_is_today(*datetime, config)? {
        Ok(datetime.with_timezone(&tz).format(FORMAT_TIME).to_string())
    } else {
        Ok(datetime.with_timezone(&tz).to_string())
    }
}

/// Parse DateTime
pub fn datetime_from_str(str: &str, timezone: Tz) -> Result<DateTime<Tz>, Error> {
    match str.len() {
        19 => parse_datetime_from_19(str, timezone),
        20 => parse_datetime_from_20(str),
        27 => parse_datetime_from_27(str),
        length => Err(Error {
            source: "datetime_from_str".to_string(),
            message: format!("cannot parse {length} length DateTime: {str}"),
        }),
    }
}

pub fn parse_datetime_from_19(str: &str, timezone: Tz) -> Result<DateTime<Tz>, Error> {
    let tz = NaiveDateTime::parse_from_str(str, FORMAT_DATETIME)?
        .and_local_timezone(timezone)
        .unwrap();
    Ok(tz)
}

pub fn parse_datetime_from_20(str: &str) -> Result<DateTime<Tz>, Error> {
    let tz = NaiveDateTime::parse_from_str(str, FORMAT_DATETIME_ZULU)?
        .and_local_timezone(Tz::UTC)
        .unwrap();
    Ok(tz)
}

// 2025-01-02T04:37:31.764000Z
pub fn parse_datetime_from_27(str: &str) -> Result<DateTime<Tz>, Error> {
    let tz = NaiveDateTime::parse_from_str(str, FORMAT_DATETIME_LONG)?
        .and_local_timezone(Tz::UTC)
        .unwrap();
    Ok(tz)
}

pub fn timezone_from_str(timezone_string: &Option<String>) -> Result<Tz, Error> {
    match timezone_string {
        None => Ok(Tz::UTC),
        Some(string) => match string.parse::<Tz>() {
            Ok(tz) => Ok(tz),
            Err(_) => parse_gmt_to_timezone(string),
        },
    }
}

/// For when we get offsets like GMT -7:00
fn parse_gmt_to_timezone(gmt: &str) -> Result<Tz, Error> {
    let split: Vec<&str> = gmt.split_whitespace().collect();
    let offset = split
        .get(1)
        .ok_or_else(|| errors::new("parse_timezone", "Could not get offset"))?;
    let offset = offset.replace(":00", "");
    let offset = offset.replace(':', "");
    let offset_num = offset.parse::<i32>()?;

    let tz_string = format!(
        "Etc/GMT{}",
        if offset_num < 0 {
            "+".to_string()
        } else {
            "-".to_string()
        } + &offset_num.abs().to_string()
    );
    tz_string.parse().map_err(Error::from)
}

/// Parse Date
pub fn date_from_str(str: &str, timezone: Tz) -> Result<NaiveDate, Error> {
    let date = match str.len() {
        10 => NaiveDate::parse_from_str(str, FORMAT_DATE)?,
        19 => NaiveDateTime::parse_from_str(str, FORMAT_DATETIME)?
            .and_local_timezone(timezone)
            .unwrap()
            .date_naive(),
        20 => NaiveDateTime::parse_from_str(str, FORMAT_DATETIME_ZULU)?
            .and_local_timezone(timezone)
            .unwrap()
            .date_naive(),
        _ => {
            return Err(errors::new(
                "date_from_str",
                "cannot parse NaiveDate, unknown length: {str}",
            ));
        }
    };

    Ok(date)
}

/// Checks if string is a date in format YYYY-MM-DD
pub fn is_date(string: &str) -> bool {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    re.is_match(string)
}

/// Checks if string is a datetime in format YYYY-MM-DD HH:MM
pub fn is_datetime(string: &str) -> bool {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$").unwrap();
    re.is_match(string)
}

#[cfg(test)]
mod tests {
    use crate::time;

    use super::*;
    use chrono::Datelike;
    use chrono_tz::Tz;

    #[test]
    fn test_is_date() {
        assert!(is_date("2022-10-05"));
        assert!(!is_date("22-10-05"));
        assert!(!is_date("2022-10-05 24:02"));
        assert!(!is_date("today"));
    }

    #[test]
    fn test_is_datetime() {
        assert!(!is_datetime("2022-10-05"));
        assert!(!is_datetime("22-10-05"));
        assert!(is_datetime("2022-10-05 24:02"));
        assert!(!is_datetime("today"));
    }

    #[test]
    fn test_timezone_from_string() {
        assert_eq!(
            timezone_from_str(&Some("America/Los_Angeles".to_string())),
            Ok(Tz::America__Los_Angeles),
        );

        assert_eq!(
            timezone_from_str(&Some("GMT -7:00".to_string())),
            Ok(Tz::Etc__GMTPlus7),
        );
    }

    #[test]
    fn test_today_date_from_tz_utc() {
        let tz = Tz::UTC;
        let result = today_date_from_tz(tz).unwrap();
        let expected = chrono::Utc::now().with_timezone(&tz).date_naive();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_today_date_from_tz_pacific() {
        let tz = Tz::America__Los_Angeles;
        let result = today_date_from_tz(tz).unwrap();
        let expected = chrono::Utc::now().with_timezone(&tz).date_naive();
        assert_eq!(result, expected);
    }

    #[test]
    fn trait_default_today_is_used() {
        let provider = SystemTimeProvider;
        let tz: Tz = "UTC".parse().unwrap();
        let expected = provider.now(tz).date_naive();
        let today = provider.today(tz);
        assert_eq!(today, expected);
    }

    #[tokio::test]
    async fn config_uses_default_system_time_provider() {
        let mut config = Config::new("test-token", None).await.unwrap();
        config.timezone = Some("UTC".to_string());
        let now = crate::time::now(&config).unwrap();
        assert!(now.year() >= 2024); // or any sanity check
    }

    #[test]
    fn fallback_to_utc_now_when_today_date_from_tz_fails() {
        let tz: Tz = chrono_tz::UTC;

        let result = time::today_date_from_tz(tz)
            .unwrap_or_else(|_| Utc::now().with_timezone(&tz).date_naive());

        let expected = Utc::now().with_timezone(&tz).date_naive();

        // Allow for edge-of-day differences
        assert!(
            result == expected || (result - expected).num_days().abs() <= 1,
            "Got {}, expected ~{}",
            result,
            expected
        );
    }
    #[tokio::test]
    async fn test_default_config_uses_system_time_provider() {
        // Create a default config
        let config = Config::default();

        // Parse a timezone (e.g., UTC)
        let tz: Tz = "UTC".parse().unwrap();

        // Call the `today` method via the `time_provider`
        let today_from_provider = config.time_provider.today(tz);

        // Get today's date directly from `SystemTimeProvider` for comparison
        let system_provider = SystemTimeProvider;
        let today_from_system = system_provider.today(tz);

        // Assert that the `time_provider` in the default config behaves like `SystemTimeProvider`
        assert_eq!(today_from_provider, today_from_system)
    }
}
