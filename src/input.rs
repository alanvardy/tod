use std::fmt::Display;

use inquire::{DateSelect, Select, Text};

pub enum DateTimeInput {
    Skip,
    None,
    Complete,
    Text(String),
}
/// Get datetime input from user
pub fn datetime(
    mock_select: Option<usize>,
    mock_string: Option<String>,
) -> Result<DateTimeInput, String> {
    let options = vec![
        "Natural Language",
        "Pick Date",
        "No Date",
        "Skip",
        "Complete",
    ];
    let description = "Set a due date";
    let selection = select(description, options, mock_select)?;

    match selection {
        "Natural Language" => {
            let entry = string("Enter datetime in natural language", mock_string)?;

            Ok(DateTimeInput::Text(entry))
        }
        "Pick Date" => {
            let string = DateSelect::new("Select Date")
                .prompt()
                .map_err(|e| e.to_string())?
                .to_string();

            Ok(DateTimeInput::Text(string))
        }

        "No Date" => Ok(DateTimeInput::None),
        "Complete" => Ok(DateTimeInput::Complete),
        "Skip" => Ok(DateTimeInput::Skip),
        _ => Err(String::from("Unrecognized input")),
    }
}

/// Get text input from user
pub fn string(desc: &str, mock_string: Option<String>) -> Result<String, String> {
    if cfg!(test) {
        if let Some(string) = mock_string {
            Ok(string)
        } else {
            panic!("Must set mock_string in config")
        }
    } else {
        Text::new(desc).prompt().map_err(|e| e.to_string())
    }
}

/// Get string input with default value
pub fn string_with_default(desc: &str, default_message: &str) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from(default_message));
    }

    Text::new(desc)
        .with_initial_value(default_message)
        .prompt()
        .map_err(|e| e.to_string())
}

/// Select an input from a list
pub fn select<T: Display>(
    desc: &str,
    options: Vec<T>,
    mock_select: Option<usize>,
) -> Result<T, String> {
    if cfg!(test) {
        if let Some(index) = mock_select {
            Ok(options
                .into_iter()
                .nth(index)
                .expect("Must provide a vector of options"))
        } else {
            panic!("Must set mock_select in config")
        }
    } else {
        Select::new(desc, options)
            .prompt()
            .map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_select() {
        let result = select("type", vec!["there", "are", "words"], Some(0));
        let expected = Ok("there");
        assert_eq!(result, expected);

        let result = select("type", vec!["there", "are", "words"], Some(1));
        let expected = Ok("are");
        assert_eq!(result, expected);
    }
}
