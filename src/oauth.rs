use futures::lock::Mutex;
use serde::Deserialize;
use uuid::Uuid;

use crate::color::green_string;
use crate::errors::Error;
use crate::tasks::format::format_url;
use crate::todoist::OAUTH_URL;
use crate::{config::Config, todoist};

use axum::{Router, extract::Query, routing::get};
use std::sync::Arc;
use tokio::sync::oneshot::{self, Sender};

pub const CLIENT_ID: &str = "2696d64dc4f745679e21181c56b489fe";
pub const CLIENT_SECRET: &str = "bfde0d10e3d740beb47f95879881634e";
const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";
const TRANSMIT_ERROR: &str = "Could not transmit";

const LOCALHOST: &str = "127.0.0.1:8080";
const SCOPE: &str = "data:read_write,data:delete,project:delete";

#[derive(Deserialize, Debug)]
struct Params {
    // returns only in the case of an error
    error: Option<String>,
    // authorization code from which we can get an access token
    code: Option<String>,
    // should match the csrf token we passed in
    state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
}

pub async fn login(config: &mut Config, test_tx: Option<Sender<()>>) -> Result<String, Error> {
    let csrf_token = print_oauth_url();
    let code = receive_callback(&csrf_token, test_tx)
        .await?
        .code
        .ok_or_else(|| Error::new("params", "no code provided"))?;
    let access_token = todoist::get_access_token(config, &code).await?;
    let result = config.set_token(access_token).await;

    // Print authentication success message to the terminal
    let check = green_string("Authentication Sucessful!");
    println!("{check}");

    result
}

fn print_oauth_url() -> String {
    let csrf_token = new_uuid();

    let url = format!(
        "https://todoist.com{OAUTH_URL}?client_id={CLIENT_ID}&scope={SCOPE}&state={csrf_token}"
    );
    let formatted_url = format_url(&url, &Config::default());

    match open::that(&url) {
        Ok(_) => {
            println!(
                "Opening {formatted_url} in the default web browser to authenticate with Todoist."
            );
        }
        Err(_) => {
            println!("Please visit the following url to authenticate with Todoist:");
            println!("{formatted_url}");
        }
    }
    csrf_token
}

async fn receive_callback(csrf_token: &str, tx: Option<Sender<()>>) -> Result<Params, Error> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_signal = Arc::new(Mutex::new(Some(shutdown_tx)));

    let (response_tx, response_rx) = oneshot::channel::<Params>();
    let response = Arc::new(Mutex::new(Some(response_tx)));

    let app = Router::new().route(
        "/",
        get(move |Query(params): Query<Params>| {
            // Send shutdown signal after handling the request
            async move {
                if let Some(tx) = shutdown_signal.lock().await.take() {
                    let _ = tx.send(());
                }

                if let Some(tx) = response.lock().await.take() {
                    if let Some(error_message) = params.error.clone() {
                        tx.send(params).expect(TRANSMIT_ERROR);
                        format!("Error from Todoist: {error_message}")
                    } else {
                        tx.send(params).expect(TRANSMIT_ERROR);
                        String::from(
                            "Success! You can close this window and return to your terminal.",
                        )
                    }
                } else {
                    String::from("Error: Could not get response tx")
                }
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind(LOCALHOST).await?;
    if let Some(tx) = tx {
        // Let test suite know that are ready to call the server
        tx.send(()).expect("failed to notify test");
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await?;

    let params = response_rx.await?;

    if let Some(message) = params.error {
        Err(Error::new("oauth get code", &message))
    } else if params.state.clone().unwrap_or_default() == csrf_token {
        Ok(params)
    } else {
        Err(Error::new(
            "oauth get code",
            "state doesn't match csrf token",
        ))
    }
}

pub fn json_to_access_token(json: String) -> Result<AccessToken, Error> {
    let token: AccessToken = serde_json::from_str(&json)?;
    Ok(token)
}

/// Create a new UUID, required for Todoist OAuth
pub fn new_uuid() -> String {
    if cfg!(test) {
        String::from(FAKE_UUID)
    } else {
        Uuid::new_v4().to_string()
    }
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::test::{self, responses::ResponseFromFile};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn login_test() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", "/oauth/access_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ResponseFromFile::AccessToken.read().await)
            .create_async()
            .await;

        let mut config = test::fixtures::config().await.with_mock_url(server.url());

        config.clone().create().await.unwrap();

        assert_eq!(config.token, Some(String::from("alreadycreated")));
        let (test_tx, test_rx) = oneshot::channel::<()>();
        let login_handle =
            tokio::spawn(async move { login(&mut config, Some(test_tx)).await.unwrap() });

        test_rx.await.unwrap();

        let params = [("code", "state"), ("state", FAKE_UUID)];
        let client = reqwest::Client::new();
        let resp = client
            .get("http://127.0.0.1:8080/")
            .query(&params)
            .send()
            .await
            .expect("Failed to send callback");

        assert!(resp.status().is_success());
        let body = resp.text().await.unwrap();
        assert!(body.contains("Success"));

        let result = login_handle.await.unwrap();
        assert_eq!(result, String::from("✓"));
        mock.assert()
    }

    #[tokio::test]
    async fn receive_callback_with_error_param() {
        let (test_tx, test_rx) = oneshot::channel::<()>();
        let csrf_token = new_uuid();
        let csrf_token_clone = csrf_token.clone();

        // Spawn the server
        let server_handle =
            tokio::spawn(async move { receive_callback(&csrf_token, Some(test_tx)).await });

        test_rx.await.unwrap();

        // Simulate callback with error
        let params = [("error", "access_denied"), ("state", &csrf_token_clone)];
        let client = reqwest::Client::new();
        let resp = client
            .get("http://127.0.0.1:8080/")
            .query(&params)
            .send()
            .await
            .expect("Failed to send callback");

        assert!(resp.status().is_success());
        let body = resp.text().await.unwrap();
        assert!(body.contains("Error"));

        let result = server_handle.await.unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("access_denied"));
    }

    #[test]
    fn test_print_oauth_url_returns_csrf_token() {
        // In test mode, new_uuid() returns FAKE_UUID
        let csrf_token = print_oauth_url();
        assert_eq!(csrf_token, FAKE_UUID);

        // Optionally, check that the formatted URL contains the CSRF token
        let expected_url_part = format!("state={FAKE_UUID}");
        let url = format!(
            "https://todoist.com{OAUTH_URL}?client_id={CLIENT_ID}&scope={SCOPE}&state={FAKE_UUID}"
        );
        let formatted_url = format_url(&url, &Config::default());
        assert!(formatted_url.contains(&expected_url_part));
    }
}
