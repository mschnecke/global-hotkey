//! Program launching functionality

use std::path::Path;
use std::process::Command;

use crate::config::schema::ProgramConfig;
use crate::error::AppError;

use super::platform;

/// Launch a program with the given configuration
pub fn launch(config: &ProgramConfig) -> Result<(), AppError> {
    let path = Path::new(&config.path);

    // Validate the path exists
    if !path.exists() {
        return Err(AppError::Process(format!(
            "Program not found: {}",
            config.path
        )));
    }

    // Build the command
    let mut command = Command::new(&config.path);

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
pub fn validate_path(path: &str) -> bool {
    let path = Path::new(path);

    if !path.exists() {
        return false;
    }

    platform::is_executable(path)
}

/// Get the executable extensions for the current platform
pub fn get_executable_extensions() -> Vec<&'static str> {
    platform::executable_extensions()
}

/// Resolve a program name to its full path (searches PATH)
#[allow(dead_code)]
pub fn resolve_program(name: &str) -> Option<String> {
    // First check if it's already a valid path
    if validate_path(name) {
        return Some(name.to_string());
    }

    // Try to find in PATH
    if let Ok(path) = which::which(name) {
        return Some(path.to_string_lossy().to_string());
    }

    None
}
