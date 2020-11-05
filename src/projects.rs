use crate::config::Config;
use crate::params::Params;
use regex::Regex;

const NAME_REGEX: &str = r"^(?P<name>\w*)$";
const NAME_NUMBER_REGEX: &str = r"^(?P<name>\w*) (?P<num>\d*)$";

/// List the projects in config
pub fn list(config: Config) {
    println!("== Projects ==");
    for (k, _) in config.projects.iter() {
        println!("{}", k);
    }
}

/// Add a project to the projects HashMap in Config
pub fn add(config: Config, params: Params) {
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

    config.add_project(name, num).save();
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, params: Params) {
    let name = Regex::new(NAME_REGEX)
        .unwrap()
        .captures(&params.text)
        .unwrap()
        .name("name")
        .unwrap()
        .as_str();

    config.remove_project(name).save();
}
