use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::Read;
use uuid::Uuid;

fn main() {
    let sentence = get_args_as_sentence();
    let token = get_or_create_token_file();
    post_request(sentence, token);
}

fn get_args_as_sentence() -> String {
    let mut sentence = String::new();
    let mut counter = 0;
    for arg in env::args() {
        if counter == 0 {
            counter += 1;
        } else if counter == 1 {
            sentence.push_str(&arg);
            counter += 1;
        } else {
            sentence.push_str(" ");
            sentence.push_str(&arg);
        }
    }
    sentence
}

#[allow(deprecated)]
fn get_or_create_token_file() -> String {
    let home_directory = env::home_dir().expect("could not get home directory");
    let home_directory_str = home_directory
        .to_str()
        .expect("could not set home directory to str");
    let path = format!("{}/todoist_token.cfg", home_directory_str);

    let mut file = File::open(&path).expect("could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read to string");

    contents
}

fn post_request(sentence: String, token: String) {
    let uuid: String = generate_uuid();
    let temp_id: String = generate_uuid();

    let body = json!({
    "token": token,
    "commands": [{"type": "item_add", "uuid": uuid, "temp_id": temp_id, "args": {"content": sentence}}]});

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

fn generate_uuid() -> String {
    Uuid::new_v4().hyphenated().to_string()
}
