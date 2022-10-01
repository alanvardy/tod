use crate::{request, time, VERSION};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::{fs, io};

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
    pub next_id: Option<u64>,
    pub timezone: Option<String>,
    pub last_version_check: Option<String>,
}

impl Config {
    pub fn new(token: &str) -> Result<Config, String> {
        let projects: HashMap<String, u32> = HashMap::new();
        Ok(Config {
            path: generate_path()?,
            token: String::from(token),
            next_id: None,
            last_version_check: None,
            timezone: None,
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

    pub fn load(path: &str) -> Result<Config, String> {
        let mut json = String::new();

        fs::File::open(path)
            .or(Err("Could not find file"))?
            .read_to_string(&mut json)
            .or(Err("Could not read to string"))?;

        serde_json::from_str::<Config>(&json).map_err(|_| String::from("Could not parse JSON"))
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

    pub fn set_next_id(&self, next_id: u64) -> Config {
        let next_id: Option<u64> = Some(next_id);

        Config {
            next_id,
            ..self.clone()
        }
    }

    pub fn clear_next_id(self) -> Config {
        let next_id: Option<u64> = None;

        Config { next_id, ..self }
    }

    fn check_for_latest_version(self: Config) -> Result<Config, String> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::today_string(&self)),
            ..self.clone()
        };

        if last_version != Some(time::today_string(&self)) {
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

    fn check_for_timezone(self: Config) -> Result<Config, String> {
        if self.timezone.is_none() {
            time::list_timezones();
            let desc = "Please enter the number of your timezone";
            let num: usize = get_input(desc)?
                .parse::<usize>()
                .map_err(|_| String::from("Could not parse string into number"))?;
            let config = Config {
                timezone: Some(time::get_timezone(num)),
                ..self
            };

            config.clone().save()?;

            Ok(config)
        } else {
            Ok(self)
        }
    }
}

pub fn get_or_create(config_path: Option<&str>) -> Result<Config, String> {
    let path: String = match config_path {
        None => generate_path()?,
        Some(path) => String::from(path).trim().to_owned(),
    };
    let desc = "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

    if !path_exists(&path) {
        // We used to store config in $HOME/.tod.cfg
        // This moves it to new path
        let legacy_path = generate_legacy_path()?;
        if path_exists(&legacy_path) {
            println!(
                "INFO: Moving the config file from \"{}\" to \"{}\".\n",
                legacy_path, path
            );
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
            Config::new(&token)?.create()?.check_for_timezone()
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
    Ok(format!("{}/{}", config_directory, filename))
}

pub fn generate_legacy_path() -> Result<String, String> {
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
        return Ok(String::from("5"));
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
                last_version_check: None,
                projects: projects.clone(),
                timezone: None,
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
                projects,
                timezone: None,
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
            timezone: Some(String::from("Asia/Pyongyang")),
        };

        assert_eq!(
            config_with_two_projects,
            Config {
                token: String::from("something"),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: None,
                projects: projects.clone(),
                timezone: Some(String::from("Asia/Pyongyang")),
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
                timezone: Some(String::from("Asia/Pyongyang")),
            }
        );
    }

    #[test]
    fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.

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
        let new_config = Config::new("faketoken").unwrap();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&path).unwrap();
        assert_eq!(created_config, loaded_config);

        // save and load
        let different_new_config = Config::new("differenttoken").unwrap();
        different_new_config.clone().save().unwrap();
        let loaded_config = Config::load(&path).unwrap();
        assert_eq!(loaded_config, different_new_config);
        delete_config(&path);

        // get_or_create (create)
        let config = get_or_create(None);
        assert_eq!(
            config.clone(),
            Ok(Config {
                token: String::from("5"),
                projects: HashMap::new(),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: None,
                timezone: Some(String::from("Africa/Asmera")),
            })
        );
        delete_config(&path);

        // get_or_create (load)
        Config::new("alreadycreated").unwrap().create().unwrap();
        let config = get_or_create(None);
        assert_eq!(
            config.clone(),
            Ok(Config {
                token: String::from("alreadycreated"),
                projects: HashMap::new(),
                path: generate_path().unwrap(),
                next_id: None,
                last_version_check: Some(time::today_string(&config.unwrap())),
                timezone: Some(String::from("Africa/Asmera")),
            })
        );
        delete_config(&path);

        // get_or_create (move legacy)
        Config::new("created in $HOME").unwrap().create().unwrap();
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
                last_version_check: Some(time::today_string(&config.unwrap())),
                timezone: Some(String::from("Africa/Asmera")),
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
            last_version_check: Some(String::from("2022-02-26")),
            projects,
            path: String::from("/home/vardy/dev/tod/tod.cfg"),
            next_id: Some(3592652665),
        };
        assert_eq!(loaded_config, config);
    }

    fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path), Ok(_));
    }
}
