use crate::config;
use crate::params;
use regex::Regex;
use serde_json::json;

const NAME_REGEX: &str = r"^(?P<name>\w*)$";
const NAME_NUMBER_REGEX: &str = r"^(?P<name>\w*) (?P<num>\d*)$";

pub fn list(config: config::Config) {
    println!("== Projects ==");
    for (k, _) in config.projects.iter() {
        println!("{}", k);
    }
}

pub fn add(params: params::Params, config: config::Config) {
    let re = Regex::new(NAME_NUMBER_REGEX).unwrap();
    let captures = re.captures(&params.text).unwrap();
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
        json: json!({ "token": config.token, "projects": projects}).to_string(),
        ..config
    };

    new_config.save();
}

pub fn remove(params: params::Params, config: config::Config) {
    let re = Regex::new(NAME_REGEX).unwrap();
    let captures = re.captures(&params.text).unwrap();
    let name = captures.name("name").unwrap().as_str();

    let mut projects = config.projects;
    projects.remove(name);

    let new_config = config::Config {
        projects: projects.clone(),
        json: json!({ "token": config.token, "projects": projects}).to_string(),
        ..config
    };

    new_config.save();
}
