//! An unofficial Todoist command-line client. Takes simple input and dumps it in your inbox or another project. Takes advantage of natural language processing to assign due dates, tags, etc. Designed for single tasking in a world filled with distraction.
//!
//! Get started with `cargo install tod`
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use cargo::Version;
use clap::{Parser, Subcommand};
use config::Config;
use errors::Error;
use input::DateTimeInput;
use lists::Flag;
use shell::Shell;
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};
use tasks::priority::Priority;
use tasks::{SortOrder, TaskAttribute, priority};
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

use crate::config::config_reset;

mod cargo;
mod color;
mod comments;
mod config;
mod debug;
mod errors;
mod filters;
mod id;
mod input;
mod labels;
mod lists;
mod oauth;
mod projects;
mod sections;
mod shell;
mod tasks;
mod test;
mod test_time;
mod time;
mod todoist;
mod users;
// Values pulled from Cargo.toml
const NAME: &str = env!("CARGO_PKG_NAME");
const LOWERCASE_NAME: &str = "tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("BUILD_TARGET"),
    "-",
    env!("BUILD_PROFILE"),
    ")"
);
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");
// Verbose values set at build time
const BUILD_TARGET: &str = env!("BUILD_TARGET");
const BUILD_PROFILE: &str = env!("BUILD_PROFILE");
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
const NO_PROJECTS_ERR: &str = "No projects in config. Add projects with `tod project import`";

#[derive(Parser, Clone)]
#[command(name = NAME)]
#[command(author = AUTHOR)]
#[command(version = LONG_VERSION)]
#[command(about = ABOUT, long_about = None)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    /// Display additional debug info while processing
    verbose: bool,

    #[arg(short, long)]
    /// Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/tod.cfg
    config: Option<PathBuf>,

    #[arg(short, long)]
    /// Time to wait for a response from API in seconds. Defaults to 30.
    timeout: Option<u64>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(subcommand)]
    #[clap(alias = "p")]
    /// (p) Commands that change projects
    Project(ProjectCommands),

    #[command(subcommand)]
    #[clap(alias = "n")]
    /// (n) Commands that change projects
    Section(SectionCommands),

    #[command(subcommand)]
    #[clap(alias = "t")]
    /// (t) Commands for individual tasks
    Task(TaskCommands),

    #[command(subcommand)]
    #[clap(alias = "l")]
    /// (l) Commands for multiple tasks
    List(ListCommands),

    #[command(subcommand)]
    #[clap(alias = "c")]
    /// (c) Commands around configuration and the app
    Config(ConfigCommands),

    #[command(subcommand)]
    #[clap(alias = "a")]
    /// (a) Commands for logging in with OAuth
    Auth(AuthCommands),

    #[command(subcommand)]
    #[clap(alias = "s")]
    /// (s) Commands for generating shell completions
    Shell(ShellCommands),

    #[command(subcommand)]
    #[clap(alias = "e")]
    /// (e) Commands for manually testing Tod against the API
    Test(TestCommands),
}

// -- PROJECTS --

#[derive(Subcommand, Debug, Clone)]
enum ProjectCommands {
    #[clap(alias = "c")]
    /// (c) Create a new project in Todoist and add to config
    Create(ProjectCreate),

    #[clap(alias = "l")]
    /// (l) List all of the projects in config
    List(ProjectList),

    #[clap(alias = "r")]
    /// (r) Remove a project from config (not Todoist)
    Remove(ProjectRemove),

    #[clap(alias = "d")]
    /// (d) Remove a project from Todoist
    Delete(ProjectDelete),

    #[clap(alias = "n")]
    /// (n) Rename a project in config (not in Todoist)
    Rename(ProjectRename),

    #[clap(alias = "i")]
    /// (i) Get projects from Todoist and prompt to add to config
    Import(ProjectImport),

    #[clap(alias = "e")]
    /// (e) Empty a project by putting tasks in other projects"
    Empty(ProjectEmpty),
}

#[derive(Parser, Debug, Clone)]
struct ProjectList {}

#[derive(Parser, Debug, Clone)]
struct ProjectCreate {
    #[arg(short, long)]
    /// Project name
    name: Option<String>,

    #[arg(short, long)]
    /// Project description
    description: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Whether the project is marked as favorite
    is_favorite: bool,
}

#[derive(Parser, Debug, Clone)]
struct ProjectImport {
    #[arg(short = 'a', long, default_value_t = false)]
    /// Add all projects to config that are not there aleady
    auto: bool,
}

#[derive(Parser, Debug, Clone)]
struct ProjectRemove {
    #[arg(short = 'a', long, default_value_t = false)]
    /// Remove all projects from config that are not in Todoist
    auto: bool,

    #[arg(short = 'r', long, default_value_t = false)]
    /// Keep repeating prompt to remove projects. Use Ctrl/CMD + c to exit.
    repeat: bool,

    #[arg(short = 'l', long, default_value_t = false)]
    /// Remove all projects from config
    all: bool,

    #[arg(short, long)]
    /// Project to remove
    project: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ProjectDelete {
    #[arg(short = 'r', long, default_value_t = false)]
    /// Keep repeating prompt to delete projects. Use Ctrl/CMD + c to exit.
    repeat: bool,

    #[arg(short, long)]
    /// Project to remove
    project: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ProjectRename {
    #[arg(short, long)]
    /// Project to remove
    project: Option<String>,
}
#[derive(Parser, Debug, Clone)]
struct ProjectEmpty {
    #[arg(short, long)]
    /// Project to remove
    project: Option<String>,
}

// -- SECTIONS --

#[derive(Subcommand, Debug, Clone)]
enum SectionCommands {
    #[clap(alias = "c")]
    /// (c) Create a new section for a project in Todoist
    Create(SectionCreate),
}

#[derive(Parser, Debug, Clone)]
struct SectionCreate {
    #[arg(short, long)]
    /// Section name
    name: Option<String>,

    #[arg(short, long)]
    /// Project to put the section in
    project: Option<String>,
}

// -- TASKS --

#[derive(Subcommand, Debug, Clone)]
enum TaskCommands {
    #[clap(alias = "q")]
    /// (q) Create a new task using NLP
    QuickAdd(TaskQuickAdd),

    #[clap(alias = "c")]
    /// (c) Create a new task (without NLP)
    Create(TaskCreate),

    #[clap(alias = "e")]
    /// (e) Edit an existing task's content
    Edit(TaskEdit),

    #[clap(alias = "n")]
    /// (n) Get the next task by priority
    Next(TaskNext),

    #[clap(alias = "o")]
    /// (o) Complete the last task fetched with the next command
    Complete(TaskComplete),

    #[clap(alias = "m")]
    /// (m) Add a comment to the last task fetched with the next command
    Comment(TaskComment),
}

#[derive(Parser, Debug, Clone)]
struct TaskQuickAdd {
    #[arg(short, long, num_args(1..))]
    /// Content for task. Add a reminder at the end by prefixing the natural language date with `!`.
    /// Example: Get milk on sunday !saturday 4pm
    content: Option<Vec<String>>,
}

#[derive(Parser, Debug, Clone)]
struct TaskCreate {
    #[arg(short, long)]
    /// The project into which the task will be added
    project: Option<String>,

    #[arg(short = 'u', long)]
    /// Date date in format YYYY-MM-DD, YYYY-MM-DD HH:MM, or natural language
    due: Option<String>,

    #[arg(short, long, default_value_t = String::new())]
    /// Description for task
    description: String,

    #[arg(short, long)]
    /// Content for task
    content: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Do not prompt for section
    no_section: bool,

    #[arg(short = 'r', long)]
    /// Priority from 1 (without priority) to 4 (highest)
    priority: Option<u8>,

    #[arg(short, long)]
    /// List of labels to choose from, to be applied to each entry. Use flag once per label
    label: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
struct TaskEdit {
    #[arg(short, long)]
    /// The project containing the task
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the task
    filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct TaskNext {
    #[arg(short, long)]
    /// The project containing the task
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the task
    filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct TaskComplete {}

#[derive(Parser, Debug, Clone)]
struct TaskComment {
    #[arg(short, long)]
    /// Content for comment
    content: Option<String>,
}

// -- LISTS --

#[derive(Subcommand, Debug, Clone)]
enum ListCommands {
    #[clap(alias = "v")]
    /// (v) View a list of tasks
    View(ListView),

    #[clap(alias = "c")]
    /// (c) Complete a list of tasks one by one in priority order
    Process(ListProcess),

    #[clap(alias = "z")]
    /// (z) Give every task a priority
    Prioritize(ListPrioritize),

    #[clap(alias = "t")]
    /// (t) Give every task at date, time, and length
    Timebox(ListTimebox),

    #[clap(alias = "l")]
    /// (l) Iterate through tasks and apply labels from defined choices. Use label flag once per label to choose from.
    Label(ListLabel),

    #[clap(alias = "s")]
    /// (s) Assign dates to all tasks individually
    Schedule(ListSchedule),

    #[clap(alias = "d")]
    /// (d) Assign deadlines to all non-recurring tasks without deadlines individually
    Deadline(ListDeadline),

    #[clap(alias = "i")]
    /// (i) Create tasks from a text file, one per line using natural language. Skips empty lines.
    Import(ListImport),
}

#[derive(Parser, Debug, Clone)]
struct ListView {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Datetime)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListProcess {
    #[arg(short, long)]
    /// Complete all tasks that are due today or undated in a project individually in priority order
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListTimebox {
    #[arg(short, long)]
    /// Timebox all tasks without durations
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks, does not filter out tasks with durations unless specified in filter. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListPrioritize {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListLabel {
    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// Labels to select from, if left blank this will be fetched from API
    label: Vec<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListSchedule {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Don't re-schedule recurring tasks that are overdue
    skip_recurring: bool,

    #[arg(short, long, default_value_t = false)]
    /// Only schedule overdue tasks
    overdue: bool,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListDeadline {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks. Can add multiple filters separated by commas.
    filter: Option<String>,

    #[arg(short = 't', long, default_value_t = SortOrder::Value)]
    /// Choose how results should be sorted
    sort: SortOrder,
}

#[derive(Parser, Debug, Clone)]
struct ListImport {
    #[arg(short, long)]
    /// The file or directory to fuzzy find in
    path: Option<String>,
}

// -- CONFIG --

#[derive(Subcommand, Debug, Clone)]
enum ConfigCommands {
    #[clap(alias = "v")]
    /// (v) Check to see if tod is on the latest version, returns exit code 1 if out of date. Does not need a configuration file.
    CheckVersion(ConfigCheckVersion),

    #[clap(alias = "r")]
    /// (r) Deletes the configuration file (if present). Errors if the file does not exist.
    Reset(ConfigReset),

    #[clap(alias = "tz")]
    /// (tz) Change the timezone in the configuration file
    SetTimezone(ConfigSetTimezone),
}

#[derive(Subcommand, Debug, Clone)]
enum AuthCommands {
    #[clap(alias = "l")]
    /// (l) Log into Todoist using OAuth
    Login(AuthLogin),
}

#[derive(Subcommand, Debug, Clone)]
enum ShellCommands {
    #[clap(alias = "b")]
    /// (b) Generate shell completions for various shells. Does not need a configuration file
    Completions(ShellCompletions),
}

#[derive(Subcommand, Debug, Clone)]
enum TestCommands {
    #[clap(alias = "a")]
    /// (a) Hit all API endpoints
    All(TestAll),
}

#[derive(Parser, Debug, Clone)]
struct ConfigCheckVersion {}

#[derive(Parser, Debug, Clone)]
struct TestAll {}

#[derive(Parser, Debug, Clone)]
struct ConfigReset {
    /// Skip confirmation and force deletion
    #[arg(long)]
    force: bool,
}

#[derive(Parser, Debug, Clone)]
struct ConfigSetTimezone {
    #[arg(short, long)]
    /// TimeZone to add, i.e. "Canada/Pacific"
    timezone: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct AuthLogin {}

#[derive(Parser, Debug, Clone)]
struct ShellCompletions {
    shell: Shell,
}

enum FlagOptions {
    Project,
    Filter,
}

impl Display for FlagOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagOptions::Project => write!(f, "Project"),
            FlagOptions::Filter => write!(f, "Filter"),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Channel for sending errors from async processes
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Error>();

    let (bell_success, bell_error, result) = select_command(cli, tx).await;
    while let Some(e) = rx.recv().await {
        eprintln!("Error from async process: {e}");
    }

    match result {
        Ok(text) => {
            if bell_success {
                terminal_bell()
            }
            println!("{text}");
            std::process::exit(0);
        }
        Err(e) => {
            if bell_error {
                terminal_bell()
            }
            eprintln!("\n\n{e}");
            std::process::exit(1);
        }
    }
}

fn terminal_bell() {
    print!("\x07");
    std::io::stdout().flush().unwrap();
}

async fn select_command(
    cli: Cli,
    tx: UnboundedSender<Error>,
) -> (bool, bool, Result<String, Error>) {
    match &cli.command {
        // Project
        Commands::Project(ProjectCommands::Create(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_create(config, args).await,
            )
        }
        Commands::Project(ProjectCommands::List(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_list(config, args).await,
            )
        }
        Commands::Project(ProjectCommands::Remove(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_remove(config, args).await,
            )
        }
        Commands::Project(ProjectCommands::Rename(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_rename(config, args).await,
            )
        }
        Commands::Project(ProjectCommands::Import(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_import(config, args).await,
            )
        }
        Commands::Project(ProjectCommands::Empty(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_empty(&config, args).await,
            )
        }
        Commands::Project(ProjectCommands::Delete(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                project_delete(config, args).await,
            )
        }

        Commands::Section(SectionCommands::Create(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                section_create(config, args).await,
            )
        }

        // Task
        Commands::Task(TaskCommands::QuickAdd(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_quick_add(config, args).await,
            )
        }
        Commands::Task(TaskCommands::Create(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_create(config, args).await,
            )
        }
        Commands::Task(TaskCommands::Edit(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_edit(config, args).await,
            )
        }
        Commands::Task(TaskCommands::Next(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_next(config, args).await,
            )
        }
        Commands::Task(TaskCommands::Complete(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_complete(config, args).await,
            )
        }
        Commands::Task(TaskCommands::Comment(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                task_comment(config, args).await,
            )
        }

        // List
        Commands::List(ListCommands::View(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_view(config, args).await,
            )
        }
        Commands::List(ListCommands::Process(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_process(config, args).await,
            )
        }
        Commands::List(ListCommands::Prioritize(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_prioritize(config, args).await,
            )
        }
        Commands::List(ListCommands::Label(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_label(config, args).await,
            )
        }
        Commands::List(ListCommands::Schedule(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_schedule(config, args).await,
            )
        }
        Commands::List(ListCommands::Deadline(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_deadline(config, args).await,
            )
        }
        Commands::List(ListCommands::Timebox(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_timebox(config, args).await,
            )
        }
        Commands::List(ListCommands::Import(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                list_import(config, args).await,
            )
        }

        // Config
        Commands::Config(ConfigCommands::CheckVersion(args)) => {
            (true, true, config_check_version(args).await)
        }

        // Command to delete the config file. Checks the default path, does not rely on the config struct.
        Commands::Config(ConfigCommands::Reset(args)) => (
            false,
            false,
            config_reset(cli.config.clone(), args.force).await,
        ),

        Commands::Config(ConfigCommands::SetTimezone(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                tz_reset(config, args).await,
            )
        }

        Commands::Auth(AuthCommands::Login(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                auth_login(config, args).await,
            )
        }

        // Shell
        Commands::Shell(ShellCommands::Completions(args)) => {
            (true, true, shell_completions(args).await)
        }

        // Test
        Commands::Test(TestCommands::All(args)) => {
            let config = match fetch_config(&cli, &tx).await {
                Ok(config) => config,
                Err(e) => return (true, true, Err(e)),
            };
            (
                config.bell_on_success,
                config.bell_on_failure,
                test_all(config, args).await,
            )
        }
    }
}

async fn auth_login(config: Config, _args: &AuthLogin) -> Result<String, Error> {
    let mut config = config;
    oauth::login(&mut config, None).await
}
async fn shell_completions(args: &ShellCompletions) -> Result<String, Error> {
    shell::generate_completions(args.shell);

    Ok(String::new())
}
async fn test_all(config: Config, _args: &TestAll) -> Result<String, Error> {
    todoist::test_all_endpoints(config).await
}

// --- TASK ---

async fn task_quick_add(config: Config, args: &TaskQuickAdd) -> Result<String, Error> {
    let TaskQuickAdd { content } = args;
    let maybe_string = content.as_ref().map(|c| c.join(" "));
    let content = fetch_string(maybe_string.as_deref(), &config, input::CONTENT)?;
    let (content, reminder) = if let Some(index) = content.find('!') {
        let (before, after) = content.split_at(index);
        // after starts with '!', so skip it
        (
            before.trim().to_string(),
            Some(after[1..].trim().to_string()),
        )
    } else {
        (content, None)
    };
    todoist::quick_create_task(&config, &content, reminder).await?;
    Ok(color::green_string("✓"))
}

/// User does not want to use sections
fn is_no_sections(args: &TaskCreate, config: &Config) -> bool {
    args.no_section || config.no_sections.unwrap_or_default()
}

async fn task_create(config: Config, args: &TaskCreate) -> Result<String, Error> {
    if no_flags_used(args) {
        let options = tasks::create_task_attributes();
        let selections = input::multi_select(input::ATTRIBUTES, options, config.mock_select)?;

        let content = fetch_string(None, &config, input::CONTENT)?;

        let description = if selections.contains(&TaskAttribute::Description) {
            fetch_string(None, &config, input::DESCRIPTION)?
        } else {
            String::new()
        };

        let priority = if selections.contains(&TaskAttribute::Priority) {
            fetch_priority(&None, &config)?
        } else {
            Priority::None
        };
        let due = if selections.contains(&TaskAttribute::Due) {
            let datetime_input = input::datetime(
                config.mock_select,
                config.mock_string.clone(),
                config.natural_language_only,
                false,
                false,
            )?;

            match datetime_input {
                DateTimeInput::Skip => unreachable!(),
                DateTimeInput::Complete => unreachable!(),
                DateTimeInput::None => None,
                DateTimeInput::Text(datetime) => Some(datetime),
            }
        } else {
            None
        };

        let labels = if selections.contains(&TaskAttribute::Labels) {
            let all_labels = labels::get_labels(&config, false).await?;
            input::multi_select(input::LABELS, all_labels, config.mock_select)?
        } else {
            Vec::new()
        }
        .into_iter()
        .map(|l| l.name.to_owned())
        .collect::<Vec<String>>();

        let project = match fetch_project(args.project.as_deref(), &config).await? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };

        let section = if is_no_sections(args, &config) {
            None
        } else {
            sections::select_section(&config, &project).await?
        };

        todoist::create_task(
            &config,
            &content,
            &project,
            section,
            priority,
            &description,
            due.as_deref(),
            &labels,
        )
        .await?;
    } else {
        let TaskCreate {
            project,
            due,
            description,
            content,
            priority,
            label: labels,
            no_section: _no_section,
        } = args;
        let project = match fetch_project(project.as_deref(), &config).await? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };

        let section = if is_no_sections(args, &config) {
            None
        } else {
            sections::select_section(&config, &project).await?
        };
        let content = fetch_string(content.as_deref(), &config, input::CONTENT)?;
        let priority = fetch_priority(priority, &config)?;

        todoist::create_task(
            &config,
            &content,
            &project,
            section,
            priority,
            description,
            due.as_deref(),
            labels,
        )
        .await?;
    }
    Ok(color::green_string("✓"))
}

fn no_flags_used(args: &TaskCreate) -> bool {
    let TaskCreate {
        project,
        due,
        description,
        content,
        no_section: _no_section,
        priority,
        label,
    } = args;

    project.is_none()
        && due.is_none()
        && description.is_empty()
        && content.is_none()
        && priority.is_none()
        && label.is_empty()
}

async fn task_edit(config: Config, args: &TaskEdit) -> Result<String, Error> {
    let TaskEdit { project, filter } = args;
    match fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await? {
        Flag::Project(project) => projects::edit_task(&config, &project).await,
        Flag::Filter(filter) => filters::edit_task(&config, filter).await,
    }
}
async fn task_next(config: Config, args: &TaskNext) -> Result<String, Error> {
    let TaskNext { project, filter } = args;
    match fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await? {
        Flag::Project(project) => projects::next_task(config, &project).await,
        Flag::Filter(filter) => filters::next_task(&config, &filter).await,
    }
}

async fn task_complete(config: Config, _args: &TaskComplete) -> Result<String, Error> {
    match config.next_task() {
        Some(task) => {
            todoist::complete_task(&config, &task, true).await?;

            Ok(color::green_string("Task completed successfully"))
        }
        None => Err(Error::new(
            "task_complete",
            "There is nothing to complete. A task must first be marked as 'next'.",
        )),
    }
}

async fn task_comment(config: Config, args: &TaskComment) -> Result<String, Error> {
    let TaskComment { content } = args;
    match config.next_task() {
        Some(task) => {
            let content = fetch_string(content.as_deref(), &config, input::CONTENT)?;
            todoist::create_comment(&config, &task, content, true).await?;
            Ok(color::green_string("Comment created successfully"))
        }
        None => Err(Error::new(
            "task_comment",
            "There is nothing to comment on. A task must first be marked as 'next'.",
        )),
    }
}

// --- PROJECT ---

async fn project_create(config: Config, args: &ProjectCreate) -> Result<String, Error> {
    let ProjectCreate {
        name,
        description,
        is_favorite,
    } = args;
    let name = fetch_string(name.as_deref(), &config, input::NAME)?;
    let description = description.clone().unwrap_or_default();
    let mut config = config;
    projects::create(&mut config, name, description, *is_favorite).await
}

async fn project_list(config: Config, _args: &ProjectList) -> Result<String, Error> {
    let mut config = config.clone();
    projects::list(&mut config).await
}

async fn project_remove(config: Config, args: &ProjectRemove) -> Result<String, Error> {
    let ProjectRemove {
        all,
        auto,
        project,
        repeat,
    } = args;
    let mut config = config.clone();
    match (all, auto) {
        (true, false) => projects::remove_all(&mut config).await,
        (false, true) => projects::remove_auto(&mut config).await,
        (false, false) => loop {
            let project = match fetch_project(project.as_deref(), &config).await? {
                Flag::Project(project) => project,
                _ => unreachable!(),
            };
            let value = projects::remove(&mut config, &project).await;

            if !repeat {
                return value;
            }
        },
        (_, _) => Err(Error::new("project_remove", "Incorrect flags provided")),
    }
}

async fn project_delete(config: Config, args: &ProjectDelete) -> Result<String, Error> {
    let ProjectDelete { project, repeat } = args;
    let mut config = config.clone();
    loop {
        let project = match fetch_project(project.as_deref(), &config).await? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };
        let tasks = todoist::all_tasks_by_project(&config, &project, None).await?;

        if !tasks.is_empty() {
            println!();
            let options = vec![input::CANCEL, input::DELETE];
            let num_tasks = tasks.len();
            let desc = format!("Project has {num_tasks} tasks, confirm deletion");
            let result = input::select(&desc, options, config.mock_select)?;

            if result == input::CANCEL {
                return Ok("Cancelled".into());
            }
        }
        let value = projects::delete(&mut config, &project).await;

        if !repeat {
            return value;
        }
    }
}

async fn project_rename(config: Config, args: &ProjectRename) -> Result<String, Error> {
    let ProjectRename { project } = args;
    let project = match fetch_project(project.as_deref(), &config).await? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    debug::maybe_print(
        &config,
        format!("Calling projects::rename with project:\n{project}"),
    );
    projects::rename(config, &project).await
}

async fn project_import(config: Config, args: &ProjectImport) -> Result<String, Error> {
    let ProjectImport { auto } = args;

    let mut config = config.clone();
    projects::import(&mut config, auto).await
}

async fn project_empty(config: &Config, args: &ProjectEmpty) -> Result<String, Error> {
    let ProjectEmpty { project } = args;
    let project = match fetch_project(project.as_deref(), config).await? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    let mut config = config.clone();
    projects::empty(&mut config, &project).await
}

async fn section_create(config: Config, args: &SectionCreate) -> Result<String, Error> {
    let SectionCreate { name, project } = args;
    let name = fetch_string(name.as_deref(), &config, input::NAME)?;

    let project = match fetch_project(project.as_deref(), &config).await? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    todoist::create_section(&config, name, &project, true).await?;
    Ok(color::green_string("Section created successfully"))
}

// --- LIST ---

async fn list_view(config: Config, args: &ListView) -> Result<String, Error> {
    let mut config = config;

    let ListView {
        project,
        filter,
        sort,
    } = args;

    let flag = fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await?;
    lists::view(&mut config, flag, sort).await
}

async fn list_label(config: Config, args: &ListLabel) -> Result<String, Error> {
    let ListLabel {
        filter,
        project,
        label: labels,
        sort,
    } = args;
    let labels = maybe_fetch_labels(&config, labels).await?;
    let flag = fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await?;
    lists::label(&config, flag, &labels, sort).await
}

async fn list_process(config: Config, args: &ListProcess) -> Result<String, Error> {
    let ListProcess {
        project,
        filter,
        sort,
    } = args;
    let flag = fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await?;
    lists::process(&config, flag, sort).await
}

async fn list_timebox(config: Config, args: &ListTimebox) -> Result<String, Error> {
    let ListTimebox {
        project,
        filter,
        sort,
    } = args;
    let flag = fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await?;
    lists::timebox(&config, flag, sort).await
}

async fn list_prioritize(config: Config, args: &ListPrioritize) -> Result<String, Error> {
    let ListPrioritize {
        project,
        filter,
        sort,
    } = args;
    let flag = fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await?;
    lists::prioritize(&config, flag, sort).await
}
async fn list_import(config: Config, args: &ListImport) -> Result<String, Error> {
    let ListImport { path } = args;
    let path = fetch_string(path.as_deref(), &config, input::PATH)?;
    let file_path = select_file(path, &config)?;
    lists::import(&config, &file_path).await
}

fn select_file(path_or_file: String, config: &Config) -> Result<String, Error> {
    let path = Path::new(&path_or_file);
    if Path::is_dir(path) {
        let mut options = WalkDir::new(path_or_file)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(is_md_file)
            .map(|e| e.path().to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        options.sort();
        options.dedup();
        let path = input::select("Select file to process", options, config.mock_select)?;

        Ok(path)
    } else if Path::is_file(path) {
        Ok(path_or_file)
    } else {
        Err(Error {
            source: "select_file".to_string(),
            message: format!("{path_or_file} is neither a file nor a directory"),
        })
    }
}

fn is_md_file(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .unwrap_or_default()
        .ends_with(".md")
}

async fn list_schedule(config: Config, args: &ListSchedule) -> Result<String, Error> {
    let ListSchedule {
        project,
        filter,
        skip_recurring,
        overdue,
        sort,
    } = args;
    match fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await? {
        Flag::Filter(filter) => filters::schedule(&config, &filter, sort).await,
        Flag::Project(project) => {
            let task_filter = if *overdue {
                projects::TaskFilter::Overdue
            } else {
                projects::TaskFilter::Unscheduled
            };

            projects::schedule(&config, &project, task_filter, *skip_recurring, sort).await
        }
    }
}

async fn list_deadline(config: Config, args: &ListDeadline) -> Result<String, Error> {
    let ListDeadline {
        project,
        filter,
        sort,
    } = args;
    match fetch_project_or_filter(project.as_deref(), filter.as_deref(), &config).await? {
        Flag::Filter(filter) => filters::deadline(&config, &filter, sort).await,
        Flag::Project(project) => projects::deadline(&config, &project, sort).await,
    }
}

// // --- CONFIG ---

async fn config_check_version(_args: &ConfigCheckVersion) -> Result<String, Error> {
    match cargo::compare_versions(None).await {
        Ok(Version::Latest) => Ok(format!("Tod is up to date with version: {VERSION}")),
        Ok(Version::Dated(version)) => Err(Error::new(
            "cargo",
            &format!("Tod is out of date with version: {VERSION}, latest is:{version}"),
        )),
        Err(e) => Err(e),
    }
}

async fn tz_reset(config: Config, _args: &ConfigSetTimezone) -> Result<String, Error> {
    match config.set_timezone().await {
        Ok(updated_config) => {
            let tz = updated_config.get_timezone()?;
            Ok(format!("Timezone set successfully to: {tz}"))
        }
        Err(e) => Err(Error::new(
            "tz_reset",
            &format!("Could not reset timezone in config. {e}"),
        )),
    }
}

// --- VALUE HELPERS ---

/// Get or create config
async fn fetch_config(cli: &Cli, tx: &UnboundedSender<Error>) -> Result<Config, Error> {
    let Cli {
        verbose,
        config: config_path,
        timeout,
        command: _,
    } = cli;

    let config_path = config_path.to_owned();
    let verbose = verbose.to_owned();
    let timeout = timeout.to_owned();

    let config = config::get_or_create(config_path, verbose, timeout, tx).await?;

    let async_config = config.clone();

    tokio::spawn(async move { async_config.check_for_latest_version().await });

    config.maybe_set_timezone().await
}

fn fetch_string(
    maybe_string: Option<&str>,
    config: &Config,
    prompt: &str,
) -> Result<String, Error> {
    match maybe_string {
        Some(string) => Ok(string.to_owned()),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

async fn fetch_project(project_name: Option<&str>, config: &Config) -> Result<Flag, Error> {
    let projects = config.projects().await?;
    if projects.is_empty() {
        return Err(Error::new("fetch_project", NO_PROJECTS_ERR));
    }

    if projects.len() == 1 {
        return Ok(Flag::Project(projects.first().unwrap().clone()));
    }

    match project_name {
        Some(project_name) => projects
            .iter()
            .find(|p| p.name == project_name)
            .map_or_else(
                || {
                    Err(Error::new(
                        "fetch_project",
                        "Could not find project in config",
                    ))
                },
                |p| Ok(Flag::Project(p.to_owned())),
            ),
        None => input::select(input::PROJECT, projects, config.mock_select).map(Flag::Project),
    }
}

fn fetch_filter(filter: Option<&str>, config: &Config) -> Result<Flag, Error> {
    match filter {
        Some(string) => Ok(Flag::Filter(string.to_owned())),
        None => {
            let string = input::string(input::FILTER, config.mock_string.clone())?;
            Ok(Flag::Filter(string))
        }
    }
}

async fn fetch_project_or_filter(
    project: Option<&str>,
    filter: Option<&str>,
    config: &Config,
) -> Result<Flag, Error> {
    match (project, filter) {
        (Some(_), None) => fetch_project(project, config).await,
        (None, Some(_)) => fetch_filter(filter, config),
        (Some(_), Some(_)) => Err(Error::new(
            "project_or_filter",
            "Must select project OR filter",
        )),
        (None, None) => {
            let options = vec![FlagOptions::Project, FlagOptions::Filter];
            match input::select(input::OPTION, options, config.mock_select)? {
                FlagOptions::Project => fetch_project(project, config).await,
                FlagOptions::Filter => fetch_filter(filter, config),
            }
        }
    }
}

fn fetch_priority(priority: &Option<u8>, config: &Config) -> Result<Priority, Error> {
    match priority::from_integer(priority) {
        Some(priority) => Ok(priority),
        None => {
            let options = vec![
                Priority::None,
                Priority::Low,
                Priority::Medium,
                Priority::High,
            ];
            input::select(input::PRIORITY, options, config.mock_select)
        }
    }
}

async fn maybe_fetch_labels(config: &Config, labels: &[String]) -> Result<Vec<String>, Error> {
    if labels.is_empty() {
        let labels = labels::get_labels(config, false)
            .await?
            .into_iter()
            .map(|l| l.name)
            .collect();
        Ok(labels)
    } else {
        Ok(labels.to_vec())
    }
}

pub fn long_version() -> String {
    format!("{NAME} ({VERSION}, {BUILD_PROFILE}, {BUILD_TARGET}, {BUILD_TIMESTAMP})")
}

#[test]
fn verify_cmd() {
    use clap::CommandFactory;
    // Mostly checks that it is not going to throw an exception because of conflicting short arguments
    Cli::try_parse().err();
    Cli::command().debug_assert();
}
