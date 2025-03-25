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
#[serde(untagged)]
pub enum Attachment {
    File(FileAttachment),
    Url(UrlAttachment),
    ShortUrl(ShortUrlAttachment),
    Video(VideoAttachment),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct FileAttachment {
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub resource_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UrlAttachment {
    description: String,
    favicon: String,
    image: String,
    image_height: u32,
    image_width: u32,
    resource_type: String,
    site_name: String,
    title: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ShortUrlAttachment {
    resource_type: String,
    title: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct VideoAttachment {
    html: String,
    resource_type: String,
    title: String,
    url: String,
    site_name: String,
}

impl Comment {
    pub fn fmt(&self, config: &Config) -> Result<String, Error> {
        let timezone = time::timezone_from_str(&config.timezone)?;
        let datetime = time::datetime_from_str(&self.posted_at, timezone)?;
        let formatted_date = time::format_datetime(&datetime, config)?;

        let link = match &self.attachment {
            None => String::new(),
            Some(Attachment::Url(UrlAttachment {
                url,
                site_name,
                title,
                ..
            })) => {
                format!("\nAttachment \x1B]8;;{url}\x1B\\[{site_name}: {title}]\x1B]8;;\x1B\\")
            }
            Some(Attachment::ShortUrl(ShortUrlAttachment {
                url,
                title,
                resource_type: _resource_type,
            })) => {
                format!("\nAttachment \x1B]8;;{url}\x1B\\[{title}]\x1B]8;;\x1B\\")
            }
            Some(Attachment::Video(VideoAttachment {
                url,
                site_name,
                title,
                ..
            })) => {
                format!("\nAttachment \x1B]8;;{url}\x1B\\[{site_name}: {title}]\x1B]8;;\x1B\\")
            }
            Some(Attachment::File(FileAttachment {
                file_name,
                file_url,
                ..
            })) => {
                format!("\nAttachment \x1B]8;;{file_url}\x1B\\[{file_name}]\x1B]8;;\x1B\\")
            }
        };
        Ok(format!(
            "Posted {}{}\n{}",
            formatted_date, link, self.content
        ))
    }
}
