//! A tiny Todoist CLI program. Takes simple input and dumps it in your inbox or another project. Takes advantage of natural language processing to assign due dates, tags, etc. Designed for single tasking in a world filled with distraction.
//!
//! Get started with `cargo install tod`
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use std::fmt::Display;

use cargo::Version;
use clap::{Parser, Subcommand};
use config::Config;
use error::Error;
use projects::Project;
use tasks::priority;
use tasks::priority::Priority;
use tokio::sync::mpsc::UnboundedSender;

mod cargo;
mod color;
mod config;
mod debug;
mod error;
mod filters;
mod input;
mod projects;
mod sections;
mod tasks;
mod test;
mod time;
mod todoist;

const NAME: &str = "Tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Todoist client";

const NO_PROJECTS_ERR: &str = "No projects in config. Add projects with `tod project import`";

#[derive(Parser, Clone)]
#[command(name = NAME)]
#[command(version = VERSION)]
#[command(about = ABOUT, long_about = None)]
#[command(author = AUTHOR, version)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    /// Display additional debug info while processing
    verbose: bool,

    #[arg(short, long)]
    /// Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/tod.cfg
    config: Option<String>,

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
}

// -- PROJECTS --

#[derive(Subcommand, Debug, Clone)]
enum ProjectCommands {
    #[clap(alias = "l")]
    /// (l) List all of the projects in config
    List(ProjectList),

    #[clap(alias = "r")]
    /// (r) Remove a project from config (not Todoist)
    Remove(ProjectRemove),

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
}

#[derive(Parser, Debug, Clone)]
struct TaskQuickAdd {
    #[arg(short, long, num_args(1..))]
    /// Content for task
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

    #[clap(alias = "l")]
    /// (l) Iterate through tasks and apply labels from defined choices
    Label(ListLabel),

    #[clap(alias = "s")]
    /// (s) Assign dates to all tasks individually
    Schedule(ListSchedule),
}

#[derive(Parser, Debug, Clone)]
struct ListView {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks
    filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ListProcess {
    #[arg(short, long)]
    /// Complete all tasks that are due today or undated in a project individually in priority order
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks
    filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ListPrioritize {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks
    filter: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ListLabel {
    #[arg(short, long)]
    /// The filter containing the tasks
    filter: Option<String>,

    #[arg(short, long)]
    /// Labels to select from
    label: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
struct ListSchedule {
    #[arg(short, long)]
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks
    filter: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Don't re-schedule recurring tasks that are overdue
    skip_recurring: bool,

    #[arg(short, long, default_value_t = false)]
    /// Only schedule overdue tasks
    overdue: bool,
}

// -- CONFIG --

#[derive(Subcommand, Debug, Clone)]
enum ConfigCommands {
    #[clap(alias = "v")]
    /// (v) Check to see if tod is on the latest version, returns exit code 1 if out of date
    CheckVersion(ConfigCheckVersion),

    #[clap(alias = "r")]
    /// (r) Delete the configuration file
    Reset(ConfigReset),

    #[clap(alias = "tz")]
    /// (tz) Change the timezone in the configuration file
    SetTimezone(ConfigSetTimezone),
}

#[derive(Parser, Debug, Clone)]
struct ConfigCheckVersion {}

#[derive(Parser, Debug, Clone)]
struct ConfigReset {}

#[derive(Parser, Debug, Clone)]
struct ConfigSetTimezone {}

enum Flag {
    Project(Project),
    Filter(String),
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

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Channel for sending errors from async processes
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Error>();

    let result: Result<String, Error> = match &cli.command {
        Commands::Project(ProjectCommands::List(args)) => project_list(cli.clone(), args, tx).await,
        Commands::Project(ProjectCommands::Remove(args)) => {
            project_remove(cli.clone(), args, tx).await
        }
        Commands::Project(ProjectCommands::Rename(args)) => {
            project_rename(cli.clone(), args, tx).await
        }
        Commands::Project(ProjectCommands::Import(args)) => {
            project_import(cli.clone(), args, tx).await
        }
        Commands::Project(ProjectCommands::Empty(args)) => {
            project_empty(cli.clone(), args, tx).await
        }

        Commands::Task(TaskCommands::QuickAdd(args)) => task_quick_add(cli.clone(), args, tx).await,
        Commands::Task(TaskCommands::Create(args)) => task_create(cli.clone(), args, tx).await,
        Commands::Task(TaskCommands::Edit(args)) => task_edit(cli.clone(), args, tx).await,
        Commands::Task(TaskCommands::Next(args)) => task_next(cli.clone(), args, tx).await,
        Commands::Task(TaskCommands::Complete(args)) => task_complete(cli.clone(), args, tx).await,

        Commands::List(ListCommands::View(args)) => list_view(cli.clone(), args, tx).await,
        Commands::List(ListCommands::Process(args)) => list_process(cli.clone(), args, tx).await,
        Commands::List(ListCommands::Prioritize(args)) => {
            list_prioritize(cli.clone(), args, tx).await
        }
        Commands::List(ListCommands::Label(args)) => list_label(cli.clone(), args, tx).await,
        Commands::List(ListCommands::Schedule(args)) => list_schedule(cli.clone(), args, tx).await,

        Commands::Config(ConfigCommands::CheckVersion(args)) => {
            config_check_version(cli.clone(), args, tx).await
        }
        Commands::Config(ConfigCommands::Reset(args)) => config_reset(cli.clone(), args, tx).await,
        Commands::Config(ConfigCommands::SetTimezone(args)) => tz_reset(cli.clone(), args, tx).await,
    };

    while let Some(e) = rx.recv().await {
        eprintln!("Error from async process: {e}");
    }

    match result {
        Ok(text) => {
            println!("{text}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("\n\n{e}");
            std::process::exit(1);
        }
    }
}

// --- TASK ---

#[cfg(not(tarpaulin_include))]
async fn task_quick_add(
    cli: Cli,
    args: &TaskQuickAdd,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let TaskQuickAdd { content } = args;
    let config = fetch_config(cli, tx).await?;

    let content = fetch_string(&content.as_ref().map(|c| c.join(" ")), &config, "CONTENT")?;
    todoist::quick_add_task(&config, &content).await?;
    Ok(color::green_string("✓"))
}

#[cfg(not(tarpaulin_include))]
async fn task_create(
    cli: Cli,
    args: &TaskCreate,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let TaskCreate {
        project,
        due,
        description,
        content,
        no_section,
        priority,
        label: labels,
    } = args;
    let config = fetch_config(cli, tx).await?;
    let content = fetch_string(content, &config, "CONTENT")?;
    let priority = fetch_priority(priority, &config)?;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    let section = if *no_section || config.no_sections.unwrap_or_default() {
        None
    } else {
        let sections = todoist::sections_for_project(&config, &project).await?;
        let mut section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
        if section_names.is_empty() {
            None
        } else {
            section_names.insert(0, "No section".to_string());
            let section_name = input::select("Select section", section_names, config.mock_select)?;
            sections
                .iter()
                .find(|x| x.name == section_name.as_str())
                .map(|s| s.to_owned())
        }
    };

    todoist::add_task(
        &config,
        &content,
        &project,
        section,
        priority,
        description,
        due,
        labels,
    )
    .await?;

    Ok(color::green_string("✓"))
}

#[cfg(not(tarpaulin_include))]
async fn task_edit(cli: Cli, args: &TaskEdit, tx: UnboundedSender<Error>) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;
    let TaskEdit { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::rename_task(&config, &project).await,
        Flag::Filter(filter) => filters::rename_task(&config, filter).await,
    }
}
#[cfg(not(tarpaulin_include))]
async fn task_next(cli: Cli, args: &TaskNext, tx: UnboundedSender<Error>) -> Result<String, Error> {
    let TaskNext { project, filter } = args;
    let config = fetch_config(cli, tx).await?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::next_task(config, &project).await,
        Flag::Filter(filter) => filters::next_task(config, &filter).await,
    }
}

#[cfg(not(tarpaulin_include))]
async fn task_complete(
    cli: Cli,
    _args: &TaskComplete,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;
    match config.next_id.as_ref() {
        Some(id) => todoist::complete_task(&config, id, true).await,
        None => Err(error::new(
            "task_complete",
            "There is nothing to complete. A task must first be marked as 'next'.",
        )),
    }
}

// --- LIST ---

#[cfg(not(tarpaulin_include))]
async fn list_view(cli: Cli, args: &ListView, tx: UnboundedSender<Error>) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;
    let ListView { project, filter } = args;

    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::all_tasks(&config, &project).await,
        Flag::Filter(filter) => filters::all_tasks(&config, &filter).await,
    }
}

// --- PROJECT ---

#[cfg(not(tarpaulin_include))]
async fn project_list(
    cli: Cli,
    _args: &ProjectList,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let mut config = fetch_config(cli, tx).await?;

    projects::list(&mut config).await
}

#[cfg(not(tarpaulin_include))]
async fn project_remove(
    cli: Cli,
    args: &ProjectRemove,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ProjectRemove {
        all,
        auto,
        project,
        repeat,
    } = args;
    let mut config = fetch_config(cli, tx).await?;
    match (all, auto) {
        (true, false) => projects::remove_all(&mut config).await,
        (false, true) => projects::remove_auto(&mut config).await,
        (false, false) => loop {
            let project = match fetch_project(project, &config)? {
                Flag::Project(project) => project,
                _ => unreachable!(),
            };
            let value = projects::remove(&mut config, &project).await;

            if !repeat {
                return value;
            }
        },
        (_, _) => Err(error::new("project_remove", "Incorrect flags provided")),
    }
}

#[cfg(not(tarpaulin_include))]
async fn project_rename(
    cli: Cli,
    args: &ProjectRename,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;
    let ProjectRename { project } = args;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    debug::print(
        &config,
        format!("Calling projects::rename with project:\n{project}"),
    );
    projects::rename(config, &project).await
}

#[cfg(not(tarpaulin_include))]
async fn project_import(
    cli: Cli,
    args: &ProjectImport,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let mut config = fetch_config(cli, tx).await?;
    let ProjectImport { auto } = args;

    projects::import(&mut config, auto).await
}

#[cfg(not(tarpaulin_include))]
async fn project_empty(
    cli: Cli,
    args: &ProjectEmpty,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ProjectEmpty { project } = args;
    let mut config = fetch_config(cli, tx.clone()).await?;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::empty(&mut config, &project, tx).await
}

// --- LIST ---

#[cfg(not(tarpaulin_include))]
async fn list_label(
    cli: Cli,
    args: &ListLabel,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ListLabel {
        filter,
        label: labels,
    } = args;
    let config = fetch_config(cli, tx.clone()).await?;
    let labels = maybe_fetch_labels(&config, labels)?;
    match fetch_filter(filter, &config)? {
        Flag::Filter(filter) => filters::label(&config, &filter, &labels, tx).await,
        _ => unreachable!(),
    }
}

#[cfg(not(tarpaulin_include))]
async fn list_process(
    cli: Cli,
    args: &ListProcess,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ListProcess { project, filter } = args;
    let config = fetch_config(cli, tx.clone()).await?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::process_tasks(&config, &filter, tx).await,
        Flag::Project(project) => projects::process_tasks(&config, &project, tx).await,
    }
}

#[cfg(not(tarpaulin_include))]
async fn list_prioritize(
    cli: Cli,
    args: &ListPrioritize,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ListPrioritize { project, filter } = args;
    let config = fetch_config(cli, tx.clone()).await?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::prioritize_tasks(&config, &filter, tx).await,
        Flag::Project(project) => projects::prioritize_tasks(&config, &project, tx).await,
    }
}

#[cfg(not(tarpaulin_include))]
async fn list_schedule(
    cli: Cli,
    args: &ListSchedule,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let ListSchedule {
        project,
        filter,
        skip_recurring,
        overdue,
    } = args;
    let config = fetch_config(cli, tx.clone()).await?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::schedule(&config, &filter, tx).await,
        Flag::Project(project) => {
            let task_filter = if *overdue {
                projects::TaskFilter::Overdue
            } else {
                projects::TaskFilter::Unscheduled
            };

            projects::schedule(&config, &project, task_filter, *skip_recurring, tx).await
        }
    }
}

// // --- CONFIG ---

#[cfg(not(tarpaulin_include))]
async fn config_check_version(
    cli: Cli,
    _args: &ConfigCheckVersion,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;

    match cargo::compare_versions(config).await {
        Ok(Version::Latest) => Ok(format!("Tod is up to date with version: {}", VERSION)),
        Ok(Version::Dated(version)) => Err(error::new(
            "cargo",
            &format!(
                "Tod is out of date with version: {}, latest is:{}",
                VERSION, version
            ),
        )),
        Err(e) => Err(e),
    }
}

#[cfg(not(tarpaulin_include))]
async fn config_reset(
    cli: Cli,
    _args: &ConfigReset,
    tx: UnboundedSender<Error>,
) -> Result<String, Error> {
    use tokio::fs;

    let config = fetch_config(cli, tx).await?;
    let path = config.path;

    match fs::remove_file(path.clone()).await {
        Ok(_) => Ok(format!("{path} deleted successfully")),
        Err(e) => Err(error::new(
            "config_reset",
            &format!("Could not delete config at path: {path}, {e}"),
        )),
    }
}

#[cfg(not(tarpaulin_include))]
async fn tz_reset(cli: Cli, _args: &ConfigSetTimezone, tx: UnboundedSender<Error>) -> Result<String, Error> {
    let config = fetch_config(cli, tx).await?;

    match config.set_timezone().await {
        Ok(_) => Ok("Timezone set successfully.".to_string()),
        Err(e) => Err(error::new(
            "tz_reset",
            &format!("Could not reset timezone in config. {e}"),
        )),
    }
}

// --- VALUE HELPERS ---

#[cfg(not(tarpaulin_include))]
async fn fetch_config(cli: Cli, tx: UnboundedSender<Error>) -> Result<Config, Error> {
    let Cli {
        verbose,
        config: config_path,
        timeout,
        command: _,
    } = cli;

    let config = config::get_or_create(config_path, verbose, timeout).await?;

    let async_config = config.clone();

    tokio::spawn(async move { async_config.check_for_latest_version(tx).await });

    config.check_for_timezone().await
}

#[cfg(not(tarpaulin_include))]
fn fetch_string(
    maybe_string: &Option<String>,
    config: &Config,
    prompt: &str,
) -> Result<String, Error> {
    match maybe_string {
        Some(string) => Ok(string.to_owned()),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_project(project: &Option<String>, config: &Config) -> Result<Flag, Error> {
    let projects = config.projects.clone().unwrap_or_default();
    if projects.is_empty() {
        return Err(error::new("fetch_project", NO_PROJECTS_ERR));
    }

    if projects.len() == 1 {
        return Ok(Flag::Project(projects.first().unwrap().clone()));
    }

    match project {
        Some(project_name) => projects
            .iter()
            .find(|p| p.name == project_name.as_str())
            .map_or_else(
                || {
                    Err(error::new(
                        "fetch_project",
                        "Could not find project in config",
                    ))
                },
                |p| Ok(Flag::Project(p.to_owned())),
            ),
        None => input::select("Select project", projects, config.mock_select).map(Flag::Project),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_filter(filter: &Option<String>, config: &Config) -> Result<Flag, Error> {
    match filter {
        Some(string) => Ok(Flag::Filter(string.to_owned())),
        None => {
            let string = input::string("Enter a filter:", config.mock_string.clone())?;
            Ok(Flag::Filter(string))
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_project_or_filter(
    project: &Option<String>,
    filter: &Option<String>,
    config: &Config,
) -> Result<Flag, Error> {
    match (project, filter) {
        (Some(_), None) => fetch_project(project, config),
        (None, Some(_)) => fetch_filter(filter, config),
        (Some(_), Some(_)) => Err(error::new(
            "project_or_filter",
            "Must select project OR filter",
        )),
        (None, None) => {
            let options = vec![FlagOptions::Project, FlagOptions::Filter];
            match input::select("Select Project or Filter:", options, config.mock_select)? {
                FlagOptions::Project => fetch_project(project, config),
                FlagOptions::Filter => fetch_filter(filter, config),
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
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
            input::select(
                "Choose a priority that should be assigned to task:",
                options,
                config.mock_select,
            )
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn maybe_fetch_labels(config: &Config, labels: &[String]) -> Result<Vec<String>, Error> {
    if labels.is_empty() {
        let labels = input::string(
            "Enter labels to select from, separated by a space",
            config.mock_string.clone(),
        )?
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        Ok(labels)
    } else {
        Ok(labels.to_vec())
    }
}
// --- TESTS ---

#[test]
fn verify_cmd() {
    use clap::CommandFactory;
    // Mostly checks that it is not going to throw an exception because of conflicting short arguments
    Cli::try_parse().err();
    Cli::command().debug_assert();
}
