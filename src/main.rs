#[cfg(test)]
#[macro_use]
extern crate matches;

extern crate clap;

use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;

mod config;
mod items;
mod projects;
mod request;
mod sections;
mod test;
mod time;

const APP: &str = "Tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Todoist client";

struct Arguments<'a> {
    new_task: Option<String>,
    config_path: Option<&'a str>,
    project: Option<&'a str>,
    next_task: bool,
    next_task_interactive: bool,
    complete_task: bool,
    list_projects: bool,
    add_project: Option<Vec<String>>,
    remove_project: Option<&'a str>,
    sort_inbox: bool,
    prioritize_tasks: bool,
    scheduled_items: bool,
}

fn main() {
    let app = Command::new(APP)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT);
    let matches = app
        .arg(
            Arg::new("new task")
                .short('t')
                .long("task")
                .required(false)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_parser(clap::value_parser!(String))
                .help(
                    "Create a new task with text. Can specify project option, defaults to inbox.",
                ),
        )
        .arg(
            Arg::new("project")
                .short('p')
                .long("project")
                .required(false)
                .value_name("PROJECT NAME")
                .help("The project namespace, for filtering other commands, use by itself to list all tasks for the project"),
        )
        .arg(
            flag_arg("next task", 'n', "next", "Get the next task by priority. Requires project option.")                    )      
        .arg(
            flag_arg("next task (interactive)", 'x', "nextinteractive", "Fetches tasks one at a time by priority, and completes with an interactive prompt. Requires project option.") )
        .arg(
            flag_arg("complete task", 'c', "complete", "Complete the last task fetched with next")
                            )
        .arg(
            flag_arg("list projects", 'l', "list", "List all projects in the local config."),
        )
        .arg(
            Arg::new("add project")
                .short('a')
                .long("add")
                .required(false)
                .action(ArgAction::Append)
                .num_args(2)
                .value_parser(clap::value_parser!(String))
                .value_names(["PROJECT NAME", "PROJECT ID"])
                .help("Add a project to config with id"),
        )
        .arg(
            Arg::new("remove project")
                .short('r')
                .long("remove")
                .num_args(1)
                .required(false)
                .value_name("PROJECT NAME")
                .help("Remove a project from config by name"),
        )
        .arg(
            flag_arg("sort inbox", 's', "sort", "Sort inbox by moving tasks into projects")
        )
        .arg(
            flag_arg("prioritize tasks", 'z', "prioritize", "Assign priorities to tasks. Can specify project option, defaults to inbox.")
        )
        .arg(
            flag_arg("scheduled items", 'e', "scheduled", "Returns items that are today and have a time. Can specify project option, defaults to inbox.")
        )
        .arg(
            Arg::new("configuration path")
                .short('o')
                .long("config")
                .num_args(1)
                .required(false)
                .value_name("CONFIGURATION PATH")
                .help("Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/tod.cfg"),
        )
        .get_matches();

    let new_task = matches
        .get_many("new task")
        .map(|values| values.cloned().collect::<Vec<String>>().join(" "));
    let add_project = matches
        .get_many("add project")
        .map(|values| values.cloned().collect::<Vec<String>>());

    let arguments = Arguments {
        new_task,
        project: matches.get_one::<String>("project").map(|s| s.as_str()),
        next_task: has_flag(matches.clone(), "next task"),
        next_task_interactive: has_flag(matches.clone(), "next task (interactive)"),
        complete_task: has_flag(matches.clone(), "complete task"),
        list_projects: has_flag(matches.clone(), "list projects"),
        sort_inbox: has_flag(matches.clone(), "sort inbox"),
        add_project,
        remove_project: matches
            .get_one::<String>("remove project")
            .map(|s| s.as_str()),
        config_path: matches
            .get_one::<String>("configuration path")
            .map(|s| s.as_str()),
        prioritize_tasks: has_flag(matches.clone(), "prioritize tasks"),
        scheduled_items: has_flag(matches.clone(), "scheduled items"),
    };

    match dispatch(arguments) {
        Ok(text) => {
            println!("{text}");
            std::process::exit(0);
        }
        Err(e) => {
            println!("{}", e.red());
            std::process::exit(1);
        }
    }
}

fn dispatch(arguments: Arguments) -> Result<String, String> {
    let config_path = arguments.config_path.map(|s| s.to_string());
    let config: config::Config = config::get_or_create(config_path)?;

    match arguments {
        Arguments {
            new_task: Some(task),
            project: Some(project),
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::add_item_to_project(config, &task, project),
        Arguments {
            new_task: Some(task),
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::add_item_to_project(config, &task, "inbox"),
        Arguments {
            new_task: None,
            project: Some(project),
            next_task: true,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::next_item(config, project),

        Arguments {
            new_task: None,
            project: Some(project),
            next_task: false,
            next_task_interactive: true,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::next_item_interactive(config, project),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: true,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => match request::complete_item(config) {
            Ok(_) => Ok(String::from("✓")),
            Err(err) => Err(err),
        },
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: true,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::list(config),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: Some(params),
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::add(config, params),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: Some(project_name),
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::remove(config, project_name),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: true,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::sort_inbox(config),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: true,
            scheduled_items: false,
            config_path: _,
        } => projects::prioritize_items(&config, "inbox"),
        Arguments {
            new_task: None,
            project: Some(project_name),
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: true,
            scheduled_items: false,
            config_path: _,
        } => projects::prioritize_items(&config, project_name),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: true,
            config_path: _,
        } => projects::scheduled_items(&config, "inbox"),
        Arguments {
            new_task: None,
            project: Some(project_name),
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: true,
            config_path: _,
        } => projects::scheduled_items(&config, project_name),
        Arguments {
            new_task: None,
            project: Some(project_name),
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::all_items(&config, project_name),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            next_task_interactive: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => Err(String::from(
            "Tod cannot be run without parameters. To see available parameters use --help",
        )),
        _ => Err(String::from(
            "Invalid parameters. For more information try --help",
        )),
    }
}

fn flag_arg(id: &'static str, short: char, long: &'static str, help: &'static str) -> Arg {
    Arg::new(id)
        .short(short)
        .long(long)
        .value_parser(["yes", "no"])
        .num_args(0..1)
        .default_value("no")
        .default_missing_value("yes")
        .required(false)
        .help(help)
}

fn has_flag(matches: ArgMatches, id: &'static str) -> bool {
    matches.get_one::<String>(id) == Some(&String::from("yes"))
}
