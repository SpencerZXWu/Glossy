//! User settings, persisted as JSON in the application config directory.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager};

/// Minimum selection length (in characters) that may trigger the popup.
pub const MIN_SELECTION_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    /// Free public Google translate endpoint, no API key required.
    #[serde(rename = "google")]
    Google,
    /// Zhipu GLM chat completions, free tier, requires an API key.
    #[serde(rename = "zhipu")]
    Zhipu,
    /// Baidu Translate, free monthly quota, requires an APP ID and a key.
    #[serde(rename = "baidu")]
    Baidu,
    /// DeepL, requires an API key.
    #[serde(rename = "deepl")]
    DeepL,
    /// OpenAI chat completions, requires an API key.
    #[serde(rename = "openai")]
    OpenAI,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::Google
    }
}

impl Provider {
    /// Stable name used as the key of the saved credentials.
    pub fn key(self) -> &'static str {
        match self {
            Provider::Google => "google",
            Provider::Zhipu => "zhipu",
            Provider::Baidu => "baidu",
            Provider::DeepL => "deepl",
            Provider::OpenAI => "openai",
        }
    }
}

/// Credentials of one provider, kept so switching back just works.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Credentials {
    /// Baidu only: the public half of the credential pair.
    pub app_id: String,
    /// API key, or Baidu's secret half.
    pub api_key: String,
}

impl Credentials {
    fn is_empty(&self) -> bool {
        self.app_id.is_empty() && self.api_key.is_empty()
    }

    fn trimmed(&self) -> Credentials {
        Credentials {
            app_id: self.app_id.trim().to_string(),
            api_key: self.api_key.trim().to_string(),
        }
    }
}

/// Language the interface itself is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiLanguage {
    /// Follow the Windows display language.
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "en")]
    English,
}

impl Default for UiLanguage {
    fn default() -> Self {
        UiLanguage::System
    }
}

/// Colour scheme used by the settings window and the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    /// Follow the Windows / WebView2 preference.
    #[serde(rename = "system")]
    System,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

/// Default popup card width in CSS pixels, kept in sync with `popup.css`.
const DEFAULT_POPUP_WIDTH: u32 = 356;

/// Default opacity (in percent) of the popup card.
const DEFAULT_POPUP_OPACITY: u32 = 100;

/// Normalizes the ignored-app list: lower case, no duplicates, no separators
/// left inside an entry.
///
/// `code` and `code.exe` describe the same program, so both spellings collapse
/// into the first one that was written.
pub fn ignored_processes(names: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for raw in names {
        for part in raw.split([',', ';', ' ', '\t', '\n', '\r']) {
            let name = part.trim().to_ascii_lowercase();
            if name.is_empty() {
                continue;
            }
            let stem = name.strip_suffix(".exe").unwrap_or(&name);
            if !result
                .iter()
                .any(|existing| existing.strip_suffix(".exe").unwrap_or(existing) == stem)
            {
                result.push(name);
            }
        }
    }
    result
}

/// True when the foreground application is on the ignored list.
///
/// Entries match the executable name with or without its `.exe` suffix.
pub fn ignores_process(names: &[String], process: &str) -> bool {
    let process = process.trim().to_ascii_lowercase();
    if process.is_empty() {
        return false;
    }
    let stem = process.strip_suffix(".exe").unwrap_or(&process);
    ignored_processes(names)
        .iter()
        .any(|name| name.strip_suffix(".exe").unwrap_or(name) == stem)
}

/// Accepts both the comma separated string older versions wrote and the list
/// the current version stores.
fn string_or_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        List(Vec<String>),
        Text(String),
    }

    Ok(match Option::<Raw>::deserialize(deserializer)? {
        None => Vec::new(),
        Some(Raw::List(list)) => list,
        Some(Raw::Text(text)) => text.split([',', ';']).map(str::to_string).collect(),
    })
}

fn default_target_lang() -> String {
    let locale = crate::platform::user_locale();
    let lower = locale.to_ascii_lowercase();
    if lower.starts_with("zh") {
        if lower.contains("tw") || lower.contains("hk") || lower.contains("hant") {
            "zh-TW".to_string()
        } else {
            "zh-CN".to_string()
        }
    } else if lower.starts_with("ja") {
        "ja".to_string()
    } else if lower.starts_with("ko") {
        "ko".to_string()
    } else if lower.starts_with("en") {
        "en".to_string()
    } else {
        "en".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Master switch for the select-to-translate feature.
    pub enabled: bool,
    /// Trigger when the mouse drags across text.
    pub trigger_on_drag: bool,
    /// Trigger when a word is double clicked.
    pub trigger_on_double_click: bool,
    /// Language the selection is translated into.
    pub target_lang: String,
    /// Translation backend.
    pub provider: Provider,
    /// Credentials of every provider that was configured so far.
    pub credentials: BTreeMap<String, Credentials>,
    /// API key written by versions that only kept one credential pair.
    #[serde(rename = "apiKey", default, skip_serializing)]
    pub legacy_api_key: String,
    /// Baidu APP ID written by versions that only kept one credential pair.
    #[serde(rename = "appId", default, skip_serializing)]
    pub legacy_app_id: String,
    /// Put the previous clipboard content back after reading a selection.
    pub restore_clipboard: bool,
    /// Show the original text above the translation.
    pub show_original: bool,
    /// Shortest selection (in characters) that still opens the popup.
    pub min_selection_len: usize,
    /// Programs that never trigger a translation.
    #[serde(default, deserialize_with = "string_or_list")]
    pub ignored_apps: Vec<String>,
    /// Language of the interface itself.
    pub ui_lang: UiLanguage,
    /// Colour scheme for both windows.
    pub theme: Theme,
    /// Popup text size as a percentage of the default.
    pub font_scale: u32,
    /// Popup card width in CSS pixels.
    pub popup_width: u32,
    /// Opacity of the popup card, in percent.
    pub popup_opacity: u32,
    /// Seconds before the popup closes itself, 0 keeps it open.
    pub auto_close_secs: u64,
    /// Hide the popup right after the translation was copied.
    pub close_after_copy: bool,
    /// Accelerator such as `Ctrl+Alt+C` that translates the clipboard.
    pub hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            enabled: true,
            trigger_on_drag: true,
            trigger_on_double_click: true,
            target_lang: default_target_lang(),
            provider: Provider::default(),
            credentials: BTreeMap::new(),
            legacy_api_key: String::new(),
            legacy_app_id: String::new(),
            restore_clipboard: true,
            show_original: true,
            min_selection_len: MIN_SELECTION_LEN,
            ignored_apps: Vec::new(),
            ui_lang: UiLanguage::default(),
            theme: Theme::default(),
            font_scale: 100,
            popup_width: DEFAULT_POPUP_WIDTH,
            popup_opacity: DEFAULT_POPUP_OPACITY,
            auto_close_secs: 0,
            close_after_copy: false,
            hotkey: "Ctrl+Alt+C".to_string(),
        }
    }
}

impl Settings {
    fn path(app: &AppHandle) -> Option<PathBuf> {
        app.path()
            .app_config_dir()
            .ok()
            .map(|dir| dir.join("settings.json"))
    }

    pub fn load(app: &AppHandle) -> Settings {
        let Some(path) = Self::path(app) else {
            return Settings::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(raw) => Self::parse(&raw),
            Err(_) => Settings::default(),
        }
    }

    /// Reads persisted JSON, falling back to the defaults for anything unusable.
    fn parse(raw: &str) -> Settings {
        // Editors on Windows like to write a UTF-8 byte order mark.
        let settings: Settings =
            serde_json::from_str(raw.trim_start_matches('\u{feff}')).unwrap_or_default();
        // Out of range or badly spelled values in the file are repaired right
        // away, so the settings window never shows something unusable.
        settings.sanitized()
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let path = Self::path(app).ok_or_else(|| "config directory unavailable".to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Credentials saved for the provider that is currently selected.
    pub fn active_credentials(&self) -> Credentials {
        self.credentials
            .get(self.provider.key())
            .cloned()
            .unwrap_or_default()
    }

    /// Moves the single credential pair written by older versions into the
    /// per-provider map, so it stays available after switching away.
    fn migrate_legacy_credentials(&mut self) {
        let legacy = Credentials {
            app_id: std::mem::take(&mut self.legacy_app_id),
            api_key: std::mem::take(&mut self.legacy_api_key),
        };
        if legacy.is_empty() {
            return;
        }
        let stored = self
            .credentials
            .entry(self.provider.key().to_string())
            .or_default();
        if stored.api_key.is_empty() {
            stored.api_key = legacy.api_key;
        }
        if stored.app_id.is_empty() {
            stored.app_id = legacy.app_id;
        }
    }

    pub fn sanitized(mut self) -> Settings {
        if self.target_lang.trim().is_empty() {
            self.target_lang = default_target_lang();
        }
        self.migrate_legacy_credentials();
        self.min_selection_len = self.min_selection_len.clamp(1, 40);
        self.ignored_apps = ignored_processes(&self.ignored_apps);
        self.font_scale = self.font_scale.clamp(80, 160);
        self.popup_width = self.popup_width.clamp(280, 560);
        self.popup_opacity = self.popup_opacity.clamp(50, 100);
        self.auto_close_secs = self.auto_close_secs.min(600);
        self.hotkey = self.hotkey.trim().to_string();
        self.credentials = self
            .credentials
            .iter()
            // The provider names come from the settings file, so normalize the
            // spelling the same way the rest of the settings are normalized.
            .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trimmed()))
            .filter(|(name, value)| !name.is_empty() && !value.is_empty())
            .collect();
        if !self.trigger_on_drag && !self.trigger_on_double_click {
            self.trigger_on_drag = true;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_previous_settings_when_the_file_starts_with_a_byte_order_mark() {
        let raw = "\u{feff}{\"targetLang\":\"ja\",\"restoreClipboard\":false}";
        let settings = Settings::parse(raw);

        assert_eq!(settings.target_lang, "ja");
        assert!(!settings.restore_clipboard);
    }

    #[test]
    fn falls_back_to_the_defaults_for_an_unreadable_file() {
        let settings = Settings::parse("{ this is not json");

        assert_eq!(settings.target_lang, Settings::default().target_lang);
        assert!(settings.enabled);
    }

    #[test]
    fn keeps_the_defaults_for_settings_written_by_an_older_version() {
        let settings = Settings::parse("{\"enabled\":true,\"provider\":\"zhipu\"}");

        assert_eq!(settings.min_selection_len, MIN_SELECTION_LEN);
        assert_eq!(settings.popup_width, DEFAULT_POPUP_WIDTH);
        assert_eq!(settings.popup_opacity, DEFAULT_POPUP_OPACITY);
        assert_eq!(settings.font_scale, 100);
        assert_eq!(settings.theme, Theme::System);
        assert!(!settings.close_after_copy);
    }

    #[test]
    fn repairs_out_of_range_values() {
        let settings = Settings {
            min_selection_len: 0,
            font_scale: 400,
            popup_width: 40,
            popup_opacity: 4,
            auto_close_secs: 100_000,
            ignored_apps: vec![
                "  Code.exe ".to_string(),
                "code".to_string(),
                " mstsc".to_string(),
            ],
            hotkey: "  Ctrl+Alt+C  ".to_string(),
            ..Settings::default()
        }
        .sanitized();

        assert_eq!(settings.min_selection_len, 1);
        assert_eq!(settings.font_scale, 160);
        assert_eq!(settings.popup_width, 280);
        assert_eq!(settings.popup_opacity, 50);
        assert_eq!(settings.auto_close_secs, 600);
        assert_eq!(settings.ignored_apps, vec!["code.exe", "mstsc"]);
        assert_eq!(settings.hotkey, "Ctrl+Alt+C");
    }

    #[test]
    fn matches_ignored_apps_with_or_without_the_exe_suffix() {
        let list = vec!["code.exe".to_string(), "mstsc".to_string()];

        assert!(ignores_process(&list, "Code.exe"));
        assert!(ignores_process(&list, "code"));
        assert!(ignores_process(&list, "MSTSC.EXE"));
        assert!(!ignores_process(&list, "chrome.exe"));
        assert!(!ignores_process(&[], "chrome.exe"));
        assert!(!ignores_process(&list, "  "));
    }

    #[test]
    fn reads_the_ignored_apps_of_an_older_version() {
        let settings = Settings::parse("{\"ignoredApps\":\"  Code.exe ,, code , mstsc \"}");

        assert_eq!(settings.ignored_apps, vec!["code.exe", "mstsc"]);
    }

    #[test]
    fn keeps_a_single_api_key_next_to_the_per_provider_map() {
        let mut settings = Settings::parse("{\"provider\":\"zhipu\",\"apiKey\":\" old-key \"}");
        assert_eq!(
            settings.active_credentials().api_key,
            "old-key".to_string()
        );

        // Switching provider and back keeps the key.
        settings.provider = Provider::Google;
        settings = settings.sanitized();
        assert!(settings.active_credentials().is_empty());
        assert_eq!(settings.credentials.get("zhipu").unwrap().api_key, "old-key");

        settings.provider = Provider::Zhipu;
        settings = settings.sanitized();
        assert_eq!(settings.active_credentials().api_key, "old-key");
    }

    #[test]
    fn forgets_empty_credentials() {
        let settings = Settings {
            credentials: [
                (
                    "baidu".to_string(),
                    Credentials {
                        app_id: " 2024 ".to_string(),
                        api_key: " secret ".to_string(),
                    },
                ),
                ("deepl".to_string(), Credentials::default()),
            ]
            .into_iter()
            .collect(),
            ..Settings::default()
        }
        .sanitized();

        assert_eq!(settings.credentials.len(), 1);
        let baidu = settings.credentials.get("baidu").unwrap();
        assert_eq!(baidu.app_id, "2024");
        assert_eq!(baidu.api_key, "secret");
    }
}
