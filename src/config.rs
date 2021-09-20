use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::*;

/// App configuration, serialized as json in ~/.tod.cfg
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    /// The Todoist Api token
    pub token: String,
    /// List of Todoist projects and their project numbers
    pub projects: HashMap<String, u32>,
    /// Path to config file
    pub path: String,
    /// The ID of the next task
    pub next_id: Option<u64>,
}

impl Config {
    pub fn new(token: &str) -> Config {
        let projects: HashMap<String, u32> = HashMap::new();
        Config {
            path: generate_path(),
            token: String::from(token),
            next_id: None,
            projects,
        }
    }

    pub fn create(self) -> Config {
        let json = json!(self).to_string();
        let _bytes = fs::File::create(&self.path)
            .expect("could not create file")
            .write(json.as_bytes())
            .expect("could not write to file");
        self
    }

    pub fn save(self) -> std::result::Result<String, String> {
        let json = json!(self).to_string();

        let bytes = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .truncate(true)
            .open(&self.path)
            .expect("Could not find config")
            .write(json.as_bytes());

        match bytes {
            Ok(_) => Ok(String::from("✓")),
            Err(_) => Err(String::from("Could not write to file")),
        }
    }

    pub fn load(path: String) -> Config {
        let mut json = String::new();

        fs::File::open(&path)
            .expect("Could not find file")
            .read_to_string(&mut json)
            .expect("Could not read to string");

        let json_output: Config = serde_json::from_str(&json).expect("Could not parse JSON");

        Config {
            token: json_output.token,
            projects: json_output.projects,
            next_id: json_output.next_id,
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

    pub fn set_next_id(self, next_id: u64) -> Config {
        let next_id: Option<u64> = Some(next_id);

        Config { next_id, ..self }
    }

    pub fn clear_next_id(self) -> Config {
        let next_id: Option<u64> = None;

        Config { next_id, ..self }
    }
}

pub fn get_or_create() -> Config {
    let path: String = generate_path();
    let desc = "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

    match fs::File::open(&path) {
        Ok(_) => Config::load(path),
        Err(_) => {
            let token = get_input(desc);
            Config::new(&token).create()
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

pub fn get_input(desc: &str) -> String {
    if cfg!(test) {
        String::from("test")
    } else {
        let mut input = String::new();
        println!("{}", desc);
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        String::from(input.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new("something");
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn set_and_clear_next_id_should_work() {
        let config = Config::new("something");
        assert_eq!(config.next_id, None);
        let config = config.set_next_id(123123);
        assert_eq!(config.next_id, Some(123123));
        let config = config.clear_next_id();
        assert_eq!(config.next_id, None);
    }

    #[test]
    fn add_project_should_work() {
        let config = Config::new("something");
        let mut projects: HashMap<String, u32> = HashMap::new();
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path(),
                next_id: None,
                projects: projects.clone(),
            }
        );
        let config = config.add_project("test", 1234);
        projects.insert(String::from("test"), 1234);
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path(),
                next_id: None,
                projects,
            }
        );
    }

    #[test]
    fn remove_project_should_work() {
        let mut projects: HashMap<String, u32> = HashMap::new();
        projects.insert(String::from("test"), 1234);
        projects.insert(String::from("test2"), 4567);
        let config_with_two_projects = Config {
            token: String::from("something"),
            path: generate_path(),
            next_id: None,
            projects: projects.clone(),
        };

        assert_eq!(
            config_with_two_projects,
            Config {
                token: String::from("something"),
                path: generate_path(),
                next_id: None,
                projects: projects.clone(),
            }
        );
        let config_with_one_project = config_with_two_projects.remove_project("test");
        let mut projects: HashMap<String, u32> = HashMap::new();
        projects.insert(String::from("test2"), 4567);
        assert_eq!(
            config_with_one_project,
            Config {
                token: String::from("something"),
                path: generate_path(),
                next_id: None,
                projects,
            }
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn config_tests() {
            // These need to be run sequentially as they write to the filesystem.

            // Save and load
            // Build path
            let home_directory = dirs::home_dir().expect("could not get home directory");
            let home_directory_str = home_directory
                .to_str()
                .expect("could not set home directory to str");
            let path = format!("{}/test", home_directory_str);
            let _ = fs::remove_file(&path);

            // create and load
            let new_config = Config::new("faketoken");
            let created_config = new_config.clone().create();
            assert_eq!(new_config, created_config);
            let loaded_config = Config::load(path.clone());
            assert_eq!(created_config, loaded_config);

            // save and load
            let different_new_config = Config::new("differenttoken");
            different_new_config.clone().save().unwrap();
            let loaded_config = Config::load(path.clone());
            assert_eq!(loaded_config, different_new_config);
            assert_matches!(fs::remove_file(&path), Ok(_));

            // get_or_create (create)
            let config = get_or_create();
            assert_eq!(config, Config::new("test"));
            assert_matches!(fs::remove_file(&path), Ok(_));

            // get_or_create (load)
            Config::new("alreadycreated").create();
            let config = get_or_create();
            assert_eq!(config, Config::new("alreadycreated"));
            assert_matches!(fs::remove_file(&path), Ok(_));
        }
    }
}
