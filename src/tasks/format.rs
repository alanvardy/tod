use regex::Regex;
use std::borrow::Cow;
use supports_hyperlinks::Stream;

use super::{priority, DateTimeInfo, Duration, Task, Unit};
use crate::{color, config::Config, projects::Project, time};

pub fn content(task: &Task, config: &Config) -> String {
    let content = match task.priority {
        priority::Priority::Low => color::blue_string(&task.content),
        priority::Priority::Medium => color::yellow_string(&task.content),
        priority::Priority::High => color::red_string(&task.content),
        priority::Priority::None => color::normal_string(&task.content),
    };

    if disable_links(config) {
        content
    } else {
        create_links(&content)
    }
}

pub fn project(task: &Task, config: &Config, buffer: &String) -> String {
    let project_icon = color::purple_string("#");
    let maybe_project = config
        .projects
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.id == task.project_id)
        .collect::<Vec<Project>>();

    match maybe_project.first() {
        Some(Project { name, .. }) => format!("\n{buffer}{project_icon} {name}"),
        None => {
            let command = color::cyan_string("tod project import --auto");
            format!("\n{buffer}{project_icon} Project not in config\nUse {command} to import missing projects")
        }
    }
}

pub fn disable_links(config: &Config) -> bool {
    config.disable_links || !supports_hyperlinks::on(Stream::Stdout)
}

pub fn labels(task: &Task) -> String {
    format!(" {} {}", color::purple_string("@"), task.labels.join(" "))
}

pub fn due(task: &Task, config: &Config, buffer: &String) -> String {
    let due_icon = color::purple_string("!");
    let recurring_icon = color::purple_string("↻");

    match &task.datetimeinfo(config) {
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

            let duration_string = match task.duration {
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
        Ok(DateTimeInfo::NoDateTime) => String::new(),
        Err(e) => e.to_string(),
    }
}

pub fn task_url(id: &str) -> String {
    let link = color::purple_string("link");
    format!("\x1B]8;;https://app.todoist.com/app/task/{id}\x1B\\[{link}]\x1B]8;;\x1B\\")
}

fn create_links(content: &str) -> String {
    // Define the regex pattern for Markdown links
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    // Use `replace_all` to replace all matches
    let result = link_regex.replace_all(content, |caps: &regex::Captures| {
        let text = &caps[1];
        let url = &caps[2];
        Cow::from(format!("\x1b]8;;{}\x07[{}]\x1b]8;;\x07", url, text))
    });

    result.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_links() {
        assert_eq!(create_links("hello"), String::from("hello"));
        assert_eq!(
            create_links("This is text [Google](https://www.google.com/)"),
            String::from("This is text \x1b]8;;https://www.google.com/\x07[Google]\x1b]8;;\x07")
        );
    }

    #[test]
    fn test_task_url() {
        assert_eq!(
            task_url("1"),
            String::from("\x1B]8;;https://app.todoist.com/app/task/1\x1B\\[link]\x1B]8;;\x1B\\")
        )
    }
}
