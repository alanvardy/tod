use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::items::Item;
use crate::{items, projects};

#[cfg(test)]
use mockito;

// TODOIST URLS
const QUICK_ADD_URL: &str = "/sync/v9/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v9/projects/get_data";
const SYNC_URL: &str = "/sync/v9/sync";
const REST_V2_TASKS_URL: &str = "/rest/v2/tasks/";

// CRATES.IO URLS
const VERSIONS_URL: &str = "/v1/crates/tod/versions";

const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

#[derive(Deserialize)]
struct CargoResponse {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    num: String,
}

/// Add a new item to the inbox with natural language support
pub fn add_item_to_inbox(config: &Config, task: &str) -> Result<Item, String> {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"text": task, "auto_reminder": true});

    let json = post_todoist_sync(config.clone(), url, body)?;
    items::json_to_item(json)
}

/// Get a vector of all items for a project
pub fn items_for_project(config: &Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({ "project_id": project_id });
    let json = post_todoist_sync(config.clone(), url, body)?;
    items::json_to_items(json)
}

/// Move an item to a different project
pub fn move_item(config: Config, item: Item, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name)?;
    let body = json!({"commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

/// Update the priority of an item by ID
pub fn update_item_priority(config: Config, item: Item, priority: u8) -> Result<String, String> {
    let body = json!({ "priority": priority });
    let url = format!("{}{}", REST_V2_TASKS_URL, item.id);

    post_todoist_rest(config, url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: Config) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_close", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist_sync(config.clone(), url, body)?;

    if !cfg!(test) {
        config.clear_next_id().save()?;
    }

    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Post to Todoist via sync API
fn post_todoist_sync(
    config: Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: String = "https://api.todoist.com".to_string();
    #[cfg(not(test))]
    let _placeholder = config.clone().mock_url;

    #[cfg(test)]
    let todoist_url: String = config.clone().mock_url.expect("Mock URL not set");

    let request_url = format!("{todoist_url}{url}");
    let token = config.token;

    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Post to Todoist via REST api
fn post_todoist_rest(
    config: Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: String = "https://api.todoist.com".to_string();

    #[cfg(test)]
    let todoist_url: String = config.mock_url.expect("Mock URL not set");

    let token = config.token;

    let request_url = format!("{todoist_url}{url}");
    let authorization: &str = &format!("Bearer {token}");

    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .header("X-Request-Id", new_uuid())
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{DateInfo, Item};
    use crate::{test, time, VERSION};
    use pretty_assertions::assert_eq;

    #[test]
    fn should_add_item_to_inbox() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/quick/add")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::item())
            .create();

        let config = Config::new("12341234", Some(server.url())).unwrap();

        assert_eq!(
            add_item_to_inbox(&config, "testy test"),
            Ok(Item {
                id: String::from("5149481867"),
                priority: 1,
                content: String::from("testy test"),
                checked: false,
                description: String::from(""),
                due: None,
                is_deleted: false,
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
            .with_body(&test::responses::items())
            .create();

        let config = Config::new("12341234", Some(server.url())).unwrap();
        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            ..config
        };

        assert_eq!(
            items_for_project(&config_with_timezone, "123123"),
            Ok(vec![Item {
                id: String::from("999999"),
                content: String::from("Put out recycling"),
                checked: false,
                description: String::from(""),
                due: Some(DateInfo {
                    date: String::from(format!(
                        "{}T23:59:00Z",
                        time::today_string(&config_with_timezone)
                    )),
                    is_recurring: true,
                    timezone: None,
                }),
                priority: 3,
                is_deleted: false,
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
            .with_body(&test::responses::sync())
            .create();

        let config = Config::new("12341234", Some(server.url()))
            .unwrap()
            .set_next_id(String::from("112233"));

        let response = complete_item(config);
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
            .with_body(&test::responses::sync())
            .create();

        let item = test::helpers::item_fixture();
        let project_name = "testy";
        let config = Config::new("12341234", Some(server.url()))
            .unwrap()
            .add_project(String::from(project_name), 1);

        let config = Config {
            mock_url: Some(server.url()),
            ..config
        };
        let response = move_item(config, item, project_name);
        mock.assert();

        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_prioritize_an_item() {
        let item = test::helpers::item_fixture();
        let url: &str = &format!("{}{}", "/rest/v2/tasks/", item.id);
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::sync())
            .create();

        let config = Config::new("12341234", Some(server.url())).unwrap();

        let response = update_item_priority(config, item, 4);
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
            .with_body(&test::responses::versions())
            .create();

        let config = Config {
            mock_url: Some(server.url()),
            ..Config::new("12341234", Some(server.url())).unwrap()
        };

        let response = get_latest_version(config);
        mock.assert();

        assert_eq!(response, Ok(String::from(VERSION)));
    }
}
