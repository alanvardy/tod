use reqwest::blocking::Client;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::items::Item;
use crate::{items, projects};

#[cfg(test)]
use mockito;

const QUICK_ADD_URL: &str = "/sync/v8/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v8/projects/get_data";
const SYNC_URL: &str = "/sync/v8/sync";
const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

/// Add a new item to the inbox with natural language support
pub fn add_item_to_inbox(config: &Config, task: &str) -> Result<Item, String> {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"token": config.token, "text": task, "auto_reminder": true});

    let json = post(url, body)?;
    items::json_to_item(json)
}

pub fn items_for_project(config: Config, project_id: &str) -> Result<Vec<Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({"token": config.token, "project_id": project_id});
    let json = post(url, body)?;
    items::json_to_items(json)
}

pub fn move_item(config: Config, item: Item, project_name: &str) -> Result<String, String> {
    let project_id = projects::project_id(&config, project_name);
    let body = json!({"token": config.token, "commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    post(url, body)
}

/// Complete the last item returned by "next item"
pub fn update_item_priority(config: Config, item: Item, priority: u8) -> Result<String, String> {
    let body = json!({"token": config.token, "commands": [{"type": "item_update", "uuid": new_uuid(), "args": {"id": item.id, "priority": priority}}]});
    let url = String::from(SYNC_URL);

    post(url, body)?;
    Ok(String::from("✓"))
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: Config) -> Result<Item, String> {
    let body = json!({"token": config.token, "commands": [{"type": "item_close", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    let json = post(url, body)?;
    config.clear_next_id().save()?;
    items::json_to_item(json)
}

/// Process an HTTP response
fn post(url: String, body: serde_json::Value) -> Result<String, String> {
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
    use crate::time;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_add_item_to_inbox() {
        let body = "\
        {\"added_by_uid\":635166,\
        \"assigned_by_uid\":null,\
        \"checked\":0,\
        \"child_order\":2,\
        \"collapsed\":0,\
        \"content\":\"testy test\",\
        \"date_added\":\"2021-09-12T19:11:07Z\",\
        \"date_completed\":null,\
        \"description\":\"\",\
        \"due\":null,\
        \"id\":5149481867,\
        \"in_history\":0,\
        \"is_deleted\":0,\
        \"labels\":[],\
        \"legacy_project_id\":333333333,\
        \"parent_id\":null,\
        \"priority\":1,\
        \"project_id\":5555555,\
        \"reminder\":null,\
        \"responsible_uid\":null,\
        \"section_id\":null,\
        \"sync_id\":null,\
        \"user_id\":111111\
    }";
        let _m = mockito::mock("POST", "/sync/v8/quick/add")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
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
        let body = format!(
            "{{\
        \"items\":\
            [
                {{\
                \"added_by_uid\":44444444,\
                \"assigned_by_uid\":null,\
                \"checked\":0,\
                \"child_order\":-5,\
                \"collapsed\":0,\
                \"content\":\"Put out recycling\",\
                \"date_added\":\"2021-06-15T13:01:28Z\",\
                \"date_completed\":null,\
                \"description\":\"\",\
                \"due\":{{\
                \"date\":\"{}\",\
                \"is_recurring\":true,\
                \"lang\":\"en\",\
                \"string\":\"every other mon at 16:30\",\
                \"timezone\":null}},\
                \"id\":999999,\
                \"in_history\":0,\"is_deleted\":0,\
                \"labels\":[],\
                \"note_count\":0,\
                \"parent_id\":null,\
                \"priority\":3,\
                \"project_id\":22222222,\
                \"responsible_uid\":null,\
                \"section_id\":333333333,\
                \"sync_id\":null,\
                \"user_id\":111111111\
                }}
            ]
        }}",
            time::today()
        );
        let _m = mockito::mock("POST", "/sync/v8/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
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
                    date: time::today(),
                    is_recurring: true,
                }),
                priority: 3,
                is_deleted: 0,
            }])
        );
    }
}
