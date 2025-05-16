#!/usr/bin/env bash
# Purpose: This script automates the process of creating a pull request for a NEW RELEASE VERSION of a Rust project. Do not use this script for minor patch/diff alone.
# Usage: VERSION=0.6.15 ./create_pr.sh
# This script assumes that the following tools are installed:
# - ambr: A tool for editing Cargo.toml files (install with 'cargo install amber')
# - gt: Graphite CLI tool for creating and managing GitHub pull requests. It integrates with Graphite.dev, a tool for managing Git workflows. Learn more at https://graphite.dev
# - gh: GitHub CLI for interacting with GitHub repositories
# - cargo: Rust's package manager and build system (install using rustup)

# Exits immediately if a command exits with a non-zero status
set -euo pipefail

# Logging functions for consistent output
log() {
    echo "[INFO] $1"
}

error() {
    echo "[ERROR] $1" >&2
    exit 1
}

# Check if the VERSION environment variable is set
if [ -z "${VERSION:-}" ]; then
    error "VERSION environment variable is not set. Usage: VERSION=0.7.6 ./create_pr.sh"
fi

log "VERSION is set to $VERSION"

# Check if required tools are installed
command -v ambr >/dev/null 2>&1 || error "ambr is not installed. Please install with 'cargo install ambr' before running this script."
command -v gt >/dev/null 2>&1 || error "gt is not installed. Please install it before running this script."
command -v gh >/dev/null 2>&1 || error "gh is not installed. Please install it before running this script."

# Update Cargo.toml with the new version
log "Editing Cargo.toml to set version to $VERSION"
ambr --regex "^version = \"\\d+\\.\\d+\\.\\d+\"" "version = \"$VERSION\"" Cargo.toml

# Update dependencies
log "Running cargo update"
cargo update

# Format the code
log "Running cargo fmt"
cargo fmt

# Run Clippy to lint the code
log "Running cargo clippy"
cargo clippy -- -D warnings

# Run tests
log "Running cargo tests"
cargo test

# Create a pull request
# Ensure the branch is up-to-date and has no uncommitted changes
log "Checking if the branch is ready for creating a pull request"
if ! git diff-index --quiet HEAD --; then
    error "There are uncommitted changes in the working directory. Please commit or stash them before proceeding."
fi

if ! git fetch origin && git status | grep -q "Your branch is up to date with"; then
    error "The branch is not up-to-date with the remote. Please pull the latest changes before proceeding."
fi

log "Creating a pull request for version $VERSION"
gt create "v$VERSION" -a -m "v$VERSION" --no-interactive || error "Failed to create pull request."
gt submit --no-interactive || error "Failed to submit pull request."

# Mark the pull request as ready for review
log "Marking the pull request as ready for review"
gh pr ready || error "Failed to mark the pull request as ready."

# Wait for checks to complete
log "Waiting for pull request checks to complete"
sleep 5
gh pr checks --watch -i 5 || error "Pull request checks failed. Please review the failed checks in the GitHub pull request page and address the issues before retrying."

# Play a notification sound (cross-platform support)
log "Playing notification sound"
if command -v afplay >/dev/null 2>&1; then
    afplay /System/Library/Sounds/Ping.aiff
elif command -v paplay >/dev/null 2>&1; then
    paplay /usr/share/sounds/freedesktop/stereo/complete.oga
elif command -v play >/dev/null 2>&1; then
    play /usr/share/sounds/freedesktop/stereo/complete.oga
else
    log "No sound player found. Skipping notification."
fi

log "Pull request creation process for version $VERSION completed successfully!"