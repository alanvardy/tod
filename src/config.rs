use crate::cargo::Version;
use crate::error::{self, Error};
use crate::projects::Project;
use crate::{cargo, color, input, time, todoist, VERSION};
use chrono_tz::TZ_VARIANTS;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedSender;

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
    pub async fn reload_projects(self: &mut Config) -> Result<String, Error> {
        let all_projects = todoist::projects(self).await?;
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

    pub async fn check_for_latest_version(
        self: Config,
        tx: UnboundedSender<Error>,
    ) -> Result<(), Error> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::today_string(&self)?),
            ..self.clone()
        };

        if last_version != Some(time::today_string(&self)?) {
            match cargo::compare_versions(self).await {
                Ok(Version::Dated(version)) => {
                    let message = format!(
                        "Latest Tod version is {}, found {}.\nRun {} to update if you installed with Cargo",
                        version,
                        VERSION,
                        color::cyan_string("cargo install tod --force")
                    );
                    tx.send(Error {
                        message,
                        source: String::from("Crates.io"),
                    })?;
                    new_config.clone().save().await?;
                }
                Ok(Version::Latest) => (),
                Err(err) => tx.send(err)?,
            };
        };

        Ok(())
    }

    pub async fn check_for_timezone(self: Config) -> Result<Config, Error> {
        if self.timezone.is_none() {
            let desc = "Please select your timezone. This should match your Timezone setting within Todoist";
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

            config.clone().save().await?;

            Ok(config)
        } else {
            Ok(self)
        }
    }

    pub fn clear_next_id(self) -> Config {
        let next_id: Option<String> = None;

        Config { next_id, ..self }
    }

    pub async fn create(self) -> Result<Config, Error> {
        let json = json!(self).to_string();
        let mut file = fs::File::create(&self.path).await?;
        // file.write_all(json.as_bytes())?;
        fs::File::write_all(&mut file, json.as_bytes()).await?;
        println!("Config successfully created in {}", &self.path);
        Ok(self)
    }

    pub async fn load(path: &str) -> Result<Config, Error> {
        let mut json = String::new();
        fs::File::open(path)
            .await?
            .read_to_string(&mut json)
            .await?;
        let config = serde_json::from_str::<Config>(&json)?;

        match config.sort_value {
            None => Ok(Config {
                sort_value: Some(SortValue::default()),
                ..config
            }),
            Some(_) => Ok(config),
        }
    }

    pub async fn new(token: &str) -> Result<Config, Error> {
        Ok(Config {
            path: generate_path().await?,
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

    pub async fn reload(&self) -> Result<Self, Error> {
        Config::load(&self.path).await
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

    pub async fn save(&mut self) -> std::result::Result<String, Error> {
        // We don't want to overwrite verbose in the config
        let config = match Config::load(&self.path).await {
            Ok(Config { verbose, .. }) => Config {
                verbose,
                ..self.clone()
            },
            _ => self.clone(),
        };

        let json = json!(config);
        let string = serde_json::to_string_pretty(&json)?;
        fs::OpenOptions::new()
            .write(true)
            .read(true)
            .truncate(true)
            .open(&self.path)
            .await?
            .write_all(string.as_bytes())
            .await?;

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

pub async fn get_or_create(
    config_path: Option<String>,
    verbose: bool,
    timeout: Option<u64>,
) -> Result<Config, Error> {
    let path: String = match config_path {
        None => generate_path().await?,
        Some(path) => maybe_expand_home_dir(path)?,
    };

    match fs::File::open(&path).await {
        Ok(_) => Config::load(&path).await,
        Err(_) => {
            let desc =
                "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";

            let token = input::string(desc, Some(String::new()))?;
            Config::new(&token).await?.create().await
        }
    }
    .map(|config| Config {
        args: Args { timeout, verbose },
        ..config
    })
}

pub async fn generate_path() -> Result<String, Error> {
    let config_directory = dirs::config_dir()
        .ok_or_else(|| error::new("dirs", "Could not find config directory"))?
        .to_str()
        .ok_or_else(|| error::new("dirs", "Could not convert config directory to string"))?
        .to_owned();
    if cfg!(test) {
        _ = fs::create_dir(format!("{config_directory}/tod_test")).await;
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
        Ok(format!("tests/{random_string}.testcfg"))
    } else {
        Ok(format!("{config_directory}/tod.cfg"))
    }
}

fn maybe_expand_home_dir(path: String) -> Result<String, Error> {
    if path.starts_with('~') {
        let home = homedir::get_my_home()?
            .ok_or_else(|| error::new("homedir", "Could not get homedir"))?;
        let mut path = path;
        path.replace_range(
            ..1,
            home.to_str()
                .ok_or_else(|| error::new("homedir", "Could not get homedir"))?,
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

    #[tokio::test]
    async fn new_should_generate_config() {
        let config = Config::new("something").await.unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[tokio::test]
    async fn reload_config_should_work() {
        let config = test::fixtures::config().await;
        let mut config = config.create().await.expect("Failed to create test config");
        let project = test::fixtures::project();
        config.add_project(project);
        let projects = config.projects.clone().unwrap_or_default();
        assert!(!&projects.is_empty());

        config.reload().await.expect("Failed to reload config");
    }

    #[tokio::test]
    async fn set_and_clear_next_id_should_work() {
        let config = test::fixtures::config().await;
        assert_eq!(config.next_id, None);
        let config = config.set_next_id(&String::from("123123"));
        assert_eq!(config.next_id, Some(String::from("123123")));
        let config = config.clear_next_id();
        assert_eq!(config.next_id, None);
    }

    #[tokio::test]
    async fn add_project_should_work() {
        let mut config = test::fixtures::config().await;
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 1);
        config.add_project(test::fixtures::project());
        let projects_count = config.projects.clone().unwrap_or_default().len();
        assert_eq!(projects_count, 2);
    }

    #[tokio::test]
    async fn remove_project_should_work() {
        let mut config = test::fixtures::config().await;
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

    #[tokio::test]
    async fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.

        let server = mockito::Server::new_async().await;

        // create and load
        let new_config = test::fixtures::config().await;
        let created_config = new_config.clone().create().await.unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&new_config.path).await.unwrap();
        assert_eq!(created_config, loaded_config);

        // get_or_create (create)
        let config = get_or_create(None, false, None).await;
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
        delete_config(&config.unwrap().path).await;

        // get_or_create (load)
        test::fixtures::config()
            .await
            .mock_url(server.url())
            .create()
            .await
            .unwrap();

        let config = get_or_create(None, false, None).await;

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

        delete_config(&config.unwrap().path).await;
    }

    async fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path).await, Ok(_));
    }
}
