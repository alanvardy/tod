use std::fmt::Display;

use crate::{
    color,
    config::Config,
    error::Error,
    projects::Project,
    tasks::{self, priority::Priority, FormatType, SortOrder, Task},
    todoist,
};
use futures::future;

#[derive(Clone)]
pub enum Flag {
    Project(Project),
    Filter(String),
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::Project(project) => write!(f, "{project}"),
            Flag::Filter(filter) => write!(f, "'{filter}'"),
        }
    }
}

/// Get a list of all tasks
pub async fn view(config: &Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => todoist::tasks_for_project(config, &project).await?,
        Flag::Filter(filter) => todoist::tasks_for_filter(config, &filter).await?,
    };

    let empty_text = format!("No tasks for {flag}");
    let title = format!("Tasks for {flag}");

    if tasks.is_empty() {
        return Ok(empty_text);
    }

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&title));
    buffer.push('\n');

    for task in tasks::sort(tasks, config, sort) {
        buffer.push('\n');
        buffer.push_str(&task.fmt(config, FormatType::List, true));
    }
    Ok(buffer)
}

/// Prioritize all unprioritized tasks
pub async fn prioritize(config: &Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => todoist::tasks_for_project(config, &project)
            .await?
            .into_iter()
            .filter(|task| task.priority == Priority::None)
            .collect::<Vec<Task>>(),
        Flag::Filter(filter) => todoist::tasks_for_filter(config, &filter).await?,
    };

    let empty_text = format!("No tasks for {flag}");
    let success = format!("Successfully prioritized {flag}");

    if tasks.is_empty() {
        return Ok(color::green_string(&empty_text));
    }

    let tasks = tasks::sort(tasks, config, sort);

    let mut handles = Vec::new();
    for task in tasks {
        println!();
        let handle = tasks::set_priority(config, task, true).await?;
        handles.push(handle);
    }
    future::join_all(handles).await;
    Ok(color::green_string(&success))
}

/// Gives tasks durations
pub async fn timebox(config: &Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => todoist::tasks_for_project(config, &project)
            .await?
            .into_iter()
            .filter(|task| task.duration.is_none())
            .collect::<Vec<Task>>(),
        Flag::Filter(filter) => todoist::tasks_for_filter(config, &filter).await?,
    };

    let empty_text = format!("No tasks for {flag}");
    let success = format!("Successfully timeboxed {flag}");

    if tasks.is_empty() {
        return Ok(color::green_string(&empty_text));
    }

    let tasks = tasks::sort(tasks, config, sort);
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        println!();
        match tasks::timebox_task(&config.reload().await?, task, &mut task_count, false).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    Ok(color::green_string(&success))
}

/// Get next tasks and give an interactive prompt for completing them one by one
pub async fn process(config: &Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => {
            let tasks = todoist::tasks_for_project(config, &project).await?;
            tasks::filter_not_in_future(tasks, config)?
        }

        Flag::Filter(filter) => todoist::tasks_for_filter(config, &filter).await?,
    };
    let tasks = tasks::reject_parent_tasks(tasks, config).await;

    let empty_text = format!("No tasks for {flag}");
    let success = format!("Successfully processed {flag}");

    if tasks.is_empty() {
        return Ok(color::green_string(&empty_text));
    }

    let tasks = tasks::sort(tasks, config, sort);
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        println!();
        match tasks::process_task(&config.reload().await?, task, &mut task_count, false).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    Ok(color::green_string(&success))
}

/// Puts labels on tasks
pub async fn label(
    config: &Config,
    flag: Flag,
    labels: &Vec<String>,
    sort: &SortOrder,
) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => todoist::tasks_for_project(config, &project).await?,
        Flag::Filter(filter) => todoist::tasks_for_filter(config, &filter).await?,
    };

    let empty_text = format!("No tasks for {flag}");
    let success = format!("Successfully labeled {flag}");

    if tasks.is_empty() {
        return Ok(color::green_string(&empty_text));
    }

    let tasks = tasks::sort(tasks, config, sort);
    let mut handles = Vec::new();
    for task in tasks {
        println!();
        let future = tasks::label_task(config, task, labels).await?;
        handles.push(future);
    }
    future::join_all(handles).await;
    Ok(color::green_string(&success))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_prioritize() {
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
        let sort = &SortOrder::Value;
        let result = prioritize(&config, Flag::Filter(filter), sort).await;
        assert_eq!(result, Ok(String::from("Successfully prioritized 'today'")));
        mock.assert();
        mock2.assert();
    }
    #[tokio::test]
    async fn test_timebox() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_unscheduled_tasks())
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
            .mock_string("tod")
            .create()
            .await
            .unwrap();

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;
        let result = timebox(&config, Flag::Project(project), sort);
        assert_eq!(result.await, Ok("Exited".to_string()));

        let config = config.mock_select(2);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().to_owned();
        let result = timebox(&config, Flag::Project(project), sort);
        assert_eq!(result.await, Ok("Exited".to_string()));

        let config = config.mock_select(3);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().to_owned();
        let result = timebox(&config, Flag::Project(project.clone()), sort).await;
        assert_eq!(result, Ok("Exited".to_string()));

        let result = timebox(&config, Flag::Project(project), sort).await;
        assert_eq!(result, Ok("Exited".to_string()));
        mock.expect(2);
        mock2.expect(2);
    }

    #[tokio::test]
    async fn test_prioritize_tasks_with_no_tasks() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;

        let result = prioritize(&config, Flag::Project(project), sort).await;
        assert_eq!(
            result,
            Ok(String::from("No tasks for myproject\nwww.google.com"))
        );
        mock.assert();
    }
    #[tokio::test]
    async fn test_process_with_filter() {
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
        let sort = &SortOrder::Value;

        let result = process(&config, Flag::Filter(filter), sort).await;
        assert_eq!(result, Ok("Successfully processed 'today'".to_string()));
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_process_with_project() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks().await)
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

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;

        let result = process(&config, Flag::Project(project), sort).await;
        assert_eq!(
            result,
            Ok("Successfully processed myproject\nwww.google.com".to_string())
        );
        mock.assert();
        mock2.assert();
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
        let sort = &SortOrder::Value;

        assert_eq!(
            label(&config_with_timezone, Flag::Filter(filter), &labels, sort).await,
            Ok(String::from("Successfully labeled 'today'"))
        );
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_view() {
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
        let sort = &SortOrder::Value;

        let tasks = view(&config_with_timezone, Flag::Filter(filter), sort)
            .await
            .unwrap();

        assert!(tasks.contains("Tasks for 'today'"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_view_with_project() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap().clone();
        let sort = &SortOrder::Value;

        let tasks = view(&config_with_timezone, Flag::Project(project), sort)
            .await
            .unwrap();

        assert!(tasks.contains("Tasks for"));
        assert!(tasks.contains("- Put out recycling\n"));
        mock.assert();
    }
}