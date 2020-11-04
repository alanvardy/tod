use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
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
    /// The Todoist Api token
    pub token: String,
    /// List of Todoist projects and their project numbers
    pub projects: HashMap<String, u32>,
    /// Json string that is stored in config file
    pub json: String,
    /// Path to config file
    pub path: String,
}

impl Config {
    fn new(token: &str) -> Config {
        let projects: HashMap<String, u32> = HashMap::new();
        Config {
            path: generate_path(".tod.cfg"),
            token: String::from(token),
            json: json!({ "token": token, "projects": projects}).to_string(),
            projects,
        }
    }

    pub fn save(self) {
        fs::remove_file(&self.path).expect("could not remove old config");
        create_file(String::from(&self.path), self);
    }
}

pub fn get_or_create_config_file() -> Config {
    let path: String = generate_path(".tod.cfg");

    let config: Config = match File::open(&path) {
        Ok(_) => read_config(&path),
        Err(_) => {
            let token = input_token();
            let config = Config::new(&token);
            create_file(path, config)
        }
    };

    config
}

pub fn generate_path(filename: &str) -> String {
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
        path: String::from(path),
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

    #[test]
    fn should_generate_config() {
        let config = Config::new("something");
        assert_eq!(config.token, String::from("something"));
        assert_eq!(
            config.json,
            String::from("{\"projects\":{},\"token\":\"something\"}")
        );
    }

    #[test]
    fn should_create_file_and_read_config() {
        let config = Config::new("something");
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
