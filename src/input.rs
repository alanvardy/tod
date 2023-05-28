use std::fmt::Display;

use inquire::{Select, Text};

/// Get text input from user
pub fn string(desc: &str) -> Result<String, String> {
    if cfg!(test) {
        return Ok(String::from("Africa/Asmera"));
    }

    Text::new(desc).prompt().map_err(|e| e.to_string())
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
pub fn select<T: Display>(desc: &str, options: Vec<T>) -> Result<T, String> {
    // Just return the first option in test
    if cfg!(test) {
        return Ok(options
            .into_iter()
            .next()
            .expect("Must provide a vector of options"));
    }
    Select::new(desc, options)
        .prompt()
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_select() {
        let result = select("type", vec!["there", "are", "words"]);
        let expected = Ok("there");
        assert_eq!(result, expected)
    }
}
