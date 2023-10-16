use crate::{
    color,
    config::Config,
    tasks::{self, FormatType},
    todoist,
};

/// All tasks for a project
pub fn all_tasks(config: &Config, filter: &String) -> Result<String, String> {
    let tasks = todoist::tasks_for_filter(config, filter)?;

    if tasks.is_empty() {
        return Ok(format!("No tasks for filter: '{filter}'"));
    }

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!(
        "Tasks for filter: '{filter}'"
    )));

    for task in tasks::sort_by_datetime(tasks, config) {
        buffer.push('\n');
        buffer.push_str(&task.fmt(config, FormatType::List));
    }
    Ok(buffer)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    /// Need to adjust this value forward or back an hour when timezone changes
    const TIME: &str = "16:59";

    #[test]
    fn test_all_tasks() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::rest_tasks())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let filter = String::from("today");

        assert_eq!(
            all_tasks(&config_with_timezone, &filter),
            Ok(format!(
                "Tasks for filter: 'today'\n- Put out recycling\n  ! {TIME} ↻ every other mon at 16:30\n"
            ))
        );
        mock.assert();
    }
}
