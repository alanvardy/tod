use serde::Deserialize;

// Projects are split into sections
#[derive(PartialEq, Deserialize, Clone, Debug)]
pub struct Section {
    pub id: String,
    pub project_id: String,
    pub order: u8,
    pub name: String,
}

pub fn json_to_sections(json: String) -> Result<Vec<Section>, String> {
    let result: Result<Vec<Section>, _> = serde_json::from_str(&json);
    match result {
        Ok(sections) => Ok(sections),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
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
