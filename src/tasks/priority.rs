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
            Priority::None => 1,
            Priority::Low => 2,
            Priority::Medium => 3,
            Priority::High => 4,
        }
    }
}

pub fn from_integer(priority: &Option<u8>) -> Option<Priority> {
    match priority {
        None => None,
        Some(1) => Some(Priority::None),
        Some(2) => Some(Priority::Low),
        Some(3) => Some(Priority::Medium),
        Some(4) => Some(Priority::High),
        Some(_) => None,
    }
}
