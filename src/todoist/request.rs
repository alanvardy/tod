use std::env;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::Response;
use serde_json::json;
use spinners::Spinner;
use spinners::Spinners;
use uuid::Uuid;

use crate::config::Args;
use crate::config::Config;
use crate::debug;
use crate::error;
use crate::error::Error;

const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";
const TODOIST_URL: &str = "https://api.todoist.com";

const SPINNER: Spinners = Spinners::Dots4;
const MESSAGE: &str = "Querying API";

/// Post to Todoist via sync API
/// We use sync when we want natural languague processing.
pub async fn post_todoist_sync(
    config: &Config,
    url: String,
    body: serde_json::Value,
    spinner: bool,
) -> Result<String, Error> {
    let base_url = get_base_url(config);
    let request_url = format!("{base_url}{url}");
    let token = &config.token;

    let spinner = maybe_start_spinner(config, spinner);
    debug::print(config, format!("POST {request_url}\nbody: {body}"));
    let response = Client::new()
        .post(request_url.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .json(&body)
        .timeout(get_timeout(config))
        .send()
        .await?;

    maybe_stop_spinner(spinner);
    handle_response(config, response, "POST", url, body).await
}

/// Post to Todoist via REST api
/// We use this when we want more options and don't need natural language processing
pub async fn post_todoist_rest(
    config: &Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, Error> {
    let base_url = get_base_url(config);
    let token = &config.token;

    let request_url = format!("{base_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config, true);

    debug::print(config, format!("POST {request_url}\nbody: {body}"));

    let response = Client::new()
        .post(request_url.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .header("X-Request-Id", new_uuid())
        .json(&body)
        .timeout(get_timeout(config))
        .send()
        .await?;

    maybe_stop_spinner(spinner);
    handle_response(config, response, "POST", url, body).await
}

pub async fn delete_todoist_rest(
    config: &Config,
    url: String,
    body: serde_json::Value,
    spinner: bool,
) -> Result<String, Error> {
    let base_url = get_base_url(config);
    let token = &config.token;

    let request_url = format!("{base_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config, spinner);

    debug::print(config, format!("DELETE {request_url}\nbody: {body}"));

    let response = Client::new()
        .delete(request_url.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .header("X-Request-Id", new_uuid())
        .json(&body)
        .timeout(get_timeout(config))
        .send()
        .await?;

    maybe_stop_spinner(spinner);
    handle_response(config, response, "DELETE", url, body).await
}

// Combine get and post into one function
/// Get Todoist via REST api
pub async fn get_todoist_rest(config: &Config, url: String) -> Result<String, Error> {
    let base_url = get_base_url(config);
    let token = config.token.clone();

    let request_url = format!("{base_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config, true);
    if config.verbose.unwrap_or_default() {
        println!("GET {request_url}")
    }
    debug::print(config, format!("GET {request_url}"));
    let response = Client::new()
        .get(request_url.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .timeout(get_timeout(config))
        .send()
        .await?;

    maybe_stop_spinner(spinner);
    handle_response(config, response, "GET", url, json!({})).await
}

async fn handle_response(
    config: &Config,
    response: Response,
    method: &str,
    url: String,
    body: serde_json::Value,
) -> Result<String, Error> {
    if response.status().is_success() {
        let text = response.text().await.unwrap();
        debug::print(config, format!("{method} {url}\nresponse: {text}"));
        Ok(text)
    } else {
        Err(error::new(
            "reqwest",
            &format!(
                "
            method: {method}
            url: {url}
            body: {body}
            Error: {:?}",
                response
            ),
        ))
    }
}

fn get_timeout(config: &Config) -> Duration {
    match config {
        Config {
            timeout: Some(timeout),
            args: Args { timeout: None, .. },
            ..
        } => Duration::from_secs(timeout.to_owned()),
        Config {
            timeout: Some(_),
            args: Args {
                timeout: Some(timeout),
                ..
            },
            ..
        } => Duration::from_secs(timeout.to_owned()),
        Config {
            timeout: None,
            args: Args { timeout: None, .. },
            ..
        } => Duration::from_secs(30),

        Config {
            timeout: None,
            args: Args {
                timeout: Some(timeout),
                ..
            },
            ..
        } => Duration::from_secs(timeout.to_owned()),
    }
}

fn get_base_url(config: &Config) -> String {
    if cfg!(test) {
        config.mock_url.clone().expect("Mock URL not set")
    } else {
        TODOIST_URL.to_string()
    }
}

fn maybe_start_spinner(config: &Config, spinner: bool) -> Option<Spinner> {
    if cfg!(test) {
        return None;
    }

    match (env::var("DISABLE_SPINNER"), config.spinners, spinner) {
        (Ok(_), _, _) => None,
        (_, Some(false), _) => None,
        (_, _, false) => None,
        _ => {
            let sp = Spinner::new(SPINNER, MESSAGE.into());
            Some(sp)
        }
    }
}
fn maybe_stop_spinner(spinner: Option<Spinner>) {
    if let Some(mut sp) = spinner {
        sp.stop();
        print!("\x1b[2K\r");
    };
}

/// Create a new UUID, required for Todoist API
pub fn new_uuid() -> String {
    if cfg!(test) {
        String::from(FAKE_UUID)
    } else {
        Uuid::new_v4().to_string()
    }
}
