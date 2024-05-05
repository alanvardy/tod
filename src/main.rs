#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use std::fmt::Display;

use cargo::Version;
use clap::{Parser, Subcommand};
use config::Config;
use projects::Project;
use tasks::priority;
use tasks::priority::Priority;

mod cargo;
mod color;
mod config;
mod debug;
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
struct ProjectImport {}

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
    content: Vec<String>,
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
}

#[derive(Parser, Debug, Clone)]
struct ConfigCheckVersion {}

#[derive(Parser, Debug, Clone)]
struct ConfigReset {}

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
fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Project(ProjectCommands::List(args)) => project_list(cli.clone(), args),
        Commands::Project(ProjectCommands::Remove(args)) => project_remove(cli.clone(), args),
        Commands::Project(ProjectCommands::Rename(args)) => project_rename(cli.clone(), args),
        Commands::Project(ProjectCommands::Import(args)) => project_import(cli.clone(), args),
        Commands::Project(ProjectCommands::Empty(args)) => project_empty(cli.clone(), args),

        Commands::Task(TaskCommands::QuickAdd(args)) => task_quick_add(cli.clone(), args),
        Commands::Task(TaskCommands::Create(args)) => task_create(cli.clone(), args),
        Commands::Task(TaskCommands::Edit(args)) => task_edit(cli.clone(), args),
        Commands::Task(TaskCommands::Next(args)) => task_next(cli.clone(), args),
        Commands::Task(TaskCommands::Complete(args)) => task_complete(cli.clone(), args),

        Commands::List(ListCommands::View(args)) => list_view(cli.clone(), args),
        Commands::List(ListCommands::Process(args)) => list_process(cli.clone(), args),
        Commands::List(ListCommands::Prioritize(args)) => list_prioritize(cli.clone(), args),
        Commands::List(ListCommands::Label(args)) => list_label(cli.clone(), args),
        Commands::List(ListCommands::Schedule(args)) => list_schedule(cli.clone(), args),

        Commands::Config(ConfigCommands::CheckVersion(args)) => {
            config_check_version(cli.clone(), args)
        }
        Commands::Config(ConfigCommands::Reset(args)) => config_reset(cli.clone(), args),
    };

    match result {
        Ok(text) => {
            println!("{text}");
            std::process::exit(0);
        }
        Err(e) => {
            println!("{}", color::red_string(&e));
            std::process::exit(1);
        }
    }
}

// --- TASK ---

#[cfg(not(tarpaulin_include))]
fn task_quick_add(cli: Cli, args: &TaskQuickAdd) -> Result<String, String> {
    let TaskQuickAdd { content } = args;
    let config = fetch_config(cli)?;

    todoist::quick_add_task(&config, &content.join(" "))?;
    Ok(color::green_string("✓"))
}

#[cfg(not(tarpaulin_include))]
fn task_create(cli: Cli, args: &TaskCreate) -> Result<String, String> {
    let TaskCreate {
        project,
        due,
        description,
        content,
        no_section,
        priority,
        label: labels,
    } = args;
    let config = fetch_config(cli)?;
    let content = fetch_string(content, &config, "CONTENT")?;
    let priority = fetch_priority(priority, &config)?;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    let section = if *no_section || config.no_sections.unwrap_or_default() {
        None
    } else {
        let sections = todoist::sections_for_project(&config, &project)?;
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
    )?;

    Ok(color::green_string("✓"))
}

#[cfg(not(tarpaulin_include))]
fn task_edit(cli: Cli, args: &TaskEdit) -> Result<String, String> {
    let config = fetch_config(cli)?;
    let TaskEdit { project, filter } = args;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::rename_task(&config, &project),
        Flag::Filter(filter) => filters::rename_task(&config, filter),
    }
}
#[cfg(not(tarpaulin_include))]
fn task_next(cli: Cli, args: &TaskNext) -> Result<String, String> {
    let TaskNext { project, filter } = args;
    let config = fetch_config(cli)?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::next_task(config, &project),
        Flag::Filter(filter) => filters::next_task(config, &filter),
    }
}

#[cfg(not(tarpaulin_include))]
fn task_complete(cli: Cli, _args: &TaskComplete) -> Result<String, String> {
    let config = fetch_config(cli)?;
    match config.next_id {
        Some(_) => todoist::complete_task(&config),
        None => {
            Err("There is nothing to complete. A task must first be marked as 'next'.".to_string())
        }
    }
}

// --- LIST ---

#[cfg(not(tarpaulin_include))]
fn list_view(cli: Cli, args: &ListView) -> Result<String, String> {
    let config = fetch_config(cli)?;
    let ListView { project, filter } = args;

    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Project(project) => projects::all_tasks(&config, &project),
        Flag::Filter(filter) => filters::all_tasks(&config, &filter),
    }
}

// --- PROJECT ---

#[cfg(not(tarpaulin_include))]
fn project_list(cli: Cli, _args: &ProjectList) -> Result<String, String> {
    let mut config = fetch_config(cli)?;

    projects::list(&mut config)
}

#[cfg(not(tarpaulin_include))]
fn project_remove(cli: Cli, args: &ProjectRemove) -> Result<String, String> {
    let ProjectRemove {
        all,
        auto,
        project,
        repeat,
    } = args;
    let mut config = fetch_config(cli)?;
    match (all, auto) {
        (true, false) => projects::remove_all(&mut config),
        (false, true) => projects::remove_auto(&mut config),
        (false, false) => loop {
            let project = match fetch_project(project, &config)? {
                Flag::Project(project) => project,
                _ => unreachable!(),
            };
            let value = projects::remove(&mut config, &project);

            if !repeat {
                return value;
            }
        },
        (_, _) => Err(String::from("Incorrect flags provided")),
    }
}

#[cfg(not(tarpaulin_include))]
fn project_rename(cli: Cli, args: &ProjectRename) -> Result<String, String> {
    let config = fetch_config(cli)?;
    let ProjectRename { project } = args;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    debug::print(
        &config,
        format!("Calling projects::rename with project:\n{project}"),
    );
    projects::rename(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_import(cli: Cli, _args: &ProjectImport) -> Result<String, String> {
    let mut config = fetch_config(cli)?;

    projects::import(&mut config)
}

#[cfg(not(tarpaulin_include))]
fn project_empty(cli: Cli, args: &ProjectEmpty) -> Result<String, String> {
    let ProjectEmpty { project } = args;
    let mut config = fetch_config(cli)?;
    let project = match fetch_project(project, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::empty(&mut config, &project)
}

// --- LIST ---

#[cfg(not(tarpaulin_include))]
fn list_label(cli: Cli, args: &ListLabel) -> Result<String, String> {
    let ListLabel {
        filter,
        label: labels,
    } = args;
    let config = fetch_config(cli)?;
    match fetch_filter(filter, &config)? {
        Flag::Filter(filter) => filters::label(&config, &filter, labels),
        _ => unreachable!(),
    }
}

#[cfg(not(tarpaulin_include))]
fn list_process(cli: Cli, args: &ListProcess) -> Result<String, String> {
    let ListProcess { project, filter } = args;
    let config = fetch_config(cli)?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::process_tasks(config, &filter),
        Flag::Project(project) => projects::process_tasks(config, &project),
    }
}

#[cfg(not(tarpaulin_include))]
fn list_prioritize(cli: Cli, args: &ListPrioritize) -> Result<String, String> {
    let ListPrioritize { project, filter } = args;
    let config = fetch_config(cli)?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::prioritize_tasks(&config, &filter),
        Flag::Project(project) => projects::prioritize_tasks(&config, &project),
    }
}

#[cfg(not(tarpaulin_include))]
fn list_schedule(cli: Cli, args: &ListSchedule) -> Result<String, String> {
    let ListSchedule {
        project,
        filter,
        skip_recurring,
        overdue,
    } = args;
    let config = fetch_config(cli)?;
    match fetch_project_or_filter(project, filter, &config)? {
        Flag::Filter(filter) => filters::schedule(&config, &filter),
        Flag::Project(project) => {
            let task_filter = if *overdue {
                projects::TaskFilter::Overdue
            } else {
                projects::TaskFilter::Unscheduled
            };

            projects::schedule(&config, &project, task_filter, *skip_recurring)
        }
    }
}

// // --- CONFIG ---

#[cfg(not(tarpaulin_include))]
fn config_check_version(cli: Cli, _args: &ConfigCheckVersion) -> Result<String, String> {
    let config = fetch_config(cli)?;

    match cargo::compare_versions(config) {
        Ok(Version::Latest) => Ok(format!("Tod is up to date with version: {}", VERSION)),
        Ok(Version::Dated(version)) => Err(format!(
            "Tod is out of date with version: {}, latest is:{}",
            VERSION, version
        )),
        Err(e) => Err(e),
    }
}

#[cfg(not(tarpaulin_include))]
fn config_reset(cli: Cli, _args: &ConfigReset) -> Result<String, String> {
    use std::fs;

    let config = fetch_config(cli)?;
    let path = config.path;

    match fs::remove_file(path.clone()) {
        Ok(_) => Ok(format!("{path} deleted successfully")),
        Err(e) => Err(format!("Could not delete config at path: {path}, {e}")),
    }
}

// --- VALUE HELPERS ---

#[cfg(not(tarpaulin_include))]
fn fetch_config(cli: Cli) -> Result<Config, String> {
    let Cli {
        verbose,
        config: config_path,
        timeout,
        command: _,
    } = cli;

    config::get_or_create(config_path, verbose, timeout)?
        .check_for_timezone()?
        .check_for_latest_version()
}

#[cfg(not(tarpaulin_include))]
fn fetch_string(
    maybe_string: &Option<String>,
    config: &Config,
    prompt: &str,
) -> Result<String, String> {
    match maybe_string {
        Some(string) => Ok(string.to_owned()),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_project(project: &Option<String>, config: &Config) -> Result<Flag, String> {
    let projects = config.projects.clone().unwrap_or_default();
    if projects.is_empty() {
        return Err(NO_PROJECTS_ERR.to_string());
    }

    if projects.len() == 1 {
        return Ok(Flag::Project(projects.first().unwrap().clone()));
    }

    match project {
        Some(project_name) => projects
            .iter()
            .find(|p| p.name == project_name.as_str())
            .map_or_else(
                || Err("Could not find project in config".to_string()),
                |p| Ok(Flag::Project(p.to_owned())),
            ),
        None => input::select("Select project", projects, config.mock_select).map(Flag::Project),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_filter(filter: &Option<String>, config: &Config) -> Result<Flag, String> {
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
) -> Result<Flag, String> {
    match (project, filter) {
        (Some(_), None) => fetch_project(project, config),
        (None, Some(_)) => fetch_filter(filter, config),
        (Some(_), Some(_)) => Err("Must select project OR filter".to_string()),
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
fn fetch_priority(priority: &Option<u8>, config: &Config) -> Result<Priority, String> {
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
// --- TESTS ---

#[test]
fn verify_cmd() {
    use clap::CommandFactory;
    // Mostly checks that it is not going to throw an exception because of conflicting short arguments
    Cli::try_parse().err();
    Cli::command().debug_assert();
}
