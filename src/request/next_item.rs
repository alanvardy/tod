use chrono::offset::Utc;
use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use chrono_tz::US::Pacific;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: u64,
    pub content: String,
    checked: u8,
    description: String,
    due: Option<DateInfo>,
    priority: u8,
    is_deleted: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DateInfo {
    date: String,
    is_recurring: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    items: Vec<Item>,
}

/// Given a json response, return the next item
pub fn determine_next_item(text: String) -> Option<Item> {
    println!("{}", text);
    let body: Body = serde_json::from_str(&text).unwrap();
    let mut items = body.items;
    items.sort_by_key(|b| Reverse(item_value(b)));

    items.first().map(|item| item.to_owned())
}

/// Determines the numeric value of an item for sorting
fn item_value(item: &Item) -> u32 {
    let date_value: u8 = determine_date_value(item);
    let priority_value: u8 = determine_priority_value(item);

    let value = date_value as u32 + priority_value as u32;
    println!("{}", value);

    date_value as u32 + priority_value as u32
}

/// Return the value of the due field
fn determine_date_value(item: &Item) -> u8 {
    match &item.due {
        // Date "2021-09-06"
        Some(DateInfo { date, is_recurring }) if date.len() == 10 => {
            let today = Utc::now()
                .with_timezone(&Pacific)
                .format("%Y-%m-%d")
                .to_string();

            let today_value = if *date == today { 100 } else { 0 };
            let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
            today_value + recurring_value
        }
        // DateTime "2021-09-06T16:00:00(Z)"
        Some(DateInfo { date, is_recurring }) => {
            let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
            let parse_string = match date.len() {
                19 => "%Y-%m-%dT%H:%M:%S",
                _ => "%Y-%m-%dT%H:%M:%SZ",
            };
            let dt: DateTime<Tz> = Pacific
                .datetime_from_str(date, parse_string)
                .expect("could not parse DateTime");

            let now = &Utc::now().with_timezone(&Pacific);

            let duration = dt - now.to_owned();
            match duration.num_minutes() {
                num if (-15..=15).contains(&num) => 200 + recurring_value,
                _ => recurring_value,
            }
        }
        None => 40,
    }
}

fn determine_priority_value(item: &Item) -> u8 {
    match &item.priority {
        2 => 1,
        3 => 3,
        4 => 4,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_next_item_should_return_an_item() {
        let json = String::from(
            "\
        {\"items\":\
            [\
                {\
                    \"added_by_uid\":635166,\
                    \"assigned_by_uid\":null,\
                    \"checked\":0,\
                    \"child_order\":0,\
                    \"collapsed\":0,\
                    \"content\":\"Find the continuum transfunctioner\",\
                    \"date_added\":\"2021-02-27T19:41:56Z\",\
                    \"date_completed\":null,\
                    \"description\":\"\",\
                    \"due\":\
                    {\
                        \"date\":\"2021-11-13\",\
                        \"is_recurring\":false,\
                        \"lang\":\"en\",\
                        \"string\":\"every 12 weeks\",\
                        \"timezone\":null\
                    },\
                    \"id\":222,\
                    \"in_history\":0,\
                    \"is_deleted\":0,\
                    \"labels\":[],\
                    \"note_count\":0,\
                    \"parent_id\":null,\
                    \"priority\":1,\
                    \"project_id\":11111111,\
                    \"responsible_uid\":null,\
                    \"section_id\":22222222,\
                    \"sync_id\":null,\
                    \"user_id\":3333333\
                },\
                {\
                    \"added_by_uid\":635166,\
                    \"assigned_by_uid\":null,\
                    \"checked\":0,\
                    \"child_order\":0,\
                    \"collapsed\":0,\
                    \"content\":\"Get gifts for the twins\",\
                    \"date_added\":\"2021-02-27T19:41:56Z\",\
                    \"date_completed\":null,\
                    \"description\":\"\",\
                    \"due\":\
                    {\
                        \"date\":\"2021-11-13\",\
                        \"is_recurring\":false,\
                        \"lang\":\"en\",\
                        \"string\":\"every 12 weeks\",\
                        \"timezone\":null\
                    },\
                    \"id\":222,\
                    \"in_history\":0,\
                    \"is_deleted\":0,\
                    \"labels\":[],\
                    \"note_count\":0,\
                    \"parent_id\":null,\
                    \"priority\":3,\
                    \"project_id\":11111111,\
                    \"responsible_uid\":null,\
                    \"section_id\":22222222,\
                    \"sync_id\":null,\
                    \"user_id\":3333333\
                }\
            ]\
        }",
        );
        assert_eq!(
            determine_next_item(json).unwrap(),
            Item {
                id: 222,
                content: String::from("Get gifts for the twins"),
                checked: 0,
                description: String::from(""),
                due: Some(DateInfo {
                    date: String::from("2021-11-13"),
                    is_recurring: false
                }),
                priority: 3,
                is_deleted: 0,
            }
        );
    }

    #[test]
    fn determine_date_value_can_handle_date() {
        let item = Item {
            id: 222,
            content: String::from("Get gifts for the twins"),
            checked: 0,
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2021-11-13"),
                is_recurring: false,
            }),
            priority: 3,
            is_deleted: 0,
        };

        assert_eq!(determine_date_value(&item), 50)
    }

    #[test]
    fn determine_date_value_can_handle_datetime() {
        let item = Item {
            id: 222,
            content: String::from("Find car"),
            checked: 0,
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2021-02-27T19:41:56Z"),
                is_recurring: false,
            }),
            priority: 3,
            is_deleted: 0,
        };

        assert_eq!(determine_date_value(&item), 50)
    }
}
