//! Hotkey conflict detection

use crate::config::schema::HotkeyBinding;

use super::manager::REGISTRY;

/// Check if a hotkey binding conflicts with existing registered hotkeys
pub fn check_conflict(binding: &HotkeyBinding) -> bool {
    check_conflict_excluding(binding, None)
}

/// Check if a hotkey binding conflicts, excluding a specific hotkey ID
pub fn check_conflict_excluding(binding: &HotkeyBinding, exclude_id: Option<&str>) -> bool {
    let registry = REGISTRY.read().unwrap();

    for (id, (_, _, config)) in registry.iter() {
        // Skip the excluded ID (useful when editing an existing hotkey)
        if let Some(exclude) = exclude_id {
            if id == exclude {
                continue;
            }
        }

        if bindings_match(&config.hotkey, binding) {
            return true;
        }
    }

    false
}

/// Check if two hotkey bindings are equivalent
fn bindings_match(a: &HotkeyBinding, b: &HotkeyBinding) -> bool {
    // Keys must match (case-insensitive)
    if a.key.to_lowercase() != b.key.to_lowercase() {
        return false;
    }

    // Modifiers must match (same set, order doesn't matter)
    let mut mods_a: Vec<String> = a
        .modifiers
        .iter()
        .map(|m| normalize_modifier(m))
        .filter(|m| !m.is_empty())
        .collect();
    let mut mods_b: Vec<String> = b
        .modifiers
        .iter()
        .map(|m| normalize_modifier(m))
        .filter(|m| !m.is_empty())
        .collect();

    mods_a.sort();
    mods_b.sort();

    mods_a == mods_b
}

/// Normalize modifier names for comparison
fn normalize_modifier(modifier: &str) -> String {
    match modifier.to_lowercase().as_str() {
        "ctrl" | "control" => "ctrl".to_string(),
        "alt" => "alt".to_string(),
        "shift" => "shift".to_string(),
        "meta" | "super" | "win" | "cmd" | "command" => "meta".to_string(),
        _ => modifier.to_lowercase(),
    }
}

/// Known system hotkeys that should be avoided
const SYSTEM_HOTKEYS: &[(&[&str], &str)] = &[
    // Windows system hotkeys
    (&["ctrl", "alt"], "delete"),
    (&["alt"], "tab"),
    (&["alt"], "f4"),
    (&["meta"], "l"),
    (&["meta"], "d"),
    (&["meta"], "e"),
    (&["meta"], "r"),
    (&["meta"], "tab"),
    (&["ctrl", "shift"], "escape"),
    // macOS system hotkeys
    (&["meta"], "q"),
    (&["meta"], "w"),
    (&["meta"], "tab"),
    (&["meta", "shift"], "3"),
    (&["meta", "shift"], "4"),
    (&["meta", "shift"], "5"),
    (&["meta"], "space"),
    (&["ctrl"], "space"),
];

/// Check if a binding conflicts with known system hotkeys
pub fn conflicts_with_system(binding: &HotkeyBinding) -> bool {
    let key = binding.key.to_lowercase();
    let mods: Vec<String> = binding
        .modifiers
        .iter()
        .map(|m| normalize_modifier(m))
        .filter(|m| !m.is_empty())
        .collect();

    for (sys_mods, sys_key) in SYSTEM_HOTKEYS {
        if *sys_key == key {
            let mut sys_mods_vec: Vec<String> = sys_mods.iter().map(|s| s.to_string()).collect();
            let mut sorted_mods = mods.clone();

            sorted_mods.sort();
            sys_mods_vec.sort();

            if sorted_mods == sys_mods_vec {
                return true;
            }
        }
    }

    false
}

/// Get a list of system hotkeys for display
#[allow(dead_code)]
pub fn get_system_hotkeys_list() -> Vec<String> {
    SYSTEM_HOTKEYS
        .iter()
        .map(|(mods, key)| {
            let mut parts: Vec<&str> = mods.to_vec();
            parts.push(key);
            parts.join(" + ")
        })
        .collect()
}
