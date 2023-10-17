#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use std::fmt::Display;

use clap::{Arg, ArgAction, ArgMatches, Command};
use config::Config;
use projects::Project;
use tasks::priority::Priority;

mod cargo;
mod color;
mod config;
mod filters;
mod input;
mod projects;
mod sections;
mod tasks;
mod test;
mod time;
mod todoist;

const APP: &str = "Tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Todoist client";

const NO_PROJECTS_ERR: &str = "No projects in config. Add projects with `tod project import`";

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
    let matches = cmd().get_matches();

    let result = match matches.subcommand() {
        None => {
            let new_task = matches
                .get_many("quickadd")
                .map(|values| values.cloned().collect::<Vec<String>>().join(" "));
            match new_task {
                None => Err(cmd().render_long_help().to_string()),
                Some(text) => quickadd(&matches, text),
            }
        }
        Some(("task", task_matches)) => match task_matches.subcommand() {
            Some(("create", m)) => task_create(m),
            Some(("edit", m)) => task_edit(m),
            Some(("list", m)) => task_list(m),
            Some(("next", m)) => task_next(m),
            Some(("complete", m)) => task_complete(m),
            _ => unreachable!(),
        },
        Some(("project", project_matches)) => match project_matches.subcommand() {
            Some(("list", m)) => project_list(m),
            Some(("remove", m)) => project_remove(m),
            Some(("rename", m)) => project_rename(m),
            Some(("process", m)) => project_process(m),
            Some(("empty", m)) => project_empty(m),
            Some(("schedule", m)) => project_schedule(m),
            Some(("prioritize", m)) => project_prioritize(m),
            Some(("import", m)) => project_import(m),
            _ => unreachable!(),
        },
        Some(("filter", filter_matches)) => match filter_matches.subcommand() {
            Some(("label", m)) => filter_label(m),
            Some(("process", m)) => filter_process(m),
            Some(("prioritize", m)) => filter_prioritize(m),
            Some(("schedule", m)) => filter_schedule(m),
            _ => unreachable!(),
        },
        Some(("version", version_matches)) => match version_matches.subcommand() {
            Some(("check", m)) => version_check(m),
            _ => unreachable!(),
        },
        _ => unreachable!(),
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

fn cmd() -> Command {
    Command::new(APP)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg_required_else_help(true)
        .propagate_version(true)
        .arg(config_arg())
        .arg(
            Arg::new("quickadd")
                .short('q')
                .long("quickadd")
                .required(false)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_parser(clap::value_parser!(String))
                .help(
                    "Create a new task with natural language processing.",
                ),
        )
        .subcommands([
            Command::new("task")
                    .arg_required_else_help(true)
                    .propagate_version(true)
                    .subcommand_required(true)
                    .subcommands([
                       Command::new("create").about("Create a new task (without NLP)")
                         .arg(config_arg())
                         .arg(priority_arg())
                         .arg(content_arg())
                         .arg(description_arg())
                         .arg(due_arg())
                         .arg(project_arg())
                         .arg(flag_arg("nosection", 's',  "Do not prompt for section"))
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing")),
                       Command::new("edit").about("Edit an exising task's content")
                         .arg(config_arg())
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                         .arg(filter_arg())
                         .arg(project_arg()),
                       Command::new("list").about("List all tasks in a project")
                         .arg(config_arg())
                         .arg(project_arg())
                         .arg(filter_arg())
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing")),
                       Command::new("next").about("Get the next task by priority")
                         .arg(config_arg())
                         .arg(filter_arg())
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                         .arg(project_arg()),
                       Command::new("complete").about("Complete the last task fetched with the next command")
                         .arg(config_arg())
                ]),
            Command::new("project")
                   .arg_required_else_help(true)
                   .propagate_version(true)
                   .subcommand_required(true)
                   .subcommands([
                       Command::new("list").about("List all projects in config")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                         .arg(config_arg()),
                       Command::new("remove").about("Remove a project from config (not Todoist)")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg())
                         .arg(flag_arg("auto", 'a',  "Remove all projects from config that are not in Todoist"))
                         .arg(flag_arg("all", 'l',  "Remove all projects from config"))
                        .arg(project_arg()),
                       Command::new("rename").about("Rename a project in config (not Todoist)")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("empty").about("Empty a project by putting tasks in other projects")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("schedule").about("Assign dates to all tasks individually")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                         .arg(flag_arg("skip-recurring", 's',  "Don't re-schedule recurring tasks that are overdue"))
                        .arg(config_arg())
                         .arg(flag_arg("overdue", 'u',  "Only schedule overdue tasks"))
                        .arg(project_arg()),
                       Command::new("prioritize").about("Give every task a priority")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("import").about("Get projects from Todoist and prompt to add to config")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg()),
                       Command::new("process").about("Complete all tasks that are due today or undated in a project individually in priority order")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(config_arg())
                        .arg(project_arg())
                ]
                    ),
            Command::new("filter")
                   .arg_required_else_help(true)
                   .propagate_version(true)
                   .subcommand_required(true)
                   .subcommands([
                       Command::new("label").about("Iterate through tasks and apply labels from defined choices")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(filter_arg())
                        .arg(label_arg())
                         .arg(config_arg()),
                       Command::new("process").about("Iterate through tasks and complete them individually in priority order")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(filter_arg())
                         .arg(config_arg()),
                       Command::new("prioritize").about("Give every task a priority")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(filter_arg())
                         .arg(config_arg()),
                       Command::new("schedule").about("Assign dates to all tasks individually")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                        .arg(filter_arg())
                         .arg(config_arg()),
                ]
                    ),
            Command::new("version")
                   .arg_required_else_help(true)
                   .propagate_version(true)
                   .subcommand_required(true)
                   .subcommands([
                       Command::new("check").about("Check to see if tod is on the latest version, returns exit code 1 if out of date")
                         .arg(flag_arg("verbose", 'v',  "Display additional debug info while processing"))
                         .arg(config_arg()),
                ]
                    )
        ]
        )
}

// --- TOP LEVEL ---

#[cfg(not(tarpaulin_include))]
fn quickadd(matches: &ArgMatches, text: String) -> Result<String, String> {
    let config = fetch_config(matches)?;

    todoist::quick_add_task(&config, &text)?;
    Ok(color::green_string("✓"))
}

// --- TASK ---

#[cfg(not(tarpaulin_include))]
fn task_create(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let content = fetch_string(matches, &config, "content", "Content")?;
    let priority = fetch_priority(matches, &config)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    let description = fetch_description(matches);
    let due = fetch_due(matches);
    let section = if has_flag(matches, "nosection") || config.no_sections.unwrap_or_default() {
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
    )?;

    Ok(color::green_string("✓"))
}

#[cfg(not(tarpaulin_include))]
fn task_edit(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match fetch_project_or_filter(matches, &config)? {
        Flag::Project(project) => projects::rename_task(&config, &project),
        Flag::Filter(filter) => filters::rename_task(&config, filter),
    }
}
#[cfg(not(tarpaulin_include))]
fn task_list(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;

    match fetch_project_or_filter(matches, &config)? {
        Flag::Project(project) => projects::all_tasks(&config, &project),
        Flag::Filter(filter) => filters::all_tasks(&config, &filter),
    }
}

#[cfg(not(tarpaulin_include))]
fn task_next(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match fetch_project(matches, &config)? {
        Flag::Project(project) => projects::next_task(config, &project),
        Flag::Filter(filter) => filters::next_task(config, &filter),
    }
}

#[cfg(not(tarpaulin_include))]
fn task_complete(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match config.next_id {
        Some(_) => todoist::complete_task(&config),
        None => {
            Err("There is nothing to complete. A task must first be marked as 'next'.".to_string())
        }
    }
}

// --- PROJECT ---

#[cfg(not(tarpaulin_include))]
fn project_list(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;

    projects::list(&mut config)
}

#[cfg(not(tarpaulin_include))]
fn project_remove(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;
    let all = has_flag(matches, "all");
    let auto = has_flag(matches, "auto");
    match (all, auto) {
        (true, false) => projects::remove_all(&mut config),
        (false, true) => projects::remove_auto(&mut config),
        (false, false) => {
            let project = match fetch_project(matches, &config)? {
                Flag::Project(project) => project,
                _ => unreachable!(),
            };
            projects::remove(&mut config, &project)
        }
        (_, _) => Err(String::from("Incorrect flags provided")),
    }
}

#[cfg(not(tarpaulin_include))]
fn project_rename(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::rename(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_process(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::process_tasks(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_import(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;

    projects::import(&mut config)
}

#[cfg(not(tarpaulin_include))]
fn project_empty(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::empty(&mut config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_prioritize(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };

    projects::prioritize_tasks(&config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_schedule(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = match fetch_project(matches, &config)? {
        Flag::Project(project) => project,
        _ => unreachable!(),
    };
    let skip_recurring = has_flag(matches, "skip-recurring");
    let filter = if has_flag(matches, "overdue") {
        projects::TaskFilter::Overdue
    } else {
        projects::TaskFilter::Unscheduled
    };

    projects::schedule(&config, &project, filter, skip_recurring)
}

// --- FILTER ---

#[cfg(not(tarpaulin_include))]
fn filter_label(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let labels = fetch_labels(matches, &config)?;
    match fetch_filter(matches, &config)? {
        Flag::Filter(filter) => filters::label(&config, &filter, labels),
        _ => unreachable!(),
    }
}

#[cfg(not(tarpaulin_include))]
fn filter_process(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match fetch_filter(matches, &config)? {
        Flag::Filter(filter) => filters::process_tasks(config, &filter),
        _ => unreachable!(),
    }
}

#[cfg(not(tarpaulin_include))]
fn filter_prioritize(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match fetch_filter(matches, &config)? {
        Flag::Filter(filter) => filters::prioritize_tasks(&config, &filter),
        _ => unreachable!(),
    }
}

#[cfg(not(tarpaulin_include))]
fn filter_schedule(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match fetch_filter(matches, &config)? {
        Flag::Filter(filter) => filters::schedule(&config, &filter),
        _ => unreachable!(),
    }
}
// --- VERSION ---

#[cfg(not(tarpaulin_include))]
fn version_check(matches: &ArgMatches) -> Result<String, String> {
    use cargo::Version;

    let config = fetch_config(matches)?;

    match cargo::compare_versions(config) {
        Ok(Version::Latest) => Ok(format!("Tod is up to date with version: {}", VERSION)),
        Ok(Version::Dated(version)) => Err(format!(
            "Tod is out of date with version: {}, latest is:{}",
            VERSION, version
        )),
        Err(e) => Err(e),
    }
}

// --- ARGUMENT HELPERS ---

#[cfg(not(tarpaulin_include))]
fn priority_arg() -> Arg {
    Arg::new("priority")
        .long("priority")
        .num_args(1)
        .required(false)
        .value_name("PRIORITY")
        .help("Priority from 1 (without priority) to 4 (highest)")
}

#[cfg(not(tarpaulin_include))]
fn flag_arg(id: &'static str, short: char, help: &'static str) -> Arg {
    Arg::new(id)
        .short(short)
        .long(id)
        .value_parser(["yes", "no"])
        .num_args(0..1)
        .default_value("no")
        .default_missing_value("yes")
        .required(false)
        .help(help)
}

#[cfg(not(tarpaulin_include))]
fn config_arg() -> Arg {
    Arg::new("config")
        .short('o')
        .long("config")
        .num_args(1)
        .required(false)
        .value_name("CONFIGURATION PATH")
        .help("Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/tod.cfg")
}

#[cfg(not(tarpaulin_include))]
fn content_arg() -> Arg {
    Arg::new("content")
        .short('c')
        .long("content")
        .num_args(1)
        .required(false)
        .value_name("TASK TEXT")
        .help("Content for task")
}

#[cfg(not(tarpaulin_include))]
fn description_arg() -> Arg {
    Arg::new("description")
        .short('d')
        .long("description")
        .num_args(1)
        .required(false)
        .value_name("DESCRIPTION TEXT")
        .help("Description for task")
}

#[cfg(not(tarpaulin_include))]
fn due_arg() -> Arg {
    Arg::new("due")
        .short('u')
        .long("due")
        .num_args(1)
        .required(false)
        .value_name("DUE DATE")
        .help("Date date in format YYYY-MM-DD, YYYY-MM-DD HH:MM, or natural language")
}

#[cfg(not(tarpaulin_include))]
fn project_arg() -> Arg {
    Arg::new("project")
        .short('p')
        .long("project")
        .num_args(1)
        .required(false)
        .value_name("PROJECT NAME")
        .help("The project into which the task will be added")
}

#[cfg(not(tarpaulin_include))]
fn filter_arg() -> Arg {
    Arg::new("filter")
        .short('f')
        .long("filter")
        .num_args(1)
        .required(false)
        .value_name("FILTER_STRING")
        .help("Filter string https://todoist.com/help/articles/205248842")
}

#[cfg(not(tarpaulin_include))]
fn label_arg() -> Arg {
    Arg::new("labels")
        .short('l')
        .long("labels")
        .num_args(1..)
        .required(false)
        .value_name("LABEL1 LABEL2")
        .help("List of labels to choose from, to be applied to each entry")
}

// --- VALUE HELPERS ---

/// Checks if the flag was used
#[cfg(not(tarpaulin_include))]
fn has_flag(matches: &ArgMatches, id: &'static str) -> bool {
    matches.get_one::<String>(id) == Some(&String::from("yes"))
}

#[cfg(not(tarpaulin_include))]
fn fetch_config(matches: &ArgMatches) -> Result<Config, String> {
    let config_path = matches.get_one::<String>("config").map(|s| s.to_owned());

    let verbose = has_flag(matches, "verbose");

    config::get_or_create(config_path, verbose)?
        .check_for_timezone()?
        .check_for_latest_version()
}

fn fetch_description(matches: &ArgMatches) -> Option<String> {
    matches
        .get_one::<String>("description")
        .map(|s| s.to_owned())
}

fn fetch_due(matches: &ArgMatches) -> Option<String> {
    matches.get_one::<String>("due").map(|s| s.to_owned())
}

#[cfg(not(tarpaulin_include))]
fn fetch_string(
    matches: &ArgMatches,
    config: &Config,
    field: &str,
    prompt: &str,
) -> Result<String, String> {
    let argument_content = matches.get_one::<String>(field).map(|s| s.to_owned());
    match argument_content {
        Some(string) => Ok(string),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_project(matches: &ArgMatches, config: &Config) -> Result<Flag, String> {
    let project_content = matches.get_one::<String>("project").map(|s| s.to_owned());
    let projects = config.projects.clone().unwrap_or_default();
    if projects.is_empty() {
        return Err(NO_PROJECTS_ERR.to_string());
    }

    if projects.len() == 1 {
        return Ok(Flag::Project(projects.first().unwrap().clone()));
    }

    match project_content {
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
fn fetch_filter(matches: &ArgMatches, config: &Config) -> Result<Flag, String> {
    match matches.get_one::<String>("filter").map(|s| s.to_owned()) {
        Some(string) => Ok(Flag::Filter(string)),
        None => {
            let string = input::string("Enter a filter:", config.mock_string.clone())?;
            Ok(Flag::Filter(string))
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_project_or_filter(matches: &ArgMatches, config: &Config) -> Result<Flag, String> {
    let project_content = matches.get_one::<String>("project").map(|s| s.to_owned());
    let filter_content = matches.get_one::<String>("filter").map(|s| s.to_owned());

    match (project_content, filter_content) {
        (Some(_), None) => fetch_project(matches, config),
        (None, Some(_)) => fetch_filter(matches, config),
        (Some(_), Some(_)) => Err("Must select project OR filter".to_string()),
        (None, None) => {
            let options = vec![FlagOptions::Project, FlagOptions::Filter];
            match input::select("Select Project or Filter:", options, config.mock_select)? {
                FlagOptions::Project => fetch_project(matches, config),
                FlagOptions::Filter => fetch_filter(matches, config),
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_labels(matches: &ArgMatches, config: &Config) -> Result<Vec<String>, String> {
    match matches.get_many::<String>("labels") {
        None => {
            let labels = input::string(
                "Enter labels separated by spaces: ",
                config.mock_string.clone(),
            )?
            .split(' ')
            .map(|s| s.to_owned())
            .collect();

            Ok(labels)
        }
        Some(items) => {
            let labels = items
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>();

            Ok(labels)
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_priority(matches: &ArgMatches, config: &Config) -> Result<Priority, String> {
    match Priority::get_from_matches(matches) {
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
    cmd().debug_assert();
}
