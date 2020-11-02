use reqwest::blocking::Client;
use serde_json::json;
use uuid::Uuid;

use crate::config;
use crate::params;

pub fn build_request(
    params: params::Params,
    config: config::Config,
) -> (String, serde_json::Value) {
    match params.project.as_str() {
        "inbox" | "in" | "i" => build_index_request(params, config),
        _ => build_project_request(params, config),
    }
}

fn build_index_request(
    params: params::Params,
    config: config::Config,
) -> (String, serde_json::Value) {
    let url = String::from("https://api.todoist.com/sync/v8/quick/add");
    let body = json!({"token": config.token, "text": params.text, "auto_reminder": true});

    (url, body)
}

fn build_project_request(
    params: params::Params,
    config: config::Config,
) -> (String, serde_json::Value) {
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
    if cfg!(test) {
        String::from("42963283-2bab-4b1f-bad2-278ef2b6ba2c")
    } else {

        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn should_build_index_request() {
        let params = params::Params {
            project: String::from("a_project"),
            text: String::from("this is text"),
        };

        let mut projects = HashMap::new();
        projects.insert(String::from("project_name"), 1234);

        let config = config::Config {
            token: String::from("1234567"),
            projects,
            json: String::from("thisistotallyajsonstring"),
        };

        let (url, body) = build_index_request(params, config);

        assert_eq!(url.as_str(), "https://api.todoist.com/sync/v8/quick/add");
        assert_eq!(format!("{:?}", body), "Object({\"auto_reminder\": Bool(true), \"text\": String(\"this is text\"), \"token\": String(\"1234567\")})");
    }
    #[test]
    fn should_build_project_request() {
        let params = params::Params {
            project: String::from("project_name"),
            text: String::from("this is text"),
        };

        let mut projects = HashMap::new();
        projects.insert(String::from("project_name"), 1234);

        let config = config::Config {
            token: String::from("1234567"),
            projects,
            json: String::from("thisistotallyajsonstring"),
        };

        let (url, body) = build_project_request(params, config);

        assert_eq!(url.as_str(), "https://api.todoist.com/sync/v8/sync");
        assert_eq!(format!("{:?}", body), "Object({\"commands\": Array([Object({\"args\": Object({\"content\": String(\"this is text\"), \"project_id\": Number(1234)}), \"temp_id\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\"), \"type\": String(\"item_add\"), \"uuid\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\")})]), \"token\": String(\"1234567\")})");
    }
}
