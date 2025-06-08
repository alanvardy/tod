use futures::lock::Mutex;
use serde::Deserialize;
use uuid::Uuid;

use crate::color::green_string;
use crate::errors::Error;
use crate::todoist::OAUTH_URL;
use crate::{config::Config, todoist};

use axum::{Router, extract::Query, routing::get};
use std::sync::Arc;
use tokio::sync::oneshot;

pub const CLIENT_ID: &str = "2696d64dc4f745679e21181c56b489fe";
pub const CLIENT_SECRET: &str = "bfde0d10e3d740beb47f95879881634e";
const FAKE_UUID: &str = "42963283-2bab-4b1f-bad2-278ef2b6ba2c";

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

pub async fn login(config: &mut Config) -> Result<String, Error> {
    let csrf_token = print_oauth_url();
    let code = receive_callback(&csrf_token)
        .await?
        .code
        .ok_or_else(|| Error::new("params", "no code provided"))?;
    let access_token = todoist::get_access_token(config, &code).await?;
    config.set_token(access_token).await
}

fn print_oauth_url() -> String {
    let csrf_token = new_uuid();

    let text = green_string("Please visit the following url to authenticate with Todoist:");
    let url = format!(
        "https://todoist.com{OAUTH_URL}?client_id={CLIENT_ID}&scope={SCOPE}&state={csrf_token}"
    );
    println!("{text}\n{url}");
    csrf_token
}

async fn receive_callback(csrf_token: &str) -> Result<Params, Error> {
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
                        tx.send(params).unwrap();
                        format!("Error from Todoist: {error_message}")
                    } else {
                        tx.send(params).unwrap();
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
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await
        .unwrap();

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

        let login_handle = tokio::spawn(async move { login(&mut config).await.unwrap() });
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

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
        assert!(body.contains("Success")); // or whatever message your handler returns

        let result = login_handle.await.unwrap();
        assert_eq!(result, String::from("✓"));
        mock.assert()
    }
}
