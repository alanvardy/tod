use crate::config::Config;
use crate::items::Item;
use crate::{items, projects, request};

const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

/// List the projects in config
pub fn list(config: Config) -> Result<String, String> {
    println!("== Projects ==");
    for (k, _) in config.projects.iter() {
        println!("{}", k);
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

/// Get the next item by priority
pub fn next_item(config: Config, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name);

    match request::items_for_project(config.clone(), &project_id) {
        Ok(items) => {
            let filtered_items = items::filter_by_time(items);
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
        Err(e) => Err(e),
    }
}

/// Sort all the items in inbox
pub fn sort_inbox(config: Config) -> Result<String, String> {
    let inbox_id = projects::project_id(&config, "inbox");

    match request::items_for_project(config.clone(), &inbox_id) {
        Ok(items) if !items.is_empty() => {
            projects::list(config.clone()).unwrap();
            for item in items.iter() {
                request::move_item_to_project(config.clone(), item.to_owned())
                    .expect("Could not move item");
            }
            Ok(String::from("Successfully sorted inbox"))
        }
        Ok(_item) => Ok(String::from("No tasks to sort in inbox")),

        Err(e) => Err(e),
    }
}

/// Prioritize all items in a project
pub fn prioritize_items(config: Config, project_name: &str) -> Result<String, String> {
    let inbox_id = projects::project_id(&config, project_name);

    match request::items_for_project(config.clone(), &inbox_id) {
        Ok(items) => {
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

        Err(e) => Err(e),
    }
}
