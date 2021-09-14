use colored::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fmt;

use crate::config::Config;
use crate::{config, items, request, time};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: u64,
    pub content: String,
    pub priority: u8,
    pub checked: u8,
    pub description: String,
    pub due: Option<DateInfo>,
    pub is_deleted: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DateInfo {
    pub date: String,
    pub is_recurring: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    items: Vec<Item>,
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let content = match self.priority {
            2 => self.content.blue(),
            3 => self.content.yellow(),
            4 => self.content.red(),
            _ => self.content.normal(),
        };

        let description = match &*self.description {
            "" => String::from(""),
            _ => format!("\n{}", self.description),
        };

        let due = match &self.due {
            Some(DateInfo {
                date,
                is_recurring: _,
            }) => format!("\nDue: {}", time::maybe_today(date)),

            None => String::from(""),
        };

        write!(formatter, "\n{}{}{}", content, description, due)
    }
}

impl Item {
    /// Determines the numeric value of an item for sorting
    fn value(&self) -> u32 {
        let date_value: u8 = self.date_value();
        let priority_value: u8 = self.priority_value();

        date_value as u32 + priority_value as u32
    }

    /// Return the value of the due field
    fn date_value(&self) -> u8 {
        match &self.due {
            // Date "2021-09-06"
            Some(DateInfo { date, is_recurring }) if date.len() == 10 => {
                let today_value = if *date == time::today() { 100 } else { 0 };
                let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
                today_value + recurring_value
            }
            // DateTime "2021-09-06T16:00:00(Z)"
            Some(DateInfo { date, is_recurring }) => {
                let recurring_value = if is_recurring.to_owned() { 0 } else { 50 };
                let dt = time::datetime_from_str(date);

                let duration = dt - time::now();
                match duration.num_minutes() {
                    -15..=15 => 200 + recurring_value,
                    _ => recurring_value,
                }
            }
            None => 80,
        }
    }

    fn priority_value(&self) -> u8 {
        match self.priority {
            2 => 1,
            3 => 3,
            4 => 4,
            _ => 2,
        }
    }
}

pub fn from_json(json: String) -> Vec<Item> {
    let body: Body = serde_json::from_str(&json).expect("Could not parse items from JSON response");
    body.items
}

pub fn sort_by_priority(mut items: Vec<Item>) -> Vec<Item> {
    items.sort_by_key(|b| Reverse(b.value()));
    items
}

pub fn set_priority(config: Config, item: items::Item) {
    println!("{}", item);

    let priority = config::get_input("Choose a priority from 1 (lowest) to 3 (highest):");

    match priority.as_str() {
        "1" => {
            let config = config.set_next_id(item.id);
            request::update_item_priority(config, item, 2).expect("could not set priority");
        }
        "2" => {
            let config = config.set_next_id(item.id);
            request::update_item_priority(config, item, 3).expect("could not set priority");
        }
        "3" => {
            let config = config.set_next_id(item.id);
            request::update_item_priority(config, item, 4).expect("could not set priority");
        }
        _ => println!("Not a valid input, please enter 1, 2, or 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn determine_next_item_should_return_an_item() {
    //     let json = String::from(
    //         "\
    //     {\"items\":\
    //         [\
    //             {\
    //                 \"added_by_uid\":635166,\
    //                 \"assigned_by_uid\":null,\
    //                 \"checked\":0,\
    //                 \"child_order\":0,\
    //                 \"collapsed\":0,\
    //                 \"content\":\"Find the continuum transfunctioner\",\
    //                 \"date_added\":\"2021-02-27T19:41:56Z\",\
    //                 \"date_completed\":null,\
    //                 \"description\":\"\",\
    //                 \"due\":\
    //                 {\
    //                     \"date\":\"2021-11-13\",\
    //                     \"is_recurring\":false,\
    //                     \"lang\":\"en\",\
    //                     \"string\":\"every 12 weeks\",\
    //                     \"timezone\":null\
    //                 },\
    //                 \"id\":222,\
    //                 \"in_history\":0,\
    //                 \"is_deleted\":0,\
    //                 \"labels\":[],\
    //                 \"note_count\":0,\
    //                 \"parent_id\":null,\
    //                 \"priority\":1,\
    //                 \"project_id\":11111111,\
    //                 \"responsible_uid\":null,\
    //                 \"section_id\":22222222,\
    //                 \"sync_id\":null,\
    //                 \"user_id\":3333333\
    //             },\
    //             {\
    //                 \"added_by_uid\":635166,\
    //                 \"assigned_by_uid\":null,\
    //                 \"checked\":0,\
    //                 \"child_order\":0,\
    //                 \"collapsed\":0,\
    //                 \"content\":\"Get gifts for the twins\",\
    //                 \"date_added\":\"2021-02-27T19:41:56Z\",\
    //                 \"date_completed\":null,\
    //                 \"description\":\"\",\
    //                 \"due\":\
    //                 {\
    //                     \"date\":\"2021-11-13\",\
    //                     \"is_recurring\":false,\
    //                     \"lang\":\"en\",\
    //                     \"string\":\"every 12 weeks\",\
    //                     \"timezone\":null\
    //                 },\
    //                 \"id\":222,\
    //                 \"in_history\":0,\
    //                 \"is_deleted\":0,\
    //                 \"labels\":[],\
    //                 \"note_count\":0,\
    //                 \"parent_id\":null,\
    //                 \"priority\":3,\
    //                 \"project_id\":11111111,\
    //                 \"responsible_uid\":null,\
    //                 \"section_id\":22222222,\
    //                 \"sync_id\":null,\
    //                 \"user_id\":3333333\
    //             }\
    //         ]\
    //     }",
    //     );
    //     assert_eq!(
    //         determine_next_item(json).unwrap(),
    //         Item {
    //             id: 222,
    //             content: String::from("Get gifts for the twins"),
    //             checked: 0,
    //             description: String::from(""),
    //             due: Some(DateInfo {
    //                 date: String::from("2021-11-13"),
    //                 is_recurring: false
    //             }),
    //             priority: 3,
    //             is_deleted: 0,
    //         }
    //     );
    // }

    #[test]
    fn date_value_can_handle_date() {
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

        // On another day
        assert_eq!(item.date_value(), 50);

        // Recurring
        let item = Item {
            due: Some(DateInfo {
                date: String::from("2021-11-13"),
                is_recurring: true,
            }),
            ..item
        };
        assert_eq!(item.date_value(), 0);

        // No date
        let item = Item { due: None, ..item };
        assert_eq!(item.date_value(), 80);
    }

    #[test]
    fn date_value_can_handle_datetime() {
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

        assert_eq!(item.date_value(), 50);
    }

    #[test]
    fn can_format_item_with_a_date() {
        let item = Item {
            id: 222,
            content: String::from("Get gifts for the twins"),
            checked: 0,
            description: String::from(""),
            due: Some(DateInfo {
                date: String::from("2021-08-13"),
                is_recurring: false,
            }),
            priority: 3,
            is_deleted: 0,
        };

        let output = "\n\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: 2021-08-13";

        // CI has color turned off by default
        control::set_override(true);
        assert_eq!(format!("{}", item), output);
        control::unset_override();
    }

    #[test]
    fn can_format_item_with_today() {
        let item = Item {
            id: 222,
            content: String::from("Get gifts for the twins"),
            checked: 0,
            description: String::from(""),
            due: Some(DateInfo {
                date: time::today(),
                is_recurring: false,
            }),
            priority: 3,
            is_deleted: 0,
        };

        let output = "\n\u{1b}[33mGet gifts for the twins\u{1b}[0m\nDue: Today";

        // CI has color turned off by default
        control::set_override(true);
        assert_eq!(format!("{}", item), output);
        control::unset_override();
    }
}
