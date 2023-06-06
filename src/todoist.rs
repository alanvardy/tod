use serde_json::json;

mod request;

use crate::config::Config;
use crate::items::priority::Priority;
use crate::items::Item;
use crate::projects::Project;
use crate::sections::Section;
use crate::{items, projects, sections};

// TODOIST URLS
const QUICK_ADD_URL: &str = "/sync/v9/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v9/projects/get_data";
const SYNC_URL: &str = "/sync/v9/sync";
pub const REST_V2_TASKS_URL: &str = "/rest/v2/tasks/";
const SECTIONS_URL: &str = "/rest/v2/sections";
const PROJECTS_URL: &str = "/rest/v2/projects";

/// Add a new item to the inbox with natural language support
pub fn quick_add_item(config: &Config, content: &str) -> Result<Item, String> {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"text": content, "auto_reminder": true});

    let json = request::post_todoist_sync(config, url, body)?;
    items::json_to_item(json)
}

/// Add item without natural language support but supports additional parameters
pub fn add_item(
    config: &Config,
    content: &str,
    priority: Priority,
    description: String,
) -> Result<Item, String> {
    let url = String::from(REST_V2_TASKS_URL);
    let body = json!({"content": content, "description": description, "auto_reminder": true, "priority": priority.to_integer()});

    let json = request::post_todoist_rest(config, url, body)?;
    items::json_to_item(json)
}

/// Get a vector of all items for a project
pub fn items_for_project(config: &Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({ "project_id": project_id });
    let json = request::post_todoist_sync(config, url, body)?;
    items::json_to_items(json)
}

pub fn sections_for_project(config: &Config, project_id: &str) -> Result<Vec<Section>, String> {
    let url = format!("{SECTIONS_URL}?project_id={project_id}");
    let json = request::get_todoist_rest(config, url)?;
    sections::json_to_sections(json)
}

pub fn projects(config: &Config) -> Result<Vec<Project>, String> {
    let json = request::get_todoist_rest(config, PROJECTS_URL.to_string())?;
    projects::json_to_projects(json)
}

/// Move an item to a different project
pub fn move_item_to_project(
    config: &Config,
    item: Item,
    project_name: &str,
) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

pub fn move_item_to_section(
    config: &Config,
    item: Item,
    section_id: &str,
) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": item.id, "section_id": section_id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

/// Update the priority of an item by ID
pub fn update_item_priority(
    config: Config,
    item: Item,
    priority: Priority,
) -> Result<String, String> {
    let body = json!({ "priority": priority });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    request::post_todoist_rest(&config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Update due date for item using natural language
pub fn update_item_due(config: &Config, item: Item, due_string: String) -> Result<String, String> {
    let body = json!({ "due_string": due_string });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    request::post_todoist_rest(config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Update the name of an item by ID
pub fn update_item_name(config: &Config, item: Item, new_name: String) -> Result<String, String> {
    let body = json!({ "content": new_name });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    request::post_todoist_rest(config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Get a vector of all completed items for a project
pub fn completed_items_for_project(config: &Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from("/sync/v9/archive/items");
    let body = json!({ "project_id": project_id });
    let json = crate::todoist::request::post_todoist_sync(config, url, body)?;
    items::json_to_items(json)
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: &Config) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_close", "uuid": request::new_uuid(), "temp_id": request::new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;

    if !cfg!(test) {
        config.clone().clear_next_id().save()?;
    }

    // Does not pass back an item
    Ok(String::from("✓"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::priority::Priority;
    use crate::items::{DateInfo, Item};
    use crate::{test, time};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_quick_add_item() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/quick/add")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::item())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        assert_eq!(
            quick_add_item(&config, "testy test"),
            Ok(Item {
                id: String::from("5149481867"),
                priority: Priority::None,
                content: String::from("testy test"),
                checked: Some(false),
                description: String::from(""),
                due: None,
                is_deleted: Some(false),
                is_completed: None,
                completed_at: None
            })
        );
        mock.assert();
    }

    #[test]
    fn should_get_items_for_project() {
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
            ..config
        };

        assert_eq!(
            items_for_project(&config_with_timezone, "123123"),
            Ok(vec![Item {
                id: String::from("999999"),
                content: String::from("Put out recycling"),
                checked: Some(false),
                description: String::from(""),
                due: Some(DateInfo {
                    date: format!("{}T23:59:00Z", time::today_string(&config_with_timezone)),
                    is_recurring: true,
                    timezone: None,
                }),
                priority: Priority::Medium,
                is_deleted: Some(false),
                is_completed: None,
                completed_at: None
            }])
        );

        mock.assert();
    }

    #[test]
    fn should_complete_an_item() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .set_next_id(&"112233".to_string());

        let response = complete_item(&config);
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_move_an_item() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let item = test::fixtures::item();
        let project_name = "testy";
        let mut config = test::fixtures::config().mock_url(server.url());
        config.add_project(String::from(project_name), 1);

        let config = Config {
            mock_url: Some(server.url()),
            ..config
        };
        let response = move_item_to_project(&config, item, project_name);
        mock.assert();

        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_prioritize_an_item() {
        let item = test::fixtures::item();
        let url: &str = &format!("{}{}", "/rest/v2/tasks/", item.id);
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = update_item_priority(config, item, Priority::High);
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_update_date_on_an_item() {
        let item = test::fixtures::item();
        let url: &str = &format!("{}{}", "/rest/v2/tasks/", item.id);
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = update_item_due(&config, item, "today".to_string());
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }
}
