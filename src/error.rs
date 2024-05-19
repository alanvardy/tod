use std::fmt::Display;

use crate::color;
use homedir::GetHomeError;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Error {
    pub message: String,
    pub source: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Error { source, message } = self;
        write!(
            f,
            "Error from {}:\n{}",
            color::yellow_string(source),
            color::red_string(message)
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            source: String::from("io"),
            message: format!("{value}"),
        }
    }
}

impl From<chrono_tz::ParseError> for Error {
    fn from(value: chrono_tz::ParseError) -> Self {
        Self {
            source: String::from("chrono_tz"),
            message: format!("{value}"),
        }
    }
}

impl From<chrono::ParseError> for Error {
    fn from(value: chrono::ParseError) -> Self {
        Self {
            source: String::from("chrono"),
            message: format!("{value}"),
        }
    }
}

impl From<GetHomeError> for Error {
    fn from(value: GetHomeError) -> Self {
        Self {
            source: String::from("homedir"),
            message: format!("{value}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self {
            source: String::from("serde_json"),
            message: format!("{value}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self {
            source: String::from("reqwest"),
            message: format!("{value}"),
        }
    }
}

impl From<inquire::InquireError> for Error {
    fn from(value: inquire::InquireError) -> Self {
        Self {
            source: String::from("inquire"),
            message: format!("{value}"),
        }
    }
}

pub fn new(source: &str, message: &str) -> Error {
    Error {
        source: String::from(source),
        message: String::from(message),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_format() {
        let error = Error {
            message: "there".to_string(),
            source: "hello".to_string(),
        };
        assert_eq!(error.to_string(), String::from("Error from hello:\nthere"))
    }
}
