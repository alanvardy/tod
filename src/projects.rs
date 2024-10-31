use futures::future;
use pad::PadStr;
use std::fmt::Display;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::error::{self, Error};
use crate::input::DateTimeInput;
use crate::sections::Section;
use crate::tasks::priority::Priority;
use crate::tasks::{FormatType, Task};
use crate::{color, input, sections, tasks, todoist};
use serde::{Deserialize, Serialize};

const PAD_WIDTH: usize = 30;

// Projects are split into sections
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: String,
    pub comment_count: u8,
    pub order: u8,
    pub is_shared: bool,
    pub is_favorite: bool,
    pub is_inbox_project: bool,
    pub is_team_inbox: bool,
    pub view_style: String,
    pub url: String,
    pub parent_id: Option<String>,
}

pub enum TaskFilter {
    /// Does not have a date or datetime on it
    Unscheduled,
    /// Date or datetime is before today
    Overdue,
    /// Is a repeating task
    Recurring,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.name, self.url)
    }
}
pub fn json_to_projects(json: String) -> Result<Vec<Project>, Error> {
    let projects: Vec<Project> = serde_json::from_str(&json)?;
    Ok(projects)
}

/// List the projects in config with task counts
pub async fn list(config: &mut Config) -> Result<String, Error> {
    config.reload_projects().await?;

    if let Some(projects) = config.projects.clone() {
        let mut project_handles = Vec::new();

        for project in projects {
            let config = config.clone();
            let handle =
                tokio::spawn(async move { project_name_with_count(&config, &project).await });

            project_handles.push(handle);
        }

        let mut projects: Vec<String> = future::join_all(project_handles)
            .await
            .into_iter()
            .map(|p| p.unwrap_or_default())
            .collect();
        if projects.is_empty() {
            return Ok(String::from("No projects found"));
        }
        projects.sort();
        let mut buffer = String::new();
        buffer.push_str(&color::green_string("Projects").pad_to_width(PAD_WIDTH + 5));
        buffer.push_str(&color::green_string("# Tasks"));

        for key in projects {
            buffer.push_str("\n - ");
            buffer.push_str(&key);
        }
        Ok(buffer)
    } else {
        Ok(String::from("No projects found"))
    }
}

/// Formats a string with project name and the count that is a standard length
async fn project_name_with_count(config: &Config, project: &Project) -> String {
    let count = match count_processable_tasks(config, project).await {
        Ok(num) => format!("{}", num),
        Err(_) => String::new(),
    };

    format!("{}{}", project.name.pad_to_width(PAD_WIDTH), count)
}

/// Gets the number of tasks for a project that are not in the future
async fn count_processable_tasks(config: &Config, project: &Project) -> Result<u8, Error> {
    let all_tasks = todoist::tasks_for_project(config, project).await?;
    let count = tasks::filter_not_in_future(all_tasks, config)?.len();

    Ok(count as u8)
}

/// Add a project to the projects HashMap in Config
pub async fn add(config: &mut Config, project: &Project) -> Result<String, Error> {
    config.add_project(project.clone());
    config.save().await
}

/// Remove a project from the projects HashMap in Config
pub async fn remove(config: &mut Config, project: &Project) -> Result<String, Error> {
    config.remove_project(project);
    config.save().await
}

/// Rename a project in config
pub async fn rename(config: Config, project: &Project) -> Result<String, Error> {
    let new_name = input::string_with_default("Input new project name", &project.name)?;

    let mut config = config;

    let new_project = Project {
        name: new_name,
        ..project.clone()
    };
    remove(&mut config, project).await?;
    add(&mut config, &new_project).await
}

/// Get the next task by priority and save its id to config
pub async fn next_task(config: Config, project: &Project) -> Result<String, Error> {
    match fetch_next_task(&config, project).await {
        Ok(Some((task, remaining))) => {
            config.set_next_id(&task.id).save().await?;
            let task_string = task.fmt(&config, FormatType::Single, false);
            Ok(format!("{task_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No tasks on list")),
        Err(e) => Err(e),
    }
}

pub async fn label(
    config: &Config,
    project: &Project,
    labels: &Vec<String>,
) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;
    let mut handles = Vec::new();
    for task in tasks::sort_by_value(tasks, config) {
        let future = tasks::label_task(config, task, labels).await?;
        handles.push(future);
    }

    future::join_all(handles).await;
    Ok(color::green_string(&format!(
        "There are no more tasks for project: '{project}'"
    )))
}

async fn fetch_next_task(
    config: &Config,
    project: &Project,
) -> Result<Option<(Task, usize)>, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;
    let filtered_tasks = tasks::filter_not_in_future(tasks, config)?;
    let tasks = tasks::sort_by_value(filtered_tasks, config);

    Ok(tasks.first().map(|task| (task.to_owned(), tasks.len())))
}

/// Removes all projects from config that don't exist in Todoist
pub async fn remove_auto(config: &mut Config) -> Result<String, Error> {
    let projects = todoist::projects(config).await?;
    let missing_projects = filter_missing_projects(config, projects);

    if missing_projects.is_empty() {
        return Ok(color::green_string("No projects to auto remove"));
    }

    for project in &missing_projects {
        config.remove_project(project);
    }
    config.save().await?;
    let project_names = missing_projects
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let message = format!("Auto removed: '{project_names}'");
    Ok(color::green_string(&message))
}

/// Removes all projects from config
pub async fn remove_all(config: &mut Config) -> Result<String, Error> {
    let options = vec!["Cancel", "Confirm"];
    let selection = input::select(
        "Confirm removing all projects from config",
        options,
        config.mock_select,
    )?;

    if selection == "Cancel" {
        return Ok(String::from("Cancelled"));
    }

    if config.projects.clone().unwrap_or_default().is_empty() {
        return Ok(color::green_string("No projects to remove"));
    }

    for project in &config.projects.clone().unwrap_or_default() {
        config.remove_project(project);
    }
    config.save().await?;
    let message = String::from("Removed all projects from config");
    Ok(color::green_string(&message))
}

/// Returns the projects that are not already in config
fn filter_missing_projects(config: &Config, projects: Vec<Project>) -> Vec<Project> {
    let project_ids: Vec<String> = projects.into_iter().map(|v| v.id).collect();
    config
        .projects
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| !project_ids.contains(&p.id))
        .collect()
}

/// Fetch projects and prompt to add them to config one by one
pub async fn import(config: &mut Config, auto: &bool) -> Result<String, Error> {
    let projects = todoist::projects(config).await?;
    let new_projects = filter_new_projects(config, projects);
    for project in new_projects {
        maybe_add_project(config, project, auto).await?;
    }
    Ok(color::green_string("No more projects"))
}

/// Returns the projects that are not already in config
fn filter_new_projects(config: &Config, projects: Vec<Project>) -> Vec<Project> {
    let project_ids: Vec<String> = config
        .projects
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|v| v.id.clone())
        .collect();
    let new_projects: Vec<Project> = projects
        .into_iter()
        .filter(|p| !project_ids.contains(&p.id))
        .collect();

    new_projects
}

/// Prompt the user if they want to add project to config and maybe add
async fn maybe_add_project(
    config: &mut Config,
    project: Project,
    auto: &bool,
) -> Result<String, Error> {
    if *auto {
        println!("Adding {}", project);
        return add(config, &project).await;
    }

    let options = vec!["add", "skip"];
    println!("{}", project);
    match input::select("Select an option", options.clone(), config.mock_select) {
        Ok(string) => {
            if string == "add" {
                add(config, &project).await
            } else if string == "skip" {
                Ok(String::from("Skipped"))
            } else {
                Err(error::new("add_project", "Invalid option"))
            }
        }
        Err(e) => Err(e)?,
    }
}

/// Get next tasks and give an interactive prompt for completing them one by one
pub async fn process_tasks(config: &Config, project: &Project) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;
    let tasks = tasks::filter_not_in_future(tasks, config)?;
    let tasks = tasks::sort_by_value(tasks, config);
    let tasks = tasks::reject_parent_tasks(tasks, config).await;
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        println!(" ");
        match tasks::process_task(&config.reload().await?, task, &mut task_count, false).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    let project_name = project.clone().name;
    Ok(color::green_string(&format!(
        "There are no more tasks in '{project_name}'"
    )))
}

// Gives all tasks durations
pub async fn timebox_tasks(config: &Config, project: &Project) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;
    let tasks = tasks::sort_by_value(tasks, config);
    let tasks = tasks
        .iter()
        .filter(|t| t.duration.is_none())
        .map(|t| t.to_owned())
        .collect::<Vec<Task>>();
    let mut task_count = tasks.len() as i32;
    let mut handles = Vec::new();
    for task in tasks {
        match tasks::timebox_task(&config.reload().await?, task, &mut task_count, false).await {
            Some(handle) => handles.push(handle),
            None => return Ok(color::green_string("Exited")),
        }
    }
    future::join_all(handles).await;
    let project_name = project.clone().name;
    Ok(color::green_string(&format!(
        "There are no more tasks in '{project_name}'"
    )))
}

pub async fn rename_task(config: &Config, project: &Project) -> Result<String, Error> {
    let project_tasks = todoist::tasks_for_project(config, project).await?;

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

    todoist::update_task_name(config, selected_task, new_task_content).await
}

/// All tasks for a project
pub async fn all_tasks(config: &Config, project: &Project) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!(
        "Tasks for '{}'",
        project.name
    )));

    for task in tasks::sort_by_datetime(tasks, config) {
        buffer.push('\n');
        buffer.push_str(&task.fmt(config, FormatType::List, false));
    }
    Ok(buffer)
}

/// Empty a project by sending tasks to other projects one at a time
pub async fn empty(config: &mut Config, project: &Project) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;

    if tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to empty from '{}'",
            project.name
        )))
    } else {
        let sections = sections::all_sections(config).await;

        let tasks = tasks
            .into_iter()
            .filter(|task| task.parent_id.is_none())
            .collect::<Vec<Task>>();

        let mut handles = Vec::new();
        for task in tasks.iter() {
            match move_task_to_project(config, task.to_owned(), &sections).await {
                Ok(handle) => handles.push(handle),
                Err(e) => return Err(e),
            };
        }
        future::join_all(handles).await;
        Ok(color::green_string(&format!(
            "Successfully emptied '{}'",
            project.name
        )))
    }
}

/// Prioritize all unprioritized tasks in a project
pub async fn prioritize_tasks(config: &Config, project: &Project) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;

    let unprioritized_tasks: Vec<Task> = tasks
        .into_iter()
        .filter(|task| task.priority == Priority::None)
        .collect::<Vec<Task>>();

    if unprioritized_tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to prioritize in '{}'",
            project.name
        )))
    } else {
        let mut handles = Vec::new();
        for task in unprioritized_tasks.iter() {
            let handle = tasks::set_priority(config, task.to_owned(), false).await?;
            handles.push(handle);
        }

        future::join_all(handles).await;
        Ok(color::green_string(&format!(
            "Successfully prioritized '{}'",
            project.name
        )))
    }
}

/// Put dates on all tasks without dates
pub async fn schedule(
    config: &Config,
    project: &Project,
    filter: TaskFilter,
    skip_recurring: bool,
) -> Result<String, Error> {
    let tasks = todoist::tasks_for_project(config, project).await?;

    let filtered_tasks: Vec<Task> = if skip_recurring {
        tasks
            .into_iter()
            .filter(|task| {
                task.filter(config, &filter) && !task.filter(config, &TaskFilter::Recurring)
            })
            .collect::<Vec<Task>>()
    } else {
        tasks
            .into_iter()
            .filter(|task| task.filter(config, &filter))
            .collect::<Vec<Task>>()
    };

    if filtered_tasks.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in '{}'",
            project.name
        )))
    } else {
        let mut handles = Vec::new();
        for task in filtered_tasks.iter() {
            println!("{}", task.fmt(config, FormatType::Single, false));
            let datetime_input = input::datetime(
                config.mock_select,
                config.mock_string.clone(),
                config.natural_language_only,
            )?;
            match datetime_input {
                input::DateTimeInput::Complete => {
                    let handle = tasks::spawn_complete_task(config.clone(), task.clone());
                    handles.push(handle);
                }

                DateTimeInput::Skip => (),

                input::DateTimeInput::Text(due_string) => {
                    let handle = tasks::spawn_update_task_due(
                        config.clone(),
                        task.clone(),
                        due_string,
                        None,
                    );
                    handles.push(handle);
                }
                input::DateTimeInput::None => {
                    let handle = tasks::spawn_update_task_due(
                        config.clone(),
                        task.clone(),
                        "No date".to_string(),
                        None,
                    );
                    handles.push(handle);
                }
            };
        }

        future::join_all(handles).await;
        Ok(color::green_string(&format!(
            "Successfully scheduled tasks in '{}'",
            project.name
        )))
    }
}

pub async fn move_task_to_project(
    config: &Config,
    task: Task,
    sections: &[Section],
) -> Result<JoinHandle<()>, Error> {
    println!("{}", task.fmt(config, FormatType::Single, false));

    let options = ["Pick project", "Complete", "Skip", "Delete"]
        .iter()
        .map(|o| o.to_string())
        .collect::<Vec<String>>();
    let selection = input::select("Choose", options, config.mock_select)?;

    match selection.as_str() {
        "Complete" => Ok(tasks::spawn_complete_task(config.clone(), task)),

        "Delete" => Ok(tasks::spawn_delete_task(config.clone(), task)),
        "Skip" => Ok(tokio::spawn(async move {})),
        _ => {
            let projects = config.projects.clone().unwrap_or_default();
            let project = input::select("Select project", projects, config.mock_select)?;

            let sections: Vec<Section> = sections
                .iter()
                .filter(|s| s.project_id == project.id)
                .cloned()
                .collect();

            let section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
            if section_names.is_empty() || config.no_sections.unwrap_or_default() {
                let config = config.clone();
                Ok(tokio::spawn(async move {
                    if let Err(e) =
                        todoist::move_task_to_project(&config, task, &project, false).await
                    {
                        config.tx().send(e).unwrap();
                    }
                }))
            } else {
                let section_name =
                    input::select("Select section", section_names, config.mock_select)?;
                let section = sections
                    .iter()
                    .find(|x| x.name == section_name.as_str())
                    .expect("Section does not exist")
                    .clone();
                let config = config.clone();
                Ok(tokio::spawn(async move {
                    if let Err(e) =
                        todoist::move_task_to_section(&config, task, &section, false).await
                    {
                        config.tx().send(e).unwrap();
                    }
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn should_add_and_remove_projects() {
        let config = test::fixtures::config().await.create().await.unwrap();

        let mut config = config;
        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = remove(&mut config, project).await;
        assert_eq!(Ok("✓".to_string()), result);
        let result = add(&mut config, project).await;
        assert_eq!(Ok("✓".to_string()), result);
    }
    #[tokio::test]
    async fn test_list() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::projects())
            .create_async()
            .await;

        let mut config = test::fixtures::config().await.mock_url(server.url());

        let str = "Projects                           # Tasks\n - Doomsday                      ";

        assert_eq!(list(&mut config).await, Ok(String::from(str)));
        mock.expect(3);
    }

    #[tokio::test]
    async fn test_get_next_task() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test2"),
            mock_url: Some(server.url()),
            ..config
        };
        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        config_with_timezone.clone().create().await.unwrap();

        let task = next_task(config_with_timezone, project).await.unwrap();

        assert!(task.contains("Put out recycling"));
        assert!(task.contains("1 task(s) remaining"));
    }

    #[tokio::test]
    async fn test_all_tasks() {
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
        let project = binding.first().unwrap();

        let tasks = all_tasks(&config_with_timezone, project).await.unwrap();

        assert!(tasks.contains("Tasks for 'myproject'\n"));
        assert!(tasks.contains("- Put out recycling\n"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_import() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::new_projects())
            .create_async()
            .await;

        let mut config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .await
            .unwrap();

        assert_eq!(
            import(&mut config, &false).await,
            Ok("No more projects".to_string())
        );
        mock.assert();

        let config = config.reload().await.unwrap();
        let config_keys: Vec<String> = config
            .projects
            .unwrap_or_default()
            .iter()
            .map(|p| p.name.to_owned())
            .collect();
        assert!(config_keys.contains(&"Doomsday".to_string()))
    }

    #[tokio::test]
    async fn test_process_tasks() {
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
        let project = binding.first().unwrap();

        let result = process_tasks(&config, project).await;
        assert_eq!(
            result,
            Ok("There are no more tasks in 'myproject'".to_string())
        );
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_remove_auto() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::new_projects())
            .create_async()
            .await;

        let mut config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .create()
            .await
            .unwrap();

        let result = remove_auto(&mut config);
        let expected: Result<String, Error> = Ok(String::from("Auto removed: 'myproject'"));
        assert_eq!(result.await, expected);
        mock.assert();
        let projects = config.projects.clone().unwrap_or_default();
        assert_eq!(projects.is_empty(), true);
    }

    #[tokio::test]
    async fn test_remove_all() {
        let mut config = test::fixtures::config()
            .await
            .mock_select(1)
            .create()
            .await
            .unwrap();

        let result = remove_all(&mut config).await;
        let expected: Result<String, Error> = Ok(String::from("Removed all projects from config"));
        assert_eq!(result, expected);

        let projects = config.projects.clone().unwrap_or_default();
        assert_eq!(projects.is_empty(), true);
    }

    #[tokio::test]
    async fn test_empty() {
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

        let mock3 = server
            .mock("GET", "/rest/v2/sections?project_id=123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sections())
            .create_async()
            .await;

        let mut config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_string("newtext")
            .mock_select(0);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = empty(&mut config, project);
        assert_eq!(
            result.await,
            Ok(String::from("Successfully emptied 'myproject'"))
        );
        mock.expect(2);
        mock2.assert();
        mock3.assert();
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
        let project = binding.first().unwrap();

        let result = prioritize_tasks(&config, project);
        assert_eq!(
            result.await,
            Ok(String::from("No tasks to prioritize in 'myproject'"))
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_move_task_to_project() {
        let config = test::fixtures::config().await.mock_select(2);
        let task = test::fixtures::task();
        let sections: Vec<Section> = Vec::new();

        move_task_to_project(&config, task, &sections)
            .await
            .unwrap()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_rename_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks().await)
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(0);
        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = rename_task(&config, project);
        assert_eq!(
            result.await,
            Ok("The content is the same, no need to change it".to_string())
        );
        mock.assert();
    }
    #[tokio::test]
    async fn test_schedule() {
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
            .mock_string("tod");

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Unscheduled, false);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'myproject'".to_string())
        );

        let config = config.mock_select(2);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Overdue, false);
        assert_eq!(
            result.await,
            Ok("No tasks to schedule in 'myproject'".to_string())
        );

        let config = config.mock_select(3);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Unscheduled, false);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'myproject'".to_string())
        );

        let result = schedule(&config, project, TaskFilter::Unscheduled, true);
        assert_eq!(
            result.await,
            Ok("Successfully scheduled tasks in 'myproject'".to_string())
        );
        mock.expect(2);
        mock2.expect(2);
    }
    #[tokio::test]
    async fn test_timebox_tasks() {
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
        let project = binding.first().unwrap();
        let result = timebox_tasks(&config, project);
        assert_eq!(result.await, Ok("Exited".to_string()));

        let config = config.mock_select(2);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = timebox_tasks(&config, project);
        assert_eq!(result.await, Ok("Exited".to_string()));

        let config = config.mock_select(3);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = timebox_tasks(&config, project).await;
        assert_eq!(result, Ok("Exited".to_string()));

        let result = timebox_tasks(&config, project).await;
        assert_eq!(result, Ok("Exited".to_string()));
        mock.expect(2);
        mock2.expect(2);
    }
}
