use crate::{request, time, VERSION};
use chrono_tz::TZ_VARIANTS;
use colored::*;
use inquire::{Select, Text};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io::{Read, Write};

/// App configuration, serialized as json in $XDG_CONFIG_HOME/tod.cfg
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    /// The Todoist Api token
    pub token: String,
    /// List of Todoist projects and their project numbers
    pub projects: HashMap<String, u32>,
    /// Path to config file
    pub path: String,
    /// The ID of the next task
    pub next_id: Option<String>,
    pub timezone: Option<String>,
    pub last_version_check: Option<String>,
    pub mock_url: Option<String>,
    // Whether spinners are enabled
    pub spinners: Option<bool>,
}

impl Config {
    pub fn new(token: &str, mock_url: Option<String>) -> Result<Config, String> {
        let projects: HashMap<String, u32> = HashMap::new();
        Ok(Config {
            path: generate_path()?,
            token: String::from(token),
            next_id: None,
            last_version_check: None,
            timezone: None,
            spinners: Some(true),
            mock_url,
            projects,
        })
    }

    pub fn create(self) -> Result<Config, String> {
        let json = json!(self).to_string();
        let mut file = fs::File::create(&self.path).or(Err("Could not create file"))?;
        file.write_all(json.as_bytes())
            .or(Err("Could not write to file"))?;
        println!("Config successfully created in {}", &self.path);
        Ok(self)
    }

    pub fn save(&mut self) -> std::result::Result<String, String> {
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

    pub fn load(path: &str) -> Result<Config, String> {
        let mut json = String::new();

        fs::File::open(path)
            .or(Err("Could not find file"))?
            .read_to_string(&mut json)
            .or(Err("Could not read to string"))?;

        serde_json::from_str::<Config>(&json).map_err(|_| format!("Could not parse JSON:\n{json}"))
    }

    pub fn reload(&self) -> Result<Self, String> {
        Config::load(&self.path)
    }

    pub fn set_path(self, path: &str) -> Config {
        Config {
            path: String::from(path),
            ..self
        }
    }

    pub fn add_project(&mut self, name: String, number: u32) {
        let projects = &mut self.projects;
        projects.insert(name, number);
    }

    pub fn remove_project(self, name: &str) -> Config {
        let mut projects = self.projects;
        projects.remove(name);

        Config { projects, ..self }
    }

    pub fn set_next_id(&self, next_id: &String) -> Config {
        let next_id: Option<String> = Some(next_id.to_owned());

        Config {
            next_id,
            ..self.clone()
        }
    }

    pub fn clear_next_id(self) -> Config {
        let next_id: Option<String> = None;

        Config { next_id, ..self }
    }

    fn check_for_latest_version(self: Config) -> Result<Config, String> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::today_string(&self)),
            ..self.clone()
        };

        if last_version != Some(time::today_string(&self)) {
            match request::get_latest_version(self) {
                Ok(version) if version.as_str() != VERSION => {
                    println!(
                        "Latest Tod version is {}, found {}.\nRun {} to update if you installed with Cargo",
                        version,
                        VERSION,
                        "cargo install tod --force".bright_cyan()
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

    fn check_for_timezone(self: Config) -> Result<Config, String> {
        if self.timezone.is_none() {
            let desc = "Please select your timezone";
            let mut options = TZ_VARIANTS
                .to_vec()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>();
            options.sort();

            let tz = select_input(desc, options)?;
            let config = Config {
                timezone: Some(tz),
                ..self
            };

            config.clone().save()?;

            Ok(config)
        } else {
            Ok(self)
        }
    }
}

pub fn get_or_create(config_path: Option<String>) -> Result<Config, String> {
    let path: String = match config_path {
        None => generate_path()?,
        Some(path) => path.trim().to_owned(),
    };
    let desc = "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

    match fs::File::open(&path) {
        Ok(_) => {
            let config = Config::load(&path)?
                .check_for_timezone()?
                .check_for_latest_version()?;

            // When we move the config file we also need to rename the path in JSON
            if config.path != path {
                let new_config = config.set_path(&path);
                new_config.clone().save()?;
                Ok(new_config)
            } else {
                Ok(config)
            }
        }
        Err(_) => {
            let token = get_input(desc)?;
            Config::new(&token, None)?.create()?.check_for_timezone()
        }
    }
}

pub fn generate_path() -> Result<String, String> {
    let config_directory = dirs::config_dir()
        .ok_or_else(|| String::from("Could not find config directory"))?
        .to_str()
        .ok_or_else(|| String::from("Could not convert config directory to string"))?
        .to_owned();
    if cfg!(test) {
        _ = fs::create_dir(format!("{config_directory}/tod_test"));
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
        Ok(format!("tests/{random_string}.testcfg"))
    } else {
        Ok(format!("{config_directory}/tod.cfg"))
    }
}

pub fn get_input(desc: &str) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from("Africa/Asmera"));
    }

    Text::new(desc).prompt().map_err(|e| e.to_string())
}
pub fn select_input<T: Display>(desc: &str, options: Vec<T>) -> Result<T, String> {
    if cfg!(test) {
        return Ok(options
            .into_iter()
            .next()
            .expect("Must provide a vector of options"));
    }
    Select::new(desc, options)
        .prompt()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new("something", None).unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn reload_config_should_work() {
        let config = crate::test::helpers::config_fixture();
        let mut config = config.create().expect("Failed to create test config");
        config.add_project("testproj".to_string(), 1);
        assert!(!&config.projects.is_empty());

        let reloaded_config = config.reload().expect("Failed to reload config");
        assert!(reloaded_config.projects.is_empty());

        delete_config(&reloaded_config.path);
    }

    #[test]
    fn set_and_clear_next_id_should_work() {
        let config = Config::new("something", None).unwrap();
        assert_eq!(config.next_id, None);
        let config = config.set_next_id(&String::from("123123"));
        assert_eq!(config.next_id, Some(String::from("123123")));
        let config = config.clear_next_id();
        assert_eq!(config.next_id, None);
    }

    #[test]
    fn add_project_should_work() {
        let mut config = Config::new("something", None).unwrap();
        let mut projects: HashMap<String, u32> = HashMap::new();
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: config.path.clone(),
                next_id: None,
                last_version_check: None,
                projects: projects.clone(),
                spinners: Some(true),
                timezone: None,
                mock_url: None,
            }
        );
        config.add_project(String::from("test"), 1234);
        projects.insert(String::from("test"), 1234);
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: config.path.clone(),
                next_id: None,
                last_version_check: None,
                spinners: Some(true),
                projects,
                timezone: None,
                mock_url: None,
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
            spinners: Some(true),
            last_version_check: None,
            projects: projects.clone(),
            timezone: Some(String::from("Asia/Pyongyang")),
            mock_url: None,
        };

        assert_eq!(
            config_with_two_projects,
            Config {
                token: String::from("something"),
                path: config_with_two_projects.path.clone(),
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                projects: projects.clone(),
                timezone: Some(String::from("Asia/Pyongyang")),
                mock_url: None,
            }
        );
        let config_with_one_project = config_with_two_projects.remove_project("test");
        let mut projects: HashMap<String, u32> = HashMap::new();
        projects.insert(String::from("test2"), 4567);
        assert_eq!(
            config_with_one_project,
            Config {
                token: String::from("something"),
                path: config_with_one_project.path.clone(),
                next_id: None,
                last_version_check: None,
                projects,
                spinners: Some(true),
                timezone: Some(String::from("Asia/Pyongyang")),
                mock_url: None,
            }
        );
    }

    #[test]
    fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.

        let server = mockito::Server::new();
        let mock_url = Some(server.url());

        // create and load
        let new_config = Config::new("faketoken", None).unwrap();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&new_config.path).unwrap();
        assert_eq!(created_config, loaded_config);

        // get_or_create (create)
        let config = get_or_create(None);
        assert_eq!(
            config,
            Ok(Config {
                token: String::from("Africa/Asmera"),
                projects: HashMap::new(),
                path: config.clone().unwrap().path,
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: Some(String::from("Africa/Abidjan")),
                mock_url: None,
            })
        );
        delete_config(&config.unwrap().path);

        // get_or_create (load)
        Config::new("alreadycreated", mock_url)
            .unwrap()
            .create()
            .unwrap();

        let config = get_or_create(None);

        assert_eq!(
            config,
            Ok(Config {
                token: String::from("Africa/Asmera"),
                projects: HashMap::new(),
                path: config.clone().unwrap().path,
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: Some(String::from("Africa/Abidjan")),
                mock_url: None,
            })
        );
        delete_config(&config.unwrap().path);
    }

    fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path), Ok(_));
    }
}
