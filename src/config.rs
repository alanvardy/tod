use crate::cargo::Version;
use crate::errors::{self, Error};
use crate::id::Resource;
use crate::projects::{LegacyProject, Project};
use crate::tasks::Task;
use crate::time::{SystemTimeProvider, TimeProviderEnum};
use crate::{VERSION, cargo, color, debug, input, time, todoist};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedSender;

#[cfg(test)]
use crate::test_time::FixedTimeProvider;

const MAX_COMMENT_LENGTH: u32 = 500;
pub const DEFAULT_DEADLINE_VALUE: u8 = 30;
pub const DEFAULT_DEADLINE_DAYS: u8 = 5;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Completed {
    count: u32,
    date: String,
}

/// App configuration, serialized as json in $XDG_CONFIG_HOME/tod.cfg
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    /// The Todoist Api token
    pub token: String,
    /// List of Todoist projects and their project numbers
    #[serde(rename = "projectsv1")]
    projects: Option<Vec<Project>>,
    /// These are from the old v9 and SYNC endpoints
    #[serde(rename = "vecprojects")]
    legacy_projects: Option<Vec<LegacyProject>>,
    /// Path to config file
    pub path: String,
    /// The ID of the next task (NO LONGER IN USE)
    next_id: Option<String>,
    /// The next task, for use with complete
    #[serde(rename = "next_taskv1")]
    next_task: Option<Task>,
    /// Whether to trigger terminal bell on success
    #[serde(default)]
    pub bell_on_success: bool,
    /// Whether to trigger terminal bell on error
    #[serde(default = "bell_on_failure")]
    pub bell_on_failure: bool,
    /// A command to to run on task creation
    pub task_create_command: Option<String>,
    /// A command to run on task completion
    pub task_complete_command: Option<String>,
    /// A command to run on task comment creation
    pub task_comment_command: Option<String>,
    /// The timezone to use for the config
    timezone: Option<String>,
    pub timeout: Option<u64>,
    /// The last time we checked crates.io for the version
    pub last_version_check: Option<String>,
    pub mock_url: Option<String>,
    pub mock_string: Option<String>,
    pub mock_select: Option<usize>,
    /// Whether spinners are enabled
    pub spinners: Option<bool>,
    #[serde(default)]
    pub disable_links: bool,
    pub completed: Option<Completed>,
    // Maximum length for printing comments
    pub max_comment_length: Option<u32>,
    pub verbose: Option<bool>,
    /// Don't ask for sections
    pub no_sections: Option<bool>,
    /// Goes straight to natural language input in datetime selection
    pub natural_language_only: Option<bool>,
    pub sort_value: Option<SortValue>,

    /// For storing arguments from the commandline
    #[serde(skip)]
    pub args: Args,

    /// For storing arguments from the commandline
    #[serde(skip)]
    pub internal: Internal,
    /// Optional TimeProvider for testing, defaults to SystemTimeProvider
    #[serde(skip)]
    pub time_provider: TimeProviderEnum,
}

fn bell_on_failure() -> bool {
    true
}
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Args {
    pub verbose: bool,
    pub timeout: Option<u64>,
}

#[derive(Default, Clone, Debug)]
pub struct Internal {
    pub tx: Option<UnboundedSender<Error>>,
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
    pub deadline_value: Option<u8>,
    pub deadline_days: Option<u8>,
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
            deadline_value: Some(DEFAULT_DEADLINE_VALUE),
            deadline_days: Some(DEFAULT_DEADLINE_DAYS),
        }
    }
}

impl Config {
    /// Set timezone on Config struct only
    pub fn with_timezone(self: &Config, timezone: &str) -> Config {
        Config {
            timezone: Some(timezone.to_string()),
            ..self.clone()
        }
    }

    /// Converts legacy projects to the new projects if necessary
    pub async fn projects(self: &Config) -> Result<Vec<Project>, Error> {
        let projects = self.projects.clone().unwrap_or_default();
        let legacy_projects = self.legacy_projects.clone().unwrap_or_default();

        if !projects.is_empty() {
            Ok(projects)
        } else if legacy_projects.is_empty() {
            Ok(Vec::new())
        } else {
            let new_projects = todoist::all_projects(self, None).await?;
            let legacy_ids = legacy_projects.into_iter().map(|lp| lp.id).collect();
            let v1_ids = todoist::get_v1_ids(self, Resource::Project, legacy_ids).await?;

            let new_projects: Vec<Project> = new_projects
                .iter()
                .filter(|p| v1_ids.contains(&p.id))
                .map(|p| p.to_owned())
                .collect();

            let mut config = self.clone();
            for project in &new_projects {
                config.add_project(project.clone());
                config.save().await?;
            }
            Ok(new_projects)
        }
    }
    pub fn max_comment_length(self: &Config) -> u32 {
        self.max_comment_length.unwrap_or(MAX_COMMENT_LENGTH)
    }
    pub async fn reload_projects(self: &mut Config) -> Result<String, Error> {
        let all_projects = todoist::all_projects(self, None).await?;
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

    /// Fetches a sender for the error channel
    /// Use this to end errors from an async process
    pub fn tx(self) -> UnboundedSender<Error> {
        self.internal.tx.expect("No tx in Config")
    }

    pub async fn check_for_latest_version(self: Config) -> Result<(), Error> {
        let last_version = self.clone().last_version_check;
        let new_config = Config {
            last_version_check: Some(time::date_string_today(&self)?),
            ..self.clone()
        };

        if last_version != Some(time::date_string_today(&self)?) {
            match cargo::compare_versions(None).await {
                Ok(Version::Dated(version)) => {
                    let message = format!(
                        "Your version of Tod is out of date
                        Latest Tod version is {}, you have {} installed.
                        Run {} to update if you installed with Cargo
                        or run {} if you installed with Homebrew",
                        version,
                        VERSION,
                        color::cyan_string("cargo install tod --force"),
                        color::cyan_string("brew update && brew upgrade tod")
                    );
                    self.tx().send(Error {
                        message,
                        source: String::from("Crates.io"),
                    })?;
                    new_config.clone().save().await?;
                }
                Ok(Version::Latest) => (),
                Err(err) => self.tx().send(err)?,
            };
        };

        Ok(())
    }

    // Get timezone from config, or API if necessary
    pub fn get_timezone(&self) -> Result<String, Error> {
        self.timezone.clone().ok_or_else(|| Error {
            message: "Must set timezone".to_string(),
            source: "get_timezone".to_string(),
        })
    }

    /// Prompt user for timezone if it does not exist and write to disk
    pub async fn maybe_set_timezone(self) -> Result<Config, Error> {
        if self.timezone.is_none() {
            self.set_timezone().await
        } else {
            Ok(self)
        }
    }

    /// Set timezone and save to disk
    pub async fn set_timezone(self) -> Result<Config, Error> {
        let user = todoist::get_user_data(&self).await?;
        let mut config = self.with_timezone(&user.tz_info.timezone);
        config.save().await?;

        Ok(config)
    }

    pub fn clear_next_task(self) -> Config {
        let next_task: Option<Task> = None;

        Config { next_task, ..self }
    }

    /// Increase the completed count for today
    pub fn increment_completed(&self) -> Result<Config, Error> {
        let date = time::naive_date_today(self)?.to_string();
        let completed = match &self.completed {
            None => Some(Completed { date, count: 1 }),
            Some(completed) => {
                if completed.date == date {
                    Some(Completed {
                        count: completed.count + 1,
                        ..completed.clone()
                    })
                } else {
                    Some(Completed { date, count: 1 })
                }
            }
        };

        Ok(Config {
            completed,
            ..self.clone()
        })
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

        let config: Config = match serde_json::from_str(&json) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "\n{}",
                    color::red_string(&format!(
                        "Error loading config file '{}':\n{}\n\
                    \nYour config may contain an invalid value (e.g., a number over 255 in a `u8` field like `sort_value`).\n\
                    Please manually fix or delete the file.",
                        path, e
                    ))
                );
                if cfg!(test) {
                    return Err(e.into());
                } else {
                    std::process::exit(1);
                }
            }
        };

        let config = if config.sort_value.is_none() {
            Config {
                sort_value: Some(SortValue::default()),
                ..config
            }
        } else {
            config
        };

        Ok(config)
    }

    pub async fn new(token: &str, tx: Option<UnboundedSender<Error>>) -> Result<Config, Error> {
        Ok(Config {
            path: generate_path().await?,
            token: String::from(token),
            next_id: None,
            next_task: None,
            last_version_check: None,
            timeout: None,
            bell_on_success: false,
            bell_on_failure: true,
            sort_value: Some(SortValue::default()),
            timezone: None,
            completed: None,
            disable_links: false,
            spinners: Some(true),
            mock_url: None,
            no_sections: None,
            natural_language_only: None,
            mock_string: None,
            mock_select: None,
            max_comment_length: None,
            verbose: None,
            internal: Internal { tx },
            args: Args {
                verbose: false,
                timeout: None,
            },
            legacy_projects: Some(Vec::new()),
            time_provider: TimeProviderEnum::System(SystemTimeProvider),
            task_comment_command: None,
            task_create_command: None,
            task_complete_command: None,
            projects: Some(Vec::new()),
        })
    }

    pub async fn reload(&self) -> Result<Self, Error> {
        Config::load(&self.path).await.map(|config| Config {
            internal: self.internal.clone(),
            time_provider: self.time_provider.clone(),
            ..config
        })
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

    pub fn set_next_task(&self, task: Task) -> Config {
        let next_task: Option<Task> = Some(task);

        Config {
            next_task,
            ..self.clone()
        }
    }

    pub fn tasks_completed(&self) -> Result<u32, Error> {
        let date = time::naive_date_today(self)?.to_string();
        match &self.completed {
            None => Ok(0),
            Some(completed) => {
                if completed.date == date {
                    Ok(completed.count)
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn next_task(&self) -> Option<Task> {
        self.next_task.clone()
    }

    pub(crate) fn deadline_days(&self) -> u8 {
        self.sort_value
            .clone()
            .unwrap_or_default()
            .deadline_days
            .unwrap_or(DEFAULT_DEADLINE_DAYS)
    }

    pub(crate) fn deadline_value(&self) -> u8 {
        self.sort_value
            .clone()
            .unwrap_or_default()
            .deadline_value
            .unwrap_or(DEFAULT_DEADLINE_VALUE)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: String::new(),
            path: String::new(),
            next_id: None,
            next_task: None,
            last_version_check: None,
            timeout: None,
            bell_on_success: false,
            bell_on_failure: true,
            task_create_command: None,
            task_complete_command: None,
            task_comment_command: None,
            sort_value: Some(SortValue::default()),
            timezone: None,
            completed: None,
            disable_links: false,
            spinners: Some(true),
            mock_url: None,
            no_sections: None,
            natural_language_only: None,
            mock_string: None,
            mock_select: None,
            max_comment_length: None,
            verbose: None,
            internal: Internal { tx: None },
            args: Args {
                verbose: false,
                timeout: None,
            },
            legacy_projects: Some(Vec::new()),
            time_provider: TimeProviderEnum::System(SystemTimeProvider),
            projects: Some(Vec::new()),
        }
    }
}
/// Fetches config from from disk and creates it if it doesn't exist
/// Prompts for Todoist API token
pub async fn get_or_create(
    config_path: Option<String>,
    verbose: bool,
    timeout: Option<u64>,
    tx: &UnboundedSender<Error>,
) -> Result<Config, Error> {
    let config = match get(config_path.clone(), verbose, timeout, tx).await {
        Ok(config) => config,
        Err(_) => {
            // File not found or unreadable — prompt for token
            let desc =
                "Please enter your Todoist API token from https://todoist.com/prefs/integrations ";
            return set_token(desc, tx).await;
        }
    };

    // 🧠 Config loaded successfully — but check if token is blank
    if config.token.trim().is_empty() {
        let desc = "Your configuration file exists but the Todoist API token is empty.\nPlease enter your Todoist API token from https://todoist.com/prefs/integrations ";
        return set_token(desc, tx).await;
    }

    Ok(config)
}
//Fn to set token
pub async fn set_token(desc: &str, tx: &UnboundedSender<Error>) -> Result<Config, Error> {
    let token = input::string(desc, Some(String::new()))?;
    Config::new(&token, Some(tx.clone())).await?.create().await
}

pub async fn get(
    config_path: Option<String>,
    verbose: bool,
    timeout: Option<u64>,
    tx: &UnboundedSender<Error>,
) -> Result<Config, Error> {
    let path: String = match config_path {
        None => generate_path().await?,
        Some(path) => maybe_expand_home_dir(path)?,
    };

    let config = match fs::File::open(&path).await {
        Ok(_) => Config::load(&path).await,
        Err(_) => Err(Error {
            message: format!("Configuration file does not exist at {path}"),
            source: "config.rs".to_string(),
        }),
    }
    .map(|config| Config {
        args: Args { timeout, verbose },
        internal: Internal {
            tx: Some(tx.clone()),
        },
        ..config
    })?;

    let redacted_config = Config {
        token: "REDACTED".to_string(),
        ..config.clone()
    };
    debug::maybe_print(&config, format!("{:#?}", redacted_config));
    Ok(config)
}
/// Generates the path to the config file
///
pub async fn generate_path() -> Result<String, Error> {
    let config_directory = dirs::config_dir()
        .ok_or_else(|| errors::new("dirs", "Could not find config directory"))?
        .to_str()
        .ok_or_else(|| errors::new("dirs", "Could not convert config directory to string"))?
        .to_owned();
    if cfg!(test) {
        _ = fs::create_dir(format!("{config_directory}/tod_test")).await;
        let random_string = Alphanumeric.sample_string(&mut rand::rng(), 30);
        Ok(format!("tests/{random_string}.testcfg"))
    } else {
        Ok(format!("{config_directory}/tod.cfg"))
    }
}

fn maybe_expand_home_dir(path: String) -> Result<String, Error> {
    if path.starts_with('~') {
        let home =
            homedir::my_home()?.ok_or_else(|| errors::new("homedir", "Could not get homedir"))?;
        let mut path = path;
        path.replace_range(
            ..1,
            home.to_str()
                .ok_or_else(|| errors::new("homedir", "Could not get homedir"))?,
        );

        Ok(path)
    } else {
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    impl Config {
        //Method for default testing
        pub fn default_test() -> Self {
            Config {
                token: "default-token".to_string(),
                path: "/tmp/test.cfg".to_string(),
                time_provider: TimeProviderEnum::Fixed(FixedTimeProvider),
                args: Args {
                    verbose: false,
                    timeout: None,
                },
                internal: Internal { tx: None },
                sort_value: Some(SortValue::default()),

                projects: Some(vec![]),
                legacy_projects: Some(vec![]),
                next_id: None,
                next_task: None,
                bell_on_success: false,
                bell_on_failure: true,
                task_create_command: None,
                task_complete_command: None,
                task_comment_command: None,

                timezone: Some("UTC".to_string()),
                timeout: None,
                last_version_check: None,
                mock_url: None,
                mock_string: None,
                mock_select: None,
                spinners: None,
                disable_links: false,
                completed: None,
                max_comment_length: None,
                verbose: None,
                no_sections: None,
                natural_language_only: None,
            }
        }
        /// add the url of the mockito server
        pub fn with_mock_url(self, url: String) -> Config {
            Config {
                mock_url: Some(url),
                ..self
            }
        }

        /// Mock out the string response
        pub fn with_mock_string(self, string: &str) -> Config {
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
        /// Set path on Config struct
        pub fn with_path(self: &Config, path: String) -> Config {
            Config {
                path,
                ..self.clone()
            }
        }

        /// Set path on Config struct
        pub fn with_projects(self: &Config, projects: Vec<Project>) -> Config {
            Config {
                projects: Some(projects),
                ..self.clone()
            }
        }
        pub fn with_time_provider(mut self, provider_type: TimeProviderEnum) -> Self {
            self.time_provider = provider_type;
            self
        }
    }
    use crate::test;

    use super::*;
    use pretty_assertions::assert_eq;

    fn tx() -> UnboundedSender<Error> {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        tx
    }

    #[tokio::test]
    async fn new_should_generate_config() {
        let config = Config::new("something", None).await.unwrap();
        assert_eq!(config.token, String::from("something"));
    }

    #[tokio::test]
    async fn reload_config_should_work() {
        let config = test::fixtures::config().await;
        let mut config = config.create().await.expect("Failed to create test config");
        let project = test::fixtures::project();
        config.add_project(project);
        let projects = config.projects().await.unwrap();
        assert!(!&projects.is_empty());

        config.reload().await.expect("Failed to reload config");
    }

    #[tokio::test]
    async fn set_and_clear_next_task_should_work() {
        let config = test::fixtures::config().await;
        assert_eq!(config.next_task, None);
        let task = test::fixtures::today_task().await;
        let config = config.set_next_task(task.clone());
        assert_eq!(config.next_task, Some(task));
        let config = config.clear_next_task();
        assert_eq!(config.next_task, None);
    }

    #[tokio::test]
    async fn add_project_should_work() {
        let mut config = test::fixtures::config().await;
        let projects_count = config.projects().await.unwrap().len();
        assert_eq!(projects_count, 1);
        config.add_project(test::fixtures::project());
        let projects_count = config.projects().await.unwrap().len();
        assert_eq!(projects_count, 2);
    }

    #[tokio::test]
    async fn remove_project_should_work() {
        let mut config = test::fixtures::config().await;
        let projects = config.projects().await.unwrap();
        let project = projects.first().unwrap();
        let projects_count = config.projects().await.unwrap().len();
        assert_eq!(projects_count, 1);
        config.remove_project(project);
        let projects_count = config.projects().await.unwrap().len();
        assert_eq!(projects_count, 0);
    }

    #[test]
    fn test_maybe_expand_home_dir() {
        let actual = maybe_expand_home_dir("/Users/tod.cfg".to_string());

        let split = actual.unwrap();
        let mut split = split.split('/');

        assert_eq!(split.next(), Some(""));
        split.next();
        assert_eq!(split.next(), Some("tod.cfg"));
    }

    #[tokio::test]
    async fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.
        let server = mockito::Server::new_async().await;

        // --- CREATE ---
        let new_config = test::fixtures::config().await;
        let created_config = Config {
            token: String::from("created"),
            ..new_config.clone()
        };
        created_config.create().await.unwrap();

        // --- LOAD ---
        let loaded_config = Config::load(&new_config.path).await.unwrap();
        assert_eq!(loaded_config.token, "created");

        // --- GET_OR_CREATE (create) ---
        let config_created = get_or_create(None, false, None, &tx())
            .await
            .expect("Could not get or create config (create)");
        delete_config(&config_created.path).await;

        // --- GET_OR_CREATE (load) ---
        let test_config = test::fixtures::config().await.with_mock_url(server.url());
        test_config.create().await.unwrap();

        let config_loaded = get_or_create(None, false, None, &tx())
            .await
            .expect("Could not get or create config (load)");

        assert!(config_loaded.internal.tx.is_some());

        // --- GET_OR_CREATE (get) ---

        let fetched_config = get(Some(config_loaded.path), false, None, &tx()).await;
        assert_matches!(fetched_config, Ok(Config { .. }));
        delete_config(&fetched_config.unwrap().path).await;
    }

    async fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path).await, Ok(_));
    }

    #[tokio::test]
    async fn load_should_fail_on_invalid_u8_value() {
        use tokio::fs::write;

        let bad_config_path = "tests/bad_config_invalid_u8.cfg";
        let contents = r#"{
        "token": "abc123",
        "path": "tests/bad_config_invalid_u8.cfg",
        "sort_value": {
            "priority_none": 500
        }
    }"#;

        write(bad_config_path, contents).await.unwrap();

        let result = Config::load(bad_config_path).await;
        assert!(result.is_err(), "Expected error from invalid u8");

        fs::remove_file(bad_config_path).await.unwrap();
    }

    #[tokio::test]
    async fn debug_impl_for_config_should_work() {
        let config = test::fixtures::config().await;
        let debug_output = format!("{:?}", config);

        // You can assert that it contains expected fields just to be thorough
        assert!(debug_output.contains("Config"));
        assert!(debug_output.contains("token"));
        assert!(debug_output.contains(&config.token));
    }
    #[test]
    fn debug_impls_for_config_components_should_work() {
        use tokio::sync::mpsc::unbounded_channel;

        let args = Args {
            verbose: true,
            timeout: Some(42),
        };
        let args_debug = format!("{:?}", args);
        assert!(args_debug.contains("Args"));
        assert!(args_debug.contains("verbose"));
        assert!(args_debug.contains("timeout"));

        let (tx, _rx) = unbounded_channel::<Error>();
        let internal = Internal { tx: Some(tx) };
        let internal_debug = format!("{:?}", internal);
        assert!(internal_debug.contains("Internal"));

        let sort_value = SortValue::default();
        let sort_value_debug = format!("{:?}", sort_value);
        assert!(sort_value_debug.contains("SortValue"));
        assert!(sort_value_debug.contains("priority_none"));
        assert!(sort_value_debug.contains("deadline_value"));
    }
    #[test]
    fn trait_impls_for_config_components_should_work() {
        // Clone
        let args = Args {
            verbose: true,
            timeout: Some(10),
        };
        let args_clone = args.clone();
        assert_eq!(args, args_clone);

        let internal = Internal { tx: None };
        let internal_clone = internal.clone();
        assert_eq!(internal.tx.is_none(), internal_clone.tx.is_none());

        let sort_value = SortValue::default();
        let sort_value_clone = sort_value.clone();
        assert_eq!(sort_value, sort_value_clone);

        // PartialEq
        assert_eq!(
            args,
            Args {
                verbose: true,
                timeout: Some(10)
            }
        );
        assert_ne!(
            args,
            Args {
                verbose: false,
                timeout: Some(5)
            }
        );

        // Default
        let default_args = Args::default();
        assert_eq!(default_args.verbose, false);
        assert_eq!(default_args.timeout, None);

        let default_internal = Internal::default();
        assert!(default_internal.tx.is_none());

        let default_sort = SortValue::default();
        assert_eq!(default_sort.priority_none, 2);
        assert_eq!(default_sort.deadline_value, Some(DEFAULT_DEADLINE_VALUE));
    }
    #[tokio::test]
    async fn test_config_with_methods() {
        let base_config = Config::new("token_value", None)
            .await
            .expect("Failed to create base config");

        let tz_config = base_config.with_timezone("America/New_York");
        assert_eq!(tz_config.timezone, Some("America/New_York".to_string()));

        let mock_url = "http://localhost:1234";
        let mock_config = base_config.clone().with_mock_url(mock_url.to_string());
        assert_eq!(mock_config.mock_url, Some(mock_url.to_string()));

        let mock_str_config = base_config.clone().with_mock_string("mock response");
        assert_eq!(
            mock_str_config.mock_string,
            Some("mock response".to_string())
        );

        let select_config = base_config.clone().mock_select(2);
        assert_eq!(select_config.mock_select, Some(2));

        let path_config = base_config.with_path("some/test/path.cfg".to_string());
        assert_eq!(path_config.path, "some/test/path.cfg".to_string());

        let projects = vec![Project {
            id: "test123".to_string(),
            can_assign_tasks: true,
            child_order: 0,
            color: "blue".to_string(),
            created_at: None,
            is_archived: false,
            is_deleted: false,
            is_favorite: false,
            is_frozen: false,
            name: "Test Project".to_string(),
            updated_at: None,
            view_style: "list".to_string(),
            default_order: 0,
            description: "desc".to_string(),
            parent_id: None,
            inbox_project: None,
            is_collapsed: false,
            is_shared: false,
        }];
        let project_config = Config {
            projects: Some(projects.clone()),
            ..base_config.clone()
        };
        assert!(project_config.projects.is_some());
    }

    #[test]
    // Test function to check the debug output of Config
    fn test_config_debug_with_time_provider() {
        let config = Config::default_test()
            .with_time_provider(TimeProviderEnum::Fixed(FixedTimeProvider))
            .with_path("/tmp/test.cfg".to_string());

        let debug_output = format!("{:?}", config);

        // Check some expected fields are present in the debug output
        assert!(debug_output.contains("Config"));
        assert!(debug_output.contains("/tmp/test.cfg"));
    }
}
