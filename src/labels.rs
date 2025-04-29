use std::fmt::Display;

use crate::{config::Config, error::Error, todoist};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub order: Option<u32>,
    pub is_favorite: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LabelResponse {
    pub results: Vec<Label>,
    pub next_cursor: Option<String>,
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.clone();
        write!(f, "{name}")
    }
}
pub async fn get_labels(config: &Config, spinner: bool) -> Result<Vec<Label>, Error> {
    todoist::list_labels(config, spinner).await
}

pub fn json_to_labels(json: String) -> Result<Vec<Label>, Error> {
    let response: LabelResponse = serde_json::from_str(&json)?;
    Ok(response.results)
}
