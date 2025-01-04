use crate::{config::Config, error::Error, time};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Comment {
    pub id: String,
    pub task_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub posted_at: String,
    pub attachment: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Attachment {
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub resource_type: String,
}
impl Comment {
    pub fn fmt(&self, config: &Config) -> Result<String, Error> {
        let timezone = time::timezone_from_str(&config.timezone)?;
        let datetime = time::datetime_from_str(&self.posted_at, timezone)?;
        let formatted_date = time::format_datetime(&datetime, config)?;

        let link = if let Some(Attachment {
            file_name,
            file_url,
            ..
        }) = self.attachment.clone()
        {
            format!("\nAttachment \x1B]8;;{file_url}\x1B\\[{file_name}]\x1B]8;;\x1B\\")
        } else {
            String::new()
        };
        Ok(format!(
            "Posted {}{}\n{}",
            formatted_date, link, self.content
        ))
    }
}
