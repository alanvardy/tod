use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::*;

#[derive(Deserialize)]
struct JsonOutput {
    token: String,
    projects: HashMap<String, u32>,
}
#[derive(Clone)]
pub struct Config {
    pub token: String,
    pub projects: HashMap<String, u32>,
    pub json: String,
}

pub fn get_or_create_token_file() -> Config {
    let path: String = generate_path(".tod.cfg");

    let contents: Config = match File::open(&path) {
        Ok(_) => read_config(&path),
        Err(_) => {
            let token = input_token();
            let config: Config = generate_config(&token);
            create_file(path, config)
        }
    };

    contents
}

fn generate_path(filename: &str) -> String {
    let home_directory = dirs::home_dir().expect("could not get home directory");
    let home_directory_str = home_directory
        .to_str()
        .expect("could not set home directory to str");
    format!("{}/{}", home_directory_str, filename)
}

fn read_config(path: &str) -> Config {
    let mut file = File::open(path).expect("Could not find file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Could not read to string");

    let json_output: JsonOutput = serde_json::from_str(&contents).unwrap();

    Config {
        token: json_output.token,
        projects: json_output.projects,
        json: contents,
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
        projects,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn should_generate_config() {
        let config = generate_config("something");
        assert_eq!(config.token, String::from("something"));
        assert_eq!(
            config.json,
            String::from("{\"projects\":{\"project_name\":1234},\"token\":\"something\"}")
        );
    }

    #[test]
    fn should_create_file_and_read_config() {
        let config = generate_config("something");
        let home_directory = dirs::home_dir().expect("could not get home directory");
        let home_directory_str = home_directory
            .to_str()
            .expect("could not set home directory to str");
        let path = format!("{}/test", home_directory_str);

        let config2 = create_file(path.clone(), config.clone());
        let config3 = read_config(&path);
        assert_eq!(config.token, config2.token);
        assert_eq!(config.json, config2.json);
        assert_eq!(config.projects, config2.projects);
        assert_eq!(config2.token, config3.token);
        assert_eq!(config2.json, config3.json);
        assert_eq!(config2.projects, config3.projects);
        assert_matches!(File::open(&path), Ok(_));
        assert_matches!(fs::remove_file(&path), Ok(_));
    }
}
