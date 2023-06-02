use std::env;

use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::json;
use spinners::{Spinner, Spinners};
use uuid::Uuid;

use crate::config::Config;
use crate::items::Item;
use crate::items::Priority;
use crate::projects::Project;
use crate::sections::Section;
use crate::{items, projects, sections};

// TODOIST URLS
const QUICK_ADD_URL: &str = "/sync/v9/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v9/projects/get_data";
const SYNC_URL: &str = "/sync/v9/sync";
const REST_V2_TASKS_URL: &str = "/rest/v2/tasks/";
const SECTIONS_URL: &str = "/rest/v2/sections";
const PROJECTS_URL: &str = "/rest/v2/projects";

// CRATES.IO URLS
const VERSIONS_URL: &str = "/v1/crates/tod/versions";

const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

const SPINNER: Spinners = Spinners::Dots4;
const MESSAGE: &str = "Querying API";

#[derive(Deserialize)]
struct CargoResponse {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    num: String,
}

/// Add a new item to the inbox with natural language support
pub fn quick_add_item(config: &Config, content: &str) -> Result<Item, String> {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"text": content, "auto_reminder": true});

    let json = post_todoist_sync(config, url, body)?;
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

    let json = post_todoist_rest(config, url, body)?;
    items::json_to_item(json)
}

/// Get a vector of all items for a project
pub fn items_for_project(config: &Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({ "project_id": project_id });
    let json = post_todoist_sync(config, url, body)?;
    items::json_to_items(json)
}

pub fn sections_for_project(config: &Config, project_id: &str) -> Result<Vec<Section>, String> {
    let url = format!("{SECTIONS_URL}?project_id={project_id}");
    let json = get_todoist_rest(config, url)?;
    sections::json_to_sections(json)
}

pub fn projects(config: &Config) -> Result<Vec<Project>, String> {
    let json = get_todoist_rest(config, PROJECTS_URL.to_string())?;
    projects::json_to_projects(json)
}

/// Move an item to a different project
pub fn move_item_to_project(
    config: &Config,
    item: Item,
    project_name: &str,
) -> Result<String, String> {
    let project_id = projects::project_id(config, project_name)?;
    let body = json!({"commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

pub fn move_item_to_section(
    config: &Config,
    item: Item,
    section_id: &str,
) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "section_id": section_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist_sync(config, url, body)?;
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

    post_todoist_rest(&config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Update due date for item using natural language
pub fn update_item_due(config: &Config, item: Item, due_string: String) -> Result<String, String> {
    let body = json!({ "due_string": due_string });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    post_todoist_rest(config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Update the name of an item by ID
pub fn update_item_name(config: &Config, item: Item, new_name: String) -> Result<String, String> {
    let body = json!({ "content": new_name });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    post_todoist_rest(config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: &Config) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_close", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist_sync(config, url, body)?;

    if !cfg!(test) {
        config.clone().clear_next_id().save()?;
    }

    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Post to Todoist via sync API
/// We use sync when we want natural languague processing.
fn post_todoist_sync(
    config: &Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: String = "https://api.todoist.com".to_string();
    #[cfg(not(test))]
    let _placeholder = &config.mock_url;

    #[cfg(test)]
    let todoist_url: String = config.mock_url.clone().expect("Mock URL not set");

    let request_url = format!("{todoist_url}{url}");
    let token = &config.token;

    let spinner = maybe_start_spinner(config);
    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Post to Todoist via REST api
/// We use this when we want more options and don't need natural language processing
fn post_todoist_rest(
    config: &Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: String = "https://api.todoist.com".to_string();

    #[cfg(test)]
    let todoist_url: String = config.mock_url.clone().expect("Mock URL not set");

    let token = &config.token;

    let request_url = format!("{todoist_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config);

    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .header("X-Request-Id", new_uuid())
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

// Combine get and post into one function
/// Get Todoist via REST api
fn get_todoist_rest(config: &Config, url: String) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: String = "https://api.todoist.com".to_string();

    #[cfg(test)]
    let todoist_url: String = config.mock_url.clone().expect("Mock URL not set");

    let token = config.token.clone();

    let request_url = format!("{todoist_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config);
    let response = Client::new()
        .get(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Get latest version number from Cargo.io
pub fn get_latest_version(config: Config) -> Result<String, String> {
    #[cfg(not(test))]
    let cargo_url: String = "https://crates.io/api".to_string();
    let _token = config.token;

    #[cfg(test)]
    let cargo_url: String = config.mock_url.expect("Mock URL not set");

    let request_url = format!("{cargo_url}{VERSIONS_URL}");

    let response = Client::new()
        .get(request_url)
        .header(USER_AGENT, "Tod")
        .send()
        .or(Err("Did not get response from server"))?;

    if response.status().is_success() {
        let cr: CargoResponse =
            serde_json::from_str(&response.text().or(Err("Could not read response text"))?)
                .or(Err("Could not serialize to CargoResponse"))?;
        Ok(cr.versions.first().unwrap().num.clone())
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Create a new UUID, required for Todoist API
fn new_uuid() -> String {
    if cfg!(test) {
        String::from(FAKE_UUID)
    } else {
        Uuid::new_v4().to_string()
    }
}

fn maybe_start_spinner(config: &Config) -> Option<Spinner> {
    match env::var("DISABLE_SPINNER") {
        Ok(_) => None,
        _ => {
            if let Some(true) = config.spinners {
                let sp = Spinner::new(SPINNER, MESSAGE.into());
                Some(sp)
            } else {
                None
            }
        }
    }
}
fn maybe_stop_spinner(spinner: Option<Spinner>) {
    if let Some(mut sp) = spinner {
        sp.stop();
        print!("\x1b[2K\r");
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{DateInfo, Item};
    use crate::{test, time, VERSION};
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
                priority: items::Priority::None,
                content: String::from("testy test"),
                checked: Some(false),
                description: String::from(""),
                due: None,
                is_deleted: Some(false),
                is_completed: None,
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
                priority: items::Priority::Medium,
                is_deleted: Some(false),
                is_completed: None,
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

    #[test]

    fn latest_version_works() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/crates/tod/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::versions())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = get_latest_version(config);
        mock.assert();

        assert_eq!(response, Ok(String::from(VERSION)));
    }
}
