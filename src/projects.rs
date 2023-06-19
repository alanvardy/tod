use pad::PadStr;
use rayon::prelude::*;
use std::fmt::Display;

use crate::config::Config;
use crate::items::priority::Priority;
use crate::items::{FormatType, Item};
use crate::{color, input, items, projects, todoist};
use serde::Deserialize;

const ADD_ERR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

const NO_PROJECTS_ERR: &str = "No projects in config, please run `tod project import`";

const PAD_WIDTH: usize = 30;

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
pub fn list(config: &Config) -> Result<String, String> {
    let mut projects: Vec<String> = config
        .projects
        .par_iter()
        .map(|(k, _)| project_name_with_count(config, k))
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
}

/// Formats a string with project name and the count that is a standard length
fn project_name_with_count(config: &Config, project_name: &str) -> String {
    let count = match count_processable_items(config, project_name) {
        Ok(num) => format!("{}", num),
        Err(_) => String::new(),
    };

    format!(
        "{}{}",
        project_name.to_owned().pad_to_width(PAD_WIDTH),
        count
    )
}

/// Gets the number of items for a project that are not in the future
fn count_processable_items(config: &Config, project_name: &str) -> Result<u8, String> {
    let project_id = projects::project_id(config, project_name)?;

    let all_items = todoist::items_for_project(config, &project_id)?;
    let count = items::filter_not_in_future(all_items, config)?.len();

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

/// Rename a project in config
pub fn rename(config: Config, project_name: &str) -> Result<String, String> {
    let new_name = input::string_with_default("Input new project name", project_name)?;

    let project_id = project_id(&config, project_name)?;
    let mut config = config;

    add(&mut config, new_name, project_id)?;
    remove(config, project_name)
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
pub fn next(config: Config, project_name: &str) -> Result<String, String> {
    match fetch_next_item(&config, project_name) {
        Ok(Some((item, remaining))) => {
            config.set_next_id(&item.id).save()?;
            let item_string = item.fmt(&config, FormatType::Single);
            Ok(format!("{item_string}\n{remaining} task(s) remaining"))
        }
        Ok(None) => Ok(color::green_string("No items on list")),
        Err(e) => Err(e),
    }
}

fn fetch_next_item(config: &Config, project_name: &str) -> Result<Option<(Item, usize)>, String> {
    let project_id = projects::project_id(config, project_name)?;
    let items = todoist::items_for_project(config, &project_id)?;
    let filtered_items = items::filter_not_in_future(items, config)?;
    let items = items::sort_by_value(filtered_items, config);

    Ok(items.first().map(|item| (item.to_owned(), items.len())))
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
            None => return Ok(color::green_string("Exited")),
        }
    }
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
pub fn scheduled_items(config: &Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &project_id)?;
    let filtered_items = items::filter_today_and_has_time(items, config);

    if filtered_items.is_empty() {
        return Ok(String::from("No scheduled items found"));
    }

    let mut buffer = String::new();
    buffer.push_str(&color::green_string(&format!(
        "Schedule for {project_name}"
    )));

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
        return Ok(color::green_string(
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
    buffer.push_str(&color::green_string(&format!("Tasks for {project_name}")));

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
        Ok(color::green_string(&format!(
            "No tasks to empty from {project_name}"
        )))
    } else {
        projects::list(config)?;
        for item in items.iter() {
            move_item_to_project(config, item.to_owned())?;
        }
        Ok(color::green_string(&format!(
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
        Ok(color::green_string(&format!(
            "No tasks to prioritize in {project_name}"
        )))
    } else {
        for item in unprioritized_items.iter() {
            items::set_priority(config, item.to_owned())?;
        }
        Ok(color::green_string(&format!(
            "Successfully prioritized {project_name}"
        )))
    }
}

/// Put dates on all items without dates
pub fn schedule(config: &Config, project_name: &str, filter: TaskFilter) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;

    let items = todoist::items_for_project(config, &project_id)?;

    let filtered_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.filter(config, &filter) && !item.filter(config, &TaskFilter::Recurring))
        .collect::<Vec<Item>>();

    if filtered_items.is_empty() {
        Ok(color::green_string(&format!(
            "No tasks to schedule in {project_name}"
        )))
    } else {
        for item in filtered_items.iter() {
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
        Ok(color::green_string(&format!(
            "Successfully scheduled tasks in {project_name}"
        )))
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
            Ok(color::green_string("✓"))
        }
        "skip" => Ok(color::green_string("Skipped")),
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
    due: Option<String>,
) -> Result<String, String> {
    let item = todoist::add_item(config, &content, priority, description, due)?;

    match project {
        "inbox" | "i" => Ok(color::green_string("✓")),
        project => {
            todoist::move_item_to_project(config, item, project)?;
            Ok(color::green_string("✓"))
        }
    }
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
    fn test_list() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::items())
            .create();

        let mut config = test::fixtures::config().mock_url(server.url());

        config.add_project(String::from("first"), 1);
        config.add_project(String::from("second"), 2);

        let str = "Projects                           # Tasks\n - first                         1\n - second                        1";

        assert_eq!(list(&config), Ok(String::from(str)));
        mock.expect(2);
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

        assert_eq!(
            next(config_with_timezone, "good"),
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

        let mut config = test::fixtures::config().mock_url(server.url());
        config.add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        // invalid project
        assert_eq!(
            scheduled_items(&config_with_timezone, "test"),
            Err(String::from(
                "Project test not found, please add it to config"
            ))
        );

        // valid project
        let result = scheduled_items(&config_with_timezone, "good");
        assert_eq!(
            result,
            Ok(format!(
                "Schedule for good\n- Put out recycling\n  Due: {TIME} ↻"
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

        let mut config = test::fixtures::config().mock_url(server.url());
        config.add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            mock_url: Some(server.url()),
            ..config
        };

        assert_eq!(
            all_items(&config_with_timezone, "good"),
            Ok(format!(
                "Tasks for good\n- Put out recycling\n  Due: {TIME} ↻"
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
            .with_body(test::responses::projects())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .create()
            .unwrap();

        assert_eq!(import(&mut config), Ok("No more projects".to_string()));
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
        assert_eq!(
            result,
            Ok("There are no more tasks in 'Project2'".to_string())
        );
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
        assert_eq!(result, Ok(String::from("Successfully emptied projectname")));
        mock.expect(2);
        mock2.assert();
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

        let mut config = test::fixtures::config().mock_url(server.url());

        config.add_project(String::from("projectname"), 123);

        let result = prioritize_items(&config, "projectname");
        assert_eq!(
            result,
            Ok(String::from("No tasks to prioritize in projectname"))
        );
        mock.assert();
    }

    #[test]
    fn test_move_item_to_project() {
        let mut config = test::fixtures::config().mock_select(1);
        let item = test::fixtures::item();
        config.add_project("projectname".to_string(), 123);

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

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);
        config.add_project("Project".to_string(), 123);

        let result = rename_items(&config, "123");
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

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0)
            .mock_string("tod");
        config.add_project("Project".to_string(), 123);

        let result = schedule(&config, "Project", TaskFilter::Unscheduled);
        assert_eq!(
            result,
            Ok("Successfully scheduled tasks in Project".to_string())
        );

        let result = schedule(&config, "Project", TaskFilter::Overdue);
        assert_eq!(result, Ok("No tasks to schedule in Project".to_string()));
        mock.expect(2);
        mock2.expect(2);
    }

    #[test]
    fn test_add_item_to_project() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/rest/v2/tasks/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::item())
            .create();

        let mock2 = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let mut config = test::fixtures::config()
            .mock_url(server.url())
            .mock_select(0);
        config.add_project("Project".to_string(), 123);

        let content = String::from("This is content");

        let result = add_item_to_project(
            &config,
            content,
            "Project",
            Priority::None,
            String::new(),
            None,
        );
        assert_eq!(result, Ok("✓".to_string()));

        mock.assert();
        mock2.assert();
    }
}
