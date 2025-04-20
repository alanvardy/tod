use serde::Deserialize;
use std::fmt::Display;

use crate::error::Error;

#[allow(dead_code)]
pub enum ID {
    Legacy(String),
    V1(String),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Resource {
    Section,
    Task,
    Comment,
    Project,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Id {
    pub old_id: String,
    pub new_id: String,
}
pub fn json_to_ids(json: String) -> Result<Vec<Id>, Error> {
    let ids: Vec<Id> = serde_json::from_str(&json)?;
    Ok(ids)
}
impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Resource::Section => "sections",
            Resource::Task => "tasks",
            Resource::Comment => "comments",
            Resource::Project => "projects",
        };
        write!(f, "{name}")
    }
}
