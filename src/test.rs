#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use crate::{
        config::{self, Config},
        items::{DateInfo, Item},
    };

    pub fn item() -> Item {
        Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: None,
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2061-11-13"),
                is_recurring: false,
                timezone: Some(String::from("America/Los_Angeles")),
            }),
            priority: crate::items::priority::Priority::Medium,
            is_deleted: None,
            is_completed: None,
        }
    }

    pub fn config() -> Config {
        Config {
            token: String::from("alreadycreated"),
            projects: HashMap::new(),
            path: config::generate_path().unwrap(),
            next_id: None,
            timezone: Some(String::from("US/Pacific")),
            last_version_check: None,
            mock_url: None,
            mock_string: None,
            mock_select: None,
            spinners: Some(true),
        }
    }
}
#[cfg(test)]
pub mod responses {
    use crate::test::fixtures;
    use crate::{time, VERSION};

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

    pub fn items() -> String {
        format!(
            "{{\
        \"items\":\
            [
                {{\
                \"added_by_uid\":44444444,\
                \"assigned_by_uid\":null,\
                \"checked\":false,\
                \"child_order\":-5,\
                \"collapsed\":false,\
                \"content\":\"Put out recycling\",\
                \"date_added\":\"2021-06-15T13:01:28Z\",\
                \"date_completed\":null,\
                \"description\":\"\",\
                \"due\":{{\
                \"date\":\"{}T23:59:00Z\",\
                \"is_recurring\":true,\
                \"lang\":\"en\",\
                \"string\":\"every other mon at 16:30\",\
                \"timezone\":null}},\
                \"id\":\"999999\",\
                \"is_deleted\":false,\
                \"labels\":[],\
                \"note_count\":0,\
                \"parent_id\":null,\
                \"priority\":3,\
                \"project_id\":22222222,\
                \"responsible_uid\":null,\
                \"section_id\":333333333,\
                \"sync_id\":null,\
                \"user_id\":111111111\
                }}
            ]
        }}",
            time::today_string(&fixtures::config())
        )
    }

    pub fn item() -> String {
        String::from(
            "\
        {\"added_by_uid\":635166,\
        \"assigned_by_uid\":null,\
        \"checked\":false,\
        \"child_order\":2,\
        \"collapsed\":false,\
        \"content\":\"testy test\",\
        \"date_added\":\"2021-09-12T19:11:07Z\",\
        \"date_completed\":null,\
        \"description\":\"\",\
        \"due\":null,\
        \"id\":\"5149481867\",\
        \"is_deleted\":false,\
        \"labels\":[],\
        \"legacy_project_id\":333333333,\
        \"parent_id\":null,\
        \"priority\":1,\
        \"project_id\":5555555,\
        \"reminder\":null,\
        \"responsible_uid\":null,\
        \"section_id\":null,\
        \"sync_id\":null,\
        \"user_id\":111111\
    }",
        )
    }
    pub fn sections() -> String {
        String::from(
            "[
              {
              \"id\": \"1234\",
              \"project_id\": \"5678\",
              \"order\": 1,
              \"name\": \"Bread\"
              },
              {
              \"id\": \"9012\",
              \"project_id\": \"3456\",
              \"order\": 2,
              \"name\": \"Meat\"
              }
            ]
            ",
        )
    }

    pub fn projects() -> String {
        String::from(
            "[
              {
              \"id\": \"1234\",
              \"project_id\": \"5678\",
              \"order\": 1,
              \"comment_count\": 1,
              \"is_shared\": false,
              \"is_favorite\": false,
              \"is_inbox_project\": false,
              \"is_team_inbox\": false,
              \"color\": \"blue\",
              \"view_style\": \"list\",
              \"url\": \"http://www.example.com/\",
              \"name\": \"Doomsday\"
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
