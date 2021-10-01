use chrono::offset::{TimeZone, Utc};
use chrono::{Date, DateTime, NaiveDate};
use chrono_tz::Tz;
use chrono_tz::US::Pacific;

pub fn now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Pacific)
}

/// Return today's date in format 2021-09-16
pub fn today_string() -> String {
    now().format("%Y-%m-%d").to_string()
}

/// Return today's date in Utc
pub fn today_date() -> Date<Tz> {
    now().date()
}

/// Returns today or date
pub fn format_date(date: &str) -> String {
    if *date == today_string() {
        String::from("Today")
    } else if date.len() == 10 {
        String::from(date)
    } else if is_today(datetime_from_str(date)) {
        datetime_from_str(date).format("%H:%M").to_string()
    } else {
        datetime_from_str(date).to_string()
    }
}

fn is_today(datetime: DateTime<Tz>) -> bool {
    datetime.date().format("%Y-%m-%d").to_string() == today_string()
}

/// Parse DateTime
pub fn datetime_from_str(str: &str) -> DateTime<Tz> {
    match str.len() {
        19 => Pacific
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .expect("could not parse DateTime"),
        20 => Utc
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .expect("could not parse DateTime")
            .with_timezone(&Pacific),
        _ => panic!("cannot parse DateTime: {}", str),
    }
}

/// Parse Date
pub fn date_from_str(str: &str) -> Result<Date<Utc>, String> {
    let date = match str.len() {
        10 => {
            let date =
                NaiveDate::parse_from_str(str, "%Y-%m-%d").or(Err("could not parse Date"))?;
            Utc.from_local_date(&date).unwrap()
        }
        19 => Utc
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%S")
            .or(Err("could not parse DateTime"))?
            .date(),

        20 => Utc
            .datetime_from_str(str, "%Y-%m-%dT%H:%M:%SZ")
            .or(Err("could not parse DateTime"))?
            .date(),
        _ => return Err(format!("cannot parse NaiveDate, unknown length: {}", str)),
    };

    Ok(date)
}
