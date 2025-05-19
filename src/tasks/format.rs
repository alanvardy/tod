use regex::Regex;
use std::borrow::Cow;
use supports_hyperlinks::Stream;

use super::{DateTimeInfo, Duration, Task, Unit, priority};
use crate::{color, comments::Comment, config::Config, errors::Error, projects::Project, time};

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

pub async fn project(task: &Task, config: &Config, buffer: &String) -> Result<String, Error> {
    let project_icon = color::purple_string("#");
    let maybe_project = config
        .projects()
        .await?
        .into_iter()
        .filter(|p| p.id == task.project_id)
        .collect::<Vec<Project>>();

    let text = match maybe_project.first() {
        Some(Project { name, .. }) => format!("\n{buffer}{project_icon} {name}"),
        None => {
            let command = color::cyan_string("tod project import --auto");
            format!(
                "\n{buffer}{project_icon} Project not in config\nUse {command} to import missing projects"
            )
        }
    };
    Ok(text)
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
            let date_string = time::date_to_string(date, config).unwrap_or_default();

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
            let datetime_string = time::datetime_to_string(datetime, config).unwrap_or_default();

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

pub fn number_comments(quantity: usize) -> String {
    let comment_icon = color::purple_string("★");
    if quantity == 1 {
        return format!("\n{comment_icon} 1 comment");
    }

    format!("\n{comment_icon} {quantity} comments")
}

pub async fn render_comments(config: &Config, comments: Vec<Comment>) -> Result<String, Error> {
    let comment_icon = color::purple_string("★");
    let mut comments = comments
        .iter()
        .map(|c| {
            c.fmt(config)
                .unwrap_or_else(|e| format!("Failed to render comment: {e:?}"))
        })
        .collect::<Vec<String>>();
    // Latest comment first
    comments.reverse();
    let comments = comments.join("\n\n");
    let mut formatted_string = format!("\n\n{comment_icon} Comments {comment_icon}\n\n{comments}");
    let max_comment_length: usize = config.max_comment_length().try_into()?;

    if formatted_string.len() > max_comment_length {
        formatted_string.truncate(max_comment_length);
        formatted_string.push_str("[TRUNCATED]");
    };

    Ok(formatted_string)
}

#[cfg(test)]
mod tests {
    use crate::test;

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

    #[tokio::test]
    async fn test_comments() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock(
                "GET",
                "/api/v1/comments/?task_id=6Xqhv4cwxgjwG9w8&limit=200",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::comments_response())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let comments = vec![test::fixtures::comment()];
        let comments = render_comments(&config, comments).await.unwrap();

        assert_matches!(
            comments.as_str(),
            "\n\n★ Comments ★\n\nPosted 2016-09-22 00:00:00 PDT\nNeed one bottle of milk"
        );
        mock.expect(1);
    }
}
