use crate::config::Config;
use crate::items::Item;
use crate::{config, items, projects, request};
use colored::*;

const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

/// List the projects in config
pub fn list(config: Config) -> Result<String, String> {
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
pub fn add(config: Config, params: Vec<String>) -> Result<String, String> {
    let mut params = params;
    let num = params
        .pop()
        .ok_or(ADD_ERROR)?
        .parse::<u32>()
        .or(Err(ADD_ERROR))?;

    let name = params.pop().ok_or(ADD_ERROR)?;

    config.add_project(name, num).save()
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
    let project_id = projects::project_id(&config, project_name)?;
    let items = request::items_for_project(&config, &project_id)?;
    let filtered_items = items::filter_not_in_future(items, &config)?;
    let maybe_item = items::sort_by_value(filtered_items, &config)
        .first()
        .map(|item| item.to_owned());

    match maybe_item {
        Some(item) => {
            config.set_next_id(item.id.clone()).save()?;
            Ok(item.fmt(&config))
        }
        None => Ok(green_string("No items on list")),
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
        buffer.push_str(&item.fmt(config));
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
        buffer.push_str(&item.fmt(config));
    }
    Ok(buffer)
}

/// Empty the inbox by sending items to other projects one at a time
pub fn sort_inbox(config: Config) -> Result<String, String> {
    let inbox_id = projects::project_id(&config, "inbox")?;

    let items = request::items_for_project(&config, &inbox_id)?;

    if items.is_empty() {
        Ok(green_string("No tasks to sort in inbox"))
    } else {
        projects::list(config.clone())?;
        for item in items.iter() {
            move_item_to_project(config.clone(), item.to_owned())?;
        }
        Ok(green_string("Successfully sorted inbox"))
    }
}

/// Prioritize all unprioritized items in a project
pub fn prioritize_items(config: &Config, project_name: &str) -> Result<String, String> {
    let inbox_id = projects::project_id(config, project_name)?;

    let items = request::items_for_project(config, &inbox_id)?;

    let unprioritized_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.priority == 1)
        .collect::<Vec<Item>>();

    if unprioritized_items.is_empty() {
        Ok(format!("No tasks to prioritize in {project_name}")
            .green()
            .to_string())
    } else {
        for item in unprioritized_items.iter() {
            items::set_priority(config.clone(), item.to_owned());
        }
        Ok(format!("Successfully prioritized {project_name}")
            .green()
            .to_string())
    }
}

/// Prioritize all unprioritized items in a project
pub fn edit_items(config: &Config, project_name: &str) -> Result<String, String> {
    let inbox_id = projects::project_id(config, project_name)?;

    let items = request::items_for_project(config, &inbox_id)?;

    if items.is_empty() {
        Ok(format!("No tasks to edit in {project_name}")
            .green()
            .to_string())
    } else {
        for item in items.iter() {
            items::edit_item(config.clone(), item.to_owned());
        }
        Ok(format!("Successfully edited items in {project_name}")
            .green()
            .to_string())
    }
}

pub fn move_item_to_project(config: Config, item: Item) -> Result<String, String> {
    println!("{}", item.fmt(&config));

    let project_name = config::get_input("Enter destination project name or (c)omplete:")?;

    match project_name.as_str() {
        "complete" | "c" => {
            request::complete_item(config.set_next_id(item.id))?;
            Ok(green_string("✓"))
        }
        _ => {
            request::move_item(config, item, &project_name)?;
            Ok(green_string("✓"))
        }
    }
}

/// Add item to project with natural language processing
pub fn add_item_to_project(config: Config, task: &str, project: &str) -> Result<String, String> {
    let item = request::add_item_to_inbox(&config, task)?;

    match project {
        "inbox" | "i" => Ok(green_string("✓")),
        project => {
            request::move_item(config, item, project)?;
            Ok(green_string("✓"))
        }
    }
}

fn green_string(str: &str) -> String {
    String::from(str).green().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use mockito;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_list_projects() {
        let config = Config::new("123123")
            .unwrap()
            .add_project(String::from("first"), 1)
            .add_project(String::from("second"), 2);

        let str = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mProjects\u{1b}[0m\n - first\n - second"
        } else {
            "Projects\n - first\n - second"
        };

        assert_eq!(list(config), Ok(String::from(str)));
    }

    #[test]
    fn should_display_scheduled_items() {
        let _m = mockito::mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::items())
            .create();

        let config = Config::new("12341234")
            .unwrap()
            .add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            ..config
        };

        assert_eq!(
            scheduled_items(&config_with_timezone, "test"),
            Err(String::from(
                "Project test not found, please add it to config"
            ))
        );

        let str = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mSchedule for good\u{1b}[0m\n\n\u{1b}[33mPut out recycling\u{1b}[0m\nDue: 15:59 ↻"
        } else {
            "Schedule for good\n\nPut out recycling\nDue: 15:59 ↻"
        };
        assert_eq!(
            scheduled_items(&config_with_timezone, "good"),
            Ok(String::from(str))
        );
    }

    #[test]
    fn should_list_all_items() {
        let _m = mockito::mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::items())
            .create();

        let config = Config::new("12341234")
            .unwrap()
            .add_project(String::from("good"), 1);

        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            ..config
        };

        let str = if test::helpers::supports_coloured_output() {
            "\u{1b}[32mTasks for good\u{1b}[0m\n\n\u{1b}[33mPut out recycling\u{1b}[0m\nDue: 15:59 ↻"
        } else {
            "Tasks for good\n\nPut out recycling\nDue: 15:59 ↻"
        };
        assert_eq!(
            all_items(&config_with_timezone, "good"),
            Ok(String::from(str))
        );
    }
}
