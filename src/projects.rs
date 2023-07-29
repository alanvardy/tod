use pad::PadStr;
use std::fmt::Display;

use crate::config::Config;
use crate::input::DateTimeInput;
use crate::items::priority::Priority;
use crate::items::{FormatType, Item};
use crate::{color, input, items, projects, todoist};
use rayon::prelude::*;
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
    /// Is a recurring task
    Recurring,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.name, self.url)
    }
}
pub fn json_to_projects(json: String) -> Result<Vec<Project>, String> {
    let result: Result<Vec<Project>, _> = serde_json::from_str(&json);
    match result {
        Ok(projects) => Ok(projects),
        Err(err) => Err(format!("Could not parse response for project: {err:?}")),
    }
}

/// List the projects in config with task counts
pub fn list(config: &mut Config) -> Result<String, String> {
    config.reload_projects()?;

    if let Some(projects) = config.projects.clone() {
        let mut projects = projects
            .par_iter()
            .map(|p| project_name_with_count(config, p))
            .collect::<Vec<String>>();
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
fn project_name_with_count(config: &Config, project: &Project) -> String {
    let count = match count_processable_items(config, project) {
        Ok(num) => format!("{}", num),
        Err(_) => String::new(),
    };

    format!("{}{}", project.name.pad_to_width(PAD_WIDTH), count)
}

/// Gets the number of items for a project that are not in the future
fn count_processable_items(config: &Config, project: &Project) -> Result<u8, String> {
    let all_items = todoist::items_for_project(config, project)?;
    let count = items::filter_not_in_future(all_items, config)?.len();

    Ok(count as u8)
}

/// Add a project to the projects HashMap in Config
pub fn add(config: &mut Config, project: &Project) -> Result<String, String> {
    config.add_project(project.clone());
    config.save()
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: &mut Config, project: &Project) -> Result<String, String> {
    config.remove_project(project);
    config.save()
}

/// Rename a project in config
pub fn rename(config: Config, project: &Project) -> Result<String, String> {
    let new_name = input::string_with_default("Input new project name", &project.name)?;

    let mut config = config;

    let new_project = Project {
        name: new_name,
        ..project.clone()
    };
    add(&mut config, &new_project)?;
    remove(&mut config, project)
}

/// Get the next item by priority and save its id to config
pub fn next(config: Config, project: &Project) -> Result<String, String> {
    match fetch_next_item(&config, project) {
        Ok(Some((item, remaining))) => {
            config.set_next_id(&item.id).save()?;
            let item_string = item.fmt(&config, FormatType::Single);
            Ok(format!("{item_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No items on list")),
        Err(e) => Err(e),
    }
}

fn fetch_next_item(config: &Config, project: &Project) -> Result<Option<(Item, usize)>, String> {
    let items = todoist::items_for_project(config, project)?;
    let filtered_items = items::filter_not_in_future(items, config)?;
    let items = items::sort_by_value(filtered_items, config);

    Ok(items.first().map(|item| (item.to_owned(), items.len())))
}

/// Removes all projects from config that don't exist in Todoist
pub fn remove_auto(config: &mut Config) -> Result<String, String> {
    let projects = todoist::projects(config)?;
    let missing_projects = filter_missing_projects(config, projects);

    if missing_projects.is_empty() {
        return Ok(color::green_string("No projects to auto remove"));
    }

    for project in &missing_projects {
        config.remove_project(project);
    }
    config.save()?;
    let project_names = missing_projects
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let message = format!("Auto removed: {project_names}");
    Ok(color::green_string(&message))
}

/// Removes all projects from config
pub fn remove_all(config: &mut Config) -> Result<String, String> {
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
    config.save()?;
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
        .clone()
        .into_iter()
        .filter(|p| !project_ids.contains(&p.id))
        .collect()
}

/// Fetch projects and prompt to add them to config one by one
pub fn import(config: &mut Config) -> Result<String, String> {
    let projects = todoist::projects(config)?;
    let new_projects = filter_new_projects(config, projects);
    for project in new_projects {
        maybe_add_project(config, project)?;
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
fn maybe_add_project(config: &mut Config, project: Project) -> Result<String, String> {
    let options = vec!["add", "skip"];
    println!("{}", project);
    match input::select("Select an option", options.clone(), config.mock_select) {
        Ok(string) => {
            if string == "add" {
                add(config, &project)
            } else if string == "skip" {
                Ok(String::from("Skipped"))
            } else {
                Err(String::from("Invalid option"))
            }
        }
        Err(e) => Err(e)?,
    }
}

/// Get next items and give an interactive prompt for completing them one by one
pub fn process_items(config: Config, project: &Project) -> Result<String, String> {
    let items = todoist::items_for_project(&config, project)?;
    let items = items::filter_not_in_future(items, &config)?;
    for item in items {
        config.set_next_id(&item.id).save()?;
        match handle_item(&config.reload()?, item) {
            Some(Ok(_)) => (),
            Some(Err(e)) => return Err(e),
            None => return Ok(color::green_string("Exited")),
        }
    }
    let project_name = project.clone().name;
    Ok(color::green_string(&format!(
        "There are no more tasks in '{project_name}'"
    )))
}

fn handle_item(config: &Config, item: Item) -> Option<Result<String, String>> {
    let options = vec!["complete", "skip", "quit"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    println!("{}", item.fmt(config, FormatType::Single));
    match input::select("Select an option", options, config.mock_select) {
        Ok(string) => {
            if string == "complete" {
                Some(todoist::complete_item(config))
            } else if string == "skip" {
                Some(Ok(color::green_string("item skipped")))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    }
}

// Scheduled that are today and have a time on them (AKA appointments)
pub fn scheduled_items(config: &Config, project: &Project) -> Result<String, String> {
    let items = todoist::items_for_project(config, project)?;
    let filtered_items = items::filter_today_and_has_time(items, config);

    if filtered_items.is_empty() {
        return Ok(String::from("No scheduled items found"));
    }

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!(
        "Schedule for {}",
        project.name
    )));

    for item in items::sort_by_datetime(filtered_items, config) {
        buffer.push('\n');
        buffer.push_str(&item.fmt(config, FormatType::List));
    }
    Ok(buffer)
}

pub fn rename_items(config: &Config, project: &Project) -> Result<String, String> {
    let project_tasks = todoist::items_for_project(config, project)?;

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

    todoist::update_item_name(config, selected_task, new_task_content)
}

/// All items for a project
pub fn all_items(config: &Config, project: &Project) -> Result<String, String> {
    let items = todoist::items_for_project(config, project)?;

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!("Tasks for {}", project.name)));

    for item in items::sort_by_datetime(items, config) {
        buffer.push('\n');
        buffer.push_str(&item.fmt(config, FormatType::List));
    }
    Ok(buffer)
}

/// Empty a project by sending items to other projects one at a time
pub fn empty(config: &mut Config, project: &Project) -> Result<String, String> {
    let items = todoist::items_for_project(config, project)?;

    if items.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to empty from {}",
            project.name
        )))
    } else {
        projects::list(config)?;
        for item in items.iter() {
            move_item_to_project(config, item.to_owned())?;
        }
        Ok(color::green_string(&format!(
            "Successfully emptied {}",
            project.name
        )))
    }
}

/// Prioritize all unprioritized items in a project
pub fn prioritize_items(config: &Config, project: &Project) -> Result<String, String> {
    let items = todoist::items_for_project(config, project)?;

    let unprioritized_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.priority == Priority::None)
        .collect::<Vec<Item>>();

    if unprioritized_items.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to prioritize in {}",
            project.name
        )))
    } else {
        for item in unprioritized_items.iter() {
            items::set_priority(config, item.to_owned())?;
        }
        Ok(color::green_string(&format!(
            "Successfully prioritized {}",
            project.name
        )))
    }
}

/// Put dates on all items without dates
pub fn schedule(config: &Config, project: &Project, filter: TaskFilter) -> Result<String, String> {
    let items = todoist::items_for_project(config, project)?;

    let filtered_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.filter(config, &filter) && !item.filter(config, &TaskFilter::Recurring))
        .collect::<Vec<Item>>();

    if filtered_items.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in {}",
            project.name
        )))
    } else {
        for item in filtered_items.iter() {
            println!("{}", item.fmt(config, FormatType::Single));
            let datetime_input = input::datetime(config.mock_select, config.mock_string.clone())?;
            match datetime_input {
                input::DateTimeInput::Complete => {
                    let config = config.set_next_id(&item.id);
                    todoist::complete_item(&config)?
                }
                DateTimeInput::Skip => "Skipped".to_string(),

                input::DateTimeInput::Text(due_string) => {
                    todoist::update_item_due(config, item.to_owned(), due_string)?
                }
                input::DateTimeInput::None => {
                    todoist::update_item_due(config, item.to_owned(), "No Date".to_string())?
                }
            };
        }
        Ok(color::green_string(&format!(
            "Successfully scheduled tasks in {}",
            project.name
        )))
    }
}

pub fn move_item_to_project(config: &Config, item: Item) -> Result<String, String> {
    println!("{}", item.fmt(config, FormatType::Single));

    let options = vec!["Pick project", "Complete", "Skip"]
        .iter()
        .map(|o| o.to_string())
        .collect::<Vec<String>>();
    let selection = input::select(
        "Enter destination project name or complete:",
        options,
        config.mock_select,
    )?;

    match selection.as_str() {
        "Complete" => {
            todoist::complete_item(&config.set_next_id(&item.id))?;
            Ok(color::green_string("✓"))
        }
        "Skip" => Ok(color::green_string("Skipped")),
        _ => {
            let projects = config.projects.clone().unwrap_or_default();
            let project = input::select("Select project", projects, config.mock_select)?;

            let sections = todoist::sections_for_project(config, &project)?;
            let section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
            if section_names.is_empty() {
                todoist::move_item_to_project(config, item, &project)
            } else {
                let section_name =
                    input::select("Select section", section_names, config.mock_select)?;
                let section_id = &sections
                    .iter()
                    .find(|x| x.name == section_name.as_str())
                    .expect("Section does not exist")
                    .id;
                todoist::move_item_to_section(config, item, section_id)
            }
        }
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
    fn should_add_and_remove_projects() {
        let config = test::fixtures::config().create().unwrap();

        let mut config = config;
        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = remove(&mut config, project);
        assert_eq!(Ok("✓".to_string()), result);
        let result = add(&mut config, project);
        assert_eq!(Ok("✓".to_string()), result);
    }
    #[test]
    fn test_list() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::projects())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());

        let str = "Projects                           # Tasks\n - Doomsday                      ";

        assert_eq!(list(&mut config), Ok(String::from(str)));
        mock.expect(3);
    }

    #[test]
    fn test_get_next_item() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test2"),
            mock_url: Some(server.url()),
            ..config
        };
        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        config_with_timezone.clone().create().unwrap();

        assert_eq!(
            next(config_with_timezone, project),
            Ok(format!(
                "Put out recycling\nDue: {TIME} ↻\n1 task(s) remaining"
            ))
        );
    }

    #[test]
    fn test_scheduled_items() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        // valid project
        let result = scheduled_items(&config_with_timezone, project);
        assert_eq!(
            result,
            Ok(format!(
                "Schedule for myproject\n- Put out recycling\n  Due: {TIME} ↻"
            ))
        );
    }

    #[test]
    fn test_all_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        assert_eq!(
            all_items(&config_with_timezone, project),
            Ok(format!(
                "Tasks for myproject\n- Put out recycling\n  Due: {TIME} ↻"
            ))
        );
        mock.assert();
    }

    #[test]
    fn test_import() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::new_projects())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .unwrap();

        assert_eq!(import(&mut config), Ok("No more projects".to_string()));
        mock.assert();

        let config = config.reload().unwrap();
        let config_keys: Vec<String> = config
            .projects
            .unwrap_or_default()
            .iter()
            .map(|p| p.name.to_owned())
            .collect();
        assert!(config_keys.contains(&"Doomsday".to_string()))
    }

    #[test]
    fn test_handle_item() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let item = test::fixtures::item();
        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);
        let result = handle_item(&config, item);
        let expected = Some(Ok(String::from("✓")));
        assert_eq!(result, expected);
        mock.assert();
    }

    #[test]
    fn test_process_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
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

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = process_items(config, project);
        assert_eq!(
            result,
            Ok("There are no more tasks in 'myproject'".to_string())
        );
        mock.assert();
        mock2.assert();
    }

    #[test]
    fn test_remove_auto() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::new_projects())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .create()
            .unwrap();

        let result = remove_auto(&mut config);
        let expected: Result<String, String> = Ok(String::from("Auto removed: myproject"));
        assert_eq!(result, expected);
        mock.assert();
        let projects = config.projects.clone().unwrap_or_default();
        assert_eq!(projects.is_empty(), true);
    }

    #[test]
    fn test_remove_all() {
        let mut config = test::fixtures::config().mock_select(1).create().unwrap();

        let result = remove_all(&mut config);
        let expected: Result<String, String> = Ok(String::from("Removed all projects from config"));
        assert_eq!(result, expected);

        let projects = config.projects.clone().unwrap_or_default();
        assert_eq!(projects.is_empty(), true);
    }

    #[test]
    fn test_empty() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mock2 = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let mock3 = server
            .mock("GET", "/rest/v2/sections?project_id=123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sections())
            .create();

        let mock4 = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::projects())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_string("newtext")
            .mock_select(0);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = empty(&mut config, project);
        assert_eq!(result, Ok(String::from("Successfully emptied myproject")));
        mock.expect(2);
        mock2.assert();
        mock3.assert();
        mock4.assert();
    }

    #[test]
    fn test_prioritize_items_with_no_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = prioritize_items(&config, project);
        assert_eq!(
            result,
            Ok(String::from("No tasks to prioritize in myproject"))
        );
        mock.assert();
    }

    #[test]
    fn test_move_item_to_project() {
        let config = test::fixtures::config().mock_select(2);
        let item = test::fixtures::item();

        let result = move_item_to_project(&config, item);
        assert_eq!(result, Ok(String::from("Skipped")));
    }

    #[test]
    fn test_rename_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);
        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        let result = rename_items(&config, project);
        assert_eq!(
            result,
            Ok("The content is the same, no need to change it".to_string())
        );
        mock.assert();
    }
    #[test]
    fn test_schedule() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::unscheduled_items())
            .create();

        let mock2 = server
            .mock("POST", "/rest/v2/tasks/999999")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::item())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(1)
            .mock_string("tod");

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Unscheduled);
        assert_eq!(
            result,
            Ok("Successfully scheduled tasks in myproject".to_string())
        );

        let config = config.mock_select(2);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Overdue);
        assert_eq!(result, Ok("No tasks to schedule in myproject".to_string()));

        let config = config.mock_select(3);

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let result = schedule(&config, project, TaskFilter::Unscheduled);
        assert_eq!(
            result,
            Ok("Successfully scheduled tasks in myproject".to_string())
        );
        mock.expect(2);
        mock2.expect(2);
    }
}
