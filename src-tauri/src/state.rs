//! Shared application state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::settings::Settings;

pub struct AppState {
    pub settings: Mutex<Settings>,
    /// Screen point the popup is anchored to, normally the mouse up position.
    pub anchor: Mutex<Option<(f64, f64)>>,
    /// Whether the global mouse hook is installed.
    pub hooked: AtomicBool,
    pub hook_error: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        AppState {
            settings: Mutex::new(settings),
            anchor: Mutex::new(None),
            hooked: AtomicBool::new(false),
            hook_error: Mutex::new(None),
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn set_settings(&self, settings: Settings) {
        if let Ok(mut guard) = self.settings.lock() {
            *guard = settings;
        }
    }

    pub fn anchor(&self) -> Option<(f64, f64)> {
        self.anchor.lock().ok().and_then(|guard| *guard)
    }

    pub fn set_anchor(&self, anchor: (f64, f64)) {
        if let Ok(mut guard) = self.anchor.lock() {
            *guard = Some(anchor);
        }
    }

    pub fn set_hook_error(&self, message: String) {
        if let Ok(mut guard) = self.hook_error.lock() {
            *guard = Some(message);
        }
        self.hooked.store(false, Ordering::Relaxed);
    }
}
