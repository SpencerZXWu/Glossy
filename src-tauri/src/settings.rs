//! User settings, persisted as JSON in the application config directory.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager};

/// Minimum selection length (in characters) that may trigger the popup.
pub const MIN_SELECTION_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Provider {
    /// Free public Google translate endpoint, no API key required.
    #[default]
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

/// The two credential fields under the names they carry in the JSON file, so
/// protecting and revealing can share one loop body.
fn credential_fields(credentials: &mut Credentials) -> [(&'static str, &mut String); 2] {
    [
        ("appId", &mut credentials.app_id),
        ("apiKey", &mut credentials.api_key),
    ]
}

/// Language the interface itself is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiLanguage {
    /// Follow the Windows display language.
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "en")]
    English,
}

/// Bundle identifier from `tauri.conf.json`. The settings file lives in
/// `%APPDATA%\<identifier>`, and a second launch has to read it before Tauri,
/// and therefore an `AppHandle`, exists.
const IDENTIFIER: &str = "com.glossy.translator";

/// The stored interface language preference, read without an `AppHandle`.
pub fn stored_ui_language() -> UiLanguage {
    #[derive(Deserialize)]
    struct Stored {
        #[serde(default, rename = "uiLang")]
        ui_lang: UiLanguage,
    }

    let Some(base) = std::env::var_os("APPDATA") else {
        return UiLanguage::default();
    };
    let path = PathBuf::from(base).join(IDENTIFIER).join("settings.json");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return UiLanguage::default();
    };
    serde_json::from_str::<Stored>(raw.trim_start_matches('\u{feff}'))
        .map(|stored| stored.ui_lang)
        .unwrap_or_default()
}

/// Turns the preference into the language the native parts actually speak,
/// resolving "follow Windows" the way the frontend's `i18n.resolve` does.
pub fn resolve_ui_language(preference: UiLanguage) -> UiLanguage {
    match preference {
        UiLanguage::System => {
            if crate::platform::user_locale()
                .to_lowercase()
                .starts_with("zh")
            {
                UiLanguage::Chinese
            } else {
                UiLanguage::English
            }
        }
        concrete => concrete,
    }
}

/// Colour scheme used by the settings window and the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    /// Follow the Windows / WebView2 preference.
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
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
    ///
    /// The whole clipboard is covered, not just its text: images and file lists
    /// survive the Ctrl+C that reads the selection.
    pub restore_clipboard: bool,
    /// Show the original text above the translation.
    pub show_original: bool,
    /// Shortest selection (in characters) that still opens the popup.
    pub min_selection_len: usize,
    /// Convert units in the selection that the reader of the target language
    /// would not use, and annotate the converted value.
    pub units_enabled: bool,
    /// Programs that never trigger a translation.
    #[serde(default, deserialize_with = "string_or_list")]
    pub ignored_apps: Vec<String>,
    /// Language of the interface itself.
    pub ui_lang: UiLanguage,
    /// Whether Glossy has never been started before. Only the very first
    /// launch opens the settings window; every later one goes straight to the
    /// notification area.
    pub first_run: bool,
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
    /// Start Glossy with Windows.
    pub autostart: bool,
    /// Ask GitHub for a newer release.
    pub check_updates: bool,
    /// How many translations the history keeps, 0 turns it off.
    pub history_limit: u32,
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
            units_enabled: true,
            ignored_apps: Vec::new(),
            ui_lang: UiLanguage::default(),
            first_run: true,
            theme: Theme::default(),
            font_scale: 100,
            popup_width: DEFAULT_POPUP_WIDTH,
            popup_opacity: DEFAULT_POPUP_OPACITY,
            auto_close_secs: 0,
            close_after_copy: false,
            autostart: false,
            check_updates: false,
            history_limit: 50,
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
            Ok(raw) => {
                let mut settings = Self::parse(&raw);
                if settings.reveal_credentials() {
                    // The file held plain text, or a credential this login
                    // cannot unlock. Both are gone from memory by now, so write
                    // the result back; failing to do so only means the next save
                    // is the one that cleans the file up.
                    if let Err(error) = settings.save(app) {
                        eprintln!("glossy: cannot rewrite the settings file: {error}");
                    }
                }
                settings
            }
            Err(_) => Settings::default(),
        }
    }

    /// Reads persisted JSON, falling back to the defaults for anything unusable.
    ///
    /// The file is merged into the defaults one setting at a time: a value that
    /// does not fit its setting (a hand edit, a number stored as a string, a
    /// provider that no longer exists) costs only that setting instead of
    /// resetting the whole file - which the next save would then write back as
    /// defaults, losing everything the user had configured.
    fn parse(raw: &str) -> Settings {
        // Editors on Windows like to write a UTF-8 byte order mark.
        let text = raw.trim_start_matches('\u{feff}');
        let Ok(serde_json::Value::Object(stored)) = serde_json::from_str(text) else {
            return Settings::default().sanitized();
        };

        let mut merged = serde_json::to_value(Settings::default())
            .ok()
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();

        for (key, value) in stored {
            // Keys written by older versions are kept: they are migrated into
            // `credentials` by `sanitized`, so they must survive the merge.
            if !merged.contains_key(&key) && !matches!(key.as_str(), "apiKey" | "appId") {
                continue;
            }
            // Deserializing the single setting tells whether it still fits; the
            // other fields come from the defaults and cannot fail the probe.
            let probe = serde_json::json!({ key.clone(): value.clone() });
            match serde_json::from_value::<Settings>(probe) {
                Ok(_) => {
                    merged.insert(key, value);
                }
                Err(error) => eprintln!("glossy: ignoring the stored setting `{key}`: {error}"),
            }
        }

        serde_json::from_value::<Settings>(serde_json::Value::Object(merged))
            .unwrap_or_default()
            .sanitized()
    }

    /// Reads a settings file written by `export_settings`.
    ///
    /// Unlike `parse`, which is meant for Glossy's own file and falls back to
    /// the defaults for anything unusable, this rejects a file that holds no
    /// setting at all: importing one would otherwise silently wipe the setup.
    pub fn import(raw: &str) -> Result<Settings, String> {
        let text = raw.trim_start_matches('\u{feff}');
        let Ok(serde_json::Value::Object(stored)) = serde_json::from_str(text) else {
            return Err("this file is not a JSON settings file".to_string());
        };
        let known = serde_json::to_value(Settings::default())
            .ok()
            .and_then(|value| value.as_object().cloned())
            .map(|defaults| defaults.keys().any(|key| stored.contains_key(key)))
            .unwrap_or(false);
        if !known {
            return Err("this file holds no Glossy setting".to_string());
        }
        Ok(Self::parse(text))
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let path = Self::path(app).ok_or_else(|| "config directory unavailable".to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&self.protected_for_storage())
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Replaces the stored API keys with the values they protect.
    ///
    /// Returns whether the file has to be written again: it held plain text
    /// written by an older version, or a credential that this Windows login
    /// cannot unlock and that was therefore dropped.
    fn reveal_credentials(&mut self) -> bool {
        let mut rewrite = false;
        for (provider, credentials) in self.credentials.iter_mut() {
            for (field, value) in credential_fields(credentials) {
                if value.is_empty() {
                    continue;
                }
                if !crate::secrets::is_protected(value) {
                    rewrite = true;
                    continue;
                }
                match crate::secrets::reveal(value) {
                    Some(plain) => *value = plain,
                    None => {
                        eprintln!(
                            "glossy: the {field} of `{provider}` was protected for another \
                             Windows login, enter it again"
                        );
                        value.clear();
                        rewrite = true;
                    }
                }
            }
        }
        rewrite
    }

    /// The copy written to disk, with every credential DPAPI protected.
    fn protected_for_storage(&self) -> Settings {
        let mut stored = self.clone();
        for (provider, credentials) in stored.credentials.iter_mut() {
            for (field, value) in credential_fields(credentials) {
                if value.is_empty() || crate::secrets::is_protected(value) {
                    continue;
                }
                match crate::secrets::protect(value) {
                    Ok(protected) => *value = protected,
                    // Storing the key unprotected beats losing it; DPAPI is part
                    // of Windows, so this only happens in a broken environment.
                    Err(error) => {
                        eprintln!("glossy: cannot protect the {field} of `{provider}`: {error}")
                    }
                }
            }
        }
        stored
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
        } else {
            // A code from another vendor (`jp`, `ZH-HANS`) or one written by
            // hand has to match an entry of the language menu, otherwise the
            // dropdown has nothing to show for it.
            self.target_lang = crate::translate::normalize_lang_code(&self.target_lang);
        }
        self.migrate_legacy_credentials();
        self.min_selection_len = self.min_selection_len.clamp(1, 40);
        self.ignored_apps = ignored_processes(&self.ignored_apps);
        self.font_scale = self.font_scale.clamp(80, 160);
        self.popup_width = self.popup_width.clamp(280, 560);
        self.popup_opacity = self.popup_opacity.clamp(50, 100);
        self.auto_close_secs = self.auto_close_secs.min(600);
        self.history_limit = self.history_limit.min(crate::history::MAX_LIMIT);
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
    fn the_hard_coded_identifier_matches_the_bundle_identifier() {
        // `stored_ui_language` finds the settings file without an AppHandle, so
        // it has to hard-code the directory Tauri derives from this value.
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("the config is JSON");

        assert_eq!(conf["identifier"], serde_json::json!(IDENTIFIER));
    }

    #[test]
    fn the_stored_language_is_read_from_the_ui_lang_key() {
        let json = serde_json::to_string(&Settings::default()).expect("settings serialize");

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).expect("valid JSON")["uiLang"],
            serde_json::json!("system")
        );
    }

    #[test]
    fn only_the_system_preference_is_resolved_against_the_locale() {
        assert_eq!(
            resolve_ui_language(UiLanguage::Chinese),
            UiLanguage::Chinese
        );
        assert_eq!(
            resolve_ui_language(UiLanguage::English),
            UiLanguage::English
        );
        assert!(matches!(
            resolve_ui_language(UiLanguage::System),
            UiLanguage::Chinese | UiLanguage::English
        ));
    }

    #[test]
    fn one_unreadable_setting_keeps_every_other_setting() {
        // `popupWidth` was written by hand as a word: only that setting may
        // fall back to its default.
        let settings = Settings::parse(
            "{\"enabled\":false,\"targetLang\":\"ja\",\"popupWidth\":\"wide\",\
             \"hotkey\":\"Ctrl+Alt+Z\"}",
        );

        assert!(!settings.enabled);
        assert_eq!(settings.target_lang, "ja");
        assert_eq!(settings.hotkey, "Ctrl+Alt+Z");
        assert_eq!(settings.popup_width, DEFAULT_POPUP_WIDTH);
    }

    #[test]
    fn one_unreadable_setting_keeps_the_saved_credentials() {
        let settings = Settings::parse(
            "{\"provider\":\"baidu\",\"fontScale\":\"big\",\
             \"credentials\":{\"baidu\":{\"appId\":\" 2024 \",\"apiKey\":\" secret \"}}}",
        );

        assert_eq!(settings.provider, Provider::Baidu);
        assert_eq!(settings.font_scale, 100);
        let baidu = settings.active_credentials();
        assert_eq!(baidu.app_id, "2024");
        assert_eq!(baidu.api_key, "secret");
    }

    #[test]
    fn ignores_settings_that_no_longer_exist() {
        let settings =
            Settings::parse("{\"unknownSetting\":42,\"targetLang\":\"fr\",\"provider\":\"gone\"}");

        assert_eq!(settings.target_lang, "fr");
        // A provider that was removed falls back to the default one, and the
        // remaining settings are untouched.
        assert_eq!(settings.provider, Provider::default());
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
    fn the_settings_window_opens_only_on_the_very_first_launch() {
        // Nothing stored yet, and a file that predates the flag: both mean the
        // settings window still has to be shown.
        assert!(Settings::default().first_run);
        assert!(Settings::parse("{\"enabled\":true}").first_run);

        let recorded = Settings::parse("{\"firstRun\":false}");
        assert!(!recorded.first_run);

        let json = serde_json::to_string(&Settings::default()).unwrap();
        assert!(json.contains("\"firstRun\":true"));
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
        assert_eq!(settings.active_credentials().api_key, "old-key".to_string());

        // Switching provider and back keeps the key.
        settings.provider = Provider::Google;
        settings = settings.sanitized();
        assert!(settings.active_credentials().is_empty());
        assert_eq!(
            settings.credentials.get("zhipu").unwrap().api_key,
            "old-key"
        );

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

    fn settings_with_one_key() -> Settings {
        Settings {
            provider: Provider::DeepL,
            credentials: [(
                "deepl".to_string(),
                Credentials {
                    app_id: String::new(),
                    api_key: "sk-secret".to_string(),
                },
            )]
            .into_iter()
            .collect(),
            ..Settings::default()
        }
    }

    #[test]
    fn writes_the_keys_dpapi_protected_and_reads_them_back() {
        let settings = settings_with_one_key();

        let stored = settings.protected_for_storage();

        let written = &stored.credentials.get("deepl").unwrap().api_key;
        assert!(crate::secrets::is_protected(written));
        assert!(!serde_json::to_string(&stored)
            .unwrap()
            .contains("sk-secret"));

        // What the rest of the application keeps in memory stays usable.
        assert_eq!(settings.active_credentials().api_key, "sk-secret");

        let mut reloaded = stored;
        // Nothing to rewrite: the file is already in its final shape.
        assert!(!reloaded.reveal_credentials());
        assert_eq!(reloaded.active_credentials().api_key, "sk-secret");
    }

    #[test]
    fn the_next_save_protects_a_key_written_by_an_older_version() {
        let mut settings = Settings::parse("{\"provider\":\"deepl\",\"apiKey\":\"sk-old\"}");
        // `parse` only reads the JSON, so the key is still plain text here.
        assert_eq!(settings.active_credentials().api_key, "sk-old");

        // Which is what makes `load` rewrite the file.
        assert!(settings.reveal_credentials());
        assert_eq!(settings.active_credentials().api_key, "sk-old");
        assert!(crate::secrets::is_protected(
            &settings
                .protected_for_storage()
                .credentials
                .get("deepl")
                .unwrap()
                .api_key
        ));
    }

    #[test]
    fn forgets_a_key_that_belongs_to_another_windows_login() {
        // Well-formed base64 that DPAPI cannot unlock in this login.
        let raw = "{\"provider\":\"deepl\",\"credentials\":{\"deepl\":\
                   {\"apiKey\":\"dpapi:bm90IGEgYmxvYg==\"}}}";
        let mut settings = Settings::parse(raw);
        assert!(crate::secrets::is_protected(
            &settings.active_credentials().api_key
        ));

        // The file has to be rewritten so the unusable blob stops being read.
        assert!(settings.reveal_credentials());
        assert!(settings.active_credentials().is_empty());
    }

    #[test]
    fn imports_a_file_this_app_exported() {
        let raw = serde_json::to_string_pretty(&Settings::parse(
            "{\"targetLang\":\"ja\",\"provider\":\"deepl\",\"apiKey\":\"sk-exported\"}",
        ))
        .unwrap();

        let imported = Settings::import(&raw).unwrap();
        assert_eq!(imported.target_lang, "ja");
        assert_eq!(imported.active_credentials().api_key, "sk-exported");
    }

    #[test]
    fn refuses_a_file_that_holds_no_setting() {
        assert!(Settings::import("not json at all").is_err());
        assert!(Settings::import("[1, 2, 3]").is_err());
        assert!(Settings::import("{\"somethingElse\":true}").is_err());
    }
}
