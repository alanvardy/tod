#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate clap;

use clap::{Arg, ArgAction, ArgMatches, Command};
use config::Config;
use items::priority::Priority;

mod cargo;
mod color;
mod config;
mod input;
mod items;
mod projects;
mod sections;
mod test;
mod time;
mod todoist;

const APP: &str = "Tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Todoist client";

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
            Some(("add", m)) => project_add(m),
            Some(("remove", m)) => project_remove(m),
            Some(("process", m)) => project_process(m),
            Some(("empty", m)) => project_empty(m),
            Some(("schedule", m)) => project_schedule(m),
            Some(("prioritize", m)) => project_prioritize(m),
            Some(("import", m)) => project_import(m),
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
                         .arg(project_arg()),
                       Command::new("edit").about("Edit an exising task's content")
                         .arg(config_arg())
                         .arg(project_arg()),
                       Command::new("list").about("List all tasks in a project")
                         .arg(config_arg())
                         .arg(project_arg())
                         .arg(flag_arg("scheduled", 's',  "Only list tasks that are scheduled for today and have a time")),
                       Command::new("next").about("Get the next task by priority")
                         .arg(config_arg())
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
                         .arg(config_arg()),
                       Command::new("add").about("Add a project to config (not Todoist)")
                        .arg(config_arg())
                        .arg(name_arg())
                        .arg(id_arg()),
                       Command::new("remove").about("Remove a project from config (not Todoist)")
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("empty").about("Empty a project by putting tasks in other projects")
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("schedule").about("Assign dates to all tasks individually")
                        .arg(config_arg())
                         .arg(flag_arg("overdue", 'v',  "Only schedule overdue tasks"))
                        .arg(project_arg()),
                       Command::new("prioritize").about("Give every task a priority")
                        .arg(config_arg())
                        .arg(project_arg()),
                       Command::new("import").about("Get projects from Todoist and prompt to add to config")
                        .arg(config_arg()),
                       Command::new("process").about("Complete all tasks that are due today or undated in a project individually in priority order")
                        .arg(config_arg())
                        .arg(project_arg())
                ]
                    ),
            Command::new("version")
                   .arg_required_else_help(true)
                   .propagate_version(true)
                   .subcommand_required(true)
                   .subcommands([
                       Command::new("check").about("Check to see if tod is on the latest version, returns exit code 1 if out of date")
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

    todoist::quick_add_item(&config, &text)?;
    Ok(color::green_string("✓"))
}

// --- TASK ---

#[cfg(not(tarpaulin_include))]
fn task_create(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let content = fetch_string(matches, &config, "content", "Content")?;
    let priority = fetch_priority(matches, &config)?;
    let project = fetch_project(matches, &config)?;

    let description = matches
        .get_one::<String>("description")
        .map(|s| s.to_owned())
        .unwrap_or(String::new());
    projects::add_item_to_project(&config, content, &project, priority, description)
}

#[cfg(not(tarpaulin_include))]
fn task_edit(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project_name = fetch_project(matches, &config)?;
    let project_id = projects::project_id(&config, &project_name)?;

    projects::rename_items(&config, &project_id)
}
#[cfg(not(tarpaulin_include))]
fn task_list(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    if has_flag(matches, "scheduled") {
        projects::scheduled_items(&config, &project)
    } else {
        projects::all_items(&config, &project)
    }
}

#[cfg(not(tarpaulin_include))]
fn task_next(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    projects::next(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn task_complete(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    match config.next_id {
        Some(_) => todoist::complete_item(&config),
        None => Err("There is nothing to complete. Try to mark a task as 'next'.".to_string()),
    }
}

// --- PROJECT ---

#[cfg(not(tarpaulin_include))]
fn project_list(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;

    projects::list(&config)
}

#[cfg(not(tarpaulin_include))]
fn project_add(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;
    let name = fetch_string(matches, &config, "name", "Enter project name or alias")?;
    let id = fetch_string(matches, &config, "id", "Enter ID of project")?;

    projects::add(&mut config, name, id)
}

#[cfg(not(tarpaulin_include))]
fn project_remove(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    projects::remove(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_process(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    projects::process_items(config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_import(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;

    projects::import(&mut config)
}

#[cfg(not(tarpaulin_include))]
fn project_empty(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    projects::empty(&config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_prioritize(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;

    projects::prioritize_items(&config, &project)
}

#[cfg(not(tarpaulin_include))]
fn project_schedule(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let project = fetch_project(matches, &config)?;
    let filter = if has_flag(matches, "overdue") {
        projects::TaskFilter::Overdue
    } else {
        projects::TaskFilter::Unscheduled
    };

    projects::schedule(&config, &project, filter)
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
fn id_arg() -> Arg {
    Arg::new("id")
        .short('i')
        .long("id")
        .num_args(1)
        .required(false)
        .value_name("ID")
        .help("Identification key")
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
fn name_arg() -> Arg {
    Arg::new("name")
        .short('n')
        .long("name")
        .num_args(1)
        .required(false)
        .value_name("PROJECT NAME")
        .help("Name of project")
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

// --- VALUE HELPERS ---

/// Checks if the flag was used
#[cfg(not(tarpaulin_include))]
fn has_flag(matches: &ArgMatches, id: &'static str) -> bool {
    matches.get_one::<String>(id) == Some(&String::from("yes"))
}

#[cfg(not(tarpaulin_include))]
fn fetch_config(matches: &ArgMatches) -> Result<Config, String> {
    let config_path = matches.get_one::<String>("config").map(|s| s.to_owned());

    config::get_or_create(config_path)?
        .check_for_timezone()?
        .check_for_latest_version()
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
fn fetch_project(matches: &ArgMatches, config: &Config) -> Result<String, String> {
    let project_content = matches.get_one::<String>("project").map(|s| s.to_owned());
    match project_content {
        Some(string) => Ok(string),
        None => {
            let options = projects::project_names(config)?;
            input::select("Select project", options, config.mock_select)
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
