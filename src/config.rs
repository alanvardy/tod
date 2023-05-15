use crate::{request, time, VERSION};
use chrono_tz::TZ_VARIANTS;
use colored::*;
use inquire::{Select, Text};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
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

    pub fn add_project(self, name: String, number: u32) -> Config {
        let mut projects = self.projects;
        projects.insert(name, number);

        Config { projects, ..self }
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
            let options = TZ_VARIANTS
                .to_vec()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>();

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

    if !path_exists(&path) {
        // We used to store config in $HOME/.tod.cfg
        // This moves it to new path
        let legacy_path = generate_legacy_path()?;
        if path_exists(&legacy_path) {
            println!("INFO: Moving the config file from \"{legacy_path}\" to \"{path}\".\n");
            fs::rename(legacy_path, &path).map_err(|e| e.to_string())?;
        }
    }

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

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

pub fn generate_path() -> Result<String, String> {
    let filename = if cfg!(test) { "test" } else { "tod.cfg" };

    let config_directory = dirs::config_dir()
        .ok_or_else(|| String::from("Could not find config directory"))?
        .to_str()
        .ok_or_else(|| String::from("Could not convert config directory to string"))?
        .to_owned();
    Ok(format!("{config_directory}/{filename}"))
}

pub fn generate_legacy_path() -> Result<String, String> {
    let filename = if cfg!(test) { "test" } else { ".tod.cfg" };

    let home_directory = dirs::home_dir()
        .ok_or_else(|| String::from("Could not find home directory"))?
        .to_str()
        .ok_or_else(|| String::from("Could not convert directory to string"))?
        .to_owned();
    Ok(format!("{home_directory}/{filename}"))
}

pub fn get_input(desc: &str) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from("Africa/Asmera"));
    }

    Text::new(desc).prompt().map_err(|e| e.to_string())
}
pub fn select_input(desc: &str, options: Vec<String>) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from("Africa/Asmera"));
    }

    Select::new(desc, options)
        .prompt()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new("something", None).unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn reload_config_should_work() {
        let mut config = crate::test::helpers::config_fixture();
        let path = format!("{}{}", config.path, "reload");
        config.path = path;
        let mut config = config.create().expect("Failed to create test config");
        config = config.add_project("testproj".to_string(), 1);
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
        let config = Config::new("something", None).unwrap();
        let mut projects: HashMap<String, u32> = HashMap::new();
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: None,
                projects: projects.clone(),
                spinners: Some(true),
                timezone: None,
                mock_url: None,
            }
        );
        let config = config.add_project(String::from("test"), 1234);
        projects.insert(String::from("test"), 1234);
        assert_eq!(
            config,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
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
                path: generate_path().unwrap(),
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
                path: generate_path().unwrap(),
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

        // Save and load
        // Build path
        let config_directory = dirs::config_dir().expect("could not get home directory");
        let config_directory_str = config_directory
            .to_str()
            .expect("could not set home directory to str");
        let path = format!("{}/test", config_directory_str);

        // Just in case there is a leftover config from a previous test run
        let _ = fs::remove_file(&path);

        // create and load
        let new_config = Config::new("faketoken", None).unwrap();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&path).unwrap();
        assert_eq!(created_config, loaded_config);

        // save and load
        let different_new_config = Config::new("differenttoken", mock_url.clone()).unwrap();
        different_new_config.clone().save().unwrap();
        let loaded_config = Config::load(&path).unwrap();
        assert_eq!(loaded_config, different_new_config);
        delete_config(&path);

        // get_or_create (create)
        let config = get_or_create(None);
        assert_eq!(
            config.clone(),
            Ok(Config {
                token: String::from("Africa/Asmera"),
                projects: HashMap::new(),
                path: generate_path().unwrap(),
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: Some(String::from("Africa/Asmera")),
                mock_url: None,
            })
        );
        delete_config(&path);

        // get_or_create (load)
        Config::new("alreadycreated", mock_url.clone())
            .unwrap()
            .create()
            .unwrap();

        let config = get_or_create(None);

        assert_eq!(
            config.clone(),
            Ok(Config {
                token: String::from("alreadycreated"),
                projects: HashMap::new(),
                path: generate_path().unwrap(),
                next_id: None,
                spinners: Some(true),
                last_version_check: Some(time::today_string(&config.unwrap())),
                timezone: Some(String::from("Africa/Asmera")),
                mock_url: mock_url.clone(),
            })
        );
        delete_config(&path);

        // get_or_create (move legacy)
        Config::new("created in $HOME", mock_url.clone())
            .unwrap()
            .create()
            .unwrap();
        let legacy_path = generate_legacy_path().unwrap();
        let proper_path = generate_path().unwrap();
        fs::rename(proper_path, &legacy_path).unwrap();
        let config = get_or_create(None);
        assert_eq!(
            config.clone(),
            Ok(Config {
                token: String::from("created in $HOME"),
                projects: HashMap::new(),
                path: generate_path().unwrap(),
                next_id: None,
                spinners: Some(true),
                last_version_check: Some(time::today_string(&config.unwrap())),
                timezone: Some(String::from("Africa/Asmera")),
                mock_url: mock_url,
            })
        );
        delete_config(&path);
    }

    #[test]
    fn custom_config_path() {
        let path = String::from("./tests/tod.cfg");
        let loaded_config = Config::load(&path).unwrap();

        let mut projects = HashMap::new();
        projects.insert(String::from("home"), 2255636821);
        projects.insert(String::from("inbox"), 337585113);
        projects.insert(String::from("work"), 2243742250);

        let config = Config {
            token: String::from("23984719029"),
            timezone: Some(String::from("US/Pacific")),
            last_version_check: Some(String::from("2023-04-01")),
            projects,
            spinners: Some(false),
            path: String::from("tests/tod.cfg"),
            next_id: None,
            mock_url: None,
        };
        assert_eq!(loaded_config, config);
    }

    fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path), Ok(_));
    }
}
