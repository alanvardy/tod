use crate::{
    color,
    config::Config,
    input::{self, DateTimeInput},
    tasks::{self, FormatType, Task},
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

pub fn rename_task(config: &Config, filter: String) -> Result<String, String> {
    let project_tasks = todoist::tasks_for_filter(config, &filter)?;

    let selected_task = input::select(
        "Choose a task of the project:",
        project_tasks,
        config.mock_select,
    )?;
    let task_content = selected_task.content.as_str();

    let new_task_content = input::string_with_default("Edit the task you selected:", task_content)?;

    if task_content == new_task_content {
        return Ok(color::green_string(
            "The content is the same, no need to change it",
        ));
    }

    todoist::update_task_name(config, selected_task, new_task_content)
}

pub fn label(config: &Config, filter: &str, labels: Vec<String>) -> Result<String, String> {
    let tasks = todoist::tasks_for_filter(config, filter)?;
    for task in tasks {
        label_task(config, task, &labels)?;
    }
    Ok(color::green_string(&format!(
        "There are no more tasks for filter: '{filter}'"
    )))
}

fn label_task(config: &Config, task: Task, labels: &Vec<String>) -> Result<String, String> {
    println!("{}", task.fmt(config, FormatType::Single));
    let label = input::select("Select label", labels.to_owned(), config.mock_select)?;

    todoist::add_task_label(config, task, label)
}

/// Get the next task by priority and save its id to config
pub fn next_task(config: Config, filter: &str) -> Result<String, String> {
    match fetch_next_task(&config, filter) {
        Ok(Some((task, remaining))) => {
            config.set_next_id(&task.id).save()?;
            let task_string = task.fmt(&config, FormatType::Single);
            Ok(format!("{task_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No tasks on list")),
        Err(e) => Err(e),
    }
}

fn fetch_next_task(config: &Config, filter: &str) -> Result<Option<(Task, usize)>, String> {
    let tasks = todoist::tasks_for_filter(config, filter)?;
    let tasks = tasks::sort_by_value(tasks, config);

    Ok(tasks.first().map(|task| (task.to_owned(), tasks.len())))
}

/// Get next tasks and give an interactive prompt for completing them one by one
pub fn process_tasks(config: Config, filter: &String) -> Result<String, String> {
    let tasks = todoist::tasks_for_filter(&config, filter)?;
    let tasks = tasks::sort_by_value(tasks, &config);
    for task in tasks {
        config.set_next_id(&task.id).save()?;
        match handle_task(&config.reload()?, task) {
            Some(Ok(_)) => (),
            Some(Err(e)) => return Err(e),
            None => return Ok(color::green_string("Exited")),
        }
    }
    Ok(color::green_string(&format!(
        "There are no more tasks for filter: '{filter}'"
    )))
}
fn handle_task(config: &Config, task: Task) -> Option<Result<String, String>> {
    let options = ["complete", "skip", "quit"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    println!("{}", task.fmt(config, FormatType::Single));
    match input::select("Select an option", options, config.mock_select) {
        Ok(string) => {
            if string == "complete" {
                Some(todoist::complete_task(config))
            } else if string == "skip" {
                Some(Ok(color::green_string("task skipped")))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    }
}

/// Prioritize all unprioritized tasks in a project
pub fn prioritize_tasks(config: &Config, filter: &String) -> Result<String, String> {
    let tasks = todoist::tasks_for_filter(config, filter)?;

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to prioritize in '{filter}'"
        )))
    } else {
        for task in tasks.iter() {
            tasks::set_priority(config, task.to_owned())?;
        }
        Ok(color::green_string(&format!(
            "Successfully prioritized '{filter}'"
        )))
    }
}

/// Put dates on all tasks without dates
pub fn schedule(config: &Config, filter: &String) -> Result<String, String> {
    let tasks = todoist::tasks_for_filter(config, filter)?;

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in '{filter}'"
        )))
    } else {
        for task in tasks.iter() {
            println!("{}", task.fmt(config, FormatType::Single));
            let datetime_input = input::datetime(
                config.mock_select,
                config.mock_string.clone(),
                config.natural_language_only,
            )?;
            match datetime_input {
                input::DateTimeInput::Complete => {
                    let config = config.set_next_id(&task.id);
                    todoist::complete_task(&config)?
                }
                DateTimeInput::Skip => "Skipped".to_string(),

                input::DateTimeInput::Text(due_string) => {
                    todoist::update_task_due(config, task.to_owned(), due_string)?
                }
                input::DateTimeInput::None => {
                    todoist::update_task_due(config, task.to_owned(), "No Date".to_string())?
                }
            };
        }
        Ok(color::green_string(&format!(
            "Successfully scheduled tasks in '{filter}'"
        )))
    }
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
            .with_body(test::responses::get_tasks())
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

    #[test]
    fn test_rename_task() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);

        let result = rename_task(&config, String::from("today"));
        assert_eq!(
            result,
            Ok("The content is the same, no need to change it".to_string())
        );
        mock.assert();
    }
    #[test]
    fn test_get_next_task() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test3"),
            mock_url: Some(server.url()),
            ..config
        };

        config_with_timezone.clone().create().unwrap();

        let filter = String::from("today");
        assert_eq!(
            next_task(config_with_timezone, &filter),
            Ok(format!(
                "Put out recycling\n! {TIME} ↻ every other mon at 16:30\n\n1 task(s) remaining"
            ))
        );
    }
    #[test]
    fn test_label() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test3"),
            mock_url: Some(server.url()),
            mock_select: Some(0),
            ..config
        };

        config_with_timezone.clone().create().unwrap();

        let filter = String::from("today");
        let labels = vec![String::from("thing")];

        assert_eq!(
            label(&config_with_timezone, &filter, labels),
            Ok(String::from("There are no more tasks for filter: 'today'"))
        );
        mock.assert();
        mock2.assert();
    }

    #[test]
    fn test_process_tasks() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let mock2 = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .unwrap();
        let filter = String::from("today");

        let result = process_tasks(config, &filter);
        assert_eq!(
            result,
            Ok("There are no more tasks for filter: 'today'".to_string())
        );
        mock.assert();
        mock2.assert();
    }

    #[test]
    fn test_schedule() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_unscheduled_tasks())
            .create();

        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(1)
            .mock_string("tod");

        let filter = String::from("today");
        let result = schedule(&config, &filter);
        assert_eq!(
            result,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        let config = config.mock_select(2);

        let filter = String::from("today");
        let result = schedule(&config, &filter);
        assert_eq!(
            result,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        mock.expect(2);
        mock2.expect(2);
    }
    #[test]
    fn test_prioritize_tasks() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();
        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(1);

        let filter = String::from("today");
        let result = prioritize_tasks(&config, &filter);
        assert_eq!(result, Ok(String::from("Successfully prioritized 'today'")));
        mock.assert();
        mock2.assert();
    }
}
