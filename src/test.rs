#[cfg(test)]
pub mod fixtures {

    use crate::comments::Comment;
    use crate::config::Config;
    use crate::errors::Error;
    use crate::labels::Label;
    use crate::projects::Project;
    use crate::sections::Section;
    use crate::tasks::priority::Priority;
    use crate::tasks::{DateInfo, Deadline, Duration, Task, Unit};
    use crate::time::{self, FORMAT_DATE};
    use chrono::Duration as ChronoDuration;

    pub fn label() -> Label {
        Label {
            id: "123".to_string(),
            name: "345".to_string(),
            color: "red".to_string(),
            order: None,
            is_favorite: false,
        }
    }
    async fn adjusted_date(days: i64) -> String {
        let config = config().await.with_timezone("America/Vancouver");
        let tomorrow = time::now(&config).unwrap() + ChronoDuration::days(days);
        tomorrow.format(FORMAT_DATE).to_string()
    }

    pub async fn today_task() -> Task {
        let date = adjusted_date(0).await;
        Task {
            id: String::from("6Xqhv4cwxgjwG9w8"),
            section_id: None,
            added_by_uid: Some("633166".to_string()),
            added_at: Some(format!("{}T22:29:34.404051Z", date)),
            child_order: 1,
            day_order: -1,
            responsible_uid: None,
            assigned_by_uid: None,
            updated_at: Some(format!("{}T22:32:46.415849Z", date)),
            deadline: Some(Deadline {
                lang: "en".to_string(),
                date: date.clone(),
            }),
            completed_at: None,
            is_collapsed: false,
            user_id: String::from("910"),
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
                date: format!("{}T12:00:00Z", date),
                lang: String::from("en"),
                is_recurring: false,
                timezone: Some(String::from("America/Vancouver")),
                string: format!("{} 15:00", date),
            }),
            priority: Priority::Medium,
            is_deleted: false,
        }
    }

    pub async fn task(days_in_future: i64) -> Task {
        let date = adjusted_date(days_in_future).await;
        Task {
            id: String::from("6Xqhv4cwxgjwG9w8"),
            section_id: None,
            added_by_uid: Some("633166".to_string()),
            added_at: Some(format!("{}T22:29:34.404051Z", date)),
            child_order: 1,
            day_order: -1,
            responsible_uid: None,
            assigned_by_uid: None,
            updated_at: Some(format!("{}T22:32:46.415849Z", date)),
            deadline: Some(Deadline {
                lang: "en".to_string(),
                date: date.clone(),
            }),
            completed_at: None,
            is_collapsed: false,
            user_id: String::from("910"),
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
                date: format!("{}T12:00:00Z", date),
                lang: String::from("en"),
                is_recurring: false,
                timezone: Some(String::from("America/Vancouver")),
                string: format!("{} 15:00", date),
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
            inbox_project: None,
            is_collapsed: false,
            is_shared: false,
        }
    }

    pub fn section() -> Section {
        Section {
            id: "1234".to_string(),
            added_at: "2020-06-11T14:51:08.056500Z".to_string(),
            user_id: "910".to_string(),
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

    pub fn comment() -> Comment {
        Comment {
            id: "2992679862".to_string(),
            posted_uid: None,
            content: "Need one bottle of milk".to_string(),
            uids_to_notify: None,
            posted_at: "2016-09-22T07:00:00.000000Z".to_string(),
            reactions: None,
            item_id: "123".to_string(),
            is_deleted: false,
            file_attachment: None,
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

    pub fn label() -> String {
        String::from(
            "{
                \"id\": \"123\",
                \"name\": \"345\",
                \"is_favorite\": false,
                \"order\": null,
                \"color\": \"red\"
            }",
        )
    }

    pub fn labels_response() -> String {
        format!(
            "
            {{\"results\":
                
                [
                    {}
                ],
                \"next_cursor\": null}}
        ",
            label()
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

    pub async fn today_task() -> String {
        let date = today_date().await;
        format!(
            "\
                {{
                        \"user_id\": \"910\",
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
                        \"added_at\": \"{}T22:29:34.404051Z\",
                        \"completed_at\": null,
                        \"updated_at\": \"{}T22:32:46.415849Z\",
                        \"due\": {{
                                \"date\": \"{}T12:00:00Z\",
                                \"timezone\": \"America/Vancouver\",
                                \"string\": \"{} 15:00\",
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
                }}",
            date, date, date, date, date
        )
    }

    pub async fn task_without_duration() -> String {
        let date = today_date().await;
        format!(
            "
                    {{\
                        \"user_id\": \"910\",
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
            date
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
                    \"user_id\":\"910\"\
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
    \"item_id\": \"123\",
    \"is_deleted\": false,
    \"attachment\": {
        \"file_name\": \"File.pdf\",
        \"file_type\": \"application/pdf\",
        \"file_url\": \"https://s3.amazonaws.com/domorebetter/Todoist+Setup+Guide.pdf\",
        \"resource_type\": \"file\"
        }
        }",
        )
    }

    pub fn comments_response() -> String {
        format!(
            "{{
                \"results\": [{}],
                \"next_cursor\": null
            }}",
            comment()
        )
    }

    pub fn user() -> String {
        String::from(
            "\
            {
                \"activated_user\":true,
                \"auto_reminder\":0,
                \"business_account_id\":null,
                \"completed_count\":36169,
                \"completed_today\":42,
                \"daily_goal\":20,
                \"date_format\":0,
                \"days_off\":[],
                \"deleted_at\":null,
                \"email\":\"me@gmail.com\",
                \"feature_identifier\":\"635166_f037865dbe43759d0a8401f917a93fa344948a84d28774c858f3da12efd337c6\",
                \"features\": {
                    \"beta\":1,
                    \"dateist_inline_disabled\":false,
                    \"dateist_lang\":null,
                    \"global.teams\":true,
                    \"gold_theme\":true,
                    \"has_push_reminders\":true,
                    \"karma_disabled\":false,
                    \"karma_vacation\":false,
                    \"kisa_consent_timestamp\":null,
                    \"restriction\":3
                },
                \"full_name\":\"This Guy\",
                \"has_magic_number\":true,
                \"has_password\":true,
                \"has_started_a_trial\":false,
                \"id\":\"111111\",
                \"image_id\":null,
                \"inbox_project_id\":\"222222\",
                \"is_celebrations_enabled\":false,
                \"is_deleted\":false,
                \"is_premium\":true,
                \"joinable_workspace\":null,
                \"joined_at\":\"2013-07-01T05:22:21.000000Z\",
                \"karma\":58121.0,
                \"karma_trend\":\"up\",
                \"lang\":\"en\",
                \"mfa_enabled\":false,
                \"next_week\":1,
                \"onboarding_level\":null,
                \"onboarding_role\":null,
                \"onboarding_team_mode\":null,
                \"onboarding_use_cases\":null,
                \"premium_status\":\"current_personal_plan\",
                \"premium_until\":\"2026-02-22T03:47:31.000000Z\",
                \"shard_id\":1,
                \"share_limit\":5,
                \"sort_order\":0,
                \"start_day\":1,
                \"start_page\":\"filter?id=2297647060\",
                \"theme_id\":\"11\",
                \"time_format\":0,
                \"token\":\"a5c4e1bc54e1c79aca0c7b8bf57c4ed2b99ba608\",
                \"tz_info\":{
                    \"gmt_string\":\"-07:00\",
                    \"hours\":-7,
                    \"is_dst\":1,
                    \"minutes\":0,
                    \"timezone\":\"America/Vancouver\"
                },
                \"unique_prefix\":1,
                \"verification_status\":\"legacy\",
                \"websocket_url\":\"wss://ws.todoist.com/ws?token=MTcxMDk0OTNS7SW7dE-1ltDRhxBc0iXJ\",
                \"weekend_start_day\":6,
                \"weekly_goal\":50
            }",
        )
    }
    pub fn section() -> String {
        String::from(
            "
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
            
            ",
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

    async fn today_date() -> String {
        let config = fixtures::config().await.with_timezone("America/Vancouver");
        time::today_string(&config).unwrap()
    }
}
