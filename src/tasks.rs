use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures::future;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt::Display;
use tokio::task::JoinHandle;

pub mod format;
pub mod priority;
use crate::color;
use crate::config::Config;
use crate::config::SortValue;
use crate::error::Error;
use crate::input::DateTimeInput;
use crate::projects;
use crate::tasks;
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
    pub project_id: String,
    pub due: Option<DateInfo>,
    /// Only on rest api return value
    pub is_completed: Option<bool>,
    pub is_deleted: Option<bool>,
    /// only on sync api return value
    pub checked: Option<bool>,
    pub duration: Option<Duration>,
}

// Update task_attributes fn when adding here
pub enum TaskAttribute {
    Content,
    Description,
    Priority,
    Due,
}
impl Display for TaskAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskAttribute::Content => write!(f, "Content"),
            TaskAttribute::Description => write!(f, "Description"),
            TaskAttribute::Priority => write!(f, "Priority"),
            TaskAttribute::Due => write!(f, "Due"),
        }
    }
}

/// Used for selecting which attribute to set or edit in a task
pub fn task_attributes() -> Vec<TaskAttribute> {
    vec![
        TaskAttribute::Content,
        TaskAttribute::Description,
        TaskAttribute::Priority,
        TaskAttribute::Due,
    ]
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
    pub amount: u32,
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
    pub fn fmt(&self, config: &Config, format: FormatType, with_project: bool) -> String {
        let content = format::content(self, config);
        let buffer = match format {
            FormatType::List => String::from("  "),
            FormatType::Single => String::new(),
        };

        let description = match &*self.description {
            "" => String::new(),
            _ => format!("\n{buffer}{}", self.description),
        };

        let project = if with_project {
            format::project(self, config, &buffer)
        } else {
            String::new()
        };

        let url = if format::disable_links(config) {
            String::new()
        } else {
            format::task_url(&self.id)
        };

        let due = format::due(self, config, &buffer);
        let prefix = match format {
            FormatType::List => String::from("- "),
            FormatType::Single => String::new(),
        };

        let labels = if self.labels.is_empty() {
            String::new()
        } else {
            format::labels(self)
        };

        format!("{prefix}{content}{description}{due}{labels}{project} {url}\n")
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
            Ok(DateTimeInfo::Date { date, .. }) => {
                let naive_datetime = date.and_hms_opt(23, 59, 00)?;

                let now = time::now(config).ok()?;

                Some(DateTime::from_naive_utc_and_offset(
                    naive_datetime,
                    *now.offset(),
                ))
            }
            Ok(DateTimeInfo::NoDateTime) => None,
            Err(_) => None,
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
    fn datetimeinfo(&self, config: &Config) -> Result<DateTimeInfo, Error> {
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
    fn is_today(&self, config: &Config) -> Result<bool, Error> {
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

    fn is_overdue(&self, config: &Config) -> Result<bool, Error> {
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

pub async fn update_task(
    config: &Config,
    task: &Task,
    attribute: &TaskAttribute,
) -> Result<String, Error> {
    match attribute {
        TaskAttribute::Content => {
            let task_content = task.content.as_str();

            let new_task_content =
                input::string_with_default("Edit the task you selected:", task_content)?;

            if task_content == new_task_content {
                return Ok(color::green_string(
                    "The content is the same, no need to change it",
                ));
            }

            todoist::update_task_content(config, task, new_task_content).await
        }
        TaskAttribute::Description => {
            let value = task.description.as_str();

            let new_value = input::string_with_default("Edit the task you selected:", value)?;

            if value == new_value {
                return Ok(color::green_string(
                    "The description is the same, no need to change it",
                ));
            }

            todoist::update_task_description(config, task, new_value).await
        }
        TaskAttribute::Priority => {
            let value = &task.priority;
            let priorities = priority::all_priorities();

            let new_value = input::select("select your priority:", priorities, config.mock_select)?;

            if *value == new_value {
                return Ok(color::green_string(
                    "The priority is the same, no need to change it",
                ));
            }

            todoist::update_task_priority(config, task, &new_value).await
        }
        TaskAttribute::Due => {
            if let Some(handle) = tasks::spawn_schedule_task(config.clone(), task.clone())? {
                handle.await?;
                Ok(String::from("Updated due"))
            } else {
                Ok(String::from("No change"))
            }
        }
    }
}

pub async fn label_task(
    config: &Config,
    task: Task,
    labels: &Vec<String>,
) -> Result<JoinHandle<()>, Error> {
    println!("{}", task.fmt(config, FormatType::Single, true));
    let mut options = labels.to_owned();
    options.push(String::from("Skip"));
    let label = input::select("Select label", options, config.mock_select)?;

    let config = config.clone();
    Ok(tokio::spawn(async move {
        if label.as_str() == "Skip" {
        } else if let Err(e) = todoist::add_task_label(&config, task, label, false).await {
            config.tx().send(e).unwrap();
        }
    }))
}

pub async fn process_task(
    config: &Config,
    task: Task,
    task_count: &mut i32,
    with_project: bool,
) -> Option<JoinHandle<()>> {
    let options = ["Complete", "Skip", "Schedule", "Delete", "Quit"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let formatted_task = task.fmt(config, FormatType::Single, with_project);
    let mut reloaded_config = config
        .reload()
        .await
        .expect("Could not reload config")
        .increment_completed()
        .expect("Could not increment config");
    let tasks_completed = reloaded_config.tasks_completed().unwrap_or_default();
    println!("{formatted_task}{tasks_completed} completed today, {task_count} remaining");
    *task_count -= 1;
    match input::select("Select an option", options, config.mock_select) {
        Ok(string) => {
            if string == "Complete" {
                reloaded_config.save().await.expect("Could not save config");
                Some(spawn_complete_task(reloaded_config, task))
            } else if string == "Delete" {
                Some(spawn_delete_task(config.clone(), task))
            } else if string == "Schedule" {
                let date = input::date().ok()?;
                Some(spawn_update_task_due(config.clone(), task, date, None))
            } else if string == "Skip" {
                // Do nothing
                Some(tokio::spawn(async move {}))
            } else {
                // The quit clause
                None
            }
        }
        Err(e) => {
            let config = config.clone();
            let handle = tokio::spawn(async move {
                config.tx().send(e).unwrap();
            });
            Some(handle)
        }
    }
}

pub async fn timebox_task(
    config: &Config,
    task: Task,
    task_count: &mut i32,
    with_project: bool,
) -> Option<JoinHandle<()>> {
    let options = ["Timebox", "Complete", "Skip", "Delete", "Quit"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let formatted_task = task.fmt(config, FormatType::Single, with_project);
    println!("{formatted_task}{task_count} task(s) remaining");
    *task_count -= 1;
    match input::select("Select an option", options, config.mock_select) {
        Ok(string) => {
            if string == "Timebox" {
                match get_timebox(config, &task) {
                    Ok((due_string, duration)) => Some(spawn_update_task_due(
                        config.clone(),
                        task,
                        due_string,
                        Some(duration),
                    )),

                    Err(e) => {
                        config.clone().tx().send(e).unwrap();
                        Some(tokio::spawn(async move {}))
                    }
                }
                // Some(spawn_complete_task(config.clone(), task))
            } else if string == "Delete" {
                Some(spawn_delete_task(config.clone(), task))
            } else if string == "Skip" {
                // Do nothing
                Some(tokio::spawn(async move {}))
            } else {
                // The quit clause
                None
            }
        }
        Err(e) => {
            let config = config.clone();
            let handle = tokio::spawn(async move {
                config.tx().send(e).unwrap();
            });
            Some(handle)
        }
    }
}

/// Returns Date, time and duration for a task, uses the date and time on task if available, otherwise prompts. Always prompts for duration.
fn get_timebox(config: &Config, task: &Task) -> Result<(String, u32), Error> {
    let datetime = match task {
        Task {
            due: Some(DateInfo { date, .. }),
            ..
        } => {
            if time::is_date(date) {
                let time =
                    input::string("Input time, i.e. 3pm or 1500", config.mock_string.clone())?;

                format!("{date} {time}")
            } else {
                let tz = time::timezone_from_str(&config.timezone)?;
                time::datetime_from_str(date, tz)?
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            }
        }
        _ => {
            let date = input::date()?;
            let time = input::string("Input time, i.e. 3pm or 1500", config.mock_string.clone())?;
            format!("{date} {time}")
        }
    };

    let duration = input::string("Input duration in minutes", config.mock_string.clone())?;

    Ok((datetime, duration.parse::<u32>()?))
}

pub fn spawn_schedule_task(config: Config, task: Task) -> Result<Option<JoinHandle<()>>, Error> {
    println!("{}", task.fmt(&config, FormatType::Single, true));
    let datetime_input = input::datetime(
        config.mock_select,
        config.mock_string.clone(),
        config.natural_language_only,
    )?;
    match datetime_input {
        input::DateTimeInput::Complete => {
            let handle = tasks::spawn_complete_task(config.clone(), task.clone());
            Ok(Some(handle))
        }
        DateTimeInput::Skip => Ok(None),

        input::DateTimeInput::Text(due_string) => {
            let handle =
                tasks::spawn_update_task_due(config.clone(), task.clone(), due_string, None);
            Ok(Some(handle))
        }
        input::DateTimeInput::None => {
            let handle = tasks::spawn_update_task_due(
                config.clone(),
                task.clone(),
                "No date".to_string(),
                None,
            );
            Ok(Some(handle))
        }
    }
}

// Completes task inside another thread
pub fn spawn_complete_task(config: Config, task: Task) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = todoist::complete_task(&config, &task.id, false).await {
            config.tx().send(e).unwrap();
        }
    })
}

// Deletes task inside another thread
pub fn spawn_delete_task(config: Config, task: Task) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = todoist::delete_task(&config, &task, false).await {
            config.tx().send(e).unwrap();
        }
    })
}

// Updates task inside another thread
pub fn spawn_update_task_due(
    config: Config,
    task: Task,
    due_string: String,
    duration: Option<u32>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) =
            todoist::update_task_due_natural_language(&config, task, due_string, duration, false)
                .await
        {
            config.tx().send(e).unwrap();
        }
    })
}

pub fn sync_json_to_tasks(json: String) -> Result<Vec<Task>, Error> {
    let body: Body = serde_json::from_str(&json)?;
    Ok(body.items)
}

pub fn rest_json_to_tasks(json: String) -> Result<Vec<Task>, Error> {
    let tasks: Vec<Task> = serde_json::from_str(&json)?;
    Ok(tasks)
}

pub fn json_to_task(json: String) -> Result<Task, Error> {
    let task: Task = serde_json::from_str(&json)?;
    Ok(task)
}

pub fn sort_by_value(mut tasks: Vec<Task>, config: &Config) -> Vec<Task> {
    tasks.sort_by_key(|b| Reverse(b.value(config)));
    tasks
}

pub fn sort_by_datetime(mut tasks: Vec<Task>, config: &Config) -> Vec<Task> {
    tasks.sort_by_key(|i| i.datetime(config));
    tasks
}

pub fn filter_not_in_future(tasks: Vec<Task>, config: &Config) -> Result<Vec<Task>, Error> {
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

// We don't want to process parent tasks when child tasks are unchecked, or child tasks when they are checked
// We additionally need to make sure that parent tasks are not in the future

pub async fn reject_parent_tasks(tasks: Vec<Task>, config: &Config) -> Vec<Task> {
    let parent_ids: Vec<String> = tasks
        .clone()
        .into_iter()
        .filter(|task| task.parent_id.is_some() && !task.checked.unwrap_or_default())
        .map(|task| task.parent_id.unwrap_or_default())
        .collect();

    let mut handles = Vec::new();

    for task in tasks.clone() {
        let config = config.clone();
        let parent_ids = parent_ids.clone();
        let tasks = tasks.clone();

        let config = config.clone();
        let handle = tokio::spawn(async move {
            if !parent_ids.contains(&task.id)
                && !task.checked.unwrap_or_default()
                && !parent_in_future(task.clone(), tasks, &config).await
            {
                Some(task)
            } else {
                None
            }
        });

        handles.push(handle);
    }

    future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|t| t.ok())
        .flatten()
        .collect::<Vec<Task>>()
}

// Need to make sure that we are not completing a subtask for a parent task that is in the future
async fn parent_in_future(task: Task, tasks: Vec<Task>, config: &Config) -> bool {
    let task_ids: Vec<String> = tasks.clone().into_iter().map(|task| task.id).collect();

    match task {
        Task {
            parent_id: None, ..
        } => false,
        Task {
            parent_id: Some(parent_id),
            ..
        } => {
            if task_ids.contains(&parent_id) {
                false
            } else {
                // look up id and see if it is in the future
                match todoist::get_task(config, &parent_id).await {
                    Err(e) => {
                        config.clone().tx().send(e).unwrap();
                        false
                    }
                    Ok(task) => {
                        !(task.is_overdue(config).unwrap_or_default()
                            || task.has_no_date()
                            || task.is_today(config).unwrap_or_default())
                    }
                }
            }
        }
    }
}

pub async fn set_priority(
    config: &Config,
    task: Task,
    with_project: bool,
) -> Result<JoinHandle<()>, Error> {
    println!("{}", task.fmt(config, FormatType::Single, with_project));

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

    let config = config.clone();
    Ok(tokio::spawn(async move {
        if let Err(e) = todoist::update_task_priority(&config, &task, &priority).await {
            config.tx().send(e).unwrap();
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn date_value_can_handle_date() {
        let config = test::fixtures::config().await;
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

    #[tokio::test]
    async fn date_value_can_handle_datetime() {
        let config = test::fixtures::config().await;
        let task = Task {
            due: Some(DateInfo {
                date: String::from("2021-02-27T19:41:56Z"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_eq!(task.date_value(&config), 50);
    }

    #[tokio::test]
    async fn can_format_task_with_a_date() {
        let config = test::fixtures::config().await;
        let task = Task {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: String::from("2021-08-13"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        let task = task.fmt(&config, FormatType::Single, false);

        assert!(task.contains("Get gifts for the twins"));
        assert!(task.contains("2021-08-13"));
    }

    #[tokio::test]
    async fn can_format_task_with_today() {
        let config = test::fixtures::config().await;
        let task = Task {
            content: String::from("Get gifts for the twins"),
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        let task_text = task.fmt(&config, FormatType::Single, true);

        assert!(task_text.contains("Today @ computer"));
    }

    #[tokio::test]
    async fn value_can_get_the_value_of_an_task() {
        let config = test::fixtures::config().await;
        let task = Task {
            due: Some(DateInfo {
                date: String::from("2021-09-06T16:00:00"),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert_matches!(task.datetime(&config), Some(DateTime { .. }));
    }

    #[tokio::test]
    async fn datetime_works_with_date() {
        let config = test::fixtures::config().await;
        let task = Task {
            due: Some(DateInfo {
                date: time::today_string(&config).unwrap(),
                ..test::fixtures::task().due.unwrap()
            }),
            ..test::fixtures::task()
        };

        assert!(task.datetime(&config).is_some());
    }

    #[tokio::test]
    async fn has_no_date_works() {
        let config = test::fixtures::config().await;
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

    #[tokio::test]
    async fn is_today_works() {
        let config = test::fixtures::config().await;
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

    #[tokio::test]
    async fn sort_by_value_works() {
        let config = test::fixtures::config().await;
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

    #[tokio::test]
    async fn sort_by_datetime_works() {
        let config = test::fixtures::config().await;
        let no_date = Task {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            parent_id: None,
            project_id: String::from("123"),
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
            present.clone(),
            past.clone(),
            no_date.clone(),
            date_not_datetime.clone(),
        ];
        let result = vec![no_date, past, present, date_not_datetime, future];

        assert_eq!(sort_by_datetime(input, &config), result);
    }

    #[tokio::test]
    async fn is_overdue_works() {
        let config = test::fixtures::config().await;
        let task = Task {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            duration: None,
            parent_id: None,
            description: String::from(""),
            project_id: String::from("123"),
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
    fn test_to_integer() {
        assert_eq!(Priority::None.to_integer(), 1);
        assert_eq!(Priority::Low.to_integer(), 2);
        assert_eq!(Priority::Medium.to_integer(), 3);
        assert_eq!(Priority::High.to_integer(), 4);
    }

    #[tokio::test]
    async fn test_set_priority() {
        let task = test::fixtures::task();
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/rest/v2/tasks/222")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create_async()
            .await;
        let config = test::fixtures::config()
            .await
            .mock_select(1)
            .mock_url(server.url());

        let future = set_priority(&config, task, false).await.unwrap();

        tokio::join!(future).0.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn test_process_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create_async()
            .await;

        let task = test::fixtures::task();
        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .await
            .unwrap();

        let mut task_count = 3;
        process_task(&config, task, &mut task_count, true)
            .await
            .unwrap()
            .await
            .unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn test_display_task() {
        let task = test::fixtures::task();
        let string = String::from("Get gifts for the twins");
        assert_eq!(string, task.to_string())
    }
}
