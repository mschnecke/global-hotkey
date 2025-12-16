//! Program launching functionality

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::schema::ProgramConfig;
use crate::error::AppError;

use super::platform;

/// Get additional PATH directories to search on macOS
/// GUI apps don't inherit the shell's PATH, so we need to check common locations
#[cfg(target_os = "macos")]
fn get_extra_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/opt/homebrew/bin"),      // Homebrew on Apple Silicon
        PathBuf::from("/opt/homebrew/sbin"),
        PathBuf::from("/usr/local/bin"),         // Homebrew on Intel, common tools
        PathBuf::from("/usr/local/sbin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ];

    // Add user's local bin
    if let Some(home) = dirs::home_dir() {
        paths.insert(0, home.join(".local/bin"));
        paths.insert(0, home.join("bin"));
    }

    paths
}

#[cfg(not(target_os = "macos"))]
fn get_extra_paths() -> Vec<PathBuf> {
    Vec::new()
}

/// Search for a program in additional PATH directories
fn find_in_extra_paths(name: &str) -> Option<PathBuf> {
    for dir in get_extra_paths() {
        let candidate = dir.join(name);
        if candidate.exists() && platform::is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Launch a program with the given configuration
pub fn launch(config: &ProgramConfig) -> Result<(), AppError> {
    // Resolve the program path - check direct path first, then PATH
    let resolved_path = resolve_program(&config.path).ok_or_else(|| {
        AppError::Process(format!("Program not found: {}", config.path))
    })?;

    let path = Path::new(&resolved_path);

    // Build the command
    let mut command = Command::new(&resolved_path);

    // Add arguments
    for arg in &config.arguments {
        if !arg.is_empty() {
            command.arg(arg);
        }
    }

    // Set working directory
    if let Some(ref working_dir) = config.working_directory {
        if !working_dir.is_empty() {
            let dir = Path::new(working_dir);
            if dir.exists() && dir.is_dir() {
                command.current_dir(dir);
            } else {
                // Fall back to program's directory if working dir doesn't exist
                if let Some(parent) = path.parent() {
                    command.current_dir(parent);
                }
            }
        }
    }

    // Apply platform-specific settings for hidden mode
    if config.hidden {
        platform::configure_hidden(&mut command);
    }

    // Detach the process from our process group
    platform::configure_detached(&mut command);

    // Spawn the process (don't wait for it)
    command.spawn().map_err(|e| {
        AppError::Process(format!("Failed to launch program '{}': {}", config.path, e))
    })?;

    Ok(())
}

/// Validate that a path exists and points to an executable
/// Also checks if the program is available in PATH or common directories
pub fn validate_path(path: &str) -> bool {
    // First check if it's a direct path that exists
    let p = Path::new(path);
    if p.exists() {
        return platform::is_executable(p);
    }

    // Check if it's available in PATH
    if which::which(path).is_ok() {
        return true;
    }

    // Check additional directories (especially for macOS GUI apps)
    find_in_extra_paths(path).is_some()
}

/// Get the executable extensions for the current platform
pub fn get_executable_extensions() -> Vec<&'static str> {
    platform::executable_extensions()
}

/// Resolve a program name to its full path (searches PATH and common directories)
pub fn resolve_program(name: &str) -> Option<String> {
    // First check if it's already a direct path that exists
    let p = Path::new(name);
    if p.exists() && platform::is_executable(p) {
        return Some(name.to_string());
    }

    // Try to find in PATH
    if let Ok(path) = which::which(name) {
        return Some(path.to_string_lossy().to_string());
    }

    // Check additional directories (especially for macOS GUI apps)
    if let Some(path) = find_in_extra_paths(name) {
        return Some(path.to_string_lossy().to_string());
    }

    None
}
