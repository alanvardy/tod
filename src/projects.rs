use crate::config::Config;
use crate::items::Item;
use crate::{config, items, projects, request};
use colored::Colorize;

const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

/// List the projects in config
pub fn list(config: Config) -> Result<String, String> {
    let mut projects: Vec<String> = config.projects.iter().map(|(k, _v)| k.to_owned()).collect();
    projects.sort();
    println!("== Projects ==");
    for key in projects {
        println!("{}", key);
    }
    Ok(String::from(""))
}

/// Add a project to the projects HashMap in Config
pub fn add(config: Config, params: Vec<&str>) -> Result<String, String> {
    let mut params = params.clone();
    let num = params
        .pop()
        .expect(ADD_ERROR)
        .parse::<u32>()
        .expect(ADD_ERROR);

    let name = params.pop().expect(ADD_ERROR);

    config.add_project(name, num).save()
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, project_name: &str) -> Result<String, String> {
    config.remove_project(project_name).save()
}

pub fn project_id(config: &Config, project_name: &str) -> String {
    let project_id = *config.projects.get(project_name).unwrap_or_else(|| {
        panic!(
            "Project {} not found, please add it to config",
            project_name
        )
    });

    project_id.to_string()
}

/// Get the next item by priority and save its id to config
pub fn next_item(config: Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name);
    let items = request::items_for_project(config.clone(), &project_id)?;
    let filtered_items = items::filter_not_in_future(items);
    let maybe_item = items::sort_by_priority(filtered_items)
        .first()
        .map(|item| item.to_owned());

    match maybe_item {
        Some(item) => {
            config
                .set_next_id(item.id)
                .save()
                .expect("could not set next_id");
            println!("{}", item);
            Ok(String::from(""))
        }
        None => Ok(String::from("No items on list")),
    }
}

// Scheduled that are today and have a time on them (AKA appointments)
pub fn scheduled_items(config: Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name);

    let items = request::items_for_project(config, &project_id)?;
    match items::filter_today_and_has_time(items) {
        results if !results.is_empty() => {
            println!("Schedule for {}", project_name.green());
            for item in items::sort_by_datetime(results) {
                println!("{}", item);
            }
            Ok(String::from(""))
        }
        _no_items => Ok(format!("No scheduled items for {}", project_name)),
    }
}

/// Empty the inbox by sending items to other projects one at a time
pub fn sort_inbox(config: Config) -> Result<String, String> {
    let inbox_id = projects::project_id(&config, "inbox");

    let items = request::items_for_project(config.clone(), &inbox_id)?;

    if items.is_empty() {
        Ok(String::from("No tasks to sort in inbox"))
    } else {
        projects::list(config.clone()).unwrap();
        for item in items.iter() {
            move_item_to_project(config.clone(), item.to_owned()).expect("Could not move item");
        }
        Ok(String::from("Successfully sorted inbox"))
    }
}

/// Prioritize all unprioritized items in a project
pub fn prioritize_items(config: Config, project_name: &str) -> Result<String, String> {
    let inbox_id = projects::project_id(&config, project_name);

    let items = request::items_for_project(config.clone(), &inbox_id)?;

    let unprioritized_items: Vec<Item> = items
        .into_iter()
        .filter(|item| item.priority == 1)
        .collect::<Vec<Item>>();

    if unprioritized_items.is_empty() {
        Ok(format!("No tasks to prioritize in {}", project_name))
    } else {
        for item in unprioritized_items.iter() {
            items::set_priority(config.clone(), item.to_owned());
        }
        Ok(format!("Successfully prioritized {}", project_name))
    }
}

pub fn move_item_to_project(config: Config, item: Item) -> Result<String, String> {
    println!("{}", item);

    let project_name = config::get_input("Enter destination project name or (c)omplete:");

    match project_name.as_str() {
        "complete" | "c" => {
            request::complete_item(config.set_next_id(item.id))?;
            Ok(String::from("✓"))
        }
        _ => {
            request::move_item(config, item, &project_name)?;
            Ok(String::from("✓"))
        }
    }
}

/// Add item to project with natural language processing
pub fn add_item_to_project(config: Config, task: &str, project: &str) -> Result<String, String> {
    let item = request::add_item_to_inbox(&config, task)?;

    match project {
        "inbox" | "i" => Ok(String::from("✓")),
        project => {
            request::move_item(config, item, project)?;
            Ok(String::from("✓"))
        }
    }
}
