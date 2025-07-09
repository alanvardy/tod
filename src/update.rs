// Thils file contains the functions used for checking for updates and automatically updating the tod CLI tool.
// Functions that attempt to detect the installation method of the current executable, used for autoupdate and debug
use std::env;

#[derive(Debug)]
pub enum InstallMethod {
    Homebrew,
    Scoop,
    Cargo,
    FromSource,
    Unknown,
}

// Public API: returns the detected or overridden install method if manually specified
pub fn get_install_method(override_arg: &Option<String>) -> InstallMethod {
    match override_arg.as_deref() {
        Some("cargo") => InstallMethod::Cargo,
        Some("scoop") => InstallMethod::Scoop,
        Some("homebrew") => InstallMethod::Homebrew,
        Some("source") => InstallMethod::FromSource,
        _ => detect_install_method(),
    }
}

fn detect_install_method() -> InstallMethod {
    let path = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return InstallMethod::Unknown,
    };

    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();

    if cfg!(debug_assertions) || components.iter().any(|c| c == "target") {
        InstallMethod::FromSource
    } else if components.iter().any(|c| c.contains(".cargo")) {
        InstallMethod::Cargo
    } else if components.iter().any(|c| c.contains("scoop")) {
        InstallMethod::Scoop
    } else if components.iter().any(|c| c.contains("homebrew")) {
        InstallMethod::Homebrew
    } else {
        InstallMethod::Unknown
    }
}

// Public API: returns the string name of how it was installed
pub fn get_install_method_string(override_arg: &Option<String>) -> &'static str {
    match get_install_method(override_arg) {
        InstallMethod::Homebrew => "homebrew",
        InstallMethod::Scoop => "scoop",
        InstallMethod::Cargo => "cargo",
        InstallMethod::FromSource => "from source",
        InstallMethod::Unknown => "unknown",
    }
}
// Public API: returns the upgrade instruction
pub fn get_upgrade_command(override_arg: &Option<String>) -> &'static str {
    match get_install_method(override_arg) {
        InstallMethod::Homebrew => "brew upgrade tod",
        InstallMethod::Scoop => "scoop update tod",
        InstallMethod::Cargo => "cargo install tod --force",
        InstallMethod::FromSource | InstallMethod::Unknown => "https://tod.cx",
    }
}

pub fn perform_auto_update() -> Result<(), String> {
    // This is where actual update logic will go later
    println!("Auto-updating (placeholder)...");
    Ok(())
}
