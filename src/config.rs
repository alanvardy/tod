use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io;
use std::io::*;

#[derive(Deserialize)]
struct Config {
    token: String,
}

pub fn get_or_create_token_file() -> String {
    let path: String = generate_path();

    let contents: String = match File::open(&path) {
        Ok(_) => read_file_and_get_token(),
        Err(_) => input_token_and_create_file(path),
    };

    contents
}

fn generate_path() -> String {
    let home_directory = dirs::home_dir().expect("could not get home directory");
    let home_directory_str = home_directory
        .to_str()
        .expect("could not set home directory to str");
    format!("{}/.tod.cfg", home_directory_str)
}

fn read_file_and_get_token() -> String {
    let config = read_config();
    config.token
}

fn input_token_and_create_file(path: String) -> String {
    let token = input_token();
    let json = token_to_json(&token);
    create_file(path, json);

    token
}

fn read_config() -> Config {
    let path: String = generate_path();
    let mut file = File::open(&path).expect("Could not find file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read to string");

    let config: Config = serde_json::from_str(&contents).unwrap();
    config
}

fn input_token() -> String {
    let mut input = String::new();
    println!("Please enter your Todoist API token from https://todoist.com/prefs/integrations ");
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    String::from(input.trim())
}

fn token_to_json(input: &str) -> String {
    json!({ "token": input }).to_string()
}

#[allow(clippy::unused_io_amount)]
fn create_file(path: String, json: String) {
    let mut file = File::create(path).expect("could not create file");
    file.write(json.as_bytes())
        .expect("could not write to file");
}
