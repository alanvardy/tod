use crate::config;
use crate::params;
use regex::Regex;

const NAME_REGEX: &str = r"^(?P<name>\w*)$";
const NAME_NUMBER_REGEX: &str = r"^(?P<name>\w*) (?P<num>\d*)$";

/// List the projects in config
pub fn list(config: config::Config) {
    println!("== Projects ==");
    for (k, _) in config.projects.iter() {
        println!("{}", k);
    }
}

/// Add a project to the projects HashMap in Config
pub fn add(params: params::Params, config: config::Config) {
    let captures = Regex::new(NAME_NUMBER_REGEX)
    .unwrap()
    .captures(&params.text)
    .unwrap();
    let name = captures.name("name").unwrap().as_str();
    let num = captures
    .name("num")
    .unwrap()
    .as_str()
    .parse::<u32>()
    .unwrap();

    let mut projects = config.projects;
    projects.insert(String::from(name), num);

    let new_config = config::Config {
        projects: projects.clone(),
        ..config
    };

    new_config.save();
}

/// Remove a project from the projects HashMap in Config
pub fn remove(params: params::Params, config: config::Config) {
    let name = Regex::new(NAME_REGEX)
        .unwrap()
        .captures(&params.text)
        .unwrap()
        .name("name")
        .unwrap()
        .as_str();

    let mut projects = config.projects;
    projects.remove(name);

    let new_config = config::Config {
        projects,
        ..config
    };

    new_config.save();
}
