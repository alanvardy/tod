use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

use crate::config::Config;

// CRATES.IO URLS
const VERSIONS_URL: &str = "/v1/crates/tod/versions";

#[derive(Deserialize)]
struct CargoResponse {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    num: String,
}
/// Get latest version number from Cargo.io
pub fn get_latest_version(config: Config) -> Result<String, String> {
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
        .or(Err("Did not get response from server"))?;

    if response.status().is_success() {
        let cr: CargoResponse =
            serde_json::from_str(&response.text().or(Err("Could not read response text"))?)
                .or(Err("Could not serialize to CargoResponse"))?;
        Ok(cr.versions.first().unwrap().num.clone())
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test, VERSION};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_latest_version() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/crates/tod/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::versions())
            .create();

        let config = test::fixtures::config().mock_url(server.url());

        let response = get_latest_version(config);
        mock.assert();

        assert_eq!(response, Ok(String::from(VERSION)));
    }
}
