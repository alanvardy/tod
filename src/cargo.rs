use reqwest::header::USER_AGENT;
use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;
use crate::error::Error;
use crate::VERSION;

// CRATES.IO URLS
const VERSIONS_URL: &str = "/v1/crates/tod/versions";

#[derive(Deserialize)]
struct CargoResponse {
    versions: Vec<CargoVersion>,
}

#[derive(Deserialize)]
struct CargoVersion {
    num: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Version {
    Latest,
    Dated(String),
}
pub async fn compare_versions(config: Config) -> Result<Version, Error> {
    match get_latest_version(config).await {
        Ok(version) if version.as_str() != VERSION => Ok(Version::Dated(version)),
        Ok(_) => Ok(Version::Latest),
        Err(err) => Err(err),
    }
}
/// Get latest version number from Cargo.io
pub async fn get_latest_version(config: Config) -> Result<String, Error> {
    #[cfg(not(test))]
    let cargo_url: String = "https://crates.io/api".to_string();
    let _token = config.token;

    #[cfg(test)]
    let cargo_url: String = config.mock_url.expect("Mock URL not set");

    let request_url = format!("{cargo_url}{VERSIONS_URL}");

    let response = Client::new()
        .get(request_url)
        .header(USER_AGENT, "Tod")
        .send()
        .await?;

    if response.status().is_success() {
        let cr: CargoResponse = serde_json::from_str(&response.text().await?)?;
        Ok(cr.versions.first().unwrap().num.clone())
    } else {
        let message = format!("Error: {:#?}", response.text().await);
        let source = "get_latest_version response failure".to_string();
        Err(Error { message, source })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test, VERSION};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_get_latest_version() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/crates/tod/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::versions())
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let response = get_latest_version(config).await;
        mock.assert();

        assert_eq!(response, Ok(String::from(VERSION)));
    }

    #[tokio::test]
    async fn test_compare_versions() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/crates/tod/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::versions())
            .create_async()
            .await;

        let config = test::fixtures::config().await.mock_url(server.url());

        let response = compare_versions(config).await;
        mock.assert();

        assert_eq!(response, Ok(Version::Latest));
    }
}
