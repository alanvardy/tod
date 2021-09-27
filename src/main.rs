#[cfg(test)]
#[macro_use]
extern crate matches;

extern crate clap;
use clap::{App, Arg};
use colored::*;

mod config;
mod items;
mod projects;
mod request;
mod test;
mod time;

const APP: &str = "Tod";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@alanvardy.com>";
const ABOUT: &str = "A tiny unofficial Todoist client";

struct Arguments<'a> {
    new_task: Option<String>,
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
    let app = App::new(APP).version(VERSION).author(AUTHOR).about(ABOUT);
    let matches = app
        .arg(
            Arg::with_name("new task")
                .short("t")
                .long("task")
                .required(false)
                .multiple(true)
                .min_values(1)
                .help(
                    "Create a new task with text. Can specify project option, defaults to inbox.",
                ),
        )
        .arg(
            Arg::with_name("project")
                .short("p")
                .long("project")
                .required(false)
                .value_name("PROJECT NAME")
                .help("The project namespace"),
        )
        .arg(
            Arg::with_name("next task")
                .short("n")
                .long("next")
                .required(false)
                .help("Get the next task by priority. Requires project option."),
        )
        .arg(
            Arg::with_name("complete task")
                .short("c")
                .long("complete")
                .required(false)
                .help("Complete the last task fetched with next"),
        )
        .arg(
            Arg::with_name("list projects")
                .short("l")
                .long("list")
                .required(false)
                .help("List all the projects in local config"),
        )
        .arg(
            Arg::with_name("add project")
                .short("a")
                .long("add")
                .required(false)
                .multiple(true)
                .min_values(2)
                .max_values(2)
                .value_names(&["PROJECT NAME", "PROJECT ID"])
                .help("Add a project to config with id"),
        )
        .arg(
            Arg::with_name("remove project")
                .short("r")
                .long("remove")
                .required(false)
                .value_name("PROJECT NAME")
                .help("Remove a project from config by name"),
        )
        .arg(
            Arg::with_name("sort inbox")
                .short("s")
                .long("sort")
                .required(false)
                .help("Sort inbox by moving tasks into projects"),
        )
        .arg(
            Arg::with_name("prioritize tasks")
                .short("z")
                .long("prioritize")
                .required(false)
                .help("Assign priorities to tasks. Can specify project option, defaults to inbox."),
        )
        .arg(
            Arg::with_name("scheduled items")
                .short("e")
                .long("scheduled")
                .required(false)
                .help("Returns items that are today and have a time. Can specify project option, defaults to inbox."),
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
        sort_inbox: matches.is_present("sort inbox"),
        prioritize_tasks: matches.is_present("prioritize tasks"),
        scheduled_items: matches.is_present("scheduled items"),
    };

    match dispatch(arguments) {
        Ok(text) => println!("{}", text.green()),
        Err(e) => println!("{}", e.red()),
    }
}

fn dispatch(arguments: Arguments) -> Result<String, String> {
    let config: config::Config = config::get_or_create();

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
        } => projects::prioritize_items(config, "inbox"),
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
        } => projects::prioritize_items(config, project_name),
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
        } => projects::scheduled_items(config, "inbox"),
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
        } => projects::scheduled_items(config, project_name),
        _ => Err(String::from(
            "Unrecognized input. For more information try --help",
        )),
    }
}
