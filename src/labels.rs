use crate::{config::Config, error::Error, todoist};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Label {
    pub name: String,
}
pub async fn get_labels(config: &Config) -> Result<Vec<Label>, Error> {
    todoist::labels(config).await
}

pub fn json_to_labels(json: String) -> Result<Vec<Label>, Error> {
    let labels: Vec<Label> = serde_json::from_str(&json)?;
    Ok(labels)
}
