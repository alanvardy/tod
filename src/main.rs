#[cfg(test)]
#[macro_use]
extern crate matches;

extern crate clap;
use clap::{App, Arg};

mod config;
mod items;
mod projects;
mod request;

const APP: &str = "Tod";
const VERSION: &str = "0.1.2";
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
                .min_values(2)
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
    };

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
        } => request::build_project_request(config, &task, project).perform(),
        Arguments {
            new_task: Some(task),
            project: None,
            next_task: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
        } => request::build_index_request(config, &task).perform(),
        Arguments {
            new_task: None,
            project: Some(project),
            next_task: true,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
        } => request::build_next_request(config, project).perform(),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            complete_task: true,
            list_projects: false,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
        } => request::build_complete_request(config).perform(),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            complete_task: false,
            list_projects: true,
            add_project: None,
            remove_project: None,
            sort_inbox: false,
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
        } => projects::add(config, params).save(),
        Arguments {
            new_task: None,
            project: None,
            next_task: false,
            complete_task: false,
            list_projects: false,
            add_project: None,
            remove_project: Some(project_name),
            sort_inbox: false,
        } => projects::remove(config, project_name).save(),
        _ => println!("Unrecognized input. For more information try --help"),
    };
}
