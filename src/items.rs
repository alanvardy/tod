use chrono::Date;
use chrono::DateTime;
use chrono_tz::Tz;
use colored::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

use crate::config::Config;
use crate::{config, items, request, time};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub id: String,
    pub content: String,
    pub priority: u8,
    pub checked: bool,
    pub description: String,
    pub due: Option<DateInfo>,
    pub is_deleted: bool,
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

enum DateTimeInfo {
    NoDateTime,
    Date {
        date: Date<Tz>,
        is_recurring: bool,
    },
    DateTime {
        datetime: DateTime<Tz>,
        is_recurring: bool,
    },
}

impl Item {
    pub fn fmt(&self, config: &Config) -> String {
        let content = match self.priority {
            2 => self.content.blue(),
            3 => self.content.yellow(),
            4 => self.content.red(),
            _ => self.content.normal(),
        };

        let description = match &*self.description {
            "" => String::from(""),
            _ => format!("\n{}", self.description),
        };

        let due = match &self.datetimeinfo(config) {
            Ok(DateTimeInfo::Date { date, is_recurring }) => {
                let recurring_icon = if *is_recurring { " ↻" } else { "" };
                let date_string = time::format_date(date, config);

                format!("\nDue: {}{}", date_string, recurring_icon)
            }
            Ok(DateTimeInfo::DateTime {
                datetime,
                is_recurring,
            }) => {
                let recurring_icon = if *is_recurring { " ↻" } else { "" };
                let datetime_string = time::format_datetime(datetime, config);

                format!("\nDue: {}{}", datetime_string, recurring_icon)
            }
            Ok(DateTimeInfo::NoDateTime) => String::from(""),
            Err(string) => string.clone(),
        };

        format!("\n{}{}{}", content, description, due)
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
        match self.priority {
            2 => 1,
            3 => 3,
            4 => 4,
            _ => 2,
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

    fn has_no_date(&self) -> bool {
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

    fn is_overdue(&self, config: &Config) -> bool {
        match self.clone().datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => false,
            Ok(DateTimeInfo::Date { date, .. }) => time::is_date_in_past(date, config),
            Ok(DateTimeInfo::DateTime { datetime, .. }) => {
                time::is_date_in_past(datetime.date(), config)
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
        Err(err) => Err(format!("Could not parse response for item: {:?}", err)),
    }
}

pub fn json_to_item(json: String) -> Result<Item, String> {
    match serde_json::from_str(&json) {
        Ok(item) => Ok(item),
        Err(err) => Err(format!("Could not parse response for item: {:?}", err)),
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

pub fn set_priority(config: Config, item: items::Item) {
    println!("{}", item.fmt(&config));

    let priority = config::get_input("Choose a priority from 1 (lowest) to 3 (highest):")
        .expect("Please enter a number from 1 to 3");

    match priority.as_str() {
        "1" => {
            let config = config.set_next_id(item.id.clone());
            request::update_item_priority(config, item, 2).expect("could not set priority");
        }
        "2" => {
            let config = config.set_next_id(item.id.clone());
            request::update_item_priority(config, item, 3).expect("could not set priority");
        }
        "3" => {
            let config = config.set_next_id(item.id.clone());
            request::update_item_priority(config, item, 4).expect("could not set priority");
        }
        _ => println!("Not a valid input, please enter 1, 2, or 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[test]
    fn date_value_can_handle_date() {
        let config = test::helpers::config_fixture();
        // On another day
        assert_eq!(test::helpers::item_fixture().date_value(&config), 50);

        // Recurring
        let item = Item {
            due: Some(DateInfo {
                is_recurring: true,
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };
        assert_eq!(item.date_value(&config), 0);

        // Overdue
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2001-11-13"),
                is_recurring: true,
                timezone: Some(String::from("America/Los_Angeles")),
            }),
            ..test::helpers::item_fixture()
        };
        assert_eq!(item.date_value(&config), 150);

        // No date
        let item = Item { due: None, ..item };
        assert_eq!(item.date_value(&config), 80);
    }

    #[test]
    fn date_value_can_handle_datetime() {
        let config = test::helpers::config_fixture();
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2021-02-27T19:41:56Z"),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };

        assert_eq!(item.date_value(&config), 50);
    }

    #[test]
    fn can_format_item_with_a_date() {
        let config = test::helpers::config_fixture();
        let item = Item {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: String::from("2021-08-13"),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };

        let output = if test::helpers::supports_coloured_output() {
            "\n\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: 2021-08-13"
        } else {
            "\nGet gifts for the twins\nDue: 2021-08-13"
        };

        assert_eq!(format!("{}", item.fmt(&config)), output);
    }

    #[test]
    fn can_format_item_with_today() {
        let config = test::helpers::config_fixture();
        let item = Item {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };

        let output = if test::helpers::supports_coloured_output() {
            "\n\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: Today"
        } else {
            "\nGet gifts for the twins\nDue: Today"
        };
        assert_eq!(format!("{}", item.fmt(&config)), output);
    }

    #[test]
    fn value_can_get_the_value_of_an_item() {
        let config = test::helpers::config_fixture();
        let item = test::helpers::item_fixture();

        assert_eq!(item.value(&config), 53);
    }

    #[test]
    fn datetime_works_with_datetime() {
        let config = test::helpers::config_fixture();
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };

        assert_matches!(item.datetime(&config), Some(DateTime { .. }));
    }

    #[test]
    fn datetime_works_with_date() {
        let config = test::helpers::config_fixture();
        let item = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };

        assert_eq!(item.datetime(&config), None);
    }

    #[test]
    fn has_no_date_works() {
        let config = test::helpers::config_fixture();
        let item = Item {
            due: None,
            ..test::helpers::item_fixture()
        };

        assert!(item.has_no_date());

        let item_today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                ..test::helpers::item_fixture().due.unwrap()
            }),
            ..test::helpers::item_fixture()
        };
        assert!(!item_today.has_no_date());
    }

    #[test]
    fn has_time_works() {
        let config = test::helpers::config_fixture();
        let item = Item {
            due: None,
            ..test::helpers::item_fixture()
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
        let config = test::helpers::config_fixture();
        let item = Item {
            due: None,
            ..test::helpers::item_fixture()
        };

        assert!(!item.is_today(&config));

        let item_today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::helpers::item_fixture()
        };
        assert!(item_today.is_today(&config));

        let item_in_past = Item {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
            }),
            ..test::helpers::item_fixture()
        };
        assert!(!item_in_past.is_today(&config));
    }

    #[test]
    fn sort_by_value_works() {
        let config = test::helpers::config_fixture();
        let today = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::helpers::item_fixture()
        };

        let today_recurring = Item {
            due: Some(DateInfo {
                date: time::today_string(&config),
                is_recurring: false,
                timezone: None,
            }),
            ..test::helpers::item_fixture()
        };

        let future = Item {
            due: Some(DateInfo {
                date: String::from("2035-12-12"),
                is_recurring: false,
                timezone: None,
            }),
            ..test::helpers::item_fixture()
        };

        let input = vec![future.clone(), today_recurring.clone(), today.clone()];
        let result = vec![today, today_recurring, future];

        assert_eq!(sort_by_value(input, &config), result);
    }

    #[test]
    fn sort_by_datetime_works() {
        let config = test::helpers::config_fixture();
        let no_date = Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: false,
            description: String::from(""),
            due: None,
            priority: 3,
            is_deleted: false,
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
        let config = test::helpers::config_fixture();
        let item = Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: false,
            description: String::from(""),
            due: None,
            priority: 3,
            is_deleted: false,
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
}
