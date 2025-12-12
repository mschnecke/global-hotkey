//! Platform-specific process handling

use std::path::Path;
use std::process::Command;

// ============================================================================
// Windows Implementation
// ============================================================================

#[cfg(target_os = "windows")]
pub fn configure_hidden(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    // CREATE_NO_WINDOW flag prevents console window from appearing
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(target_os = "windows")]
pub fn configure_detached(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    // DETACHED_PROCESS creates a new process group
    const DETACHED_PROCESS: u32 = 0x00000008;

    // Get current flags and add DETACHED_PROCESS
    // Note: We might already have CREATE_NO_WINDOW set
    command.creation_flags(DETACHED_PROCESS);
}

#[cfg(target_os = "windows")]
pub fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "exe" | "bat" | "cmd" | "ps1" | "com" | "msi")
    } else {
        // Windows executables typically have extensions
        false
    }
}

#[cfg(target_os = "windows")]
pub fn executable_extensions() -> Vec<&'static str> {
    vec!["exe", "bat", "cmd", "ps1", "com"]
}

// ============================================================================
// macOS Implementation
// ============================================================================

#[cfg(target_os = "macos")]
pub fn configure_hidden(command: &mut Command) {
    // On macOS, we redirect stdin/stdout/stderr to /dev/null for hidden mode
    use std::process::Stdio;

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
}

#[cfg(target_os = "macos")]
pub fn configure_detached(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // Create a new process group
    unsafe {
        command.pre_exec(|| {
            // Create new session (detach from terminal)
            libc::setsid();
            Ok(())
        });
    }
}

#[cfg(target_os = "macos")]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    // Check for .app bundles
    if let Some(ext) = path.extension() {
        if ext == "app" {
            return path.is_dir();
        }
    }

    // Check if file has executable permission
    if path.is_file() {
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }

    false
}

#[cfg(target_os = "macos")]
pub fn executable_extensions() -> Vec<&'static str> {
    vec!["app", ""]
}

// ============================================================================
// Linux/Other Unix Implementation
// ============================================================================

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn configure_hidden(command: &mut Command) {
    use std::process::Stdio;

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn configure_detached(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if path.is_file() {
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }

    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn executable_extensions() -> Vec<&'static str> {
    vec!["", "sh", "AppImage"]
}

// ============================================================================
// Cross-platform utilities
// ============================================================================

/// Launch a .app bundle on macOS using the `open` command
#[cfg(target_os = "macos")]
pub fn launch_app_bundle(path: &Path, hidden: bool) -> std::io::Result<std::process::Child> {
    let mut command = Command::new("open");

    if hidden {
        command.arg("-g"); // Don't bring app to foreground
    }

    command.arg(path).spawn()
}

/// Check if a path points to a macOS .app bundle
#[cfg(target_os = "macos")]
pub fn is_app_bundle(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "app") && path.is_dir()
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn is_app_bundle(_path: &Path) -> bool {
    false
}
