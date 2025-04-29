use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct User {
    pub tz_info: TzInfo,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct TzInfo {
    pub timezone: String,
}
