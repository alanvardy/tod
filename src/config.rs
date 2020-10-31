use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io;
use std::io::*;
use std::collections::HashMap;


#[derive(Deserialize)]
struct JsonOutput {
    token: String,
    projects: HashMap<String, u32>
}
pub struct Config {
    pub token: String,
    pub projects: HashMap<String, u32>,
    json: String
}

pub fn get_or_create_token_file() -> Config {
    let path: String = generate_path();

    let contents: Config = match File::open(&path) {
        Ok(_) => read_config(),
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

fn input_token_and_create_file(path: String) -> Config {
    let token = input_token();
    let config: Config = generate_config(&token);
    create_file(path, config)
}

fn read_config() -> Config {
    let path: String = generate_path();
    let mut file = File::open(&path).expect("Could not find file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Could not read to string");

    let json_output: JsonOutput = serde_json::from_str(&contents).unwrap();

    Config {
        token: json_output.token,
        projects: json_output.projects,
        json: contents
    }
}

fn input_token() -> String {
    let mut input = String::new();
    println!("Please enter your Todoist API token from https://todoist.com/prefs/integrations ");
    io::stdin()
    .read_line(&mut input)
    .expect("error: unable to read user input");

    String::from(input.trim())
}

fn generate_config(token: &str) -> Config {
    let mut projects = HashMap::new();
    projects.insert(String::from("project_name"), 1234);

    Config {
        token: String::from(token),
        json: json!({ "token": token, "projects": {"project_name": 1234}}).to_string(),
        projects
    }
}

#[allow(clippy::unused_io_amount)]
fn create_file(path: String, config: Config) -> Config {
    let directory = &String::from(&path);
    let mut file = File::create(path).expect("could not create file");
    file.write(config.json.as_bytes())
    .expect("could not write to file");
    println!("Config written to {}", directory);
    config
}
