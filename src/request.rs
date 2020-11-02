use reqwest::blocking::Client;
use serde_json::json;
use uuid::Uuid;

use crate::params;
use crate::config;

pub fn build_request(params: params::Params, config: config::Config) -> (String, serde_json::Value) {
    match params.project.as_str() {
            "inbox" | "in" | "i" => build_index_request(params, config),
            _ => build_project_request(params, config),
        }
}

fn build_index_request(params: params::Params, config: config::Config) -> (String, serde_json::Value) {
    let url = String::from("https://api.todoist.com/sync/v8/quick/add");
    let body = json!({"token": config.token, "text": params.text, "auto_reminder": true});

    (url, body)
}

fn build_project_request(params: params::Params, config: config::Config) -> (String, serde_json::Value) {
    let url = String::from("https://api.todoist.com/sync/v8/sync");

    let body = match params.project.as_str() {
        "inbox" | "in" | "i" => {
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": gen_uuid(), "temp_id": gen_uuid(), "args": {"content": params.text}}]})
        }
        _ => {
            let project_id = config
                .projects
                .get(&params.project)
                .expect("Project not found");
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": gen_uuid(), "temp_id": gen_uuid(), "args": {"content": params.text, "project_id": project_id}}]})
        }
    };

    (url, body)
}

pub fn do_request(url: &str, body: serde_json::Value) {
    let response = Client::new()
        .post(url)
        .json(&body)
        .send()
        .expect("Did not get response from server");

    if response.status().is_success() {
        println!("✓");
    } else {
        println!("Error: {:#?}", response.text());
    }
}

fn gen_uuid() -> String {
    Uuid::new_v4().to_string()
}
