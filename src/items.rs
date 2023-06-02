use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use clap::ArgMatches;
use colored::*;
use inquire::Select;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt::Display;

use crate::config::Config;
use crate::{items, request, time};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub id: String,
    pub content: String,
    pub priority: Priority,
    pub description: String,
    pub due: Option<DateInfo>,
    /// Only on rest api return value
    pub is_completed: Option<bool>,
    pub is_deleted: Option<bool>,
    /// only on sync api return value
    pub checked: Option<bool>,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Priority {
    None = 1,
    Low = 2,
    Medium = 3,
    High = 4,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::None => write!(f, "NONE"),
            Priority::Low => write!(f, "LOW"),
            Priority::Medium => write!(f, "MEDIUM"),
            Priority::High => write!(f, "HIGH"),
        }
    }
}

impl Priority {
    pub fn to_integer(&self) -> u8 {
        match self {
            Priority::None => 4,
            Priority::Low => 3,
            Priority::Medium => 2,
            Priority::High => 1,
        }
    }

    pub fn get_from_matches(matches: &ArgMatches) -> Option<Self> {
        let priority_arg = &matches.get_one::<String>("priority").map(|s| s.to_owned());
        match priority_arg {
            None => None,
            Some(priority) => serde_json::from_str(priority).ok(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DateInfo {
    pub date: String,
    pub is_recurring: bool,
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    items: Vec<Item>,
}

pub enum FormatType {
    List,
    Single,
}

enum DateTimeInfo {
    NoDateTime,
    Date {
        date: NaiveDate,
        is_recurring: bool,
    },
    DateTime {
        datetime: DateTime<Tz>,
        is_recurring: bool,
    },
}

impl Item {
    pub fn fmt(&self, config: &Config, format: FormatType) -> String {
        let content = match self.priority {
            Priority::Low => self.content.blue(),
            Priority::Medium => self.content.yellow(),
            Priority::High => self.content.red(),
            Priority::None => self.content.normal(),
        };

        let buffer = match format {
            FormatType::List => String::from("  "),
            FormatType::Single => String::from(""),
        };

        let description = match &*self.description {
            "" => String::from(""),
            _ => format!("\n{buffer}{}", self.description),
        };
        let due = match &self.datetimeinfo(config) {
            Ok(DateTimeInfo::Date { date, is_recurring }) => {
                let recurring_icon = if *is_recurring { " ↻" } else { "" };
                let date_string = time::format_date(date, config);

                format!("\n{buffer}Due: {date_string}{recurring_icon}")
            }
            Ok(DateTimeInfo::DateTime {
                datetime,
                is_recurring,
            }) => {
                let recurring_icon = if *is_recurring { " ↻" } else { "" };
                let datetime_string = time::format_datetime(datetime, config);

                format!("\n{buffer}Due: {datetime_string}{recurring_icon}")
            }
            Ok(DateTimeInfo::NoDateTime) => String::from(""),
            Err(string) => string.clone(),
        };

        let prefix = match format {
            FormatType::List => String::from("- "),
            FormatType::Single => String::from(""),
        };
        format!("{prefix}{content}{description}{due}")
    }

    /// Determines the numeric value of an item for sorting
    fn value(&self, config: &Config) -> u32 {
        let date_value: u8 = self.date_value(config);
        let priority_value: u8 = self.priority_value();

        date_value as u32 + priority_value as u32
    }

    /// Return the value of the due field
    fn date_value(&self, config: &Config) -> u8 {
        match &self.datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => 80,
            Ok(DateTimeInfo::Date { date, is_recurring }) => {
                let today_value = if *date == time::today_date(config) {
                    100
                } else {
                    0
                };
                let overdue_value = if self.is_overdue(config) { 150 } else { 0 };
                let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
                today_value + overdue_value + recurring_value
            }
            Ok(DateTimeInfo::DateTime {
                datetime,
                is_recurring,
            }) => {
                let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
                let duration = *datetime - time::now(config);
                match duration.num_minutes() {
                    -15..=15 => 200 + recurring_value,
                    _ => recurring_value,
                }
            }
            Err(_) => 50,
        }
    }

    /// Return the value of the due field
    fn datetime(&self, config: &Config) -> Option<DateTime<Tz>> {
        match self.datetimeinfo(config) {
            Ok(DateTimeInfo::DateTime { datetime, .. }) => Some(datetime),
            _ => None,
        }
    }

    fn priority_value(&self) -> u8 {
        match &self.priority {
            Priority::None => 2,
            Priority::Low => 1,
            Priority::Medium => 3,
            Priority::High => 4,
        }
    }

    /// Converts the JSON date representation into Date or Datetime
    fn datetimeinfo(&self, config: &Config) -> Result<DateTimeInfo, String> {
        let tz = match (self.clone().due, config.clone().timezone) {
            (None, Some(tz_string)) => time::timezone_from_str(&Some(tz_string)),
            (None, None) => Tz::UTC,
            (Some(DateInfo { timezone: None, .. }), Some(tz_string)) => time::timezone_from_str(&Some(tz_string)),
            (Some(DateInfo { timezone: None, .. }), None) => Tz::UTC,
            (Some(DateInfo {
                timezone: Some(tz_string),
                ..
                // Remove the Some here
            }), _) => time::timezone_from_str(&Some(tz_string)),
        };
        match self.clone().due {
            None => Ok(DateTimeInfo::NoDateTime),
            Some(DateInfo {
                date, is_recurring, ..
            }) if date.len() == 10 => Ok(DateTimeInfo::Date {
                date: time::date_from_str(&date, tz)?,
                is_recurring,
            }),
            Some(DateInfo {
                date, is_recurring, ..
            }) => Ok(DateTimeInfo::DateTime {
                datetime: time::datetime_from_str(&date, tz)?,
                is_recurring,
            }),
        }
    }

    pub fn has_no_date(&self) -> bool {
        self.due.is_none()
    }

    // Returns true if the datetime is today and there is a time
    fn is_today(&self, config: &Config) -> bool {
        match self.datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => false,
            Ok(DateTimeInfo::Date { date, .. }) => date == time::today_date(config),
            Ok(DateTimeInfo::DateTime { datetime, .. }) => {
                time::datetime_is_today(datetime, config)
            }
            Err(_) => false,
        }
    }

    pub fn is_overdue(&self, config: &Config) -> bool {
        match self.clone().datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => false,
            Ok(DateTimeInfo::Date { date, .. }) => time::is_date_in_past(date, config),
            Ok(DateTimeInfo::DateTime { datetime, .. }) => {
                time::is_date_in_past(datetime.date_naive(), config)
            }
            Err(_) => false,
        }
    }

    /// Returns true when it is a datetime, otherwise false
    fn has_time(&self, config: &Config) -> bool {
        matches!(
            self.clone().datetimeinfo(config),
            Ok(DateTimeInfo::DateTime { .. })
        )
    }
}

pub fn json_to_items(json: String) -> Result<Vec<Item>, String> {
    let result: Result<Body, _> = serde_json::from_str(&json);
    match result {
        Ok(body) => Ok(body.items),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
}

pub fn json_to_item(json: String) -> Result<Item, String> {
    match serde_json::from_str(&json) {
        Ok(item) => Ok(item),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
}

pub fn sort_by_value(mut items: Vec<Item>, config: &Config) -> Vec<Item> {
    items.sort_by_key(|b| Reverse(b.value(config)));
    items
}

pub fn sort_by_datetime(mut items: Vec<Item>, config: &Config) -> Vec<Item> {
    items.sort_by_key(|i| i.datetime(config));
    items
}

pub fn filter_not_in_future(items: Vec<Item>, config: &Config) -> Result<Vec<Item>, String> {
    let items = items
        .into_iter()
        .filter(|item| item.is_today(config) || item.has_no_date() || item.is_overdue(config))
        .collect();

    Ok(items)
}

pub fn filter_today_and_has_time(items: Vec<Item>, config: &Config) -> Vec<Item> {
    items
        .into_iter()
        .filter(|item| item.is_today(config) && item.has_time(config))
        .collect()
}

pub fn set_priority(config: &Config, item: items::Item) {
    println!("{}", item.fmt(config, FormatType::Single));

    let options = vec![Priority::Low, Priority::Medium, Priority::High];
    let priority = Select::new(
        "Choose a priority that should be assigned to task:",
        options,
    )
    .prompt()
    .map_err(|e| e.to_string())
    .expect("Failed to create option list of priorities");

    let config = config.set_next_id(&item.id);
    match priority {
        Priority::Low => {
            request::update_item_priority(config, item, Priority::Low)
                .expect("could not set priority");
        }
        Priority::Medium => {
            request::update_item_priority(config, item, Priority::Medium)
                .expect("could not set priority");
        }
        Priority::High => {
            request::update_item_priority(config, item, Priority::High)
                .expect("could not set priority");
        }
        _ => println!("Not a valid input, please pick one of the options"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[test]
    fn date_value_can_handle_date() {
        let config = test::fixtures::config();
        // On another day
        assert_eq!(test::fixtures::item().date_value(&config), 50);

        // Recurring
        let item = Item {
            due: Some(DateInfo {
                is_recurring: true,
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };
        assert_eq!(item.date_value(&config), 0);

        // Overdue
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2001-11-13"),
                is_recurring: true,
                timezone: Some(String::from("America/Los_Angeles")),
            }),
            ..test::fixtures::item()
        };
        assert_eq!(item.date_value(&config), 150);

        // No date
        let item = Item { due: None, ..item };
        assert_eq!(item.date_value(&config), 80);
    }

    #[test]
    fn date_value_can_handle_datetime() {
        let config = test::fixtures::config();
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2021-02-27T19:41:56Z"),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };

        assert_eq!(item.date_value(&config), 50);
    }

    #[test]
    fn can_format_item_with_a_date() {
        let config = test::fixtures::config();
        let item = Item {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: String::from("2021-08-13"),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };

        let output = if test::helpers::supports_coloured_output() {
            "\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: 2021-08-13"
        } else {
            "Get gifts for the twins\nDue: 2021-08-13"
        };

        assert_eq!(format!("{}", item.fmt(&config, FormatType::Single)), output);
    }

    #[test]
    fn can_format_item_with_today() {
        let config = test::fixtures::config();
        let item = Item {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };

        let output = if test::helpers::supports_coloured_output() {
            "\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: Today"
        } else {
            "Get gifts for the twins\nDue: Today"
        };
        assert_eq!(format!("{}", item.fmt(&config, FormatType::Single)), output);
    }

    #[test]
    fn value_can_get_the_value_of_an_item() {
        let config = test::fixtures::config();
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };

        assert_matches!(item.datetime(&config), Some(DateTime { .. }));
    }

    #[test]
    fn datetime_works_with_date() {
        let config = test::fixtures::config();
        let item = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };

        assert_eq!(item.datetime(&config), None);
    }

    #[test]
    fn has_no_date_works() {
        let config = test::fixtures::config();
        let item = Item {
            due: None,
            ..test::fixtures::item()
        };

        assert!(item.has_no_date());

        let item_today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::fixtures::item().due.unwrap()
            }),
            ..test::fixtures::item()
        };
        assert!(!item_today.has_no_date());
    }

    #[test]
    fn has_time_works() {
        let config = test::fixtures::config();
        let item = Item {
            due: None,
            ..test::fixtures::item()
        };

        assert!(!item.has_time(&config));

        let item_with_date = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..item.clone()
        };
        assert!(!item_with_date.has_time(&config));

        let item_with_datetime = Item {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..item
        };
        assert!(item_with_datetime.has_time(&config));
    }

    #[test]
    fn is_today_works() {
        let config = test::fixtures::config();
        let item = Item {
            due: None,
            ..test::fixtures::item()
        };

        assert!(!item.is_today(&config));

        let item_today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::fixtures::item()
        };
        assert!(item_today.is_today(&config));

        let item_in_past = Item {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..test::fixtures::item()
        };
        assert!(!item_in_past.is_today(&config));
    }

    #[test]
    fn sort_by_value_works() {
        let config = test::fixtures::config();
        let today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::fixtures::item()
        };

        let today_recurring = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::fixtures::item()
        };

        let future = Item {
            due: Some(DateInfo {
                date: String::from("2035-12-12"),
                is_recurring: false,
                timezone: None,
            }),
            ..test::fixtures::item()
        };

        let input = vec![future.clone(), today_recurring.clone(), today.clone()];
        let result = vec![today, today_recurring, future];

        assert_eq!(sort_by_value(input, &config), result);
    }

    #[test]
    fn sort_by_datetime_works() {
        let config = test::fixtures::config();
        let no_date = Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            description: String::from(""),
            due: None,
            priority: Priority::Medium,
            is_deleted: None,
            is_completed: None,
        };

        let date_not_datetime = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..no_date.clone()
        };

        let present = Item {
            due: Some(DateInfo {
                date: String::from("2020-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..no_date.clone()
        };

        let future = Item {
            due: Some(DateInfo {
                date: String::from("2035-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..no_date.clone()
        };

        let past = Item {
            due: Some(DateInfo {
                date: String::from("2015-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..no_date.clone()
        };

        let input = vec![
            future.clone(),
            past.clone(),
            present.clone(),
            no_date.clone(),
            date_not_datetime.clone(),
        ];
        let result = vec![no_date, date_not_datetime, past, present, future];

        assert_eq!(sort_by_datetime(input, &config), result);
    }

    #[test]
    fn is_overdue_works() {
        let config = test::fixtures::config();
        let item = Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            description: String::from(""),
            due: None,
            priority: Priority::Medium,
            is_deleted: None,
            is_completed: None,
        };

        assert!(!item.is_overdue(&config));

        let item_today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..item.clone()
        };
        assert!(!item_today.is_overdue(&config));

        let item_future = Item {
            due: Some(DateInfo {
                date: String::from("2035-12-12"),
                is_recurring: false,
                timezone: None,
            }),
            ..item.clone()
        };
        assert!(!item_future.is_overdue(&config));

        let item_today = Item {
            due: Some(DateInfo {
                date: String::from("2020-12-20"),
                is_recurring: false,
                timezone: None,
            }),
            ..item
        };
        assert!(item_today.is_overdue(&config));
    }

    #[test]
    fn json_to_items_works() {
        let json = String::from("2{.e");
        let error_text = String::from("Could not parse response for item: Error(\"invalid type: integer `2`, expected struct Body\", line: 1, column: 1)");
        assert_eq!(json_to_items(json), Err(error_text));
    }

    #[test]
    fn json_to_item_works() {
        let json = String::from("2{.e");
        let error_text = String::from("Could not parse response for item: Error(\"invalid type: integer `2`, expected struct Item\", line: 1, column: 1)");
        assert_eq!(json_to_item(json), Err(error_text));
    }

    #[test]
    fn test_to_integer() {
        assert_eq!(Priority::None.to_integer(), 4);
        assert_eq!(Priority::Low.to_integer(), 3);
        assert_eq!(Priority::Medium.to_integer(), 2);
        assert_eq!(Priority::High.to_integer(), 1);
    }
}
