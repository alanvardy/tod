use std::collections::HashMap;

use serde_json::{json, Number, Value};

mod request;

use crate::config::Config;
use crate::projects::Project;
use crate::sections::Section;
use crate::tasks::priority::Priority;
use crate::tasks::Task;
use crate::{projects, sections, tasks, time};

// TODOIST URLS
const QUICK_ADD_URL: &str = "/sync/v9/quick/add";
const PROJECT_DATA_URL: &str = "/sync/v9/projects/get_data";
const SYNC_URL: &str = "/sync/v9/sync";
pub const REST_V2_TASKS_URL: &str = "/rest/v2/tasks/";
const SECTIONS_URL: &str = "/rest/v2/sections";
const PROJECTS_URL: &str = "/rest/v2/projects";

/// Add a new task to the inbox with natural language support
pub fn quick_add_task(config: &Config, content: &str) -> Result<Task, String> {
    let url = String::from(QUICK_ADD_URL);
    let body = json!({"text": content, "auto_reminder": true});

    let json = request::post_todoist_sync(config, url, body)?;
    tasks::json_to_task(json)
}

/// Add Task without natural language support but supports additional parameters
#[allow(clippy::too_many_arguments)]
pub fn add_task(
    config: &Config,
    content: &str,
    project: &Project,
    section: Option<Section>,
    priority: Priority,
    description: Option<String>,
    due: Option<String>,
    labels: Vec<String>,
) -> Result<Task, String> {
    let url = String::from(REST_V2_TASKS_URL);
    let description = description.unwrap_or_default();
    let mut body: HashMap<String, Value> = HashMap::new();
    body.insert("content".to_owned(), Value::String(content.to_owned()));
    body.insert("description".to_owned(), Value::String(description));
    body.insert("project_id".to_owned(), Value::String(project.id.clone()));

    body.insert("auto_reminder".to_owned(), Value::Bool(true));
    body.insert(
        "priority".to_owned(),
        Value::Number(Number::from(priority.to_integer())),
    );
    let labels = labels.iter().map(|l| Value::String(l.to_owned())).collect();
    body.insert("labels".to_owned(), Value::Array(labels));

    if let Some(date) = due {
        if time::is_date(&date) || time::is_datetime(&date) {
            body.insert("due_date".to_owned(), Value::String(date));
        } else {
            body.insert("due_string".to_owned(), Value::String(date));
        }
    }

    if let Some(section) = section {
        body.insert("section_id".to_owned(), Value::String(section.id.clone()));
    }

    let body = json!(body);

    let json = request::post_todoist_rest(config, url, body)?;
    tasks::json_to_task(json)
}

/// Get a vector of all tasks for a project
pub fn tasks_for_project(config: &Config, project: &Project) -> Result<Vec<Task>, String> {
    let url = String::from(PROJECT_DATA_URL);
    let body = json!({ "project_id": project.id });
    let json = request::post_todoist_sync(config, url, body)?;
    tasks::sync_json_to_tasks(json)
}

pub fn tasks_for_filter(config: &Config, filter: &str) -> Result<Vec<Task>, String> {
    use urlencoding::encode;

    let encoded = encode(filter);
    let url = format!("{REST_V2_TASKS_URL}?filter={encoded}");
    let json = request::get_todoist_rest(config, url)?;
    tasks::rest_json_to_tasks(json)
}

pub fn sections_for_project(config: &Config, project: &Project) -> Result<Vec<Section>, String> {
    let project_id = &project.id;
    let url = format!("{SECTIONS_URL}?project_id={project_id}");
    let json = request::get_todoist_rest(config, url)?;
    sections::json_to_sections(json)
}

pub fn projects(config: &Config) -> Result<Vec<Project>, String> {
    let json = request::get_todoist_rest(config, PROJECTS_URL.to_string())?;
    projects::json_to_projects(json)
}

/// Move an task to a different project
pub fn move_task_to_project(
    config: &Config,
    task: Task,
    project: &Project,
) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": task.id, "project_id": project.id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

pub fn move_task_to_section(
    config: &Config,
    task: Task,
    section: &Section,
) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": task.id, "section_id": section.id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;
    Ok(String::from("✓"))
}

/// Update the priority of an task by ID
pub fn update_task_priority(
    config: Config,
    task: Task,
    priority: Priority,
) -> Result<String, String> {
    let body = json!({ "priority": priority });
    let url = format!("{}{}", REST_V2_TASKS_URL, task.id);

    request::post_todoist_rest(&config, url, body)?;
    // Does not pass back an task
    Ok(String::from("✓"))
}

/// Add a label to task by ID
pub fn add_task_label(config: &Config, task: Task, label: String) -> Result<String, String> {
    let mut labels = task.labels;
    labels.push(label);
    let body = json!({ "labels": labels});
    let url = format!("{}{}", REST_V2_TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body)?;
    // Does not pass back an task
    Ok(String::from("✓"))
}

/// Update due date for task using natural language
pub fn update_task_due(config: &Config, task: Task, due_string: String) -> Result<String, String> {
    let due_string = if task.is_recurring() {
        format!("{} starting {due_string}", task.due.unwrap().string)
    } else {
        due_string
    };
    let body = json!({ "due_string": due_string });
    let url = format!("{}{}", REST_V2_TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body)?;
    // Does not pass back an task
    Ok(String::from("✓"))
}

/// Update the name of an task by ID
pub fn update_task_name(config: &Config, task: Task, new_name: String) -> Result<String, String> {
    let body = json!({ "content": new_name });
    let url = format!("{}{}", REST_V2_TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body)?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

/// Complete the last task returned by "next task"
pub fn complete_task(config: &Config) -> Result<String, String> {
    let body = json!({"commands": [{"type": "item_close", "uuid": request::new_uuid(), "temp_id": request::new_uuid(), "args": {"id": config.next_id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body)?;

    if !cfg!(test) {
        config.clone().clear_next_id().save()?;
    }

    // Does not pass back a task
    Ok(String::from("✓"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::priority::{self, Priority};
    use crate::tasks::{DateInfo, Task};
    use crate::{test, time};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_quick_add_task() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/quick/add")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        assert_eq!(
            quick_add_task(&config, "testy test"),
            Ok(Task {
                id: String::from("5149481867"),
                priority: Priority::None,
                content: String::from("testy test"),
                labels: vec![],
                checked: Some(false),
                description: String::from(""),
                due: None,
                is_deleted: Some(false),
                is_completed: None,
            })
        );
        mock.assert();
    }

    #[test]
    fn test_add_task() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/rest/v2/tasks/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let project = test::fixtures::project();

        let priority = priority::Priority::None;
        let section = test::fixtures::section();
        assert_eq!(
            add_task(
                &config,
                "New task",
                &project,
                Some(section),
                priority,
                None,
                None,
                vec![]
            ),
            Ok(Task {
                id: String::from("5149481867"),
                priority: Priority::None,
                content: String::from("testy test"),
                checked: Some(false),
                labels: vec![],
                description: String::from(""),
                due: None,
                is_deleted: Some(false),
                is_completed: None,
            })
        );
        mock.assert();
    }

    #[test]
    fn should_get_tasks_for_project() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/sync/v9/projects/get_data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::post_tasks())
            .create();

        let config = test::fixtures::config().mock_url(server.url());
        let config_with_timezone = Config {
            timezone: Some(String::from("US/Pacific")),
            ..config
        };
        let binding = config_with_timezone.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();

        assert_eq!(
            tasks_for_project(&config_with_timezone, project),
            Ok(vec![Task {
                id: String::from("999999"),
                content: String::from("Put out recycling"),
                checked: Some(false),
                labels: vec![],
                description: String::from(""),
                due: Some(DateInfo {
                    date: format!("{}T23:59:00Z", time::today_string(&config_with_timezone)),
                    is_recurring: true,
                    timezone: None,
                    string: String::from("every other mon at 16:30"),
                }),
                priority: Priority::Medium,
                is_deleted: Some(false),
                is_completed: None,
            }])
        );

        mock.assert();
    }

    #[test]
    fn should_complete_a_task() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config()
            .mock_url(server.url())
            .set_next_id(&"112233".to_string());

        let response = complete_task(&config);
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_move_a_task() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let task = test::fixtures::task();
        let config = test::fixtures::config().mock_url(server.url());

        let config = Config {
            mock_url: Some(server.url()),
            ..config
        };

        let binding = config.projects.clone().unwrap_or_default();
        let project = binding.first().unwrap();
        let response = move_task_to_project(&config, task, project);
        mock.assert();

        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_prioritize_a_task() {
        let task = test::fixtures::task();
        let url: &str = &format!("{}{}", "/rest/v2/tasks/", task.id);
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = update_task_priority(config, task, Priority::High);
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[test]
    fn should_update_date_on_a_task() {
        let task = test::fixtures::task();
        let url: &str = &format!("{}{}", "/rest/v2/tasks/", task.id);
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = update_task_due(&config, task, "today".to_string());
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }
}
