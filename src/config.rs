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
    /// Path to config file
    pub path: String,
}

impl Config {
    pub fn new(token: &str) -> Config {
        let projects: HashMap<String, u32> = HashMap::new();
        Config {
            path: generate_path(),
            token: String::from(token),
            projects,
        }
    }

    pub fn create_file(self) -> Config {
        let json = json!({ "token": self.token, "projects": self.projects}).to_string();
        let bytes = File::create(&self.path)
            .expect("could not create file")
            .write(json.as_bytes())
            .expect("could not write to file");
        println!("{} bytes written to {}", bytes, &self.path);
        self
    }

    pub fn save(self) -> Config {
        fs::remove_file(&self.path).expect("could not remove old config");
        self.create_file()
    }

    pub fn load() -> Config {
        let path: String = generate_path();
        let mut json = String::new();

        File::open(&path)
            .expect("Could not find file")
            .read_to_string(&mut json)
            .expect("Could not read to string");

        let json_output: JsonOutput = serde_json::from_str(&json).expect("Could not parse JSON");

        Config {
            token: json_output.token,
            projects: json_output.projects,
            path,
        }
    }

    pub fn add_project(self, name: &str, number: u32) -> Config {
        let mut projects = self.projects;
        projects.insert(String::from(name), number);

        Config { projects, ..self }
    }

    pub fn remove_project(self, name: &str) -> Config {
        let mut projects = self.projects;
        projects.remove(name);

        Config { projects, ..self }
    }
}

pub fn get_or_create_config_file() -> Config {
    let path: String = generate_path();

    match File::open(&path) {
        Ok(_) => Config::load(),
        Err(_) => {
            let token = input_token();
            Config::new(&token).create_file()
        }
    }
}

pub fn generate_path() -> String {
    let filename = if cfg!(test) { "test" } else { ".tod.cfg" };

    let home_directory = dirs::home_dir().expect("could not get home directory");
    let home_directory_str = home_directory
        .to_str()
        .expect("could not set home directory to str");
    format!("{}/{}", home_directory_str, filename)
}

fn input_token() -> String {
    let mut input = String::new();
    println!("Please enter your Todoist API token from https://todoist.com/prefs/integrations ");
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    String::from(input.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_config() {
        let config = Config::new("something");
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn should_create_file_and_read_config() {
        let config = Config::new("faketoken");
        let home_directory = dirs::home_dir().expect("could not get home directory");
        let home_directory_str = home_directory
            .to_str()
            .expect("could not set home directory to str");
        let path = format!("{}/test", home_directory_str);

        let config2 = config.clone().create_file();
        let config3 = Config::load();
        assert_eq!(config.token, config2.token);
        assert_eq!(config.projects, config2.projects);
        assert_eq!(config2.token, config3.token);
        assert_eq!(config2.projects, config3.projects);
        assert_matches!(File::open(&path), Ok(_));
        assert_matches!(fs::remove_file(&path), Ok(_));
    }
}
