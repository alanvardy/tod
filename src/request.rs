use colored::*;
use reqwest::blocking::Client;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::params::Params;

mod next_item;

const QUICK_ADD_URL: &str = "https://api.todoist.com/sync/v8/quick/add";
const PROJECT_DATA_URL: &str = "https://api.todoist.com/sync/v8/projects/get_data";
const SYNC_URL: &str = "https://api.todoist.com/sync/v8/sync";
const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

pub enum RequestType {
    // Adding a new item to Todoist
    AddItem,
    // Obtaining the next item from Todoist
    NextItem,
}

pub struct Request {
    url: String,
    body: serde_json::Value,
    request_type: RequestType,
    config: Config,
}

impl Request {
    pub fn new(params: Params, config: Config) -> Request {
        match params.command.as_str() {
            "inbox" | "in" | "i" => build_index_request(params, config),
            "--next" | "-n" => build_next_request(params, config),
            _ => build_project_request(params, config),
        }
    }

    pub fn perform(self) {
        let response = Client::new()
            .post(&self.url)
            .json(&self.body)
            .send()
            .expect("Did not get response from server");

        if response.status().is_success() {
            match &self.request_type {
                RequestType::AddItem => println!("{}", "✓".green()),
                RequestType::NextItem => {
                    let text = response.text().expect("could not read response");
                    match next_item::determine_next_item(text) {
                        Some(item) => {
                            let config = self.config.set_next_id(item.id);
                            config.save();
                            println!("{}", item);
                        }
                        None => print!("No items on list"),
                    }
                }
            }
        } else {
            println!("Error: {:#?}", response.text());
        }
    }
}

fn build_index_request(params: Params, config: Config) -> Request {
    Request {
        url: String::from(QUICK_ADD_URL),
        body: json!({"token": config.token, "text": params.text, "auto_reminder": true}),
        request_type: RequestType::AddItem,
        config,
    }
}

fn build_next_request(params: Params, config: Config) -> Request {
    let project_id = config
        .projects
        .get(&params.text)
        .expect("Project not found")
        .to_string();

    Request {
        url: String::from(PROJECT_DATA_URL),
        body: json!({"token": config.token, "project_id": project_id}),
        request_type: RequestType::NextItem,
        config,
    }
}

fn build_project_request(params: Params, config: Config) -> Request {
    let body = match params.command.as_str() {
        "inbox" | "in" | "i" => {
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": gen_uuid(), "temp_id": gen_uuid(), "args": {"content": params.text}}]})
        }
        _ => {
            let project_id = config
                .projects
                .get(&params.command)
                .expect("Project not found");
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": gen_uuid(), "temp_id": gen_uuid(), "args": {"content": params.text, "project_id": project_id}}]})
        }
    };

    Request {
        url: String::from(SYNC_URL),
        body,
        request_type: RequestType::AddItem,
        config,
    }
}

fn gen_uuid() -> String {
    if cfg!(test) {
        String::from(FAKE_UUID)
    } else {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::collections::HashMap;

    #[test]
    fn should_build_index_request() {
        let params = Params {
            command: String::from("a_project"),
            text: String::from("this is text"),
        };

        let mut projects = HashMap::new();
        projects.insert(String::from("project_name"), 1234);

        let config = Config {
            token: String::from("1234567"),
            path: config::generate_path(),
            next_id: None,
            projects,
        };

        let request = build_index_request(params, config);

        assert_eq!(request.url.as_str(), QUICK_ADD_URL);
        assert_eq!(format!("{:?}", request.body), "Object({\"auto_reminder\": Bool(true), \"text\": String(\"this is text\"), \"token\": String(\"1234567\")})");
    }

    #[test]
    fn should_build_next_request() {
        let params = Params {
            command: String::from("--next"),
            text: String::from("project_name"),
        };

        let mut projects = HashMap::new();
        projects.insert(String::from("project_name"), 1234);

        let config = Config {
            token: String::from("1234567"),
            projects,
            path: config::generate_path(),
            next_id: None,
        };

        let request = build_next_request(params, config);

        assert_eq!(request.url.as_str(), PROJECT_DATA_URL);
        assert_eq!(
            format!("{:?}", request.body),
            "Object({\"project_id\": String(\"1234\"), \"token\": String(\"1234567\")})"
        );
    }

    #[test]
    fn should_build_project_request() {
        let params = Params {
            command: String::from("project_name"),
            text: String::from("this is text"),
        };

        let mut projects = HashMap::new();
        projects.insert(String::from("project_name"), 1234);

        let config = Config {
            token: String::from("1234567"),
            projects,
            path: config::generate_path(),
            next_id: None,
        };

        let request = build_project_request(params, config);

        assert_eq!(request.url.as_str(), SYNC_URL);
        assert_eq!(format!("{:?}", request.body), "Object({\"commands\": Array([Object({\"args\": Object({\"content\": String(\"this is text\"), \"project_id\": Number(1234)}), \"temp_id\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\"), \"type\": String(\"item_add\"), \"uuid\": String(\"42963283-2bab-4b1f-bad2-278ef2b6ba2c\")})]), \"token\": String(\"1234567\")})");
    }
}
