use serde::Deserialize;

/// https://developer.todoist.com/sync/v9/#user
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct SyncResponse {
    pub user: User,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct User {
    pub tz_info: TzInfo,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct TzInfo {
    pub timezone: String,
}
