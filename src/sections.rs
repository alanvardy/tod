use crate::{config::Config, error::Error, todoist};
use futures::future;
use serde::Deserialize;

// Projects are split into sections
#[derive(PartialEq, Deserialize, Clone, Debug)]
pub struct Section {
    pub id: String,
    pub project_id: String,
    pub order: u8,
    pub name: String,
}

// Fetch all sections for all projects
pub async fn all_sections(config: &Config) -> Vec<Section> {
    let projects = config.projects.clone().unwrap_or_default();

    let mut handles = Vec::new();
    for project in projects.iter() {
        let handle = todoist::sections_for_project(config, project);

        handles.push(handle);
    }

    let var = future::join_all(handles).await;

    var.into_iter().filter_map(Result::ok).flatten().collect()
}

pub fn json_to_sections(json: String) -> Result<Vec<Section>, Error> {
    let sections: Vec<Section> = serde_json::from_str(&json)?;
    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;
    /// Need to adjust this value forward or back an hourwhen timezone changes

    #[test]
    fn should_convert_json_to_sections() {
        let sections = vec![
            Section {
                id: "1234".to_string(),
                project_id: "5678".to_string(),
                order: 1,
                name: "Bread".to_string(),
            },
            Section {
                id: "9012".to_string(),
                project_id: "3456".to_string(),
                order: 2,
                name: "Meat".to_string(),
            },
        ];
        let result = json_to_sections(test::responses::sections());
        assert_eq!(result, Ok(sections));
    }
}
