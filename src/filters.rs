use futures::future;

use crate::{
    color,
    config::Config,
    error::Error,
    input::{self},
    tasks::{self, FormatType, Task},
    todoist, SortOrder,
};

pub async fn edit_task(config: &Config, filter: String) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filters(config, &filter)
        .await?
        .into_iter()
        .flat_map(|(_, tasks)| tasks.to_owned())
        .collect::<Vec<Task>>();

    let task = input::select(input::TASK, tasks, config.mock_select)?;

    let options = tasks::edit_task_attributes();

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

/// Get the next task by priority and save its id to config
pub async fn next_task(config: Config, filter: &str) -> Result<String, Error> {
    match fetch_next_task(&config, filter).await {
        Ok(Some((task, remaining))) => {
            config.set_next_id(&task.id).save().await?;
            let task_string = task.fmt(&config, FormatType::Single, true, true).await?;
            Ok(format!("{task_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No tasks on list")),
        Err(e) => Err(e),
    }
}

async fn fetch_next_task(config: &Config, filter: &str) -> Result<Option<(Task, usize)>, Error> {
    let tasks = todoist::tasks_for_filters(config, filter)
        .await?
        .into_iter()
        .flat_map(|(_, tasks)| tasks.to_owned())
        .collect::<Vec<Task>>();

    let tasks = tasks::sort_by_value(tasks, config);

    Ok(tasks.first().map(|task| (task.to_owned(), tasks.len())))
}

/// Put dates on all tasks without dates
pub async fn schedule(config: &Config, filter: &String, sort: &SortOrder) -> Result<String, Error> {
    let tasks = todoist::tasks_for_filters(config, filter)
        .await?
        .into_iter()
        .flat_map(|(_, tasks)| tasks.to_owned())
        .collect::<Vec<Task>>();

    let tasks = tasks::sort(tasks, config, sort);

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in '{filter}'"
        )))
    } else {
        let mut handles = Vec::new();
        for task in tasks.iter() {
            if let Some(handle) = tasks::spawn_schedule_task(config.clone(), task.clone()).await? {
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
        let sort = &SortOrder::Value;
        let result = schedule(&config, &filter, sort);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        let config = config.mock_select(2);

        let filter = String::from("today");
        let result = schedule(&config, &filter, sort);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'today'".to_string())
        );

        mock.expect(2);
        mock2.expect(2);
    }
}
