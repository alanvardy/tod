use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io;
use std::io::*;

fn main() {
    let sentence = get_args_as_sentence();
    let token = get_or_create_token_file();
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

fn get_or_create_token_file() -> String {
    let home_directory = dirs::home_dir().expect("could not get home directory");
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
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    let mut file = File::create(path).expect("could not create file");
    file.write(input.as_bytes())
        .expect("could not write to file");

    input
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
