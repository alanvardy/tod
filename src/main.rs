use reqwest::blocking::Client;
use serde_json::json;
use std::env;

mod config;

fn main() {
    let sentence = get_args_as_sentence();
    let token = config::get_or_create_token_file();
    post_request(sentence, token);
}

fn get_args_as_sentence() -> String {
    let mut sentence = String::new();
    for (index, arg) in env::args().enumerate() {
        match index {
            1 => sentence.push_str(&arg),
            num if num > 1 => {
                sentence.push_str(" ");
                sentence.push_str(&arg);
            }
            _ => (),
        }
    }
    sentence
}

fn post_request(text: String, token: String) {
    let body = json!({"token": token, "text": text, "auto_reminder": true});

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
