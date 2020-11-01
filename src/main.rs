use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use uuid::Uuid;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;

struct Params {
    project: String,
    text: String,
}

fn main() {
    let params: Params = get_params_from_args();
    let config: config::Config = config::get_or_create_token_file();

    match params.project.as_str() {
        "inbox" | "in" | "i" => post_index_request(params, config),
        _ => post_project_request(params, config),
    };
}

fn get_params_from_args() -> Params {
    let mut text = String::new();
    let mut project = String::new();
    for (index, arg) in env::args().enumerate() {
        match index {
            0 => (),
            1 => project.push_str(&arg),
            2 => text.push_str(&arg),
            num if num > 2 => {
                text.push_str(" ");
                text.push_str(&arg);
            }
            _ => (),
        }
    }

    Params { project, text }
}

fn post_index_request(params: Params, config: config::Config) {
    let body = json!({"token": config.token, "text": params.text, "auto_reminder": true});

    let request_url = "https://api.todoist.com/sync/v8/quick/add";
    let response = Client::new()
        .post(request_url)
        .json(&body)
        .send()
        .expect("Did not get response from server");

    if response.status().is_success() {
        println!("✓");
    } else {
        println!("Error: {:#?}", response.text());
    }
}

fn post_project_request(params: Params, config: config::Config) {
    let body = match params.project.as_str() {
        "inbox" | "in" | "i" => {
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": Uuid::new_v4().to_string(), "temp_id": Uuid::new_v4().to_string(), "args": {"content": params.text}}]})
        }
        _ => {
            let project_id = config
                .projects
                .get(&params.project)
                .expect("Project not found");
            json!({"token": config.token, "commands": [{"type": "item_add", "uuid": Uuid::new_v4().to_string(), "temp_id": Uuid::new_v4().to_string(), "args": {"content": params.text, "project_id": project_id}}]})
        }
    };

    let request_url = "https://api.todoist.com/sync/v8/sync";
    let response = Client::new()
        .post(request_url)
        .json(&body)
        .send()
        .expect("Did not get response from server");

    if response.status().is_success() {
        println!("✓");
    } else {
        println!("Error: {:#?}", response.text());
    }
}
