use serde::Deserialize;

// Projects are split into sections
#[derive(Deserialize, Clone)]
pub struct Section {
    pub id: String,
    pub project_id: String,
    pub order: u8,
    pub name: String,
}

pub fn json_to_sections(json: String) -> Result<Vec<Section>, String> {
    let result: Result<Vec<Section>, _> = serde_json::from_str(&json);
    match result {
        Ok(body) => Ok(body),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
}
