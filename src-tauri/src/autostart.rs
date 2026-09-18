//! "Start with Windows" support.
//!
//! The login item itself is owned by `tauri-plugin-autostart`; this module adds
//! the two things Glossy needs on top of it: the argument its own starts carry,
//! and a way to repair an entry that points at a build that has moved.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Argument the login item launches Glossy with. It tells a start that came
/// from the system apart from one the user made himself, which is the
/// difference between coming up silently and showing the "Glossy is running"
/// hint over whatever is on screen at login.
pub const FLAG: &str = "--autostart";

/// True when this process was started by the login item.
pub fn started_by_system() -> bool {
    std::env::args().any(|arg| arg == FLAG)
}

/// Turns the login item on or off and reports the state the system ends up
/// with, which is what the settings file stores: the registry entry, not the
/// checkbox, decides whether Glossy comes up with the next login.
pub fn apply(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let manager = app.autolaunch();
    if manager.is_enabled().unwrap_or(false) == enabled {
        return Ok(enabled);
    }
    let change = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    change.map_err(|error| error.to_string())?;
    manager.is_enabled().map_err(|error| error.to_string())
}
