use colored::*;
use reqwest::blocking::Client;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::{config, items, projects};

const QUICK_ADD_URL: &str = "https://api.todoist.com/sync/v8/quick/add";
const PROJECT_DATA_URL: &str = "https://api.todoist.com/sync/v8/projects/get_data";
const SYNC_URL: &str = "https://api.todoist.com/sync/v8/sync";
const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

/// Add a new item to the inbox with natural language support
pub fn add_item_to_inbox(config: Config, task: &str) {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"token": config.token, "text": task, "auto_reminder": true});

    match get_response(url, body) {
        Ok(_) => print_green_checkmark(),
        Err(e) => println!("{}", e),
    }
}

pub fn items_for_project(config: Config, project_id: &str) -> Result<Vec<items::Item>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({"token": config.token, "project_id": project_id});
    match get_response(url, body) {
        Ok(text) => Ok(items::from_json(text)),
        Err(err) => Err(err),
    }
}

pub fn move_item_to_project(config: Config, item: items::Item) {
    println!("{}", item);

    let project = config::get_input("Enter destination project name or (c)omplete:");

    match project.as_str() {
        "complete" | "c" => {
            let config = config.set_next_id(item.id);
            complete_item(config);
        }
        _ => {
            let project_id = projects::project_id(&config, &project);
            let body = json!({"token": config.token, "commands": [{"type": "item_move", "uuid": new_uuid(), "args": {"id": item.id, "project_id": project_id}}]});
            let url = String::from(SYNC_URL);

            match get_response(url, body) {
                Ok(_) => print_green_checkmark(),
                Err(e) => println!("{}", e),
            }
        }
    }
}

/// Complete the last item returned by "next item"
pub fn update_item_priority(config: Config, item: items::Item, priority: u8) {
    let body = json!({"token": config.token, "commands": [{"type": "item_update", "uuid": new_uuid(), "args": {"id": item.id, "priority": priority}}]});
    let url = String::from(SYNC_URL);

    match get_response(url, body) {
        Ok(_) => print_green_checkmark(),
        Err(e) => println!("{}", e),
    }
}

/// Complete the last item returned by "next item"
pub fn complete_item(config: Config) {
    let body = json!({"token": config.token, "commands": [{"type": "item_close", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    match get_response(url, body) {
        Ok(_) => {
            config.clear_next_id().save();
            print_green_checkmark();
        }
        Err(e) => println!("{}", e),
    }
}

/// Add item to project without natural language processing
pub fn add_item_to_project(config: Config, task: &str, project: &str) {
    let project_id = config.projects.get(project).expect("Project not found");
    let body = json!({"token": config.token, "commands": [{"type": "item_add", "uuid": new_uuid(), "temp_id": new_uuid(), "args": {"content": task, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    match get_response(url, body) {
        Ok(_) => print_green_checkmark(),
        Err(e) => println!("{}", e),
    }
}

/// Process an HTTP response
fn get_response(url: String, body: serde_json::Value) -> Result<String, String> {
    let response = Client::new()
        .post(&url)
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

/// Print a green checkmark to the terminal
fn print_green_checkmark() {
    println!("{}", "✓".green())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::config;
//     use std::collections::HashMap;

//     #[test]
//     fn should_build_index_request() {
//         let text = "this is text";

//         let mut projects = HashMap::new();
//         projects.insert(String::from("project_name"), 1234);

//         let config = Config {
//             token: String::from("1234567"),
//             path: config::generate_path(),
//             next_id: None,
//             projects,
//         };

//         let request = build_add_item_to_index_request(config, text);

//         assert_eq!(request.url.as_str(), QUICK_ADD_URL);
//         assert_eq!(format!("{:?}", request.body), "Object({\"auto_reminder\": Bool(true), \"text\": String(\"this is text\"), \"token\": String(\"1234567\")})");
//     }

//     #[test]
//     fn should_build_next_request() {
//         let project_name = "project_name";

//         let mut projects = HashMap::new();
//         projects.insert(String::from("project_name"), 1234);

//         let config = Config {
//             token: String::from("1234567"),
//             projects,
//             path: config::generate_path(),
//             next_id: None,
//         };

//         let request = build_next_item_request(config, project_name);

//         assert_eq!(request.url.as_str(), PROJECT_DATA_URL);
//         assert_eq!(
//             format!("{:?}", request.body),
//             "Object({\"project_id\": String(\"1234\"), \"token\": String(\"1234567\")})"
//         );
//     }

//     #[test]
//     fn should_build_send_task_to_project_request() {
//         let mut projects = HashMap::new();
//         projects.insert(String::from("project_name"), 1234);

//         let config = Config {
//             token: String::from("1234567"),
//             projects,
//             path: config::generate_path(),
//             next_id: None,
//         };

//         let request = build_add_item_to_project_request(config, "this is text", "project_name");

//         assert_eq!(request.url.as_str(), SYNC_URL);
//         assert_eq!(format!("{:?}", request.body), "Object({\"commands\": Array([Object({\"args\": Object({\"content\": String(\"this is text\"), \"project_id\": Number(1234)}), \"temp_id\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\"), \"type\": String(\"item_add\"), \"uuid\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\")})]), \"token\": String(\"1234567\")})");
//     }

//     #[test]
//     fn should_build_complete_request() {
//         let mut projects = HashMap::new();
//         projects.insert(String::from("project_name"), 1234);

//         let config = Config {
//             token: String::from("1234567"),
//             projects,
//             path: config::generate_path(),
//             next_id: Some(123123),
//         };

//         let request = build_complete_request(config);

//         assert_eq!(request.url.as_str(), SYNC_URL);
//         assert_eq!(format!("{:?}", request.body), "Object({\"commands\": Array([Object({\"args\": Object({\"id\": Number(123123)}), \"temp_id\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\"), \"type\": String(\"item_close\"), \"uuid\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\")})]), \"token\": String(\"1234567\")})");
//     }
// }
