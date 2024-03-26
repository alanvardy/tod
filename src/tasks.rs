use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt::Display;

pub mod priority;
use crate::color;
use crate::config::Config;
use crate::config::SortValue;
use crate::projects;
use crate::tasks::priority::Priority;
use crate::{input, time, todoist};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub priority: Priority,
    pub description: String,
    pub labels: Vec<String>,
    pub parent_id: Option<String>,
    pub due: Option<DateInfo>,
    /// Only on rest api return value
    pub is_completed: Option<bool>,
    pub is_deleted: Option<bool>,
    /// only on sync api return value
    pub checked: Option<bool>,
    pub duration: Option<Duration>,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DateInfo {
    pub date: String,
    pub is_recurring: bool,
    pub string: String,
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Duration {
    pub amount: u8,
    pub unit: Unit,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    items: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Unit {
    #[serde(rename(deserialize = "minute"))]
    Minute,
    #[serde(rename(deserialize = "day"))]
    Day,
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
        string: String,
    },
    DateTime {
        datetime: DateTime<Tz>,
        is_recurring: bool,
        string: String,
    },
}

impl Task {
    pub fn fmt(&self, config: &Config, format: FormatType) -> String {
        let content = match self.priority {
            priority::Priority::Low => color::blue_string(&self.content),
            priority::Priority::Medium => color::yellow_string(&self.content),
            priority::Priority::High => color::red_string(&self.content),
            priority::Priority::None => color::normal_string(&self.content),
        };

        let buffer = match format {
            FormatType::List => String::from("  "),
            FormatType::Single => String::from(""),
        };

        let description = match &*self.description {
            "" => String::from(""),
            _ => format!("\n{buffer}{}", self.description),
        };

        let due_icon = color::purple_string("!");
        let recurring_icon = color::purple_string("↻");
        let due = match &self.datetimeinfo(config) {
            Ok(DateTimeInfo::Date {
                date,
                is_recurring,
                string,
            }) => {
                let recurring_icon = if *is_recurring {
                    format!(" {recurring_icon} {string}")
                } else {
                    String::new()
                };
                let date_string = time::format_date(date, config).unwrap_or_default();

                format!("\n{buffer}{due_icon} {date_string}{recurring_icon}")
            }
            Ok(DateTimeInfo::DateTime {
                datetime,
                is_recurring,
                string,
            }) => {
                let recurring_icon = if *is_recurring {
                    format!(" {recurring_icon} {string}")
                } else {
                    String::new()
                };
                let datetime_string = time::format_datetime(datetime, config).unwrap_or_default();

                let duration_string = match self.duration {
                    None => String::new(),
                    Some(Duration {
                        amount: 1,
                        unit: Unit::Day,
                    }) => String::from(" for 1 day"),
                    Some(Duration {
                        amount,
                        unit: Unit::Day,
                    }) => format!(" for {amount} days"),
                    Some(Duration {
                        amount,
                        unit: Unit::Minute,
                    }) => format!(" for {amount} min"),
                };

                format!("\n{buffer}{due_icon} {datetime_string}{duration_string}{recurring_icon}")
            }
            Ok(DateTimeInfo::NoDateTime) => String::from(""),
            Err(string) => string.clone(),
        };

        let prefix = match format {
            FormatType::List => String::from("- "),
            FormatType::Single => String::from(""),
        };

        let labels = if self.labels.is_empty() {
            String::new()
        } else {
            format!(" {} {}", color::purple_string("@"), self.labels.join(" "))
        };

        format!("{prefix}{content}{description}{due}{labels}\n")
    }

    /// Determines the numeric value of an task for sorting
    fn value(&self, config: &Config) -> u32 {
        let date_value: u8 = self.date_value(config);
        let priority_value: u8 = self.priority_value(config);

        date_value as u32 + priority_value as u32
    }

    /// Return the value of the due field
    fn date_value(&self, config: &Config) -> u8 {
        let SortValue {
            no_due_date,
            today,
            overdue,
            now,
            not_recurring,
            ..
        } = config.sort_value.clone().unwrap_or_default();

        match &self.datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => no_due_date,
            Ok(DateTimeInfo::Date {
                date, is_recurring, ..
            }) => {
                let today_value = if *date == time::today_date(config).unwrap_or_default() {
                    today
                } else {
                    0
                };
                let overdue_value = if self.is_overdue(config).unwrap_or_default() {
                    overdue
                } else {
                    0
                };
                let recurring_value = if is_recurring.to_owned() {
                    0
                } else {
                    not_recurring
                };
                today_value + overdue_value + recurring_value
            }
            Ok(DateTimeInfo::DateTime {
                datetime,
                is_recurring,
                ..
            }) => {
                let recurring_value = if is_recurring.to_owned() {
                    0
                } else {
                    not_recurring
                };

                let duration = match time::now(config) {
                    Ok(tz) => (*datetime - tz).num_minutes(),
                    _ => 0,
                };

                match duration {
                    -15..=15 => now + recurring_value,
                    _ => recurring_value,
                }
            }
            Err(_) => not_recurring,
        }
    }

    /// Return the value of the due field
    fn datetime(&self, config: &Config) -> Option<DateTime<Tz>> {
        match self.datetimeinfo(config) {
            Ok(DateTimeInfo::DateTime { datetime, .. }) => Some(datetime),
            _ => None,
        }
    }

    fn priority_value(&self, config: &Config) -> u8 {
        let SortValue {
            priority_none,
            priority_low,
            priority_medium,
            priority_high,
            ..
        } = config.sort_value.clone().unwrap_or_default();
        match &self.priority {
            Priority::None => priority_none,
            Priority::Low => priority_low,
            Priority::Medium => priority_medium,
            Priority::High => priority_high,
        }
    }

    /// Converts the JSON date representation into Date or Datetime
    fn datetimeinfo(&self, config: &Config) -> Result<DateTimeInfo, String> {
        let tz = match (self.clone().due, config.clone().timezone) {
            (None, Some(tz_string)) => time::timezone_from_str(&Some(tz_string))?,
            (None, None) => Tz::UTC,
            (Some(DateInfo { timezone: None, .. }), Some(tz_string)) => time::timezone_from_str(&Some(tz_string))?,
            (Some(DateInfo { timezone: None, .. }), None) => Tz::UTC,
            (Some(DateInfo {
                timezone: Some(tz_string),
                ..
                // Remove the Some here
            }), _) => time::timezone_from_str(&Some(tz_string))?,
        };
        match self.clone().due {
            None => Ok(DateTimeInfo::NoDateTime),
            Some(DateInfo {
                date,
                is_recurring,
                string,
                ..
            }) if date.len() == 10 => Ok(DateTimeInfo::Date {
                date: time::date_from_str(&date, tz)?,
                is_recurring,
                string,
            }),
            Some(DateInfo {
                date,
                is_recurring,
                string,
                ..
            }) => Ok(DateTimeInfo::DateTime {
                datetime: time::datetime_from_str(&date, tz)?,
                is_recurring,
                string,
            }),
        }
    }

    pub fn filter(&self, config: &Config, filter: &projects::TaskFilter) -> bool {
        match filter {
            projects::TaskFilter::Unscheduled => {
                self.has_no_date() || self.is_overdue(config).unwrap_or_default()
            }
            projects::TaskFilter::Overdue => self.is_overdue(config).unwrap_or_default(),
            projects::TaskFilter::Recurring => self.is_recurring(),
        }
    }

    fn has_no_date(&self) -> bool {
        self.due.is_none()
    }

    // Returns true if the datetime is today and there is a time
    fn is_today(&self, config: &Config) -> Result<bool, String> {
        let boolean = match self.datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => false,
            Ok(DateTimeInfo::Date { date, .. }) => date == time::today_date(config)?,
            Ok(DateTimeInfo::DateTime { datetime, .. }) => {
                time::datetime_is_today(datetime, config)?
            }
            Err(_) => false,
        };

        Ok(boolean)
    }

    fn is_overdue(&self, config: &Config) -> Result<bool, String> {
        let boolean = match self.clone().datetimeinfo(config) {
            Ok(DateTimeInfo::NoDateTime) => false,
            Ok(DateTimeInfo::Date { date, .. }) => time::is_date_in_past(date, config)?,
            Ok(DateTimeInfo::DateTime { datetime, .. }) => {
                time::is_date_in_past(datetime.date_naive(), config)?
            }
            Err(_) => false,
        };

        Ok(boolean)
    }

    /// Returns true if it is a recurring task
    pub fn is_recurring(&self) -> bool {
        match self.due {
            None => false,
            Some(DateInfo { is_recurring, .. }) => is_recurring,
        }
    }
}

pub fn sync_json_to_tasks(json: String) -> Result<Vec<Task>, String> {
    let result: Result<Body, _> = serde_json::from_str(&json);
    match result {
        Ok(body) => Ok(body.items),
        Err(err) => Err(format!("Could not parse response for task: {err:?}")),
    }
}

pub fn rest_json_to_tasks(json: String) -> Result<Vec<Task>, String> {
    let result: Result<Vec<Task>, String> = serde_json::from_str(&json).map_err(|e| e.to_string());

    result
}

pub fn json_to_task(json: String) -> Result<Task, String> {
    match serde_json::from_str(&json) {
        Ok(task) => Ok(task),
        Err(err) => Err(format!("Could not parse response for task: {err:?}")),
    }
}

pub fn sort_by_value(mut tasks: Vec<Task>, config: &Config) -> Vec<Task> {
    tasks.sort_by_key(|b| Reverse(b.value(config)));
    tasks
}

pub fn sort_by_datetime(mut tasks: Vec<Task>, config: &Config) -> Vec<Task> {
    tasks.sort_by_key(|i| i.datetime(config));
    tasks
}

pub fn filter_not_in_future(tasks: Vec<Task>, config: &Config) -> Result<Vec<Task>, String> {
    let tasks = tasks
        .into_iter()
        .filter(|task| {
            task.is_today(config).unwrap_or_default()
                || task.has_no_date()
                || task.is_overdue(config).unwrap_or_default()
        })
        .collect();

    Ok(tasks)
}

pub fn set_priority(config: &Config, task: Task) -> Result<String, String> {
    println!("{}", task.fmt(config, FormatType::Single));

    let options = vec![
        Priority::None,
        Priority::Low,
        Priority::Medium,
        Priority::High,
    ];
    let priority = input::select(
        "Choose a priority that should be assigned to task: ",
        options,
        config.mock_select,
    )?;

    let config = config.set_next_id(&task.id);
    todoist::update_task_priority(config, task, priority)
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
        assert_eq!(test::fixtures::task().date_value(&config), 50);

        // Recurring
        let task = Task {
            due: Some(DateInfo {
                is_recurring: true,
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };
        assert_eq!(task.date_value(&config), 0);

        // Overdue
        let task = Task {
            due: Some(DateInfo {
                date: String::from("2001-11-13"),
                is_recurring: true,
                timezone: Some(String::from("America/Los_Angeles")),
                string: String::from("Every 2 weeks"),
            }),
            ..test::fixtures::task()
        };
        assert_eq!(task.date_value(&config), 150);

        // No date
        let task = Task { due: None, ..task };
        assert_eq!(task.date_value(&config), 80);
    }

    #[test]
    fn date_value_can_handle_datetime() {
        let config = test::fixtures::config();
        let task = Task {
            due: Some(DateInfo {
                date: String::from("2021-02-27T19:41:56Z"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_eq!(task.date_value(&config), 50);
    }

    #[test]
    fn can_format_task_with_a_date() {
        let config = test::fixtures::config();
        let task = Task {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: String::from("2021-08-13"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_eq!(
            format!("{}", task.fmt(&config, FormatType::Single)),
            "Get gifts for the twins\n! 2021-08-13 @ computer\n"
        );
    }

    #[test]
    fn can_format_task_with_today() {
        let config = test::fixtures::config();
        let task = Task {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_eq!(
            format!("{}", task.fmt(&config, FormatType::Single)),
            "Get gifts for the twins\n! Today @ computer\n"
        );
    }

    #[test]
    fn value_can_get_the_value_of_an_task() {
        let config = test::fixtures::config();
        let task = Task {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_matches!(task.datetime(&config), Some(DateTime { .. }));
    }

    #[test]
    fn datetime_works_with_date() {
        let config = test::fixtures::config();
        let task = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_eq!(task.datetime(&config), None);
    }

    #[test]
    fn has_no_date_works() {
        let config = test::fixtures::config();
        let task = Task {
            due: None,
            ..test::fixtures::task()
        };

        assert!(task.has_no_date());

        let task_today = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };
        assert!(!task_today.has_no_date());
    }

    #[test]
    fn is_today_works() {
        let config = test::fixtures::config();
        let task = Task {
            due: None,
            ..test::fixtures::task()
        };

        assert!(!task.is_today(&config).unwrap());

        let task_today = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..test::fixtures::task()
        };
        assert!(task_today.is_today(&config).unwrap());

        let task_in_past = Task {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                is_recurring: false,
                timezone: None,
                string: String::from("Every 2 weeks"),
            }),
            ..test::fixtures::task()
        };
        assert!(!task_in_past.is_today(&config).unwrap());
    }

    #[test]
    fn sort_by_value_works() {
        let config = test::fixtures::config();
        let today = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                is_recurring: false,
                timezone: None,
                string: String::from("Every 2 weeks"),
            }),
            ..test::fixtures::task()
        };

        let today_recurring = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..test::fixtures::task()
        };

        let future = Task {
            due: Some(DateInfo {
                date: String::from("2035-12-12"),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..test::fixtures::task()
        };

        let input = vec![future.clone(), today_recurring.clone(), today.clone()];
        let result = vec![today, today_recurring, future];

        assert_eq!(sort_by_value(input, &config), result);
    }

    #[test]
    fn sort_by_datetime_works() {
        let config = test::fixtures::config();
        let no_date = Task {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            parent_id: None,
            description: String::from(""),
            duration: Some(Duration {
                amount: 123,
                unit: Unit::Minute,
            }),
            due: None,
            labels: vec![String::from("computer")],
            priority: Priority::Medium,
            is_deleted: None,
            is_completed: None,
        };

        let date_not_datetime = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..no_date.clone()
        };

        let present = Task {
            due: Some(DateInfo {
                date: String::from("2020-09-06T16:00:00"),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..no_date.clone()
        };

        let future = Task {
            due: Some(DateInfo {
                date: String::from("2035-09-06T16:00:00"),
                string: String::from("Every 2 weeks"),
                is_recurring: false,
                timezone: None,
            }),
            ..no_date.clone()
        };

        let past = Task {
            due: Some(DateInfo {
                date: String::from("2015-09-06T16:00:00"),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
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
        let task = Task {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            duration: None,
            parent_id: None,
            description: String::from(""),
            labels: vec![String::from("computer")],
            due: None,
            priority: Priority::Medium,
            is_deleted: None,
            is_completed: None,
        };

        assert!(!task.is_overdue(&config).unwrap());

        let task_today = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                string: String::from("Every 2 weeks"),
                is_recurring: false,
                timezone: None,
            }),
            ..task.clone()
        };
        assert!(!task_today.is_overdue(&config).unwrap());

        let task_future = Task {
            due: Some(DateInfo {
                date: String::from("2035-12-12"),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..task.clone()
        };
        assert!(!task_future.is_overdue(&config).unwrap());

        let task_today = Task {
            due: Some(DateInfo {
                date: String::from("2020-12-20"),
                is_recurring: false,
                string: String::from("Every 2 weeks"),
                timezone: None,
            }),
            ..task
        };
        assert!(task_today.is_overdue(&config).unwrap());
    }

    #[test]
    fn json_to_tasks_works() {
        let json = String::from("2{.e");
        let error_text = String::from("Could not parse response for task: Error(\"invalid type: integer `2`, expected struct Body\", line: 1, column: 1)");
        assert_eq!(sync_json_to_tasks(json), Err(error_text));
    }

    #[test]
    fn json_to_task_works() {
        let json = String::from("2{.e");
        let error_text = String::from("Could not parse response for task: Error(\"invalid type: integer `2`, expected struct Task\", line: 1, column: 1)");
        assert_eq!(json_to_task(json), Err(error_text));
    }

    #[test]
    fn test_to_integer() {
        assert_eq!(Priority::None.to_integer(), 1);
        assert_eq!(Priority::Low.to_integer(), 2);
        assert_eq!(Priority::Medium.to_integer(), 3);
        assert_eq!(Priority::High.to_integer(), 4);
    }

    #[test]
    fn test_set_priority() {
        let task = test::fixtures::task();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/rest/v2/tasks/222")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create();
        let config = test::fixtures::config()
            .mock_select(1)
            .mock_url(server.url());

        let result = set_priority(&config, task);
        assert_eq!(result, Ok("✓".to_string()));
        mock.assert();
    }
}
