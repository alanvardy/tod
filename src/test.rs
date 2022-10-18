#[cfg(test)]
pub mod helpers {
    use crate::config;
    use crate::config::Config;
    use crate::items::{DateInfo, Item};
    use std::collections::HashMap;

    /// Checks if environment supports colored output (GitHub Actions does not)
    pub fn supports_coloured_output() -> bool {
        colored::control::SHOULD_COLORIZE.should_colorize()
    }

    pub fn item_fixture() -> Item {
        Item {
            id: String::from("222"),
            content: String::from("Get gifts for the twins"),
            checked: false,
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2061-11-13"),
                is_recurring: false,
                timezone: Some(String::from("America/Los_Angeles")),
            }),
            priority: 3,
            is_deleted: false,
        }
    }

    pub fn config_fixture() -> Config {
        Config {
            token: String::from("alreadycreated"),
            projects: HashMap::new(),
            path: config::generate_path().unwrap(),
            next_id: None,
            last_version_check: None,
            timezone: Some(String::from("US/Pacific")),
        }
    }
}
#[cfg(test)]
pub mod responses {
    use crate::test::helpers;
    use crate::{time, VERSION};

    pub fn sync() -> String {
        String::from("\
        {\
            \"full_sync\":true,\
            \"sync_status\":\
            {\
                \"1111111-22222-417e-a919-b7c581bfcb0b\":\"ok\"\
            },\
                \"sync_token\":\"IGd2LFGherkilOWU0dv2y9w7AwgXQ_pyjuu-RVVaIWAVTD63UN3SzKmHdb4Myx8A0k7-aIjlAvbEUSCKONDJP7GIAXFgf_OgONSNM8bp_k3hJCmBqw\",\
                \"temp_id_mapping\":{}\
            }")
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
            time::today_string(&helpers::config_fixture())
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
