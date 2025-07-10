#[cfg(test)]
pub mod fixtures;
#[cfg(test)]
pub mod responses;

#[cfg(test)]
async fn today_date() -> String {
    let config = fixtures::config().await.with_timezone("America/Vancouver");
    crate::time::date_string_today(&config).unwrap()
}

// Tests for testing assertions and CMD line arguments

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::str::{contains, starts_with};

    use std::time::Duration;

    #[tokio::test]
    async fn test_auth_login_url() {
        let mut cmd = Command::cargo_bin("tod").unwrap();
        cmd.arg("auth")
            .arg("login")
            .timeout(Duration::from_secs(2))
            .assert()
            .stdout(contains(
                "Please visit the following url to authenticate with Todoist:",
            ));
    }

    #[tokio::test]
    async fn test_version_flag() {
        let mut cmd = Command::cargo_bin("tod").unwrap();
        cmd.arg("-V")
            .timeout(Duration::from_secs(2))
            .assert()
            .stdout(starts_with("tod "));
    }
}
