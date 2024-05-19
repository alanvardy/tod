use std::fmt::Display;

use crate::error::Error;
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
    natural_language_only: Option<bool>,
) -> Result<DateTimeInput, Error> {
    let selection = if natural_language_only.unwrap_or_default() {
        "Natural Language"
    } else {
        let options = vec![
            "Pick Date",
            "Natural Language",
            "No Date",
            "Skip",
            "Complete",
        ];
        let description = "Set a due date";
        select(description, options, mock_select)?
    };

    match selection {
        "Natural Language" => {
            let entry = string(
                "Enter datetime in natural language, or one of:\n[none (n), skip (s), complete (c)]",
                mock_string,
            )?;

            match entry.as_str() {
                "none" => Ok(DateTimeInput::None),
                "n" => Ok(DateTimeInput::None),
                "complete" => Ok(DateTimeInput::Complete),
                "c" => Ok(DateTimeInput::Complete),
                "skip" => Ok(DateTimeInput::Skip),
                "s" => Ok(DateTimeInput::Skip),
                _ => Ok(DateTimeInput::Text(entry)),
            }
        }
        "Pick Date" => {
            let string = DateSelect::new("Select Date")
                .prompt()
                .map_err(Error::from)?
                .to_string();

            Ok(DateTimeInput::Text(string))
        }

        "No Date" => Ok(DateTimeInput::None),
        "Complete" => Ok(DateTimeInput::Complete),
        "Skip" => Ok(DateTimeInput::Skip),
        _ => Err(Error {
            message: String::from("Unrecognized input"),
            source: String::from("Datetime Input"),
        }),
    }
}

/// Get text input from user
pub fn string(desc: &str, mock_string: Option<String>) -> Result<String, Error> {
    if cfg!(test) {
        if let Some(string) = mock_string {
            Ok(string)
        } else {
            panic!("Must set mock_string in config")
        }
    } else {
        Text::new(desc).prompt().map_err(Error::from)
    }
}

/// Get string input with default value
pub fn string_with_default(desc: &str, default_message: &str) -> Result<String, Error> {
    if cfg!(test) {
        return Ok(String::from(default_message));
    }

    Text::new(desc)
        .with_initial_value(default_message)
        .prompt()
        .map_err(Error::from)
}

/// Select an input from a list
pub fn select<T: Display>(
    desc: &str,
    options: Vec<T>,
    mock_select: Option<usize>,
) -> Result<T, Error> {
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
        Select::new(desc, options).prompt().map_err(Error::from)
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
