#[cfg(test)]
pub mod fixtures {

    use crate::config::Config;
    use crate::error::Error;
    use crate::projects::Project;
    use crate::sections::Section;
    use crate::tasks::priority::Priority;
    use crate::tasks::{DateInfo, Deadline, Duration, Task, Unit};

    pub fn task() -> Task {
        Task {
            id: String::from("6Xqhv4cwxgjwG9w8"),
            section_id: None,
            added_by_uid: Some("633166".to_string()),
            added_at: Some("2025-04-26T22:29:34.404051Z".to_string()),
            child_order: 1,
            day_order: -1,
            responsible_uid: None,
            assigned_by_uid: None,
            updated_at: Some("2025-04-26T22:32:46.415849Z".to_string()),
            deadline: Some(Deadline {
                lang: "en".to_string(),
                date: "2025-04-26".to_string(),
            }),
            completed_at: None,
            is_collapsed: false,
            user_id: String::from("635161"),
            content: String::from("TEST"),
            checked: false,
            duration: Some(Duration {
                amount: 15,
                unit: Unit::Minute,
            }),
            parent_id: None,
            note_count: 0,
            project_id: String::from("6VRRxv8CM6GVmmgf"),
            labels: vec![String::from("computer")],
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2025-04-26T22:00:00Z"),
                lang: String::from("en"),
                is_recurring: false,
                timezone: Some(String::from("America/Vancouver")),
                string: String::from("2025-04-26 15:00"),
            }),
            priority: Priority::Medium,
            is_deleted: false,
        }
    }

    pub async fn config() -> Config {
        let (tx, mut _rx) = tokio::sync::mpsc::unbounded_channel::<Error>();

        Config::new("alreadycreated", Some(tx))
            .await
            .expect("Could not generate directory")
            .with_projects(vec![project()])
    }

    pub fn project() -> Project {
        Project {
            id: "123".to_string(),
            can_assign_tasks: true,
            child_order: 0,
            color: "blue".to_string(),
            created_at: None,
            is_archived: false,
            is_deleted: false,
            is_favorite: false,
            is_frozen: false,
            name: "myproject".to_string(),
            updated_at: None,
            view_style: "List".to_string(),
            default_order: 0,
            description: "Something".to_string(),
            parent_id: None,
            inbox_project: false,
            is_collapsed: false,
            is_shared: false,
        }
    }

    pub fn section() -> Section {
        Section {
            id: "1234".to_string(),
            added_at: "2020-06-11T14:51:08.056500Z".to_string(),
            user_id: "1234".to_string(),
            project_id: "5678".to_string(),
            section_order: 1,
            name: "Bread".to_string(),
            updated_at: None,
            archived_at: None,
            is_archived: false,
            is_deleted: false,
            is_collapsed: false,
        }
    }
}
#[cfg(test)]
pub mod responses {
    use crate::test::fixtures;
    use crate::{VERSION, time};

    pub fn sync() -> String {
        String::from(
            "{
    \"creator_id\": \"2671355\",
    \"created_at\": \"2019-12-11T22:36:50.000000Z\",
    \"assignee_id\": \"2671362\",
    \"assigner_id\": \"2671355\",
    \"comment_count\": 10,
    \"is_completed\": false,
    \"content\": \"Buy Coffee\",
    \"description\": \"\",
    \"due\": {
        \"date\": \"2016-09-01\",
        \"is_recurring\": false,
        \"datetime\": \"2016-09-01T12:00:00.000000Z\",
        \"string\": \"tomorrow at 12\",
        \"timezone\": \"Europe/Moscow\"
    },
    \"duration\": {
        \"unit\": \"minute\",
        \"amount\": 42
    },
    \"id\": \"2995104339\",
    \"labels\": [\"Food\", \"Shopping\"],
    \"order\": 1,
    \"priority\": 1,
    \"project_id\": \"2203306141\",
    \"section_id\": \"7025\",
    \"parent_id\": \"2995104589\",
    \"url\": \"https://todoist.com/showTask?id=2995104339\"
}",
        )
    }

    pub async fn today_tasks_response() -> String {
        format!(
            "
            {{\"results\":
                
                [
                {}
                ],
                \"next_cursor\": null}}
        ",
            today_task().await
        )
    }

    pub async fn tasks_without_duration_response() -> String {
        format!(
            "
            {{\"results\":
                
                [
                {}
                ],
                \"next_cursor\": null}}
        ",
            task_without_duration().await
        )
    }
    pub async fn unscheduled_tasks_response() -> String {
        format!(
            "
            {{\"results\":
                
                [
                {}
                ],
                \"next_cursor\": null}}
        ",
            unscheduled_task().await
        )
    }

    pub fn task() -> String {
        String::from(
            "\
{
                        \"user_id\": \"635161\",
                        \"id\": \"6Xqhv4cwxgjwG9w8\",
                        \"project_id\": \"6VRRxv8CM6GVmmgf\",
                        \"section_id\": null,
                        \"parent_id\": null,
                        \"added_by_uid\": \"633166\",
                        \"assigned_by_uid\": null,
                        \"responsible_uid\": null,
                        \"labels\": [\"computer\"],
                        \"deadline\": {
                                \"date\": \"2025-04-26\",
                                \"lang\": \"en\"
                        },
                        \"duration\": {
                                \"amount\": 15,
                                \"unit\": \"minute\"
                        },
                        \"checked\": false,
                        \"is_deleted\": false,
                        \"added_at\": \"2025-04-26T22:29:34.404051Z\",
                        \"completed_at\": null,
                        \"updated_at\": \"2025-04-26T22:32:46.415849Z\",
                        \"due\": {
                                \"date\": \"2025-04-26T22:00:00Z\",
                                \"timezone\": \"America/Vancouver\",
                                \"string\": \"2025-04-26 15:00\",
                                \"lang\": \"en\",
                                \"is_recurring\": false
                        },
                        \"priority\": 3,
                        \"child_order\": 1,
                        \"content\": \"TEST\",
                        \"description\": \"\",
                        \"note_count\": 0,
                        \"day_order\": -1,
                        \"is_collapsed\": false
                }",
        )
    }

    pub async fn today_task() -> String {
        let config = fixtures::config().await.with_timezone("America/Vancouver");
        let date = time::today_string(&config).unwrap();

        format!(
            "
                    {{\
                        \"user_id\": \"635161\",
                        \"id\": \"6Xqhv4cwxgjwG9w8\",
                        \"project_id\": \"6VRRxv8CM6GVmmgf\",
                        \"section_id\": null,
                        \"parent_id\": null,
                        \"added_by_uid\": \"633166\",
                        \"assigned_by_uid\": null,
                        \"responsible_uid\": null,
                        \"labels\": [\"computer\"],
                        \"deadline\": {{
                                \"date\": \"{}\",
                                \"lang\": \"en\"
                        }},
                        \"duration\": {{
                                \"amount\": 15,
                                \"unit\": \"minute\"
                        }},
                        \"checked\": false,
                        \"is_deleted\": false,
                        \"added_at\": \"2025-04-26T22:29:34.404051Z\",
                        \"completed_at\": null,
                        \"updated_at\": \"2025-04-26T22:32:46.415849Z\",
                        \"due\": {{
                                \"date\": \"{}T22:00:00Z\",
                                \"timezone\": \"America/Vancouver\",
                                \"string\": \"2025-04-26 15:00\",
                                \"lang\": \"en\",
                                \"is_recurring\": false
                        }},
                        \"priority\": 3,
                        \"child_order\": 1,
                        \"content\": \"TEST\",
                        \"description\": \"\",
                        \"note_count\": 0,
                        \"day_order\": -1,
                        \"is_collapsed\": false
                    }}
        ",
            date, date
        )
    }

    pub async fn task_without_duration() -> String {
        format!(
            "
                    {{\
                        \"user_id\": \"635161\",
                        \"id\": \"6Xqhv4cwxgjwG9w8\",
                        \"project_id\": \"6VRRxv8CM6GVmmgf\",
                        \"section_id\": null,
                        \"parent_id\": null,
                        \"added_by_uid\": \"633166\",
                        \"assigned_by_uid\": null,
                        \"responsible_uid\": null,
                        \"labels\": [\"computer\"],
                        \"deadline\": {{
                                \"date\": \"2025-04-26\",
                                \"lang\": \"en\"
                        }},
                        \"duration\": null,
                        \"checked\": false,
                        \"is_deleted\": false,
                        \"added_at\": \"2025-04-26T22:29:34.404051Z\",
                        \"completed_at\": null,
                        \"updated_at\": \"2025-04-26T22:32:46.415849Z\",
                        \"due\": {{
                                \"date\": \"{}T22:00:00Z\",
                                \"timezone\": \"America/Vancouver\",
                                \"string\": \"2025-04-26 15:00\",
                                \"lang\": \"en\",
                                \"is_recurring\": false
                        }},
                        \"priority\": 3,
                        \"child_order\": 1,
                        \"content\": \"TEST\",
                        \"description\": \"\",
                        \"note_count\": 0,
                        \"day_order\": -1,
                        \"is_collapsed\": false
                    }}
        ",
            time::today_string(&fixtures::config().await).unwrap()
        )
    }

    pub async fn unscheduled_task() -> String {
        "
                    {\
                    \"added_by_uid\":\"44444444\",\
                    \"assigned_by_uid\":null,\
                    \"checked\":false,\
                    \"child_order\":-5,\
                    \"day_order\":-5,\
                    \"is_collapsed\":false,\
                    \"content\":\"Put out recycling\",\
                    \"date_added\":\"2021-06-15T13:01:28Z\",\
                    \"date_completed\":null,\
                    \"description\":\"\",\
                    \"due\": null,\
                    \"id\":\"999999\",\
                    \"is_deleted\":false,\
                    \"labels\":[],\
                    \"note_count\":0,\
                    \"parent_id\":null,\
                    \"priority\":3,\
                    \"project_id\":\"22222222\",\
                    \"responsible_uid\":null,\
                    \"section_id\":\"333333333\",\
                    \"sync_id\":null,\
                    \"user_id\":\"111111111\"\
                    }
        "
        .to_string()
    }

    pub fn comment() -> String {
        String::from(
            "{
    \"content\": \"Need one bottle of milk\",
    \"id\": \"2992679862\",
    \"posted_at\": \"2016-09-22T07:00:00.000000Z\",
    \"project_id\": null,
    \"task_id\": \"2995104339\",
    \"attachment\": {
        \"file_name\": \"File.pdf\",
        \"file_type\": \"application/pdf\",
        \"file_url\": \"https://s3.amazonaws.com/domorebetter/Todoist+Setup+Guide.pdf\",
        \"resource_type\": \"file\"
        }
        }",
        )
    }

    pub fn comments() -> String {
        String::from(
            "[{
    \"content\": \"Need one bottle of milk\",
    \"id\": \"2992679862\",
    \"posted_at\": \"2016-09-22T07:00:00.000000Z\",
    \"project_id\": null,
    \"task_id\": \"2995104339\",
    \"attachment\": {
        \"file_name\": \"File.pdf\",
        \"file_type\": \"application/pdf\",
        \"file_url\": \"https://s3.amazonaws.com/domorebetter/Todoist+Setup+Guide.pdf\",
        \"resource_type\": \"file\"
        }
        }]",
        )
    }

    pub fn user() -> String {
        String::from(
            "\
                {\"user\": {
                  \"tz_info\": {
                    \"timezone\": \"America/Vancouver\"
                  }
                }
            }",
        )
    }
    pub fn sections() -> String {
        String::from(
            "[
              {
              \"id\": \"1234\",
              \"project_id\": \"5678\",
              \"user_id\": \"910\",
              \"section_order\": 1,
              \"name\": \"Bread\",
              \"added_at\": \"2020-06-11T14:51:08.056500Z\",
              \"updated_at\": null,
              \"archived_at\": null,
              \"is_archived\": false,
              \"is_deleted\": false,
              \"is_collapsed\": false
              }
            ]
            ",
        )
    }
    pub fn sections_response() -> String {
        format!(
            "{{
              \"results\": {},
              \"next_cursor\": null
              }}
            ",
            sections()
        )
    }

    pub fn projects() -> String {
        String::from(
            "[
              {
              \"can_assign_tasks\": false,
              \"child_order\": 1,
              \"color\": \"blue\",
              \"created_at\": null,
              \"default_order\": 1,
              \"description\": \"Bad guy\",
              \"id\": \"123\",
              \"inbox_project\": false,
              \"is_archived\": false,
              \"is_collapsed\": false,
              \"is_deleted\": false,
              \"is_favorite\": false,
              \"is_frozen\": false,
              \"is_shared\": false,
              \"is_team_inbox\": false,
              \"name\": \"Doomsday\",
              \"parent_id\": \"5678\",
              \"updated_at\": null,
              \"view_style\": \"list\"
              
              }
            ]
            ",
        )
    }

    /// Has a new ID
    pub fn new_projects() -> String {
        String::from(
            "[
              {
              \"can_assign_tasks\": false,
              \"child_order\": 1,
              \"color\": \"blue\",
              \"created_at\": null,
              \"default_order\": 1,
              \"description\": \"Bad guy\",
              \"id\": \"890\",
              \"inbox_project\": false,
              \"is_archived\": false,
              \"is_collapsed\": false,
              \"is_deleted\": false,
              \"is_favorite\": false,
              \"is_frozen\": false,
              \"is_shared\": false,
              \"is_team_inbox\": false,
              \"name\": \"Doomsday\",
              \"parent_id\": \"5678\",
              \"updated_at\": null,
              \"view_style\": \"list\"
              
              }
            ]
            ",
        )
    }

    pub fn new_projects_response() -> String {
        format!(
            "{{
            \"results\":
                {},
                \"next_cursor\": null
            }}
            ",
            new_projects()
        )
    }

    pub fn projects_response() -> String {
        format!(
            "{{
            \"results\":
                {},
                \"next_cursor\": null
            }}
            ",
            projects()
        )
    }

    pub fn ids() -> String {
        String::from(
            "[
              {
              \"new_id\": \"7852696547\",
              \"old_id\": \"6V2J6Qhgq47phxHG\"
              }
            ]
            ",
        )
    }

    pub fn versions() -> String {
        format!(
            "{{\"versions\":[{{\
                \"audit_actions\":[{{\
                    \"action\":\"publish\",\
                    \"time\":\"2021-09-25T20:57:23.608723+00:00\",\
                    \"user\":{{\
                        \"avatar\":\"https://avatars.githubusercontent.com/u/38899847?v=4\",\
                        \"id\":105078,\
                        \"login\":\"alanvardy\",\
                        \"name\":\"Alan Vardy\",\
                        \"url\":\"https://github.com/alanvardy\"\
                    }}}}],\
                    \"crate\":\"tod\",\
                    \"crate_size\":22875,\
                    \"created_at\":\"2021-09-25T20:57:23.608723+00:00\",\
                    \"dl_path\":\"/api/v1/crates/tod/0.2.2/download\",\
                    \"downloads\":15,\
                    \"features\":{{}},\
                    \"id\":\"429968\",\
                    \"license\":\"MIT\",\
                    \"links\":{{\
                        \"authors\":\"/api/v1/crates/tod/0.2.2/authors\",\
                        \"dependencies\":\"/api/v1/crates/tod/0.2.2/dependencies\",\
                        \"version_downloads\":\"/api/v1/crates/tod/0.2.2/downloads\"\
                    }},\
                    \"num\":\"{}\",\
                    \"published_by\":{{\
                        \"avatar\":\"https://avatars.githubusercontent.com/u/38899847?v=4\",\
                        \"id\":\"105078\",\
                        \"login\":\"alanvardy\",\
                        \"name\":\"Alan Vardy\",\
                        \"url\":\"https://github.com/alanvardy\"\
                    }},\
                    \"readme_path\":\"/api/v1/crates/tod/0.2.2/readme\",\
                    \"updated_at\":\"2021-09-25T20:57:23.608723+00:00\",\
                    \"yanked\":false}},\
                    {{\"audit_actions\":[{{\
                        \"action\":\"publish\",\
                        \"time\":\"2021-09-20T16:16:21.682425+00:00\",\
                        \"user\":{{\
                            \"avatar\":\"https://avatars.githubusercontent.com/u/38899847?v=4\",\
                            \"id\":\"105078\",\
                            \"login\":\"alanvardy\",\
                            \"name\":\"Alan Vardy\",\
                            \"url\":\"https://github.com/alanvardy\"\
                        }}}}],\
                        \"crate\":\"tod\",\
                        \"crate_size\":21686,\
                        \"created_at\":\"2021-09-20T16:16:21.682425+00:00\",\
                        \"dl_path\":\"/api/v1/crates/tod/0.2.1/download\",\
                        \"downloads\":18,\
                        \"features\":{{}},\
                        \"id\":\"428020\",\
                        \"license\":\"MIT\",\
                        \"links\":{{\
                            \"authors\":\"/api/v1/crates/tod/0.2.1/authors\",\
                        \"dependencies\":\"/api/v1/crates/tod/0.2.1/dependencies\",\
                        \"version_downloads\":\"/api/v1/crates/tod/0.2.1/downloads\"\
                    }},\
                    \"num\":\"0.2.1\",\
                    \"published_by\":{{\
                        \"avatar\":\"https://avatars.githubusercontent.com/u/38899847?v=4\",\
                        \"id\":\"105078\",\
                        \"login\":\"alanvardy\",\
                        \"name\":\"Alan Vardy\",\
                        \"url\":\"https://github.com/alanvardy\"\
                    }},\
                    \"readme_path\":\"/api/v1/crates/tod/0.2.1/readme\",\
                    \"updated_at\":\"2021-09-20T16:16:21.682425+00:00\",\
                    \"yanked\":false}}]}}",
            VERSION
        )
    }
}
