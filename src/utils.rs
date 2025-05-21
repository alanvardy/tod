//File for utility functions used local to the system, such as command exeution

use std::process::Stdio;
use tokio::process::Command;
/// Executes a local system command  async with the given arguments and suppresses stdout.
/// Captures stderr and prints it if the command fails.
pub fn execute_command(command: &str) {
    // Spawn the command execution in the background
    let command = command.to_string(); // Clone the command string for the async task
    tokio::spawn(async move {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdout(Stdio::null()) // Suppress stdout
            .stderr(Stdio::piped()) // Capture stderr
            .output()
            .await;

        if let Err(e) = output {
            eprintln!("Failed to execute command '{}': {}", command, e);
        } else if let Ok(output) = output {
            if !output.status.success() {
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    eprintln!("Command '{}' failed: {}", command, stderr);
                } else {
                    eprintln!("Command '{}' failed with non-UTF-8 output.", command);
                }
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_command_success() {
        // This should succeed and produce no stderr output.
        execute_command("echo 'Hello, world!'");
    }

    #[tokio::test]
    async fn test_execute_command_failure() {
        // This should fail and print an error to stderr.
        execute_command("nonexistent_command_12345");
    }

    #[tokio::test]
    async fn test_execute_command_with_stderr() {
        // This should fail and print the error message from ls.
        execute_command("ls /nonexistent_directory");
    }

    #[tokio::test]
    async fn test_execute_command_non_utf8_stderr() {
        // Try to cat a binary file to produce non-UTF-8 output in stderr.
        // This test may not always trigger non-UTF-8, but it's a best effort.
        execute_command("cat /bin/ls 2>&1 1>/dev/null");
    }
}
