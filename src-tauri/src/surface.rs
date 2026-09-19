//! Window material: the Mica backdrop behind the settings window and the
//! colour of its title bar.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager, Theme as TauriTheme};

use crate::settings::Theme;

/// Whether the settings window really got a backdrop. The stylesheet only turns
/// transparent when this is true, so a Windows build too old to draw Mica keeps
/// painting an opaque background of its own instead of showing the desktop
/// through the window.
static BACKDROP: AtomicBool = AtomicBool::new(false);

/// Whether `prepare` has run, i.e. whether `BACKDROP` holds the outcome of the
/// attempt instead of its initial guess. The window starts loading before the
/// Rust `setup` hook runs, so the page has to wait for this before it can
/// trust `backdrop` and whether to poll for it.
static READY: AtomicBool = AtomicBool::new(false);

/// What the page needs to know about the surface it is drawn in.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceInfo {
    pub backdrop: bool,
    pub ready: bool,
}

/// Gives `label` a Mica backdrop. Needs Windows 11 build 22523 or newer; on
/// anything else this reports false and the window keeps its own colour.
pub fn apply_backdrop(app: &AppHandle, label: &str, dark: bool) -> bool {
    let Some(window) = app.get_webview_window(label) else {
        return false;
    };
    // Mica is drawn by Windows behind the window, so the page has to stop
    // painting over it: the window is already transparent (see
    // `tauri.conf.json`), but the webview fills its client area until it is
    // told otherwise.
    let applied = window_vibrancy::apply_mica(&window, Some(dark)).is_ok()
        && window
            .set_background_color(Some(tauri::window::Color(0, 0, 0, 0)))
            .is_ok();
    BACKDROP.store(applied, Ordering::Relaxed);
    applied
}

/// Matches the title bar to the colour scheme the page is using. `System`
/// hands the decision back to Windows.
pub fn set_theme(app: &AppHandle, label: &str, theme: Theme) -> bool {
    let Some(window) = app.get_webview_window(label) else {
        return false;
    };
    let wanted = match theme {
        Theme::Light => Some(TauriTheme::Light),
        Theme::Dark => Some(TauriTheme::Dark),
        Theme::System => None,
    };
    if window.set_theme(wanted).is_err() {
        return false;
    }
    // The backdrop is tinted by the frame around it, so it has to be redrawn
    // whenever that frame changes colour. `theme()` reports what Windows
    // settled on, which is what `System` resolves to.
    if BACKDROP.load(Ordering::Relaxed) {
        let dark = matches!(window.theme(), Ok(TauriTheme::Dark));
        apply_backdrop(app, label, dark);
    }
    true
}

/// Colours the title bar and lays the backdrop behind the window. Called once
/// while the window is still hidden.
pub fn prepare(app: &AppHandle, label: &str, theme: Theme) -> bool {
    let applied = if !set_theme(app, label, theme) {
        false
    } else {
        // `set_theme` redraws an existing backdrop, so ask the window what
        // Windows settled on rather than guessing at `System`.
        let dark = app
            .get_webview_window(label)
            .and_then(|window| window.theme().ok())
            .is_some_and(|theme| theme == TauriTheme::Dark);
        apply_backdrop(app, label, dark)
    };
    READY.store(true, Ordering::Relaxed);
    applied
}

#[tauri::command]
pub fn surface_info() -> SurfaceInfo {
    SurfaceInfo {
        backdrop: BACKDROP.load(Ordering::Relaxed),
        ready: READY.load(Ordering::Relaxed),
    }
}

/// Called by the page whenever the colour scheme setting changes.
#[tauri::command]
pub fn set_window_theme(app: AppHandle, label: String, theme: Theme) -> bool {
    set_theme(&app, &label, theme)
}
