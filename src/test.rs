#[cfg(test)]
pub mod helpers {
    /// Checks if environment supports colored output (GitHub Actions does not)
    pub fn is_colored() -> bool {
        colored::control::SHOULD_COLORIZE.should_colorize()
    }
}
#[cfg(test)]
pub mod responses {
    use crate::time;

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
                \"checked\":0,\
                \"child_order\":-5,\
                \"collapsed\":0,\
                \"content\":\"Put out recycling\",\
                \"date_added\":\"2021-06-15T13:01:28Z\",\
                \"date_completed\":null,\
                \"description\":\"\",\
                \"due\":{{\
                \"date\":\"{}T13:01:28Z\",\
                \"is_recurring\":true,\
                \"lang\":\"en\",\
                \"string\":\"every other mon at 16:30\",\
                \"timezone\":null}},\
                \"id\":999999,\
                \"in_history\":0,\"is_deleted\":0,\
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
            time::today_string()
        )
    }

    pub fn item() -> String {
        String::from(
            "\
        {\"added_by_uid\":635166,\
        \"assigned_by_uid\":null,\
        \"checked\":0,\
        \"child_order\":2,\
        \"collapsed\":0,\
        \"content\":\"testy test\",\
        \"date_added\":\"2021-09-12T19:11:07Z\",\
        \"date_completed\":null,\
        \"description\":\"\",\
        \"due\":null,\
        \"id\":5149481867,\
        \"in_history\":0,\
        \"is_deleted\":0,\
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
}
