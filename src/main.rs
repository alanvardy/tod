//! A tiny Todoist CLI program. Takes simple input and dumps it in your inbox or another project. Takes advantage of natural language processing to assign due dates, tags, etc. Designed for single tasking in a world filled with distraction.
//!
//! Get started with `cargo install tod`
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use std::fmt::Display;
use std::io::Write;

use cargo::Version;
use clap::{Parser, Subcommand};
use config::Config;
use error::Error;
use input::DateTimeInput;
use projects::Project;
use tasks::priority::Priority;
use tasks::{priority, TaskAttribute};
use tokio::sync::mpsc::UnboundedSender;

mod cargo;
mod color;
mod config;
mod debug;
mod error;
mod filters;
mod input;
mod labels;
mod projects;
mod sections;
mod tasks;
mod test;
mod time;
mod todoist;
mod user;

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

    #[clap(alias = "t")]
    /// (t) Give every task at date, time, and length
    Timebox(ListTimebox),

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
struct ListTimebox {
    #[arg(short, long)]
    /// Timebox all tasks without durations
    project: Option<String>,

    #[arg(short, long)]
    /// The filter containing the tasks, does not filter out tasks with durations unless specified in filter
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
    /// The project containing the tasks
    project: Option<String>,

    #[arg(short, long)]
    /// Labels to select from, if left blank this will be fetched from API
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
struct ConfigSetTimezone {
    #[arg(short, long)]
    /// TimeZone to add, i.e. "Canada/Pacific"
    timezone: Option<String>,
}

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Channel for sending errors from async processes
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Error>();

    match select_command(cli, tx).await {
        (bell_success, _bell_error, Ok(text)) => {
            while let Some(e) = rx.recv().await {
                eprintln!("Error from async process: {e}");
            }

            if bell_success {
                terminal_bell()
            }
            println!("{text}");
            std::process::exit(0);
        }
        (_bell_success, bell_error, Err(e)) => {
            while let Some(e) = rx.recv().await {
                eprintln!("Error from async process: {e}");
            }

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
    match fetch_config(&cli, tx).await {
        Err(e) => (true, true, Err(e)),
        Ok(config) => {
            let bell_on_success = config.bell_on_success;
            let bell_on_failure = config.bell_on_failure;
            let result: Result<String, Error> = match &cli.command {
                // Project
                Commands::Project(ProjectCommands::List(args)) => project_list(config, args).await,
                Commands::Project(ProjectCommands::Remove(args)) => {
                    project_remove(config, args).await
                }
                Commands::Project(ProjectCommands::Rename(args)) => {
                    project_rename(config, args).await
                }
                Commands::Project(ProjectCommands::Import(args)) => {
                    project_import(config, args).await
                }
                Commands::Project(ProjectCommands::Empty(args)) => {
                    project_empty(config, args).await
                }
                Commands::Project(ProjectCommands::Delete(args)) => {
                    project_delete(config, args).await
                }

                // Task
                Commands::Task(TaskCommands::QuickAdd(args)) => task_quick_add(config, args).await,
                Commands::Task(TaskCommands::Create(args)) => task_create(config, args).await,
                Commands::Task(TaskCommands::Edit(args)) => task_edit(config, args).await,
                Commands::Task(TaskCommands::Next(args)) => task_next(config, args).await,
                Commands::Task(TaskCommands::Complete(args)) => task_complete(config, args).await,

                // List
                Commands::List(ListCommands::View(args)) => list_view(config, args).await,
                Commands::List(ListCommands::Process(args)) => list_process(config, args).await,
                Commands::List(ListCommands::Prioritize(args)) => {
                    list_prioritize(config, args).await
                }
                Commands::List(ListCommands::Label(args)) => list_label(config, args).await,
                Commands::List(ListCommands::Schedule(args)) => list_schedule(config, args).await,
                Commands::List(ListCommands::Timebox(args)) => list_timebox(config, args).await,

                // Config
                Commands::Config(ConfigCommands::CheckVersion(args)) => {
                    config_check_version(config, args).await
                }
                Commands::Config(ConfigCommands::Reset(args)) => config_reset(config, args).await,
                Commands::Config(ConfigCommands::SetTimezone(args)) => tz_reset(config, args).await,
            };

            (bell_on_success, bell_on_failure, result)
        }
    }
}

// --- TASK ---

async fn task_quick_add(config: Config, args: &TaskQuickAdd) -> Result<String, Error> {
    let TaskQuickAdd { content } = args;

    let content = fetch_string(
        &content.as_ref().map(|c| c.join(" ")),
        &config,
        input::CONTENT,
    )?;
    todoist::quick_add_task(&config, &content).await?;
    Ok(color::green_string("✓"))
}

/// User does not want to use sections
fn is_no_sections(args: &TaskCreate, config: &Config) -> bool {
    args.no_section || config.no_sections.unwrap_or_default()
}

async fn task_create(config: Config, args: &TaskCreate) -> Result<String, Error> {
    if no_flags_used(args) {
        let options = tasks::task_attributes()
            .into_iter()
            .filter(|t| t != &TaskAttribute::Content)
            .collect();

        let selections = input::multi_select(input::ATTRIBUTES, options, config.mock_select)?;

        if selections.is_empty() {
            return Err(Error {
                message: "Nothing selected".to_string(),
                source: "edit_task".to_string(),
            });
        }

        let content = fetch_string(&None, &config, input::CONTENT)?;

        let description = if selections.contains(&TaskAttribute::Description) {
            fetch_string(&None, &config, input::DESCRIPTION)?
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

        let project = match fetch_project(&args.project, &config)? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };

        let section = if is_no_sections(args, &config) {
            None
        } else {
            sections::select_section(&config, &project).await?
        };

        todoist::add_task(
            &config,
            &content,
            &project,
            section,
            priority,
            &description,
            &due,
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
        let project = match fetch_project(project, &config)? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };

        let section = if is_no_sections(args, &config) {
            None
        } else {
            sections::select_section(&config, &project).await?
        };
        let content = fetch_string(content, &config, input::CONTENT)?;
        let priority = fetch_priority(priority, &config)?;

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
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::edit_task(&config, &project).await,
        Flag::Filter(filter) => filters::edit_task(&config, filter).await,
    }
}
async fn task_next(config: Config, args: &TaskNext) -> Result<String, Error> {
    let TaskNext { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::next_task(config, &project).await,
        Flag::Filter(filter) => filters::next_task(config, &filter).await,
    }
}

async fn task_complete(config: Config, _args: &TaskComplete) -> Result<String, Error> {
    match config.next_id.as_ref() {
        Some(id) => todoist::complete_task(&config, id, true).await,
        None => Err(error::new(
            "task_complete",
            "There is nothing to complete. A task must first be marked as 'next'.",
        )),
    }
}

// --- LIST ---

async fn list_view(config: Config, args: &ListView) -> Result<String, Error> {
    let ListView { project, filter } = args;

    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::all_tasks(&config, &project).await,
        Flag::Filter(filter) => filters::all_tasks(&config, &filter).await,
    }
}

// --- PROJECT ---

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

async fn project_delete(config: Config, args: &ProjectDelete) -> Result<String, Error> {
    let ProjectDelete { project, repeat } = args;
    let mut config = config.clone();
    loop {
        let project = match fetch_project(project, &config)? {
            Flag::Project(project) => project,
            _ => unreachable!(),
        };
        let tasks = todoist::tasks_for_project(&config, &project).await?;

        if !tasks.is_empty() {
            println!();
            let options = vec![input::CANCEL, input::DELETE];
            let num_tasks = tasks.len();
            let desc = format!("Project has {num_tasks} tasks, confirm deletion");
            let result = input::select(&desc, options, config.mock_select)?;

            if result == input::CANCEL {
                return Ok(String::from("Cancelled"));
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

async fn project_import(config: Config, args: &ProjectImport) -> Result<String, Error> {
    let ProjectImport { auto } = args;

    let mut config = config.clone();
    projects::import(&mut config, auto).await
}

async fn project_empty(config: Config, args: &ProjectEmpty) -> Result<String, Error> {
    let ProjectEmpty { project } = args;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    let mut config = config.clone();
    projects::empty(&mut config, &project).await
}

// --- LIST ---

async fn list_label(config: Config, args: &ListLabel) -> Result<String, Error> {
    let ListLabel {
        filter,
        project,
        label: labels,
    } = args;
    let labels = maybe_fetch_labels(&config, labels).await?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::label(&config, &filter, &labels).await,
        Flag::Project(project) => projects::label(&config, &project, &labels).await,
    }
}

async fn list_process(config: Config, args: &ListProcess) -> Result<String, Error> {
    let ListProcess { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::process_tasks(&config, &filter).await,
        Flag::Project(project) => projects::process_tasks(&config, &project).await,
    }
}

async fn list_timebox(config: Config, args: &ListTimebox) -> Result<String, Error> {
    let ListTimebox { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::timebox_tasks(&config, &filter).await,
        Flag::Project(project) => projects::timebox_tasks(&config, &project).await,
    }
}

async fn list_prioritize(config: Config, args: &ListPrioritize) -> Result<String, Error> {
    let ListPrioritize { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::prioritize_tasks(&config, &filter).await,
        Flag::Project(project) => projects::prioritize_tasks(&config, &project).await,
    }
}

async fn list_schedule(config: Config, args: &ListSchedule) -> Result<String, Error> {
    let ListSchedule {
        project,
        filter,
        skip_recurring,
        overdue,
    } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::schedule(&config, &filter).await,
        Flag::Project(project) => {
            let task_filter = if *overdue {
                projects::TaskFilter::Overdue
            } else {
                projects::TaskFilter::Unscheduled
            };

            projects::schedule(&config, &project, task_filter, *skip_recurring).await
        }
    }
}

// // --- CONFIG ---

async fn config_check_version(config: Config, _args: &ConfigCheckVersion) -> Result<String, Error> {
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

async fn config_reset(config: Config, _args: &ConfigReset) -> Result<String, Error> {
    use tokio::fs;

    let path = config.path;

    match fs::remove_file(path.clone()).await {
        Ok(_) => Ok(format!("{path} deleted successfully")),
        Err(e) => Err(error::new(
            "config_reset",
            &format!("Could not delete config at path: {path}, {e}"),
        )),
    }
}

async fn tz_reset(config: Config, _args: &ConfigSetTimezone) -> Result<String, Error> {
    match config.set_timezone().await {
        Ok(_) => Ok("Timezone set successfully.".to_string()),
        Err(e) => Err(error::new(
            "tz_reset",
            &format!("Could not reset timezone in config. {e}"),
        )),
    }
}

// --- VALUE HELPERS ---

async fn fetch_config(cli: &Cli, tx: UnboundedSender<Error>) -> Result<Config, Error> {
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

    config.check_for_timezone().await
}

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
        None => input::select(input::PROJECT, projects, config.mock_select).map(Flag::Project),
    }
}

fn fetch_filter(filter: &Option<String>, config: &Config) -> Result<Flag, Error> {
    match filter {
        Some(string) => Ok(Flag::Filter(string.to_owned())),
        None => {
            let string = input::string(input::FILTER, config.mock_string.clone())?;
            Ok(Flag::Filter(string))
        }
    }
}

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
            match input::select(input::OPTION, options, config.mock_select)? {
                FlagOptions::Project => fetch_project(project, config),
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
// --- TESTS ---

#[test]
fn verify_cmd() {
    use clap::CommandFactory;
    // Mostly checks that it is not going to throw an exception because of conflicting short arguments
    Cli::try_parse().err();
    Cli::command().debug_assert();
}
