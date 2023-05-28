use std::fmt::Display;

use crate::config::Config;
use crate::items::{FormatType, Item, Priority};
use crate::{config, items, projects, request};
use colored::*;
use serde::Deserialize;

const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

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

/// Add a project to the projects HashMap in Config
pub fn add(config: &mut Config, name: String, id: String) -> Result<String, String> {
    let id = id.parse::<u32>().or(Err(ADD_ERROR))?;

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
    let items = request::items_for_project(config, &project_id)?;
    let filtered_items = items::filter_not_in_future(items, config)?;
    let items = items::sort_by_value(filtered_items, config);

    Ok(items.first().map(|item| item.to_owned()))
}

/// Fetch projects and prompt to add them to config one by one
pub fn import(config: &mut Config) -> Result<String, String> {
    let projects = request::projects(config)?;
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
    match config::select_input("Select an option", options.clone()) {
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
    let items = request::items_for_project(&config, &project_id)?;
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
    match config::select_input("Select an option", options) {
        Ok(string) => {
            if string == "complete" {
                Some(request::complete_item(config))
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

    let items = request::items_for_project(config, &project_id)?;
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

/// All items for a project
pub fn all_items(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = request::items_for_project(config, &project_id)?;

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

    let items = request::items_for_project(config, &id)?;

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

    let items = request::items_for_project(config, &inbox_id)?;

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
            items::set_priority(config, item.to_owned());
        }
        Ok(format!("Successfully prioritized {project_name}")
            .green()
            .to_string())
    }
}

/// Put dates on all items without dates
pub fn schedule(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = request::items_for_project(config, &project_id)?;

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
            let due_string = config::get_input("Input a date in natural language or (c)omplete")?;
            match due_string.as_str() {
                "complete" | "c" => {
                    let config = config.set_next_id(&item.id);
                    request::complete_item(&config)?
                }
                _ => request::update_item_due(config, item.to_owned(), due_string)?,
            };
        }
        Ok(format!("Successfully dated {project_name}")
            .green()
            .to_string())
    }
}
pub fn move_item_to_project(config: &Config, item: Item) -> Result<String, String> {
    println!("{}", item.fmt(config, FormatType::Single));

    let mut options = config
        .projects
        .keys()
        .map(|k| k.to_owned())
        .collect::<Vec<String>>();

    options.push("complete".to_string());
    options.reverse();

    let project_name =
        config::select_input("Enter destination project name or complete:", options)?;

    match project_name.as_str() {
        "complete" => {
            request::complete_item(&config.set_next_id(&item.id))?;
            Ok(green_string("✓"))
        }
        _ => {
            let project_id = projects::project_id(config, &project_name)?;
            let sections = request::sections_for_project(config, &project_id)?;
            let section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
            if section_names.is_empty() {
                request::move_item_to_project(config, item, &project_name)
            } else {
                let section_name = config::select_input("Select section", section_names)?;
                let section_id = &sections
                    .iter()
                    .find(|x| x.name == section_name.as_str())
                    .expect("Section does not exist")
                    .id;
                request::move_item_to_section(config, item, section_id)
            }
        }
    }
}

/// Add item to project with natural language processing
pub fn add_item_to_project(
    config: &Config,
    content: String,
    project: &str,
    priority: Priority,
) -> Result<String, String> {
    let item = request::add_item_to_inbox(config, &content, priority)?;

    match project {
        "inbox" | "i" => Ok(green_string("✓")),
        project => {
            request::move_item_to_project(config, item, project)?;
            Ok(green_string("✓"))
        }
    }
}

pub fn green_string(str: &str) -> String {
    String::from(str).green().to_string()
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
        let config = Config::new("123123", None).unwrap().create().unwrap();

        let mut config = config;

        let result = add(&mut config, "cool_project".to_string(), "1".to_string());
        assert_eq!(result, Ok("✓".to_string()));

        let result = remove(config, "cool_project");
        assert_eq!(Ok("✓".to_string()), result);
    }
    #[test]
    fn should_list_projects() {
        let mut config = Config::new("123123", None).unwrap();

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

        let mut config = Config::new("12341234", Some(server.url())).unwrap();

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

        let mut config = Config::new("12341234", Some(server.url())).unwrap();
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

        let mut config = Config::new("12341234", Some(server.url())).unwrap();
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
    fn should_import_projects() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/rest/v2/projects")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::projects())
            .create();

        let mut config = Config::new("12341234", Some(server.url()))
            .unwrap()
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
}
