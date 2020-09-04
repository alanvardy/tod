use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::fs::File;
use uuid::Uuid;
use std::io;
use std::io::*;

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

    let contents: String = match File::open(&path) {
        Ok(file) => read_file(file),
        Err(_) => create_file(path),
    };

    contents
}

fn read_file(file: File) -> String {
    let mut contents = String::new();
    let mut file = file;
    file.read_to_string(&mut contents)
        .expect("Could not read to string");

        contents
}

#[allow(clippy::unused_io_amount)]
fn create_file(path: String) -> String {
    let mut input = String::new();
    println!("Please enter your Todoist API token from https://todoist.com/prefs/integrations ");
    io::stdin().read_line(&mut input).expect("error: unable to read user input");

    let mut file = File::create(path).expect("could not create file");
    file.write(input.as_bytes()).expect("could not write to file");

    input
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
