use crate::cargo::Version;
use crate::{cargo, color, input, time, VERSION};
use chrono_tz::TZ_VARIANTS;
use rand::distributions::{Alphanumeric, DistString};
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
    pub mock_string: Option<String>,
    pub mock_select: Option<usize>,
    // Whether spinners are enabled
    pub spinners: Option<bool>,
}

impl Config {
    pub fn add_project(&mut self, name: String, number: u32) {
        let projects = &mut self.projects;
        projects.insert(name, number);
    }

    pub fn check_for_latest_version(self: Config) -> Result<Config, String> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::today_string(&self)),
            ..self.clone()
        };

        if last_version != Some(time::today_string(&self)) {
            match cargo::compare_versions(self) {
                Ok(Version::Dated(version)) => {
                    println!(
                        "Latest Tod version is {}, found {}.\nRun {} to update if you installed with Cargo",
                        version,
                        VERSION,
                        color::cyan_string("cargo install tod --force")
                    );
                    new_config.clone().save().unwrap();
                }
                Ok(Version::Latest) => (),
                Err(err) => println!(
                    "{}, {:?}",
                    color::red_string("Could not fetch Tod version from Cargo.io"),
                    err
                ),
            };
        }

        Ok(new_config)
    }

    pub fn check_for_timezone(self: Config) -> Result<Config, String> {
        if self.timezone.is_none() {
            let desc = "Please select your timezone";
            let mut options = TZ_VARIANTS
                .to_vec()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>();
            options.sort();

            let tz = input::select(desc, options, self.mock_select)?;
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

    pub fn clear_next_id(self) -> Config {
        let next_id: Option<String> = None;

        Config { next_id, ..self }
    }

    pub fn create(self) -> Result<Config, String> {
        let json = json!(self).to_string();
        let mut file = fs::File::create(&self.path).or(Err("Could not create file"))?;
        file.write_all(json.as_bytes())
            .or(Err("Could not write to file"))?;
        println!("Config successfully created in {}", &self.path);
        Ok(self)
    }

    pub fn load(path: &str) -> Result<Config, String> {
        let mut json = String::new();

        fs::File::open(path)
            .or(Err("Could not find file"))?
            .read_to_string(&mut json)
            .or(Err("Could not read to string"))?;

        serde_json::from_str::<Config>(&json).map_err(|_| format!("Could not parse JSON:\n{json}"))
    }

    pub fn new(token: &str) -> Result<Config, String> {
        let projects: HashMap<String, u32> = HashMap::new();
        Ok(Config {
            path: generate_path()?,
            token: String::from(token),
            next_id: None,
            last_version_check: None,
            timezone: None,
            spinners: Some(true),
            mock_url: None,
            mock_string: None,
            mock_select: None,
            projects,
        })
    }

    pub fn reload(&self) -> Result<Self, String> {
        Config::load(&self.path)
    }

    pub fn remove_project(self, name: &str) -> Config {
        let mut projects = self.projects;
        projects.remove(name);

        Config { projects, ..self }
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

        Ok(color::green_string("✓"))
    }

    pub fn set_next_id(&self, next_id: &String) -> Config {
        let next_id: Option<String> = Some(next_id.to_owned());

        Config {
            next_id,
            ..self.clone()
        }
    }
}

pub fn get_or_create(config_path: Option<String>) -> Result<Config, String> {
    let path: String = match config_path {
        None => generate_path()?,
        Some(path) => path.trim().to_owned(),
    };

    match fs::File::open(&path) {
        Ok(_) => Config::load(&path),
        Err(_) => {
            let desc =
                "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

            let token = input::string(desc, Some(String::new()))?;
            Config::new(&token)?.create()
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

#[cfg(test)]
mod tests {

    impl Config {
        /// add the url of the mockito server
        pub fn mock_url(self, url: String) -> Config {
            Config {
                mock_url: Some(url),
                ..self
            }
        }

        /// Mock out the string response
        pub fn mock_string(self, string: &str) -> Config {
            Config {
                mock_string: Some(string.to_string()),
                ..self
            }
        }

        /// Mock out the select response, setting the index of the response
        pub fn mock_select(self, index: usize) -> Config {
            Config {
                mock_select: Some(index),
                ..self
            }
        }
    }

    use crate::test;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new("something").unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[test]
    fn reload_config_should_work() {
        let config = test::fixtures::config();
        let mut config = config.create().expect("Failed to create test config");
        config.add_project("testproj".to_string(), 1);
        assert!(!&config.projects.is_empty());

        let reloaded_config = config.reload().expect("Failed to reload config");
        assert!(reloaded_config.projects.is_empty());

        delete_config(&reloaded_config.path);
    }

    #[test]
    fn set_and_clear_next_id_should_work() {
        let config = test::fixtures::config();
        assert_eq!(config.next_id, None);
        let config = config.set_next_id(&String::from("123123"));
        assert_eq!(config.next_id, Some(String::from("123123")));
        let config = config.clear_next_id();
        assert_eq!(config.next_id, None);
    }

    #[test]
    fn add_project_should_work() {
        let mut config = test::fixtures::config();
        let mut projects: HashMap<String, u32> = HashMap::new();
        assert_eq!(
            config,
            Config {
                token: String::from("alreadycreated"),
                path: config.path.clone(),
                next_id: None,
                last_version_check: None,
                projects: projects.clone(),
                spinners: Some(true),
                timezone: Some(String::from("US/Pacific")),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            }
        );
        config.add_project(String::from("test"), 1234);
        projects.insert(String::from("test"), 1234);
        assert_eq!(
            config,
            Config {
                token: String::from("alreadycreated"),
                path: config.path.clone(),
                next_id: None,
                last_version_check: None,
                spinners: Some(true),
                projects,
                timezone: Some(String::from("US/Pacific")),
                mock_url: None,
                mock_string: None,
                mock_select: None,
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
            mock_string: None,
            mock_select: None,
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
                mock_string: None,
                mock_select: None,
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
                mock_string: None,
                mock_select: None,
            }
        );
    }

    #[test]
    fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.

        let server = mockito::Server::new();

        // create and load
        let new_config = test::fixtures::config();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&new_config.path).unwrap();
        assert_eq!(created_config, loaded_config);

        // get_or_create (create)
        let config = get_or_create(None);
        assert_eq!(
            config,
            Ok(Config {
                token: String::new(),
                projects: HashMap::new(),
                path: config.clone().unwrap().path,
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: None,
                mock_url: None,
                mock_string: None,
                mock_select: None,
            })
        );
        delete_config(&config.unwrap().path);

        // get_or_create (load)
        test::fixtures::config()
            .mock_url(server.url())
            .create()
            .unwrap();

        let config = get_or_create(None);

        assert_eq!(
            config,
            Ok(Config {
                token: String::new(),
                projects: HashMap::new(),
                path: config.clone().unwrap().path,
                next_id: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: None,
                mock_url: None,
                mock_string: None,
                mock_select: None,
            })
        );
        delete_config(&config.unwrap().path);
    }

    fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path), Ok(_));
    }
}
