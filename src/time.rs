use crate::config::Config;
use crate::error::{self, Error};
use chrono::offset::Utc;
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use chrono_tz::Tz;
use regex::Regex;

pub fn now(config: &Config) -> Result<DateTime<Tz>, Error> {
    let tz = timezone_from_str(&config.timezone)?;
    Ok(Utc::now().with_timezone(&tz))
}

/// Return today's date in format 2021-09-16
pub fn today_string(config: &Config) -> Result<String, Error> {
    Ok(now(config)?.format("%Y-%m-%d").to_string())
}

/// Return today's date in Utc
pub fn today_date(config: &Config) -> Result<NaiveDate, Error> {
    Ok(now(config)?.date_naive())
}

pub fn datetime_is_today(datetime: DateTime<Tz>, config: &Config) -> Result<bool, Error> {
    date_is_today(datetime.date_naive(), config)
}

pub fn date_is_today(date: NaiveDate, config: &Config) -> Result<bool, Error> {
    let date_string = date.format("%Y-%m-%d").to_string();
    let today_string = today_string(config)?;
    Ok(date_string == today_string)
}

pub fn is_date_in_past(date: NaiveDate, config: &Config) -> Result<bool, Error> {
    let num_days = date.signed_duration_since(today_date(config)?).num_days();
    Ok(num_days < 0)
}

pub fn format_date(date: &NaiveDate, config: &Config) -> Result<String, Error> {
    if date_is_today(*date, config)? {
        Ok(String::from("Today"))
    } else {
        Ok(date.format("%Y-%m-%d").to_string())
    }
}

pub fn format_datetime(datetime: &DateTime<Tz>, config: &Config) -> Result<String, Error> {
    let tz = timezone_from_str(&config.timezone)?;
    if datetime_is_today(*datetime, config)? {
        Ok(datetime.with_timezone(&tz).format("%H:%M").to_string())
    } else {
        Ok(datetime.with_timezone(&tz).to_string())
    }
}

/// Parse DateTime
pub fn datetime_from_str(str: &str, timezone: Tz) -> Result<DateTime<Tz>, Error> {
    let datetime = match str.len() {
        19 => NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .expect("could not parse DateTime")
            .and_local_timezone(timezone),
        20 => NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .expect("could not parse DateTime")
            .and_local_timezone(Tz::UTC),
        _ => {
            return Err(error::new(
                "datetime_from_str",
                "cannot parse DateTime: {str}",
            ))
        }
    };

    Ok(datetime.unwrap())
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
        .ok_or_else(|| error::new("parse_timezone", "Could not get offset"))?;
    let offset = offset.replace(":00", "");
    let offset = offset.replace(':', "");
    let offset_num = offset.parse::<i32>().unwrap();

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
        // 19 => NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%S")
        //     .expect("could not parse DateTime")
        //     .and_local_timezone(timezone),
        10 => NaiveDate::parse_from_str(str, "%Y-%m-%d")?,
        19 => NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%S")?
            .and_local_timezone(timezone)
            .unwrap()
            .date_naive(),
        20 => NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%SZ")?
            .and_local_timezone(timezone)
            .unwrap()
            .date_naive(),
        _ => {
            return Err(error::new(
                "date_from_str",
                "cannot parse NaiveDate, unknown length: {str}",
            ))
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
    use super::*;

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
}
