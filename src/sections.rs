use crate::{config::Config, error::Error, input, projects::LegacyProject, todoist};
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
    let projects = config.legacy_projects.clone().unwrap_or_default();

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

pub async fn select_section(
    config: &Config,
    project: &LegacyProject,
) -> Result<Option<Section>, Error> {
    let sections = todoist::sections_for_project(config, project).await?;
    let mut section_names: Vec<String> = sections.clone().into_iter().map(|x| x.name).collect();
    if section_names.is_empty() {
        Ok(None)
    } else {
        section_names.insert(0, "No section".to_string());
        let section_name = input::select(input::SECTION, section_names, config.mock_select)?;

        let section = sections
            .iter()
            .find(|x| x.name == section_name.as_str())
            .map(|s| s.to_owned());
        Ok(section)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

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

    #[tokio::test]
    async fn test_select_section() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/v2/sections?project_id=456")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::sections())
            .create_async()
            .await;

        let config = test::fixtures::config()
            .await
            .mock_url(server.url())
            .mock_select(1);
        let project = test::fixtures::project();

        let expected = Ok(Some(Section {
            id: "1234".to_string(),
            project_id: "5678".to_string(),
            order: 1,
            name: "Bread".to_string(),
        }));
        let result = select_section(&config, &project).await;
        assert_eq!(expected, result);
        mock.assert();
    }
}
