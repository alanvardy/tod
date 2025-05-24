//File for shell functions used local to the system, such as command exeution and shell completions
use crate::{Cli, LOWERCASE_NAME};
use clap::CommandFactory;
use std::{io, process::Stdio};
use tokio::process::Command;

#[derive(clap::ValueEnum, Debug, Copy, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[allow(clippy::enum_variant_names)]
    PowerShell,
    Elvish,
}

/// Executes a local system command  async with the given arguments and suppresses stdout.
/// Captures stderr and prints it if the command fails.
pub fn execute_command(command: &str) {
    // Spawn the command execution in the background
    let command = command.to_string(); // Clone the command string for the async task
    tokio::spawn(async move {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdout(if cfg!(test) {
                // Only capture stdout in tests for test case validation
                Stdio::piped()
            } else {
                Stdio::null()
            }) // Suppress stdout
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

pub(crate) fn generate_completions(shell: Shell) {
    let mut cli = Cli::command();

    match shell {
        Shell::Bash => {
            let shell = clap_complete::shells::Bash;
            clap_complete::generate(shell, &mut cli, LOWERCASE_NAME, &mut io::stdout());
        }
        Shell::Fish => {
            let shell = clap_complete::shells::Fish;
            clap_complete::generate(shell, &mut cli, LOWERCASE_NAME, &mut io::stdout());
        }
        Shell::Zsh => {
            let shell = clap_complete::shells::Zsh;
            clap_complete::generate(shell, &mut cli, LOWERCASE_NAME, &mut io::stdout());
        }
        Shell::PowerShell => {
            let shell = clap_complete::shells::PowerShell;
            clap_complete::generate(shell, &mut cli, LOWERCASE_NAME, &mut io::stdout());
        }
        Shell::Elvish => {
            let shell = clap_complete::shells::Elvish;
            clap_complete::generate(shell, &mut cli, LOWERCASE_NAME, &mut io::stdout());
        }
    };
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
