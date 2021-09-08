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
pub fn add(config: Config, params: Params) -> Config {
    let captures = Regex::new(NAME_NUMBER_REGEX)
        .expect(ADD_ERROR)
        .captures(&params.text)
        .expect(ADD_ERROR);

    let name = captures.name("name").expect(ADD_ERROR).as_str();

    let num = captures
        .name("num")
        .expect(ADD_ERROR)
        .as_str()
        .parse::<u32>()
        .expect(ADD_ERROR);

    config.add_project(name, num)
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, params: Params) -> Config {
    let name = Regex::new(NAME_REGEX)
        .expect(REMOVE_ERROR)
        .captures(&params.text)
        .expect(REMOVE_ERROR)
        .name("name")
        .expect(REMOVE_ERROR)
        .as_str();

    config.remove_project(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::collections::HashMap;

    #[test]
    fn add_and_remove_project_should_work() {
        // Add a project
        let config = Config::new("abcd");
        let params = Params::new(vec![
            String::from("--add"),
            String::from("some_project"),
            String::from("1234"),
        ]);

        let mut projects: HashMap<String, u32> = HashMap::new();
        projects.insert(String::from("some_project"), 1234);
        let new_config = config::Config {
            path: config::generate_path(),
            token: String::from("abcd"),
            next_id: None,
            projects: projects.clone(),
        };

        let config_with_one_project = add(config, params);

        assert_eq!(config_with_one_project, new_config);

        // Add a second project
        projects.insert(String::from("some_other_project"), 2345);
        let params = Params::new(vec![
            String::from("--add"),
            String::from("some_other_project"),
            String::from("3456"),
        ]);

        let config_with_two_projects = add(config_with_one_project, params);

        // Remove the first project
        let params = Params::new(vec![String::from("--remove"), String::from("some_project")]);
        let config_with_other_project = remove(config_with_two_projects, params);

        let mut projects: HashMap<String, u32> = HashMap::new();
        projects.insert(String::from("some_other_project"), 3456);
        let new_config = config::Config {
            path: config::generate_path(),
            token: String::from("abcd"),
            next_id: None,
            projects,
        };

        assert_eq!(config_with_other_project, new_config);
    }
}
