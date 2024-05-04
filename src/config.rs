use crate::cargo::Version;
use crate::projects::Project;
use crate::{cargo, color, input, time, todoist, VERSION};
use chrono_tz::TZ_VARIANTS;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{Read, Write};

/// App configuration, serialized as json in $XDG_CONFIG_HOME/tod.cfg
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    /// The Todoist Api token
    pub token: String,
    /// List of Todoist projects and their project numbers
    #[serde(rename = "vecprojects")]
    pub projects: Option<Vec<Project>>,
    /// Path to config file
    pub path: String,
    /// The ID of the next task
    pub next_id: Option<String>,
    pub timezone: Option<String>,
    pub timeout: Option<u64>,
    /// The last time we checked crates.io for the version
    pub last_version_check: Option<String>,
    pub mock_url: Option<String>,
    pub mock_string: Option<String>,
    pub mock_select: Option<usize>,
    /// Whether spinners are enabled
    pub spinners: Option<bool>,
    pub verbose: Option<bool>,
    /// Don't ask for sections
    pub no_sections: Option<bool>,
    /// Goes straight to natural language input in datetime selection
    pub natural_language_only: Option<bool>,
    pub sort_value: Option<SortValue>,

    /// For storing arguments from the commandline
    #[serde(skip)]
    pub args: Args,
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Args {
    pub verbose: bool,
    pub timeout: Option<u64>,
}

// Determining how
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct SortValue {
    /// Task has one of these priorities
    pub priority_none: u8,
    pub priority_low: u8,
    pub priority_medium: u8,
    pub priority_high: u8,
    pub no_due_date: u8,
    pub not_recurring: u8,
    pub today: u8,
    pub overdue: u8,
    /// Happens now plus or minus 15min
    pub now: u8,
}

impl Default for SortValue {
    fn default() -> Self {
        SortValue {
            priority_none: 2,
            priority_low: 1,
            priority_medium: 3,
            priority_high: 4,
            no_due_date: 80,
            overdue: 150,
            not_recurring: 50,
            today: 100,
            now: 200,
        }
    }
}
impl Config {
    pub fn reload_projects(self: &mut Config) -> Result<String, String> {
        let all_projects = todoist::projects(self)?;
        let current_projects = self.projects.clone().unwrap_or_default();
        let current_project_ids: Vec<String> =
            current_projects.iter().map(|p| p.id.to_owned()).collect();

        let updated_projects = all_projects
            .iter()
            .filter(|p| current_project_ids.contains(&p.id))
            .map(|p| p.to_owned())
            .collect::<Vec<Project>>();

        self.projects = Some(updated_projects);

        Ok(color::green_string("✓"))
    }

    pub fn check_for_latest_version(self: Config) -> Result<Config, String> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::today_string(&self)?),
            ..self.clone()
        };

        if last_version != Some(time::today_string(&self)?) {
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

        let config = serde_json::from_str::<Config>(&json)
            .map_err(|_| format!("Could not parse JSON:\n{json}"))?;

        match config.sort_value {
            None => Ok(Config {
                sort_value: Some(SortValue::default()),
                ..config
            }),
            Some(_) => Ok(config),
        }
    }

    pub fn new(token: &str) -> Result<Config, String> {
        Ok(Config {
            path: generate_path()?,
            token: String::from(token),
            next_id: None,
            last_version_check: None,
            timeout: None,
            sort_value: Some(SortValue::default()),
            timezone: None,
            spinners: Some(true),
            mock_url: None,
            no_sections: None,
            natural_language_only: None,
            mock_string: None,
            mock_select: None,
            verbose: None,
            args: Args {
                verbose: false,
                timeout: None,
            },
            projects: Some(Vec::new()),
        })
    }

    pub fn reload(&self) -> Result<Self, String> {
        Config::load(&self.path)
    }

    pub fn add_project(&mut self, project: Project) {
        let option_projects = &mut self.projects;
        match option_projects {
            Some(projects) => {
                projects.push(project);
            }
            None => self.projects = Some(vec![project]),
        }
    }

    pub fn remove_project(&mut self, project: &Project) {
        let projects = self
            .projects
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|p| p.id != project.id)
            .map(|p| p.to_owned())
            .collect::<Vec<Project>>();

        self.projects = Some(projects);
    }

    pub fn save(&mut self) -> std::result::Result<String, String> {
        // We don't want to overwrite verbose in the config
        let config = match Config::load(&self.path) {
            Ok(Config { verbose, .. }) => Config {
                verbose,
                ..self.clone()
            },
            _ => self.clone(),
        };

        let json = json!(config);
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

pub fn get_or_create(
    config_path: Option<String>,
    verbose: bool,
    timeout: Option<u64>,
) -> Result<Config, String> {
    let path: String = match config_path {
        None => generate_path()?,
        Some(path) => maybe_expand_home_dir(path)?,
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
    .map(|config| Config {
        args: Args { timeout, verbose },
        ..config
    })
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

fn maybe_expand_home_dir(path: String) -> Result<String, String> {
    if path.starts_with('~') {
        let home = homedir::get_my_home()
            .or(Err(String::from("Could not get homedir")))?
            .ok_or_else(|| String::from("Could not get homedir"))?;
        let mut path = path;
        path.replace_range(
            ..1,
            home.to_str()
                .ok_or_else(|| String::from("Could not get homedir"))?,
        );

        Ok(path)
    } else {
        Ok(path)
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
        let project = test::fixtures::project();
        config.add_project(project);
        let projects = config.projects.clone().unwrap_or_default();
        assert!(!&projects.is_empty());

        config.reload().expect("Failed to reload config");
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
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 1);
        config.add_project(test::fixtures::project());
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 2);
    }

    #[test]
    fn remove_project_should_work() {
        let mut config = test::fixtures::config();
        let projects = config.projects.clone().unwrap_or_default();
        let project = projects.first().unwrap();
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 1);
        config.remove_project(project);
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 0);
    }

    #[test]
    fn test_maybe_expand_home_dir() {
        let expected = Ok(String::from("/home/vardy/tod.cfg"));
        let actual = maybe_expand_home_dir(expected.clone().unwrap());
        assert_eq!(expected, actual);

        let actual = maybe_expand_home_dir("~/tod.cfg".to_string());

        let split = actual.unwrap();
        let mut split = split.split('/');

        assert_eq!(split.next(), Some(""));
        assert_eq!(split.next(), Some("home"));
        // This is machine dependent
        split.next();
        assert_eq!(split.next(), Some("tod.cfg"));
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
        let config = get_or_create(None, false, None);
        assert_eq!(
            config,
            Ok(Config {
                token: String::new(),
                projects: Some(Vec::new()),
                path: config.clone().unwrap().path,
                timeout: None,
                no_sections: None,
                next_id: None,
                args: Args {
                    verbose: false,
                    timeout: None,
                },
                spinners: Some(true),
                sort_value: Some(SortValue::default()),
                last_version_check: None,
                timezone: None,
                natural_language_only: None,
                mock_url: None,
                verbose: None,
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

        let config = get_or_create(None, false, None);

        assert_eq!(
            config,
            Ok(Config {
                token: String::new(),
                projects: Some(Vec::new()),
                path: config.clone().unwrap().path,
                sort_value: Some(SortValue::default()),
                next_id: None,
                timeout: None,
                args: Args {
                    verbose: false,
                    timeout: None,
                },
                no_sections: None,
                spinners: Some(true),
                last_version_check: None,
                timezone: None,
                natural_language_only: None,
                verbose: None,
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
