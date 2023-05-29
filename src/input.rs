use std::fmt::Display;

use inquire::{Select, Text};

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
    // Just return the first option in test

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
