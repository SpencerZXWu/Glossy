//! Window material: the Mica backdrop behind the settings window and the
//! colour of its title bar.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager, Theme as TauriTheme};

use crate::platform;
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

/// Called by the page whenever the palette setting changes, with the window
/// colour that palette paints — or nothing at all for the default one.
///
/// A palette has a window colour of its own, and the two things Windows draws
/// around the page have to be told: the title bar (DWM, or it keeps the colour
/// of the scheme the window is not in) and the backdrop (Mica is tinted by the
/// desktop, which is exactly what a palette with its own window colour cannot
/// have). The default palette sends `None`, which gives the caption back to
/// Windows and lays the backdrop again.
///
/// The colour comes from the page rather than from a table here, because
/// `tokens.css` is where a palette is defined — a second copy in Rust is a
/// second thing to keep in step.
#[tauri::command]
pub fn set_window_surface(app: AppHandle, label: String, colour: Option<String>) -> bool {
    set_surface(&app, &label, colour.as_deref())
}

fn set_surface(app: &AppHandle, label: &str, colour: Option<&str>) -> bool {
    let Some(window) = app.get_webview_window(label) else {
        return false;
    };
    let handle = platform::desktop::Handle(window.hwnd().map(|h| h.0 as isize).unwrap_or(0));
    let rgb = colour.and_then(parse_colour);
    match rgb {
        Some((red, green, blue)) => {
            let _ = window_vibrancy::clear_mica(&window);
            BACKDROP.store(false, Ordering::Relaxed);
            // The page paints this too; the window is set as well so that the
            // frame, the title bar and the moment before the page paints are
            // the same colour rather than the previous palette's.
            let _ = window.set_background_color(Some(tauri::window::Color(red, green, blue, 255)));
            platform::desktop::set_caption_color(handle, Some((red, green, blue)))
        }
        None => {
            let dark = matches!(window.theme(), Ok(TauriTheme::Dark));
            let cleared = platform::desktop::set_caption_color(handle, None);
            apply_backdrop(app, label, dark) && cleared
        }
    }
}

/// `#rrggbb` as three channels, or nothing when the page sent something else.
fn parse_colour(value: &str) -> Option<(u8, u8, u8)> {
    let text = value.trim().trim_start_matches('#');
    if text.len() != 6 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |at: usize| u8::from_str_radix(&text[at..at + 2], 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_colour_is_read_only_from_a_plain_hex_value() {
        assert_eq!(parse_colour("#4c6a92"), Some((0x4c, 0x6a, 0x92)));
        assert_eq!(parse_colour("  #FFFFFF "), Some((255, 255, 255)));
        assert_eq!(parse_colour("4c6a92"), Some((0x4c, 0x6a, 0x92)));
        // Anything else is left to Windows rather than painted half-read.
        assert_eq!(parse_colour("rgba(255, 255, 255, 0.7)"), None);
        assert_eq!(parse_colour("#fff"), None);
        assert_eq!(parse_colour(""), None);
        assert_eq!(parse_colour("#12345g"), None);
    }
}
