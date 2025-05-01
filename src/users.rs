use crate::errors::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct User {
    pub tz_info: TzInfo,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct TzInfo {
    pub timezone: String,
}

pub fn json_to_user(json: String) -> Result<User, Error> {
    let user: User = serde_json::from_str(&json)?;
    Ok(user)
}
