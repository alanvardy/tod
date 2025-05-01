use std::fmt::Display;

use crate::{
    color,
    config::Config,
    errors::Error,
    projects::Project,
    tasks::{self, FormatType, SortOrder, Task, priority::Priority},
    todoist,
};
use futures::future;
use tokio::{fs, io::AsyncReadExt};

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
pub async fn view(config: &mut Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let list_of_tasks = match flag.clone() {
        Flag::Project(project) => vec![(
            project.name.clone(),
            todoist::all_tasks_by_project(config, &project, None).await?,
        )],
        Flag::Filter(filter) => todoist::all_tasks_by_filters(config, &filter).await?,
    };

    let mut buffer = String::new();

    for (query, tasks) in list_of_tasks {
        let title = format!("Tasks for {query}");
        buffer.push('\n');
        buffer.push_str(&color::green_string(&title));
        buffer.push('\n');
        for task in tasks::sort(tasks, config, sort) {
            let text = task.fmt(config, FormatType::List, true, false).await?;
            buffer.push('\n');
            buffer.push_str(&text);
        }
    }
    Ok(buffer)
}

/// Prioritize all unprioritized tasks
pub async fn prioritize(config: &Config, flag: Flag, sort: &SortOrder) -> Result<String, Error> {
    let tasks = match flag.clone() {
        Flag::Project(project) => todoist::all_tasks_by_project(config, &project, None)
            .await?
            .into_iter()
            .filter(|task| task.priority == Priority::None)
            .collect::<Vec<Task>>(),
        Flag::Filter(filter) => todoist::all_tasks_by_filters(config, &filter)
            .await?
            .iter()
            .flat_map(|(_, tasks)| tasks.to_owned())
            .collect::<Vec<Task>>(),
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
        Flag::Project(project) => todoist::all_tasks_by_project(config, &project, None)
            .await?
            .into_iter()
            .filter(|task| task.duration.is_none())
            .collect::<Vec<Task>>(),
        Flag::Filter(filter) => todoist::all_tasks_by_filters(config, &filter)
            .await?
            .into_iter()
            .flat_map(|(_, tasks)| tasks.to_owned())
            .collect::<Vec<Task>>(),
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
        match tasks::timebox_task(&config.reload().await?, task, &mut task_count, false).await? {
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
            let tasks = todoist::all_tasks_by_project(config, &project, None).await?;
            tasks::filter_not_in_future(tasks, config)?
        }

        Flag::Filter(filter) => todoist::all_tasks_by_filters(config, &filter)
            .await?
            .into_iter()
            .flat_map(|(_, tasks)| tasks.to_owned())
            .collect::<Vec<Task>>(),
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
        match tasks::process_task(&config.reload().await?, task, &mut task_count, false).await? {
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
        Flag::Project(project) => todoist::all_tasks_by_project(config, &project, None).await?,
        Flag::Filter(filter) => todoist::all_tasks_by_filters(config, &filter)
            .await?
            .into_iter()
            .flat_map(|(_, tasks)| tasks.to_owned())
            .collect::<Vec<Task>>(),
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

pub async fn import(config: &Config, file_path: &String) -> Result<String, Error> {
    let mut lines = String::new();
    fs::File::open(file_path)
        .await?
        .read_to_string(&mut lines)
        .await?;

    let lines: Vec<String> = lines
        .split('\n')
        .map(|s| s.to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    for line in lines {
        todoist::quick_create_task(config, &line).await?;
    }

    Ok(String::from("✓"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_import() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/tasks/quick")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());
        config.clone().create().await.unwrap();

        assert_eq!(import(&config, &config.path).await, Ok(String::from("✓")));

        mock.assert();
    }

    #[tokio::test]
    async fn test_prioritize() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/tasks/filter?query=today&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;
        let mock2 = server
            .mock("POST", "/api/v1/tasks/6Xqhv4cwxgjwG9w8")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .with_mock_url(server.url())
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
            .mock("GET", "/api/v1/tasks/?project_id=123&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::tasks_without_duration_response().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/api/v1/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let mock3 = server
            .mock("GET", "/api/v1/id_mappings/projects/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::ids())
            .create_async()
            .await;
        let config = test::fixtures::config()
            .await
            .with_mock_url(server.url())
            .mock_select(1)
            .with_mock_string("tod")
            .create()
            .await
            .unwrap();

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;
        let result = timebox(&config, Flag::Project(project), sort).await;
        assert_matches!(result, Ok(x) if x.contains("Successfully timeboxed"));

        let config = config.mock_select(2);

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap().to_owned();
        let result = timebox(&config, Flag::Project(project), sort).await;
        assert_matches!(result, Ok(x) if x.contains("Successfully timeboxed"));

        let config = config.mock_select(3);

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap().to_owned();
        let result = timebox(&config, Flag::Project(project.clone()), sort).await;
        assert_matches!(result, Ok(x) if x.contains("Successfully timeboxed"));

        let result = timebox(&config, Flag::Project(project), sort).await;
        assert_matches!(result, Ok(x) if x.contains("Successfully timeboxed"));
        mock.expect(2);
        mock2.expect(2);
        mock3.expect(1);
    }

    #[tokio::test]
    async fn test_prioritize_tasks_with_no_tasks() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/tasks/?project_id=123&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;

        let result = prioritize(&config, Flag::Project(project), sort).await;
        assert_eq!(
            result,
            Ok(String::from(
                "No tasks for myproject\nhttps://app.todoist.com/app/project/123"
            ))
        );
        mock.assert();
    }
    #[tokio::test]
    async fn test_process_with_filter() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/tasks/filter?query=today&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/api/v1/tasks/6Xqhv4cwxgjwG9w8/close")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .with_mock_url(server.url())
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
            .mock("GET", "/api/v1/tasks/?project_id=123&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/api/v1/tasks/6Xqhv4cwxgjwG9w8/close")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .with_mock_url(server.url())
            .mock_select(0)
            .create()
            .await
            .unwrap();

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap().to_owned();
        let sort = &SortOrder::Value;

        let result = process(&config, Flag::Project(project), sort).await;
        assert_eq!(
            result,
            Ok(
                "Successfully processed myproject\nhttps://app.todoist.com/app/project/123"
                    .to_string()
            )
        );
        mock.assert();
        mock2.assert();
    }
    #[tokio::test]
    async fn test_label() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/tasks/filter?query=today&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let mock2 = server
            .mock("POST", "/api/v1/tasks/6Xqhv4cwxgjwG9w8")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = config
            .with_timezone("US/Pacific")
            .with_path(format!("{config_dir}/test3"))
            .with_mock_url(server.url())
            .mock_select(0);

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
            .mock("GET", "/api/v1/tasks/filter?query=today&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let mut config_with_timezone = config
            .with_timezone("US/Pacific")
            .with_mock_url(server.url());
        let filter = String::from("today");
        let sort = &SortOrder::Value;

        let tasks = view(&mut config_with_timezone, Flag::Filter(filter), sort)
            .await
            .unwrap();

        assert!(tasks.contains("Tasks for today"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_view_with_project() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/tasks/?project_id=123&limit=200")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let mut config_with_timezone = config
            .with_timezone("US/Pacific")
            .with_mock_url(server.url());

        let binding = config_with_timezone.projects().await.unwrap();
        let project = binding.first().unwrap().clone();
        let sort = &SortOrder::Value;

        let tasks = view(&mut config_with_timezone, Flag::Project(project), sort)
            .await
            .unwrap();

        assert!(tasks.contains("Tasks for"));
        assert!(tasks.contains("- TEST\n"));
        mock.assert();
    }
}
