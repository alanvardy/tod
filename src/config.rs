use crate::{request, time, VERSION};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::{fs, io};

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
    last_version_check: Option<String>,
}

impl Config {
    pub fn new(token: &str) -> Result<Config, String> {
        let projects: HashMap<String, u32> = HashMap::new();
        Ok(Config {
            path: generate_path()?,
            token: String::from(token),
            next_id: None,
            last_version_check: Some(time::today_string()),
            projects,
        })
    }

    pub fn create(self) -> Result<Config, String> {
        let json = json!(self).to_string();
        let mut file = fs::File::create(&self.path).or(Err("Could not create file"))?;
        file.write_all(json.as_bytes())
            .or(Err("Could not write to file"))?;
        Ok(self)
    }

    pub fn save(self) -> std::result::Result<String, String> {
        let json = json!(self);
        let string = serde_json::to_string_pretty(&json).or(Err("Could not convert to JSON"))?;

        fs::OpenOptions::new()
            .write(true)
            .read(true)
            .truncate(true)
            .open(&self.path)
            .or(Err("Could not find config"))?
            .write_all(string.as_bytes())
            .or(Err("Could not write to file"))?;

        Ok(String::from("✓"))
    }

    pub fn load(path: String) -> Result<Config, String> {
        let mut json = String::new();

        fs::File::open(&path)
            .or(Err("Could not find file"))?
            .read_to_string(&mut json)
            .or(Err("Could not read to string"))?;

        let config: Config = serde_json::from_str(&json).or(Err("Could not parse JSON"))?;

        check_for_latest_version(config)
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

pub fn get_or_create() -> Result<Config, String> {
    let path: String = generate_path()?;
    let desc = "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

    match fs::File::open(&path) {
        Ok(_) => Config::load(path),
        Err(_) => {
            let token = get_input(desc)?;
            Config::new(&token)?.create()
        }
    }
}

fn check_for_latest_version(config: Config) -> Result<Config, String> {
    let last_version = config.clone().last_version_check;
    let new_config = Config {
        last_version_check: Some(time::today_string()),
        ..config
    };

    if last_version != Some(time::today_string()) {
        match request::get_latest_version() {
            Ok(version) if version.as_str() != VERSION => {
                println!(
                    "Latest Tod version is {}, found {}.\nRun {} to update if you installed with Cargo",
                    version,
                    VERSION,
                    "cargo install tod".bright_cyan()
                );
                new_config.clone().save().unwrap();
            }
            Ok(_) => (),
            Err(err) => println!(
                "{}, {:?}",
                "Could not fetch Tod version from Cargo.io".red(),
                err
            ),
        };
    }

    Ok(new_config)
}

pub fn generate_path() -> Result<String, String> {
    let filename = if cfg!(test) { "test" } else { ".tod.cfg" };

    let home_directory = dirs::home_dir()
        .ok_or_else(|| String::from("Could not find home directory"))?
        .to_str()
        .ok_or_else(|| String::from("Could not convert directory to string"))?
        .to_owned();
    Ok(format!("{}/{}", home_directory, filename))
}

pub fn get_input(desc: &str) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from("test"));
    }

    let mut input = String::new();
    println!("{}", desc);
    io::stdin()
        .read_line(&mut input)
        .or(Err("error: unable to read user input"))?;

    Ok(String::from(input.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new("something").unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn set_and_clear_next_id_should_work() {
        let config = Config::new("something").unwrap();
        assert_eq!(config.next_id, None);
        let config = config.set_next_id(123123);
        assert_eq!(config.next_id, Some(123123));
        let config = config.clear_next_id();
        assert_eq!(config.next_id, None);
    }

    #[test]
    fn add_project_should_work() {
        let config = Config::new("something").unwrap();
        let mut projects: HashMap<String, u32> = HashMap::new();
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: Some(time::today_string()),
                projects: projects.clone(),
            }
        );
        let config = config.add_project("test", 1234);
        projects.insert(String::from("test"), 1234);
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: Some(time::today_string()),
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
            path: generate_path().unwrap(),
            next_id: None,
            last_version_check: None,
            projects: projects.clone(),
        };

        assert_eq!(
            config_with_two_projects,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: None,
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
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: None,
                projects,
            }
        );
    }

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
        let new_config = Config::new("faketoken").unwrap();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(path.clone()).unwrap();
        assert_eq!(created_config, loaded_config);

        // save and load
        let different_new_config = Config::new("differenttoken").unwrap();
        different_new_config.clone().save().unwrap();
        let loaded_config = Config::load(path.clone()).unwrap();
        assert_eq!(loaded_config, different_new_config);
        assert_matches!(fs::remove_file(&path), Ok(_));

        // get_or_create (create)
        let config = get_or_create();
        assert_eq!(config, Config::new("test"));
        assert_matches!(fs::remove_file(&path), Ok(_));

        // get_or_create (load)
        Config::new("alreadycreated").unwrap().create().unwrap();
        let config = get_or_create();
        assert_eq!(config, Config::new("alreadycreated"));
        assert_matches!(fs::remove_file(&path), Ok(_));
    }
}
