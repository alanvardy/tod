use std::fmt::Display;

use crate::{config::Config, error::Error, todoist};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Label {
    pub name: String,
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.clone();
        write!(f, "{name}")
    }
}
pub async fn get_labels(config: &Config, spinner: bool) -> Result<Vec<Label>, Error> {
    todoist::labels(config, spinner).await
}

pub fn json_to_labels(json: String) -> Result<Vec<Label>, Error> {
    let labels: Vec<Label> = serde_json::from_str(&json)?;
    Ok(labels)
}
