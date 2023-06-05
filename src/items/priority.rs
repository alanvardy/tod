use clap::ArgMatches;
use std::fmt::Display;

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Priority {
    None = 1,
    Low = 2,
    Medium = 3,
    High = 4,
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::None => write!(f, "NONE (P4)"),
            Priority::Low => write!(f, "LOW (P3)"),
            Priority::Medium => write!(f, "MEDIUM (P2)"),
            Priority::High => write!(f, "HIGH (P1)"),
        }
    }
}

impl Priority {
    pub fn to_integer(&self) -> u8 {
        match self {
            Priority::None => 4,
            Priority::Low => 3,
            Priority::Medium => 2,
            Priority::High => 1,
        }
    }

    pub fn get_from_matches(matches: &ArgMatches) -> Option<Self> {
        let priority_arg = &matches.get_one::<String>("priority").map(|s| s.to_owned());
        match priority_arg {
            None => None,
            Some(priority) => serde_json::from_str(priority).ok(),
        }
    }
}
