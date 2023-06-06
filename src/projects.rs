use rayon::prelude::*;
use std::fmt::Display;

use crate::config::Config;
use crate::items::priority::Priority;
use crate::items::{FormatType, Item};
use crate::{input, items, projects, todoist};
use colored::*;
use serde::Deserialize;

const ADD_ERR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

const NO_PROJECTS_ERR: &str = "No projects in config, please run `tod project import`";

// Projects are split into sections
#[derive(PartialEq, Deserialize, Clone, Debug)]
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

/// List the projects in config
pub fn list(config: &Config) -> Result<String, String> {
    let result: Vec<String> = config
        .projects
        .par_iter()
        .map(|(k, _)| project_name_with_count(config, k))
        .collect::<Vec<String>>();
    dbg!(result);
    let mut projects: Vec<String> = config.projects.keys().map(|k| k.to_owned()).collect();
    if projects.is_empty() {
        return Ok(String::from("No projects found"));
    }
    projects.sort();
    let mut buffer = String::new();
    buffer.push_str(&green_string("Projects"));

    for key in projects {
        buffer.push_str("\n - ");
        buffer.push_str(&key);
    }
    Ok(buffer)
}

fn project_name_with_count(config: &Config, project_name: &str) -> String {
    let count = match get_item_count(config, project_name) {
        Ok(num) => format!("{}", num),
        Err(_) => String::new(),
    };

    let padding = 30 - project_name.len();

    format!("{project_name} {:padding$}", "")
}

fn get_item_count(config: &Config, project_name: &str) -> Result<u8, String> {
    let project_id = projects::project_id(config, project_name)?;

    let count = todoist::items_for_project(config, &project_id)?.len();

    Ok(count as u8)
}

/// Add a project to the projects HashMap in Config
pub fn add(config: &mut Config, name: String, id: String) -> Result<String, String> {
    let id = id.parse::<u32>().or(Err(ADD_ERR))?;

    config.add_project(name, id);
    config.save()
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, project_name: &str) -> Result<String, String> {
    config.remove_project(project_name).save()
}

pub fn project_id(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = config
        .projects
        .get(project_name)
        .ok_or(format!(
            "Project {project_name} not found, please add it to config"
        ))?
        .to_string();

    Ok(project_id)
}

/// Get the next item by priority and save its id to config
pub fn next_item(config: Config, project_name: &str) -> Result<String, String> {
    match fetch_next_item(&config, project_name) {
        Ok(Some(item)) => {
            config.set_next_id(&item.id).save()?;
            Ok(item.fmt(&config, FormatType::Single))
        }
        Ok(None) => Ok(green_string("No items on list")),
        Err(e) => Err(e),
    }
}

fn fetch_next_item(config: &Config, project_name: &str) -> Result<Option<Item>, String> {
    let project_id = projects::project_id(config, project_name)?;
    let items = todoist::items_for_project(config, &project_id)?;
    let filtered_items = items::filter_not_in_future(items, config)?;
    let items = items::sort_by_value(filtered_items, config);

    Ok(items.first().map(|item| item.to_owned()))
}

/// Fetch projects and prompt to add them to config one by one
pub fn import(config: &mut Config) -> Result<String, String> {
    let projects = todoist::projects(config)?;
    let new_projects = filter_new_projects(config, projects);
    for project in new_projects {
        maybe_add_project(config, project)?;
    }
    Ok(green_string("No more projects"))
}

/// Returns the projects that are not already in config
fn filter_new_projects(config: &Config, projects: Vec<Project>) -> Vec<Project> {
    let project_ids: Vec<String> = config.projects.values().map(|v| v.to_string()).collect();
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
                add(config, project.name, project.id)
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
pub fn process_items(config: Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name)?;
    let items = todoist::items_for_project(&config, &project_id)?;
    let items = items::filter_not_in_future(items, &config)?;
    for item in items {
        config.set_next_id(&item.id).save()?;
        match handle_item(&config.reload()?, item) {
            Some(Ok(_)) => (),
            Some(Err(e)) => return Err(e),
            None => return Ok(green_string("Exited")),
        }
    }
    Ok(green_string(&format!(
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
                Some(Ok(green_string("item skipped")))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    }
}

// Scheduled that are today and have a time on them (AKA appointments)
pub fn scheduled_items(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &project_id)?;
    let filtered_items = items::filter_today_and_has_time(items, config);

    if filtered_items.is_empty() {
        return Ok(String::from("No scheduled items found"));
    }

    let mut buffer = String::new();
    buffer.push_str(&green_string(&format!("Schedule for {project_name}")));

    for item in items::sort_by_datetime(filtered_items, config) {
        buffer.push('\n');
        buffer.push_str(&item.fmt(config, FormatType::List));
    }
    Ok(buffer)
}

pub fn rename_items(config: &Config, project_id: &str) -> Result<String, String> {
    let project_tasks = todoist::items_for_project(config, project_id)?;

    let selected_task = input::select(
        "Choose a task of the project:",
        project_tasks,
        config.mock_select,
    )?;
    let task_content = selected_task.content.as_str();

    let new_task_content = input::string_with_default("Edit the task you selected:", task_content)?;

    if task_content == new_task_content {
        return Ok(green_string(
            "The content is the same, no need to change it",
        ));
    }

    todoist::update_item_name(config, selected_task, new_task_content)
}

/// All items for a project
pub fn all_items(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &project_id)?;

    let mut buffer = String::new();
    buffer.push_str(&green_string(&format!("Tasks for {project_name}")));

    for item in items::sort_by_datetime(items, config) {
        buffer.push('\n');
        buffer.push_str(&item.fmt(config, FormatType::List));
    }
    Ok(buffer)
}

/// Empty a project by sending items to other projects one at a time
pub fn empty(config: &Config, project_name: &str) -> Result<String, String> {
    let id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &id)?;

    if items.is_empty() {
        Ok(green_string(&format!(
            "No tasks to empty from {project_name}"
        )))
    } else {
        projects::list(config)?;
        for item in items.iter() {
            move_item_to_project(config, item.to_owned())?;
        }
        Ok(green_string(&format!(
            "Successfully emptied {project_name}"
        )))
    }
}

/// Prioritize all unprioritized items in a project
pub fn prioritize_items(config: &Config, project_name: &str) -> Result<String, String> {
    let inbox_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &inbox_id)?;

    let unprioritized_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.priority == Priority::None)
        .collect::<Vec<Item>>();

    if unprioritized_items.is_empty() {
        Ok(format!("No tasks to prioritize in {project_name}")
            .green()
            .to_string())
    } else {
        for item in unprioritized_items.iter() {
            items::set_priority(config, item.to_owned())?;
        }
        Ok(format!("Successfully prioritized {project_name}")
            .green()
            .to_string())
    }
}

/// Put dates on all items without dates
pub fn schedule(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &project_id)?;

    let undated_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.has_no_date() || item.is_overdue(config))
        .collect::<Vec<Item>>();

    if undated_items.is_empty() {
        Ok(format!("No tasks to date in {project_name}")
            .green()
            .to_string())
    } else {
        for item in undated_items.iter() {
            println!("{}", item.fmt(config, FormatType::Single));
            let due_string = input::string(
                "Input a date in natural language, (s)kip or (c)omplete",
                config.mock_string.clone(),
            )?;
            match due_string.as_str() {
                "complete" | "c" => {
                    let config = config.set_next_id(&item.id);
                    todoist::complete_item(&config)?
                }
                "skip" | "s" => "Skipped".to_string(),

                _ => todoist::update_item_due(config, item.to_owned(), due_string)?,
            };
        }
        Ok(format!("Successfully dated {project_name}")
            .green()
            .to_string())
    }
}

pub fn move_item_to_project(config: &Config, item: Item) -> Result<String, String> {
    println!("{}", item.fmt(config, FormatType::Single));

    let mut options = project_names(config)?;
    options.reverse();
    options.push("skip".to_string());
    options.push("complete".to_string());
    options.reverse();

    let project_name = input::select(
        "Enter destination project name or complete:",
        options,
        config.mock_select,
    )?;

    match project_name.as_str() {
        "complete" => {
            todoist::complete_item(&config.set_next_id(&item.id))?;
            Ok(green_string("✓"))
        }
        "skip" => Ok(green_string("Skipped")),
        _ => {
            let project_id = projects::project_id(config, &project_name)?;
            let sections = todoist::sections_for_project(config, &project_id)?;
            let section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
            if section_names.is_empty() {
                todoist::move_item_to_project(config, item, &project_name)
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

/// Add item to project without natural language processing
pub fn add_item_to_project(
    config: &Config,
    content: String,
    project: &str,
    priority: Priority,
    description: String,
) -> Result<String, String> {
    let item = todoist::add_item(config, &content, priority, description)?;

    match project {
        "inbox" | "i" => Ok(green_string("✓")),
        project => {
            todoist::move_item_to_project(config, item, project)?;
            Ok(green_string("✓"))
        }
    }
}

pub fn green_string(str: &str) -> String {
    String::from(str).green().to_string()
}

pub fn project_names(config: &Config) -> Result<Vec<String>, String> {
    let mut names = config
        .projects
        .keys()
        .map(|k| k.to_owned())
        .collect::<Vec<String>>();
    names.sort();
    if names.is_empty() {
        Err(NO_PROJECTS_ERR.to_string())
    } else {
        Ok(names)
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

        let result = add(&mut config, "cool_project".to_string(), "1".to_string());
        assert_eq!(result, Ok("✓".to_string()));

        let result = remove(config, "cool_project");
        assert_eq!(Ok("✓".to_string()), result);
    }
    #[test]
    fn should_list_projects() {
        let mut config = test::fixtures::config();

        config.add_project(String::from("first"), 1);
        config.add_project(String::from("second"), 2);

        let str = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mProjects\u{1b}[0m\n - first\n - second"
        } else {
            "Projects\n - first\n - second"
        };

        assert_eq!(list(&config), Ok(String::from(str)));
    }

    #[test]
    fn should_get_next_item() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());

        config.add_project(String::from("good"), 1);

        let config_dir = dirs::config_dir().unwrap().to_str().unwrap().to_owned();

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            path: format!("{config_dir}/test2"),
            mock_url: Some(server.url()),
            ..config
        };

        config_with_timezone.clone().create().unwrap();

        let string = if test::helpers::supports_coloured_output() {
            format!("\u{1b}[33mPut out recycling\u{1b}[0m\nDue: {TIME} ↻")
        } else {
            format!("Put out recycling\nDue: {TIME} ↻")
        };

        assert_eq!(next_item(config_with_timezone, "good"), Ok(string));
    }

    #[test]
    fn should_display_scheduled_items() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());
        config.add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        assert_eq!(
            scheduled_items(&config_with_timezone, "test"),
            Err(String::from(
                "Project test not found, please add it to config"
            ))
        );

        let string = if test::helpers::supports_coloured_output() {
            format!("\u{1b}[32mSchedule for good\u{1b}[0m\n- \u{1b}[33mPut out recycling\u{1b}[0m\n  Due: {TIME} ↻")
        } else {
            format!("Schedule for good\n- Put out recycling\n  Due: {TIME} ↻")
        };
        let result = scheduled_items(&config_with_timezone, "good");
        assert_eq!(result, Ok(string));
    }

    #[test]
    fn should_list_all_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());
        config.add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        let string = if test::helpers::supports_coloured_output() {
            format!("\u{1b}[32mTasks for good\u{1b}[0m\n- \u{1b}[33mPut out recycling\u{1b}[0m\n  Due: {TIME} ↻")
        } else {
            format!("Tasks for good\n- Put out recycling\n  Due: {TIME} ↻")
        };
        assert_eq!(all_items(&config_with_timezone, "good"), Ok(string));
        mock.assert();
    }

    #[test]
    fn test_import() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::projects())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .unwrap();

        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mNo more projects\u{1b}[0m".to_string()
        } else {
            "No more projects".to_string()
        };
        assert_eq!(import(&mut config), Ok(string));
        mock.assert();

        let config = config.reload().unwrap();
        let config_keys: Vec<String> = config.projects.keys().map(|k| k.to_string()).collect();
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

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .unwrap();
        let project_name = String::from("Project2");
        config.add_project(project_name.clone(), 123);

        let result = process_items(config, &project_name);
        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mThere are no more tasks in 'Project2'\u{1b}[0m"
        } else {
            "There are no more tasks in 'Project2'"
        };
        assert_eq!(result, Ok(string.to_string()));
        mock.assert();
        mock2.assert();
    }

    #[test]
    fn test_project_names() {
        let mut config = test::fixtures::config();
        let result = project_names(&config);
        let expected = Err(String::from(NO_PROJECTS_ERR));
        assert_eq!(result, expected);

        config.add_project(String::from("NEWPROJECT"), 123);

        let result = project_names(&config);
        let expected: Result<Vec<String>, String> = Ok(vec![String::from("NEWPROJECT")]);
        assert_eq!(result, expected);
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

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_string("newtext")
            .mock_select(0);

        config.add_project(String::from("projectname"), 123);

        let result = empty(&config, "projectname");
        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mSuccessfully emptied projectname\u{1b}[0m"
        } else {
            "Successfully emptied projectname"
        };
        assert_eq!(result, Ok(String::from(string)));
        mock.assert();
        mock2.assert();
    }

    #[test]
    fn test_prioritize_items() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());

        config.add_project(String::from("projectname"), 123);

        let result = prioritize_items(&config, "projectname");
        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mNo tasks to prioritize in projectname\u{1b}[0m"
        } else {
            "No tasks to prioritize in projectname"
        };
        assert_eq!(result, Ok(String::from(string)));
        mock.assert();
    }

    #[test]
    fn test_move_item_to_project() {
        let mut config = test::fixtures::config().mock_select(1);
        let item = test::fixtures::item();
        config.add_project("projectname".to_string(), 123);

        let result = move_item_to_project(&config, item);
        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mSkipped\u{1b}[0m"
        } else {
            "Skipped"
        };
        assert_eq!(result, Ok(String::from(string)));
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

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);
        config.add_project("Project".to_string(), 123);

        let result = rename_items(&config, "123");
        let string = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mThe content is the same, no need to change it\u{1b}[0m"
        } else {
            "The content is the same, no need to change it"
        };
        assert_eq!(result, Ok(string.to_string()));
        mock.assert();
    }
}
