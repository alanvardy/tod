use crate::{config::Config, errors::Error, time};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Comment {
    pub id: String,
    pub posted_uid: Option<String>,
    pub content: String,
    pub uids_to_notify: Option<Vec<String>>,
    pub is_deleted: bool,
    pub posted_at: String,
    pub reactions: Option<Reactions>,
    pub item_id: String,
    pub file_attachment: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CommentResponse {
    pub results: Vec<Comment>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Reactions {}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum Attachment {
    File(FileAttachment),
    Url(UrlAttachment),
    ShortUrl(ShortUrlAttachment),
    Video(VideoAttachment),
    Image(ImageAttachment),
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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ImageAttachment {
    pub resource_type: String,
    pub url: String,
    pub image: String,
    pub image_height: u32,
    pub image_width: u32,
    pub site_name: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub tn_s: Option<serde_json::Value>,
    #[serde(default)]
    pub tn_m: Option<serde_json::Value>,
    #[serde(default)]
    pub tn_l: Option<serde_json::Value>,
}

impl Comment {
    pub fn fmt(&self, config: &Config) -> Result<String, Error> {
        let timezone = time::timezone_from_str(&config.timezone)?;
        let datetime = time::datetime_from_str(&self.posted_at, timezone)?;
        let formatted_date = time::format_datetime(&datetime, config)?;

        let link = match &self.file_attachment {
            None => String::new(),
            Some(Attachment::Url(UrlAttachment {
                url,
                site_name,
                title,
                ..
            })) => Self::render_link(url, &format!("{site_name}: {title}")),
            Some(Attachment::ShortUrl(ShortUrlAttachment { url, title, .. })) => {
                Self::render_link(url, title)
            }
            Some(Attachment::Video(VideoAttachment {
                url,
                site_name,
                title,
                ..
            })) => Self::render_link(url, &format!("{site_name}: {title}")),
            Some(Attachment::File(FileAttachment {
                file_url,
                file_name,
                ..
            })) => Self::render_link(file_url, file_name),
            Some(Attachment::Image(ImageAttachment {
                url,
                site_name,
                title,
                ..
            })) => {
                let site = site_name.as_deref().unwrap_or("Image");
                let title = title.as_deref().unwrap_or(url);
                Self::render_link(url, &format!("{site}: {title}"))
            }
        };

        Ok(format!(
            "Posted {}{}\n{}",
            formatted_date, link, self.content
        ))
    }

    fn render_link(url: &str, label: &str) -> String {
        format!("\nAttachment \x1B]8;;{url}\x1B\\[{label}]\x1B]8;;\x1B\\")
    }
}

pub fn json_to_comment_response(json: String) -> Result<CommentResponse, Error> {
    let response: CommentResponse = serde_json::from_str(&json)?;
    Ok(response)
}

pub fn json_to_comment(json: String) -> Result<Comment, Error> {
    let comment: Comment = serde_json::from_str(&json)?;
    Ok(comment)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::json_to_comment_response;
    use crate::test::fixtures; // 👈 access fixtures::config()
    use std::fs;

    async fn load_comments() -> Vec<Comment> {
        let json = fs::read_to_string("tests/fixtures/comments_all_types.json").unwrap();
        let response = json_to_comment_response(json).unwrap();
        response
            .results
            .into_iter()
            .filter(|c| !c.is_deleted)
            .collect()
    }

    #[tokio::test]
    async fn test_filters_deleted_comment() {
        let comments = load_comments().await;
        assert_eq!(
            comments.len(),
            7,
            "One deleted comment should be filtered out"
        );
    }

    #[tokio::test]
    async fn test_fmt_file_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "file-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("file.pdf"));
    }

    #[tokio::test]
    async fn test_fmt_video_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "video-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("Test Video"));
    }

    #[tokio::test]
    async fn test_fmt_image_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "image-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("Example Image"));
    }

    #[tokio::test]
    async fn test_fmt_url_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "url-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("Interesting Article"));
    }

    #[tokio::test]
    async fn test_fmt_short_url_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "shorturl-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("Shortened Link"));
    }

    #[tokio::test]
    async fn test_fmt_rich_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "rich-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("News Story"));
    }

    #[tokio::test]
    async fn test_fmt_no_attachment() {
        let config = fixtures::config().await;
        let comment = load_comments()
            .await
            .into_iter()
            .find(|c| c.id == "noattach-1")
            .unwrap();
        let output = comment.fmt(&config).unwrap();
        assert!(output.contains("Just a plain comment"));
    }
}
