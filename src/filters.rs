use futures::future;

use crate::{
    color,
    config::Config,
    error::Error,
    input::{self},
    tasks::{self, FormatType, Task},
    todoist,
};

/// All tasks for a project
pub async fn all_tasks(config: &Config, filter: &String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;

    if tasks.is_empty() {
        return Ok(format!("No tasks for filter: '{filter}'"));
    }

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!(
        "Tasks for filter: '{filter}'"
    )));

    for task in tasks::sort_by_datetime(tasks, config) {
        buffer.push('\n');
        buffer.push_str(&task.fmt(config, FormatType::List, true));
    }
    Ok(buffer)
}

pub async fn edit_task(config: &Config, filter: String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, &filter).await?;

    let task = input::select(input::TASK, tasks, config.mock_select)?;

    let options = tasks::task_attributes();

    let selections = input::multi_select(input::ATTRIBUTES, options, config.mock_select)?;

    if selections.is_empty() {
        return Err(Error {
            message: "Nothing selected".to_string(),
            source: "edit_task".to_string(),
        });
    }

    let mut handles = Vec::new();
    for attribute in selections {
        // Stops the inputs from rolling over each other in terminal
        println!();
        if let Some(handle) = tasks::update_task(config, &task, &attribute).await? {
            handles.push(handle);
        }
    }

    future::join_all(handles).await;
    Ok(String::from("Finished editing task"))
}

pub async fn label(config: &Config, filter: &str, labels: &Vec<String>) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;
    let mut handles = Vec::new();
    for task in tasks::sort_by_value(tasks, config) {
        let future = tasks::label_task(config, task, labels).await?;
        handles.push(future);
    }

    future::join_all(handles).await;
    Ok(color::green_string(&format!(
        "There are no more tasks for filter: '{filter}'"
    )))
}

/// Get the next task by priority and save its id to config
pub async fn next_task(config: Config, filter: &str) -> Result<String, Error> {
    match fetch_next_task(&config, filter).await {
        Ok(Some((task, remaining))) => {
            config.set_next_id(&task.id).save().await?;
            let task_string = task.fmt(&config, FormatType::Single, true);
            Ok(format!("{task_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No tasks on list")),
        Err(e) => Err(e),
    }
}

async fn fetch_next_task(config: &Config, filter: &str) -> Result<Option<(Task, usize)>, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;
    let tasks = tasks::sort_by_value(tasks, config);

    Ok(tasks.first().map(|task| (task.to_owned(), tasks.len())))
}

/// Get next tasks and give an interactive prompt for completing them one by one
pub async fn process_tasks(config: &Config, filter: &String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;
    let tasks = tasks::sort_by_value(tasks, config);
    let tasks = tasks::reject_parent_tasks(tasks, config).await;
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        println!(" ");
        match tasks::process_task(config, task, &mut task_count, true).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    Ok(color::green_string(&format!(
        "There are no more tasks for filter: '{filter}'"
    )))
}

// Gives all tasks durations
pub async fn timebox_tasks(config: &Config, filter: &String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;
    let tasks = tasks::sort_by_value(tasks, config);
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        println!(" ");
        match tasks::timebox_task(config, task, &mut task_count, true).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    Ok(color::green_string(&format!(
        "There are no more tasks for filter: '{filter}'"
    )))
}

/// Prioritize all unprioritized tasks in a project
pub async fn prioritize_tasks(config: &Config, filter: &String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to prioritize in '{filter}'"
        )))
    } else {
        let mut handles = Vec::new();
        for task in tasks.iter() {
            let handle = tasks::set_priority(config, task.to_owned(), true).await?;
            handles.push(handle);
        }
        future::join_all(handles).await;
        Ok(color::green_string(&format!(
            "Successfully prioritized '{filter}'"
        )))
    }
}

/// Put dates on all tasks without dates
pub async fn schedule(config: &Config, filter: &String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filter(config, filter).await?;

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in '{filter}'"
        )))
    } else {
        let mut handles = Vec::new();
        for task in tasks.iter() {
            if let Some(handle) = tasks::spawn_schedule_task(config.clone(), task.clone())? {
                handles.push(handle);
            }
        }

        future::join_all(handles).await;
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

    #[tokio::test]
    async fn test_all_tasks() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let filter = String::from("today");

        let tasks = all_tasks(&config_with_timezone, &filter).await.unwrap();
        //     Ok(format!(
        //         "Tasks for filter: 'today'\n- Put out recycling\n  ! {TIME} ↻ every other mon at 16:30\n# Project not in config\nUse tod project import --auto to import missing projects\n"
        //     ))
        // );

        assert!(tasks.contains("Tasks for filter"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_rename_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(0);

        let result = edit_task(&config, String::from("today"));
        assert_eq!(result.await, Ok("Finished editing task".to_string()));
        mock.assert();
    }
    #[tokio::test]
    async fn test_get_next_task() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test3"),
            mock_url: Some(server.url()),
            ..config
        };

        config_with_timezone.clone().create().await.unwrap();

        let filter = String::from("today");
        let task = next_task(config_with_timezone, &filter).await.unwrap();

        assert!(task.contains("Put out recycling"));
        assert!(task.contains("every other mon at 16:30"));
    }
    #[tokio::test]
    async fn test_label() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test3"),
            mock_url: Some(server.url()),
            mock_select: Some(0),
            ..config
        };

        config_with_timezone.clone().create().await.unwrap();

        let filter = String::from("today");
        let labels = vec![String::from("thing")];

        assert_eq!(
            label(&config_with_timezone, &filter, &labels).await,
            Ok(String::from("There are no more tasks for filter: 'today'"))
        );
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_process_tasks() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .await
            .unwrap();
        let filter = String::from("today");

        let result = process_tasks(&config, &filter);
        assert_eq!(
            result.await,
            Ok("There are no more tasks for filter: 'today'".to_string())
        );
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_schedule() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_unscheduled_tasks())
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(1)
            .mock_string("tod");

        let filter = String::from("today");
        let result = schedule(&config, &filter);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        let config = config.mock_select(2);

        let filter = String::from("today");
        let result = schedule(&config, &filter);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        mock.expect(2);
        mock2.expect(2);
    }
    #[tokio::test]
    async fn test_prioritize_tasks() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/tasks/?filter=today")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;
        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::get_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(1);

        let filter = String::from("today");
        let result = prioritize_tasks(&config, &filter);
        assert_eq!(
            result.await,
            Ok(String::from("Successfully prioritized 'today'"))
        );
        mock.assert();
        mock2.assert();
    }
}
