use serde::Deserialize;
use std::fmt::Display;

use crate::errors::Error;

#[derive(Clone)]
pub enum Resource {
    Project,
}

#[derive(Deserialize)]
pub struct Id {
    pub new_id: String,
}
pub fn json_to_ids(json: String) -> Result<Vec<Id>, Error> {
    let ids: Vec<Id> = serde_json::from_str(&json)?;
    Ok(ids)
}
impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Resource::Project => "projects",
        };
        write!(f, "{name}")
    }
}
