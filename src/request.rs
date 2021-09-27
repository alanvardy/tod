use reqwest::blocking::Client;
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
const QUICK_ADD_URL: &str = "/sync/v8/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v8/projects/get_data";
const SYNC_URL: &str = "/sync/v8/sync";

// CRATES.IO URLS
const VERSIONS_URL: &str = "https://crates.io/api/v1/crates/tod/versions";

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
    let body = json!({"token": config.token, "text": task, "auto_reminder": true});

    let json = post_todoist(url, body)?;
    items::json_to_item(json)
}

pub fn items_for_project(config: Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({"token": config.token, "project_id": project_id});
    let json = post_todoist(url, body)?;
    items::json_to_items(json)
}

pub fn move_item(config: Config, item: Item, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name);
    let body = json!({"token": config.token, "commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist(url, body)?;
    Ok(String::from("✓"))
}

/// Complete the last item returned by "next item"
pub fn update_item_priority(config: Config, item: Item, priority: u8) -> Result<String, String> {
    let body = json!({"token": config.token, "commands": [{"type": "item_update", "uuid": new_uuid(), "args": {"id": item.id, "priority": priority}}]});
    let url = String::from(SYNC_URL);

    post_todoist(url, body)?;
    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: Config) -> Result<String, String> {
    let body = json!({"token": config.token, "commands": [{"type": "item_close", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    post_todoist(url, body)?;

    if !cfg!(test) {
        config.clear_next_id().save()?;
    }

    // Does not pass back an item
    Ok(String::from("✓"))
}

/// Post to Todoist
fn post_todoist(url: String, body: serde_json::Value) -> Result<String, String> {
    #[cfg(not(test))]
    let todoist_url: &str = "https://api.todoist.com";

    #[cfg(test)]
    let todoist_url: &str = &mockito::server_url();

    let request_url = format!("{}{}", todoist_url, url);

    let response = Client::new()
        .post(&request_url)
        .json(&body)
        .send()
        .expect("Did not get response from server");

    if response.status().is_success() {
        Ok(response.text().expect("could not read response"))
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Get request
pub fn get_latest_version() -> Result<String, String> {
    let response = Client::new()
        .get(VERSIONS_URL)
        .header(USER_AGENT, "Tod")
        .send()
        .expect("Did not get response from server");

    if response.status().is_success() {
        let cr: CargoResponse = serde_json::from_str(&response.text().unwrap()).unwrap();
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
        let _m = mockito::mock("POST", "/sync/v8/quick/add")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::item())
            .create();

        let config = Config::new("12341234");

        assert_eq!(
            add_item_to_inbox(&config, "testy test"),
            Ok(Item {
                id: 5149481867,
                priority: 1,
                content: String::from("testy test"),
                checked: 0,
                description: String::from(""),
                due: None,
                is_deleted: 0,
            })
        );
    }

    #[test]
    fn should_get_items_for_project() {
        let _m = mockito::mock("POST", "/sync/v8/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::items())
            .create();

        let config = Config::new("12341234");

        assert_eq!(
            items_for_project(config, "123123"),
            Ok(vec![Item {
                id: 999999,
                content: String::from("Put out recycling"),
                checked: 0,
                description: String::from(""),
                due: Some(DateInfo {
                    date: time::today_string(),
                    is_recurring: true,
                }),
                priority: 3,
                is_deleted: 0,
            }])
        );
    }

    #[test]
    fn should_complete_an_item() {
        let _m = mockito::mock("POST", "/sync/v8/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::sync())
            .create();

        let config = Config::new("12341234").set_next_id(112233);
        let response = complete_item(config);
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_prioritize_an_item() {
        let _m = mockito::mock("POST", "/sync/v8/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&test::responses::sync())
            .create();

        let config = Config::new("12341234");
        let item = Item {
            id: 999999,
            content: String::from("Put out recycling"),
            checked: 0,
            description: String::from(""),
            due: Some(DateInfo {
                date: time::today_string(),
                is_recurring: true,
            }),
            priority: 1,
            is_deleted: 0,
        };
        let response = update_item_priority(config, item, 4);
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]

    fn latest_version_works() {
        let response = get_latest_version();

        assert_eq!(response, Ok(String::from(VERSION)));
    }
}
