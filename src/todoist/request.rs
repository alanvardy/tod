use std::env;

use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use spinners::Spinner;
use spinners::Spinners;
use uuid::Uuid;

use crate::config::Config;

const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";
const TODOIST_URL: &str = "https://api.todoist.com";

const SPINNER: Spinners = Spinners::Dots4;
const MESSAGE: &str = "Querying API";

/// Post to Todoist via sync API
/// We use sync when we want natural languague processing.
pub fn post_todoist_sync(
    config: &Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    let base_url = get_base_url(config);
    let request_url = format!("{base_url}{url}");
    let token = &config.token;

    let spinner = maybe_start_spinner(config);
    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response))
    }
}

/// Post to Todoist via REST api
/// We use this when we want more options and don't need natural language processing
pub fn post_todoist_rest(
    config: &Config,
    url: String,
    body: serde_json::Value,
) -> Result<String, String> {
    let base_url = get_base_url(config);
    let token = &config.token;

    let request_url = format!("{base_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config);

    let response = Client::new()
        .post(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .header("X-Request-Id", new_uuid())
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response))
    }
}

// Combine get and post into one function
/// Get Todoist via REST api
pub fn get_todoist_rest(config: &Config, url: String) -> Result<String, String> {
    let base_url = get_base_url(config);
    let token = config.token.clone();

    let request_url = format!("{base_url}{url}");
    let authorization: &str = &format!("Bearer {token}");
    let spinner = maybe_start_spinner(config);
    let response = Client::new()
        .get(request_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response))
    }
}

fn get_base_url(config: &Config) -> String {
    if cfg!(test) {
        config.mock_url.clone().expect("Mock URL not set")
    } else {
        TODOIST_URL.to_string()
    }
}

fn maybe_start_spinner(config: &Config) -> Option<Spinner> {
    match env::var("DISABLE_SPINNER") {
        Ok(_) => None,
        _ => {
            if let Some(true) = config.spinners {
                let sp = Spinner::new(SPINNER, MESSAGE.into());
                Some(sp)
            } else {
                None
            }
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
