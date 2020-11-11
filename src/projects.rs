use crate::config::Config;
use crate::params::Params;
use regex::Regex;

const NAME_REGEX: &str = r"^(?P<name>\w*)$";
const NAME_NUMBER_REGEX: &str = r"^(?P<name>\w*) (?P<num>\d*)$";
const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";
const REMOVE_ERROR: &str = "Must provide project name, i.e. tod --remove projectname";

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
        .expect(ADD_ERROR)
        .captures(&params.text)
        .expect(ADD_ERROR);

    let name = captures.name("name").unwrap().as_str();

    let num = captures
        .name("num")
        .expect(ADD_ERROR)
        .as_str()
        .parse::<u32>()
        .expect(ADD_ERROR);

    config.add_project(name, num).save();
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, params: Params) {
    let name = Regex::new(NAME_REGEX)
        .expect(REMOVE_ERROR)
        .captures(&params.text)
        .expect(REMOVE_ERROR)
        .name("name")
        .expect(REMOVE_ERROR)
        .as_str();

    config.remove_project(name).save();
}
