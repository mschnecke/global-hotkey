//! Post-action execution logic

use std::thread;
use std::time::Duration;

use crate::config::schema::{
    PostAction, PostActionTrigger, PostActionType, PostActionsConfig, ProgramConfig,
};
use crate::error::AppError;
use crate::process;

use super::input::InputSimulator;

/// Execute a program with post-actions
pub fn execute_with_post_actions(
    program_config: &ProgramConfig,
    post_actions: &PostActionsConfig,
    hotkey_name: &str,
) -> Result<(), AppError> {
    // If no post-actions enabled, just launch normally
    if !post_actions.enabled || post_actions.actions.is_empty() {
        return process::spawner::launch(program_config);
    }

    match &post_actions.trigger {
        PostActionTrigger::OnExit => {
            // Launch and wait for process to exit
            let exit_code = launch_and_wait(program_config)?;

            if exit_code == 0 {
                execute_actions(&post_actions.actions, hotkey_name)?;
            } else {
                eprintln!(
                    "Hotkey '{}': process exited with code {}, skipping post-actions",
                    hotkey_name, exit_code
                );
            }
        }
        PostActionTrigger::AfterDelay { delay_ms } => {
            // Launch process (don't wait)
            process::spawner::launch(program_config)?;

            // Wait for delay then execute post-actions
            thread::sleep(Duration::from_millis(*delay_ms));
            execute_actions(&post_actions.actions, hotkey_name)?;
        }
    }

    Ok(())
}

/// Launch a program and wait for it to exit, returning the exit code
fn launch_and_wait(config: &ProgramConfig) -> Result<i32, AppError> {
    use std::process::Command;

    let resolved_path = process::spawner::resolve_program(&config.path)
        .ok_or_else(|| AppError::Process(format!("Program not found: {}", config.path)))?;

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
            let dir = std::path::Path::new(working_dir);
            if dir.exists() && dir.is_dir() {
                command.current_dir(dir);
            }
        }
    }

    // Apply hidden mode if configured
    if config.hidden {
        process::platform::configure_hidden(&mut command);
    }

    // NOTE: Don't detach - we need to wait for this process
    let output = command.output().map_err(|e| {
        AppError::Process(format!("Failed to launch program '{}': {}", config.path, e))
    })?;

    Ok(output.status.code().unwrap_or(-1))
}

/// Execute a sequence of post-actions
fn execute_actions(actions: &[PostAction], hotkey_name: &str) -> Result<(), AppError> {
    let mut simulator = InputSimulator::new()?;

    for action in actions {
        if !action.enabled {
            continue;
        }

        // Small delay before simulating input to ensure window focus is stable
        thread::sleep(Duration::from_millis(50));

        match &action.action_type {
            PostActionType::PasteClipboard => {
                simulator.paste()?;
            }
            PostActionType::SimulateKeystroke { keystroke } => {
                simulator.simulate_keystroke(keystroke)?;
            }
            PostActionType::Delay { delay_ms } => {
                thread::sleep(Duration::from_millis(*delay_ms));
            }
        }
    }

    eprintln!("Hotkey '{}': post-actions completed", hotkey_name);
    Ok(())
}
