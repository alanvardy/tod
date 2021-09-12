use crate::config::Config;

const ADD_ERROR: &str = "Must provide project name and number, i.e. tod --add projectname 12345";

/// List the projects in config
pub fn list(config: Config) {
    println!("== Projects ==");
    for (k, _) in config.projects.iter() {
        println!("{}", k);
    }
}

/// Add a project to the projects HashMap in Config
pub fn add(config: Config, params: Vec<&str>) {
    let mut params = params.clone();
    let num = params
        .pop()
        .expect(ADD_ERROR)
        .parse::<u32>()
        .expect(ADD_ERROR);

    let name = params.pop().expect(ADD_ERROR);

    config.add_project(name, num).save()
}

/// Remove a project from the projects HashMap in Config
pub fn remove(config: Config, project_name: &str) {
    config.remove_project(project_name).save()
}

pub fn project_id(config: &Config, project_name: &str) -> String {
    let project_id = *config.projects.get(project_name).unwrap_or_else(|| {
        panic!(
            "Project {} not found, please add it to config",
            project_name
        )
    });

    project_id.to_string()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::config;
//     use std::collections::HashMap;

//     #[test]
//     fn add_and_remove_project_should_work() {
//         // Add a project
//         let config = Config::new("abcd");
//         let params = vec!["some_project", "1234"];

//         let mut projects: HashMap<String, u32> = HashMap::new();
//         projects.insert(String::from("some_project"), 1234);
//         let new_config = config::Config {
//             path: config::generate_path(),
//             token: String::from("abcd"),
//             next_id: None,
//             projects: projects.clone(),
//         };

//         let config_with_one_project = add(config, params);

//         assert_eq!(config_with_one_project, new_config);

//         // Add a second project
//         projects.insert(String::from("some_other_project"), 2345);
//         let params = vec!["some_other_project", "3456"];

//         let config_with_two_projects = add(config_with_one_project, params);

//         // Remove the first project
//         let config_with_other_project = remove(config_with_two_projects, "some_project");

//         let mut projects: HashMap<String, u32> = HashMap::new();
//         projects.insert(String::from("some_other_project"), 3456);
//         let new_config = config::Config {
//             path: config::generate_path(),
//             token: String::from("abcd"),
//             next_id: None,
//             projects,
//         };

//         assert_eq!(config_with_other_project, new_config);
//     }
// }
