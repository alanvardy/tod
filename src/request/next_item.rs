use chrono::offset::Utc;
use chrono_tz::US::Pacific;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    id: u64,
    checked: u8,
    content: String,
    due: Option<DateInfo>,
    priority: u8,
    is_deleted: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct DateInfo {
    date: String,
    is_recurring: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    items: Vec<Item>,
}

pub fn print(text: String) {
    let body: Body = serde_json::from_str(&text).unwrap();
    let mut items = body.items;
    items.sort_by_key(|b| Reverse(item_value(b)));
    match items.first() {
        Some(item) => println!("{}: {}", item.id, item.content),
        None => print!("No items on list"),
    }
}

fn item_value(item: &Item) -> u32 {
    let date_value: u8 = set_date_value(item);
    let priority_value: u8 = set_priority_value(item);

    date_value as u32 + priority_value as u32
}

fn set_date_value(item: &Item) -> u8 {
    let today = Utc::now()
        .with_timezone(&Pacific)
        .format("%Y-%m-%d")
        .to_string();

    match &item.due {
        Some(DateInfo {
            date,
            is_recurring: false,
        }) => {
            if *date == today {
                100
            } else {
                0
            }
        }
        None => 40,
        Some(DateInfo {
            date,
            is_recurring: true,
        }) => {
            if *date == today {
                50
            } else {
                0
            }
        }
    }
}

fn set_priority_value(item: &Item) -> u8 {
    match &item.priority {
        2 => 1,
        3 => 3,
        4 => 4,
        _ => 2,
    }
}
