use futures::future;
use serde_json::{Number, Value, json};
use std::collections::HashMap;
use urlencoding::encode;
mod request;

use crate::comment::Comment;
use crate::config::Config;
use crate::error::Error;
use crate::id::{self, ID, Id, Resource};
use crate::labels::{self, Label};
use crate::projects::Project;
use crate::sections::Section;
use crate::tasks::Task;
use crate::tasks::priority::Priority;
use crate::user::{SyncResponse, User};
use crate::{color, projects, sections, tasks, time};

// TODOIST URLS
const SYNC_URL: &str = "/sync/v9/sync";
pub const TASKS_URL: &str = "/api/v1/tasks/";
pub const COMMENTS_URL: &str = "/rest/v2/comments/";
const SECTIONS_URL: &str = "/api/v1/sections";
const PROJECTS_URL: &str = "/api/v1/projects";
const LABELS_URL: &str = "/rest/v2/labels";
const IDS_URL: &str = "/api/v1/id_mappings/";

/// Used to sanity check all the Todoist API endpoints to make sure that we are able to process the JSON payloads they are sending back.
pub async fn test_all_endpoints(config: Config) -> Result<String, Error> {
    let name = "TEST".to_string();
    let priority = Priority::None;
    let labels = vec![String::from("one"), String::from("two")];

    println!("Creating project");
    let project = create_project(&config, name.clone(), false).await?;

    println!("List projects");
    let _projects = projects(&config).await?;

    println!("Creating task with add_task");
    let task = add_task(
        &config,
        &name,
        &project,
        None,
        priority.clone(),
        &name,
        &None,
        &[],
    )
    .await?;

    println!("Getting task with get_task");
    let task = get_task(&config, &task.id).await?;

    println!("Commenting on task");
    let _comment = comment_task(&config, ID::V1(task.id.clone()), name.clone(), false).await?;

    println!("Getting comments for task");
    let _comments = comments(&config, &task).await?;

    println!("Deleting task");
    delete_task(&config, &task, false).await?;

    println!("Creating task with quick_add_task");
    let task = quick_add_task(&config, &name).await?;

    println!("Finding tasks with tasks_for_project");
    let _tasks = tasks_for_project(&config, &project).await?;

    println!("Finding tasks with tasks_for_filter");
    let _tasks = tasks_for_filter(&config, "tod").await?;

    println!("Updating task priority");
    let _task = update_task_priority(&config, &task, &priority, false).await?;

    println!("Updating task content");
    let _task = update_task_content(&config, &task, name.clone(), false).await?;

    println!("Updating task description");
    let _task = update_task_description(&config, &task, name, false).await?;

    println!("Updating task labels");
    let _task = update_task_labels(&config, &task, labels, false).await?;

    println!("Adding task label");
    let _task = add_task_label(&config, task.clone(), String::from("three"), false).await?;

    println!("Updating task due with natural language");
    let _task =
        update_task_due_natural_language(&config, task.clone(), "today".to_string(), None, false)
            .await?;

    println!("Moving task to project");
    let _task = move_task_to_project(&config, task.clone(), &project, false).await?;

    println!("Completing task");
    let _task = complete_task(&config, &task, false).await?;

    println!("Deleting task");
    delete_task(&config, &task, false).await?;

    println!("Deleting project");
    delete_project(&config, &project, false).await?;

    println!("List labels");
    let _labels = list_labels(&config, false).await?;

    println!("Get user data");
    let _data = get_user_data(&config).await?;

    // Still to be implemented:
    // - sections_for_project
    // - move_task_to_section
    Ok(color::green_string("Completed successfully"))
}

pub async fn get_legacy_id(config: &Config, resource: Resource, id: ID) -> Result<String, Error> {
    match id {
        ID::Legacy(id) => Ok(id),
        ID::V1(id) => {
            let url = format!("{IDS_URL}{resource}/{id}");
            let json = request::get_todoist_rest(config, url, true).await?;
            match id::json_to_ids(json)?.pop() {
                None => Err(Error {
                    source: "get_legacy_id".to_string(),
                    message: format!("Could not convert {id} to legacy id"),
                }),
                Some(Id { old_id, .. }) => Ok(old_id),
            }
        }
    }
}

pub async fn get_v1_ids(
    config: &Config,
    resource: Resource,
    ids: Vec<String>,
) -> Result<Vec<String>, Error> {
    let ids = ids.join(",");
    let url = format!("{IDS_URL}{resource}/{ids}");
    let json = request::get_todoist_rest(config, url, true).await?;
    let ids = id::json_to_ids(json)?
        .into_iter()
        .map(|i| i.new_id)
        .collect();

    Ok(ids)
}

/// Add a new task to the inbox with natural language support
pub async fn quick_add_task(config: &Config, content: &str) -> Result<Task, Error> {
    let url = format!("{TASKS_URL}quick");
    let body = json!({"text": content, "auto_reminder": true});

    let json = request::post_todoist_sync(config, url, body, true).await?;
    tasks::json_to_task(json)
}

pub async fn get_task(config: &Config, id: &str) -> Result<Task, Error> {
    let url = format!("{TASKS_URL}{id}");
    let json = request::get_todoist_rest(config, url, true).await?;
    tasks::json_to_task(json)
}

/// Add Task without natural language support but supports additional parameters
#[allow(clippy::too_many_arguments)]
pub async fn add_task(
    config: &Config,
    content: &String,
    project: &Project,
    section: Option<Section>,
    priority: Priority,
    description: &String,
    due: &Option<String>,
    labels: &[String],
) -> Result<Task, Error> {
    let project_id = project.id.clone();
    let url = String::from(TASKS_URL);
    let mut body: HashMap<String, Value> = HashMap::new();
    body.insert("content".to_owned(), Value::String(content.to_owned()));
    body.insert(
        "description".to_owned(),
        Value::String(description.to_owned()),
    );
    body.insert("project_id".to_owned(), Value::String(project_id));

    body.insert("auto_reminder".to_owned(), Value::Bool(true));
    body.insert(
        "priority".to_owned(),
        Value::Number(Number::from(priority.to_integer())),
    );
    let labels = labels.iter().map(|l| Value::String(l.to_owned())).collect();
    body.insert("labels".to_owned(), Value::Array(labels));

    if let Some(date) = due {
        if time::is_date(date) || time::is_datetime(date) {
            body.insert("due_date".to_owned(), Value::String(date.to_owned()));
        } else {
            body.insert("due_string".to_owned(), Value::String(date.to_owned()));
        }
    }

    if let Some(section) = section {
        body.insert("section_id".to_owned(), Value::String(section.id.clone()));
    }

    let body = json!(body);

    let json = request::post_todoist_rest(config, url, body, true).await?;
    tasks::json_to_task(json)
}

/// Get a vector of all tasks for a project
pub async fn tasks_for_project(config: &Config, project: &Project) -> Result<Vec<Task>, Error> {
    let project_id = project.id.clone();
    let url = format!("{TASKS_URL}?project_id={project_id}");
    let json = request::get_todoist_rest(config, url, true).await?;
    tasks::json_to_tasks(json)
}

pub async fn tasks_for_filters(
    config: &Config,
    filter: &str,
) -> Result<Vec<(String, Vec<Task>)>, Error> {
    let filters: Vec<_> = filter
        .split(',')
        .map(|f| tasks_for_filter(config, f))
        .collect();

    let mut acc = Vec::new();
    for result in future::join_all(filters).await {
        acc.push(result?);
    }

    Ok(acc)
}

pub async fn tasks_for_filter(config: &Config, filter: &str) -> Result<(String, Vec<Task>), Error> {
    let encoded = encode(filter);
    let url = format!("{TASKS_URL}filter?query={encoded}");
    let json = request::get_todoist_rest(config, url, true).await?;
    let tasks = tasks::json_to_tasks(json)?;
    Ok((filter.to_string(), tasks))
}

pub async fn sections_for_project(
    config: &Config,
    project: &Project,
) -> Result<Vec<Section>, Error> {
    let project_id = project.id.clone();
    let url = format!("{SECTIONS_URL}?project_id={project_id}");
    let json = request::get_todoist_rest(config, url, true).await?;
    sections::json_to_sections(json)
}

pub async fn projects(config: &Config) -> Result<Vec<Project>, Error> {
    let json = request::get_todoist_rest(config, PROJECTS_URL.to_string(), true).await?;
    projects::json_to_projects(json)
}

pub async fn list_labels(config: &Config, spinner: bool) -> Result<Vec<Label>, Error> {
    let json = request::get_todoist_rest(config, LABELS_URL.to_string(), spinner).await?;
    labels::json_to_labels(json)
}

/// Move an task to a different project
pub async fn move_task_to_project(
    config: &Config,
    task: Task,
    project: &Project,
    spinner: bool,
) -> Result<String, Error> {
    let project_id = get_legacy_id(config, Resource::Project, ID::V1(project.id.clone())).await?;
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": task.id, "project_id": project_id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body, spinner).await?;
    Ok(String::from("✓"))
}

pub async fn move_task_to_section(
    config: &Config,
    task: Task,
    section: &Section,
    spinner: bool,
) -> Result<String, Error> {
    let body = json!({"commands": [{"type": "item_move", "uuid": request::new_uuid(), "args": {"id": task.id, "section_id": section.id}}]});
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body, spinner).await?;
    Ok(String::from("✓"))
}

/// Update the priority of an task by ID
pub async fn update_task_priority(
    config: &Config,
    task: &Task,
    priority: &Priority,
    spinner: bool,
) -> Result<String, Error> {
    let body = json!({ "priority": priority });
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back an task
    Ok(String::from("✓"))
}

/// Add a label to task by ID
pub async fn add_task_label(
    config: &Config,
    task: Task,
    label: String,
    spinner: bool,
) -> Result<String, Error> {
    let mut labels = task.labels;
    labels.push(label);
    let body = json!({ "labels": labels});
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back an task
    Ok(String::from("✓"))
}

/// Update due date for task using natural language
pub async fn update_task_due_natural_language(
    config: &Config,
    task: Task,
    due_string: String,
    duration: Option<u32>,
    spinner: bool,
) -> Result<String, Error> {
    let due_string = if task.is_recurring() {
        format!("{} starting {due_string}", task.due.unwrap().string)
    } else {
        due_string
    };
    let body = if let Some(duration) = duration {
        json!({ "due_string": due_string, "duration": duration, "duration_unit": "minute" })
    } else {
        json!({ "due_string": due_string })
    };
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

/// Update the content of a task by ID
pub async fn update_task_content(
    config: &Config,
    task: &Task,
    content: String,
    spinner: bool,
) -> Result<String, Error> {
    let body = json!({ "content": content});
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

/// Update the description of a task by ID
pub async fn update_task_description(
    config: &Config,
    task: &Task,
    description: String,
    spinner: bool,
) -> Result<String, Error> {
    let body = json!({ "description": description});
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

/// Update the labels of a task by ID
/// Replaces the old labels
pub async fn update_task_labels(
    config: &Config,
    task: &Task,
    labels: Vec<String>,
    spinner: bool,
) -> Result<String, Error> {
    let body = json!({ "labels": labels});
    let url = format!("{}{}", TASKS_URL, task.id);

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

/// Complete the last task returned by "next task"
pub async fn complete_task(config: &Config, task: &Task, spinner: bool) -> Result<String, Error> {
    let body = if task.is_recurring() {
        json!({"commands": [{"type": "item_update_date_complete", "uuid": request::new_uuid(), "temp_id": request::new_uuid(), "args": {"id": task.id, "reset_subtasks": 1}}]})
    } else {
        json!({"commands": [{"type": "item_close", "uuid": request::new_uuid(), "temp_id": request::new_uuid(), "args": {"id": task.id}}]})
    };
    let url = String::from(SYNC_URL);

    request::post_todoist_sync(config, url, body, spinner).await?;

    if !cfg!(test) {
        config.reload().await?.clear_next_task().save().await?;
    }

    // Does not pass back a task
    Ok(String::from("✓"))
}

pub async fn delete_task(config: &Config, task: &Task, spinner: bool) -> Result<String, Error> {
    let body = json!({});
    let url = format!("{}{}", TASKS_URL, task.id);

    request::delete_todoist_rest(config, url, body, spinner).await?;
    Ok(String::from("✓"))
}

pub async fn delete_project(
    config: &Config,
    project: &Project,
    spinner: bool,
) -> Result<String, Error> {
    let url = format!("{}/{}", PROJECTS_URL, project.id);
    let body = json!({});

    request::delete_todoist_rest(config, url, body, spinner).await?;
    Ok(String::from("✓"))
}
pub async fn create_project(
    config: &Config,
    name: String,
    spinner: bool,
) -> Result<Project, Error> {
    let url = PROJECTS_URL.to_string();
    let body = json!({"name": name});

    let json = request::post_todoist_rest(config, url, body, spinner).await?;
    projects::json_to_project(json)
}

pub async fn comment_task(
    config: &Config,
    id: ID,
    content: String,
    spinner: bool,
) -> Result<String, Error> {
    let resource = Resource::Task;
    let id = get_legacy_id(config, resource, id).await?;
    let body = json!({"task_id": id, "content": content});
    let url = COMMENTS_URL.to_string();

    request::post_todoist_rest(config, url, body, spinner).await?;
    // Does not pass back a task
    Ok(String::from("✓"))
}

pub async fn get_user_data(config: &Config) -> Result<User, Error> {
    let url = SYNC_URL.to_string();
    let body = json!({"resource_types": ["user"], "sync_token": "*"});
    let json = request::post_todoist_sync(config, url, body, true).await?;
    sync_json_to_user(json)
}

pub async fn comments(config: &Config, task: &Task) -> Result<Vec<Comment>, Error> {
    let task_id = &task.id;
    let url = format!("{COMMENTS_URL}?task_id={task_id}");
    let json = request::get_todoist_rest(config, url, true).await?;
    rest_json_to_comments(json)
}

pub fn sync_json_to_user(json: String) -> Result<User, Error> {
    let sync_response: SyncResponse = serde_json::from_str(&json)?;
    Ok(sync_response.user)
}

pub fn rest_json_to_comments(json: String) -> Result<Vec<Comment>, Error> {
    let comments: Vec<Comment> = serde_json::from_str(&json)?;
    Ok(comments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::priority::{self, Priority};
    use crate::test;
    use crate::user::TzInfo;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_get_user_data() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::user())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        assert_eq!(
            get_user_data(&config).await,
            Ok(User {
                tz_info: TzInfo {
                    timezone: "America/Vancouver".to_string()
                }
            })
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_add_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/tasks/quick")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        assert_eq!(
            quick_add_task(&config, "testy test").await,
            Ok(test::fixtures::task())
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_add_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/tasks/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let project = test::fixtures::project();

        let priority = priority::Priority::None;
        let section = test::fixtures::section();
        assert_eq!(
            add_task(
                &config,
                &String::from("New task"),
                &project,
                Some(section),
                priority,
                &String::new(),
                &None,
                &[]
            )
            .await,
            Ok(test::fixtures::task())
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_comment_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/rest/v2/comments/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::comment())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        assert_eq!(
            comment_task(
                &config,
                ID::Legacy("123".to_string()),
                String::from("New comment"),
                true
            )
            .await,
            Ok(String::from("✓"))
        );
        mock.assert();
    }

    #[tokio::test]
    async fn should_get_tasks_for_project() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/api/v1/tasks/?project_id=123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());
        let config_with_timezone = config.with_timezone("US/Pacific");
        let binding = config_with_timezone.projects().await.unwrap();
        let project = binding.first().unwrap();

        assert_eq!(
            tasks_for_project(&config_with_timezone, project).await,
            Ok(vec![test::fixtures::task()])
        );

        mock.assert();
    }

    #[tokio::test]
    async fn should_complete_a_task() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let task = test::fixtures::task();
        let response = complete_task(&config, &task, false).await;
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[tokio::test]
    async fn should_move_a_task() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", "/sync/v9/sync")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sync())
            .create_async()
            .await;

        let mock2 = server
            .mock("GET", "/api/v1/id_mappings/projects/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::ids())
            .create_async()
            .await;

        let task = test::fixtures::task();
        let config = test::fixtures::config().await.with_mock_url(server.url());

        let binding = config.projects().await.unwrap();
        let project = binding.first().unwrap();
        let response = move_task_to_project(&config, task, project, false).await;

        assert_eq!(response, Ok(String::from("✓")));
        mock.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_delete_task() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("DELETE", "/api/v1/tasks/6Xqhv4cwxgjwG9w8")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let task = test::fixtures::task();
        let config = test::fixtures::config().await.with_mock_url(server.url());

        let response = delete_task(&config, &task, false).await;
        mock.assert();

        assert_eq!(response, Ok(String::from("✓")));
    }

    #[tokio::test]
    async fn test_get_task() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/api/v1/tasks/5149481867")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::task())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let response = get_task(&config, "5149481867").await.unwrap();
        mock.assert();

        assert_eq!(response.id, String::from("6Xqhv4cwxgjwG9w8"));
        assert_eq!(response.project_id, String::from("6VRRxv8CM6GVmmgf"));
    }

    #[tokio::test]
    async fn should_prioritize_a_task() {
        let task = test::fixtures::task();
        let url: &str = &format!("{}{}", "/api/v1/tasks/", task.id);
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_task().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let response = update_task_priority(&config, &task, &Priority::High, true).await;
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[tokio::test]
    async fn should_update_date_on_a_task() {
        let task = test::fixtures::task();
        let url: &str = &format!("{}{}", "/api/v1/tasks/", task.id);
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", url)
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body(test::responses::today_tasks_response().await)
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());

        let response =
            update_task_due_natural_language(&config, task, "today".to_string(), None, true).await;
        mock.assert();
        assert_eq!(response, Ok(String::from("✓")));
    }

    #[tokio::test]
    async fn should_get_legacy_id() {
        let task = test::fixtures::task();
        let url: &str = &format!("{}{}", "/api/v1/id_mappings/tasks/", task.id);
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", url)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::ids())
            .create_async()
            .await;

        let config = test::fixtures::config().await.with_mock_url(server.url());
        let resource = Resource::Task;

        // Makes the request when converting a new ID to old
        let response = get_legacy_id(&config, resource.clone(), ID::V1(task.id.clone())).await;
        mock.assert();
        assert_eq!(response, Ok(String::from("6V2J6Qhgq47phxHG")));

        // Makes no request when passed an old ID
        let response = get_legacy_id(&config, resource, ID::Legacy(task.id)).await;
        mock.expect(0);
        assert_eq!(response, Ok(String::from("6Xqhv4cwxgjwG9w8")));
    }
}
