#[cfg(test)]
#[macro_use]
extern crate matches;

extern crate clap;
use clap::{Arg, Command};
use colored::*;

mod config;
mod items;
mod projects;
mod request;
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
    complete_task: bool,
    list_projects: bool,
    add_project: Option<Vec<&'a str>>,
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
                .multiple_occurrences(true)
                .min_values(1)
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
            Arg::new("next task")
                .short('n')
                .long("next")
                .required(false)
                .help("Get the next task by priority. Requires project option."),
        )
        .arg(
            Arg::new("complete task")
                .short('c')
                .long("complete")
                .required(false)
                .help("Complete the last task fetched with next"),
        )
        .arg(
            Arg::new("list projects")
                .short('l')
                .long("list")
                .required(false)
                .help("List all the projects in local config"),
        )
        .arg(
            Arg::new("add project")
                .short('a')
                .long("add")
                .required(false)
                .multiple_occurrences(true)
                .min_values(2)
                .max_values(2)
                .value_names(&["PROJECT NAME", "PROJECT ID"])
                .help("Add a project to config with id"),
        )
        .arg(
            Arg::new("remove project")
                .short('r')
                .long("remove")
                .required(false)
                .value_name("PROJECT NAME")
                .help("Remove a project from config by name"),
        )
        .arg(
            Arg::new("sort inbox")
                .short('s')
                .long("sort")
                .required(false)
                .help("Sort inbox by moving tasks into projects"),
        )
        .arg(
            Arg::new("prioritize tasks")
                .short('z')
                .long("prioritize")
                .required(false)
                .help("Assign priorities to tasks. Can specify project option, defaults to inbox."),
        )
        .arg(
            Arg::new("scheduled items")
                .short('e')
                .long("scheduled")
                .required(false)
                .help("Returns items that are today and have a time. Can specify project option, defaults to inbox."),
        )
        .arg(
            Arg::new("configuration path")
                .short('o')
                .long("config")
                .required(false)
                .value_name("CONFIGURATION PATH")
                .help("Absolute path of configuration. Defaults to ~/.tod.cfg."),
        )
        .get_matches();

    let new_task = matches
        .values_of("new task")
        .map(|values| values.collect::<Vec<&str>>().join(" "));
    let add_project = matches
        .values_of("add project")
        .map(|values| values.collect::<Vec<&str>>());

    let arguments = Arguments {
        new_task,
        project: matches.value_of("project"),
        next_task: matches.is_present("next task"),
        complete_task: matches.is_present("complete task"),
        list_projects: matches.is_present("list projects"),
        add_project,
        remove_project: matches.value_of("remove project"),
        config_path: matches.value_of("configuration path"),
        sort_inbox: matches.is_present("sort inbox"),
        prioritize_tasks: matches.is_present("prioritize tasks"),
        scheduled_items: matches.is_present("scheduled items"),
    };

    match dispatch(arguments) {
        Ok(text) => {
            println!("{}", text);
            std::process::exit(0);
        }
        Err(e) => {
            println!("{}", e.red());
            std::process::exit(1);
        }
    }
}

fn dispatch(arguments: Arguments) -> Result<String, String> {
    let config: config::Config = config::get_or_create(arguments.config_path)?;

    match arguments {
        Arguments {
            new_task: Some(task),
            project: Some(project),
            next_task: false,
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
            project: None,
            next_task: false,
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
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
            prioritize_tasks: false,
            scheduled_items: false,
            config_path: _,
        } => projects::all_items(&config, project_name),
        _ => Err(String::from(
            "Unrecognized input. For more information try --help",
        )),
    }
}
