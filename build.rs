//This file runs before the build process to set environment variables and perform other build-time tasks.
// This file is part of the project and is executed by Cargo during the build process.
// It sets environment variables that can be used in the Rust code, such as the build target, profile, and timestamp.

use std::env;

fn main() {
    // Set TARGET and PROFILE as build-time env vars
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".into());

    println!("cargo:rustc-env=BUILD_TARGET={target}");
    println!("cargo:rustc-env=BUILD_PROFILE={profile}");

    // Add build timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");
}
