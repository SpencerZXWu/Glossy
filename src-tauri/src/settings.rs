//! User settings, persisted as JSON in the application config directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager};

use crate::log::note;

/// Minimum selection length (in characters) that may trigger the popup.
pub const MIN_SELECTION_LEN: usize = 2;

/// Version of the settings file format, written into every file as
/// `formatVersion`.
///
/// `contract/contract.json` publishes this number next to the keys this build
/// writes and the IPC commands it answers to, and a test fails when the file and
/// the code disagree. The number only moves when the stored shape changes in a
/// way `parse` cannot absorb on its own â€” a key that is renamed or removed, or
/// one that changes meaning. Added keys come from `#[serde(default)]` and leave
/// the number alone. When it does move, `migrate` carries every file written
/// before it up to it, so a settings file from any `1.3.0` or later release
/// keeps loading.
///
/// Version 2 is the release that folds the four fields naming the translation
/// service â€” `channel`, `cloudProvider`, `cloudVendor` and `provider` â€” into the
/// single `service` this build writes, and drops the two fields nothing has read
/// for years: the address of the relay, which the build carries, and the
/// credential map, which no service in this build asks for. Everything a file
/// said with them is migrated into `service`; nothing else about a `1.x` file
/// changes.
pub const FORMAT_VERSION: u32 = 2;

/// Where a translation comes from.
///
/// The three channels are the three ways a desktop app can offer translation:
/// on somebody else's account, on the user's own, or on this machine. Every
/// vendor forbids handing a free allowance on to third parties, so the cloud
/// channel cannot simply resell one of their keys; it runs on an account Glossy
/// pays for, or on a model running on this machine.
///
/// Version 1 of the file stored this as `channel` plus three more fields; the
/// `service` they added up to is what version 2 keeps, which is what
/// [`Service`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Service {
    /// Glossy's own server, asking Baidu.
    #[serde(rename = "cloud-baidu")]
    CloudBaidu,
    /// Glossy's own server, asking Youdao.
    #[serde(rename = "cloud-youdao")]
    CloudYoudao,
    /// The free public Google endpoint.
    Google,
    /// Baidu, on the APP ID and key the reader filled in themselves. The same
    /// engine the relay asks, on somebody else's account: which one paid is
    /// what the two entries are named for.
    #[serde(rename = "api-baidu")]
    ApiBaidu,
    /// An OpenAI compatible endpoint the reader named, which is one entry
    /// because one entry covers every service that speaks that protocol.
    #[serde(rename = "api-openai")]
    ApiOpenAi,
    /// The models on this machine, which translate Chinese and English and
    /// nothing else.
    Offline,
}

impl Service {
    /// Every entry, in the order the dropdown shows them.
    ///
    /// The window builds its dropdown from the markup, so what walks this list
    /// is [`Service::from_id`] and the tests. The two entries that need
    /// credentials of the reader's own are in it as well: what keeps them off
    /// the dropdown until they are filled in is the window and
    /// [`Settings::sanitized`], not this list.
    pub const ALL: [Service; 6] = [
        Service::CloudBaidu,
        Service::CloudYoudao,
        Service::Google,
        Service::ApiBaidu,
        Service::ApiOpenAi,
        Service::Offline,
    ];

    /// The id the window and the settings file use.
    pub fn id(self) -> &'static str {
        match self {
            Service::CloudBaidu => "cloud-baidu",
            Service::CloudYoudao => "cloud-youdao",
            Service::Google => "google",
            Service::ApiBaidu => "api-baidu",
            Service::ApiOpenAi => "api-openai",
            Service::Offline => "offline",
        }
    }

    /// The name of the engine that answers, the way a result carries it.
    ///
    /// The two cloud entries are one relay in front of two engines, so `id` â€”
    /// which names the *entry* â€” is not what a card should show: what answered
    /// is Baidu or Youdao. The two entries of the reader's own are named after
    /// the protocol for the same reason: the endpoint behind `api-openai` is
    /// whatever they pointed it at, and which account paid is the one thing the
    /// card can say for certain.
    pub fn provider(self) -> &'static str {
        match self {
            Service::CloudBaidu => "baidu",
            Service::CloudYoudao => "youdao",
            Service::Google => "google",
            Service::ApiBaidu => "api-baidu",
            Service::ApiOpenAi => "api-openai",
            Service::Offline => "offline",
        }
    }

    /// Whether the entry only exists once the reader has filled something in.
    pub fn needs_credentials(self) -> bool {
        matches!(self, Service::ApiBaidu | Service::ApiOpenAi)
    }

    /// The entry an id names; the inverse of [`Service::id`].
    ///
    /// `None` means the id is not one of the four, which is what a window or a
    /// shortcut carrying a stale id has to hear instead of a silent fallback.
    pub fn from_id(id: &str) -> Option<Service> {
        let wanted = id.trim();
        Service::ALL
            .iter()
            .copied()
            .find(|service| service.id() == wanted)
    }

    /// The service a file written before the one-field shape named.
    ///
    /// A `1.x` file spread the choice over `channel`, `cloudProvider`,
    /// `cloudVendor` and `provider`, and named entries this build no longer
    /// offers â€” a provider that wanted the user's own key, a vendor the server
    /// dropped, a model reached over an OpenAI-compatible endpoint. All of them
    /// read as the entry that is nearest to what they meant, so a window always
    /// has something to show and the first save writes the current shape.
    ///
    /// `cloudProvider` is not read at all: its two values were the built-in
    /// engine, which is the only one of the two this build has, and a file that
    /// named the local model was already read as the built-in engine in `1.x`.
    ///
    /// `migrate` is the only caller: this is the version-to-version bridge, not
    /// part of reading a file this build wrote.
    pub fn from_legacy(
        channel: Option<&str>,
        cloud_vendor: Option<&str>,
        provider: Option<&str>,
    ) -> Self {
        let named = |value: Option<&str>, wanted: &str| {
            value
                .map(|value| value.trim().eq_ignore_ascii_case(wanted))
                .unwrap_or(false)
        };
        // The two windows of `1.x` that went somewhere other than Glossy's own
        // server: this machine, and the free endpoint.
        if named(channel, "offline") {
            return Service::Offline;
        }
        // The api channel was the free endpoint and everything that wanted a
        // key of the user's; only the free endpoint is an entry now, and a
        // provider this build does not offer reads as the built-in engine.
        if named(channel, "api") {
            return if named(provider, "google") {
                Service::Google
            } else {
                Service::CloudBaidu
            };
        }
        // A file written before the channels existed names only a provider, and
        // the free endpoint is the one of those that is an entry of its own.
        if channel.is_none() && named(provider, "google") {
            return Service::Google;
        }
        if named(cloud_vendor, "youdao") {
            return Service::CloudYoudao;
        }
        Service::CloudBaidu
    }
}

/// The pair of credentials Baidu hands out, both of which it needs.
///
/// The values are the reader's own: what is in memory is the key, and what
/// reaches the disk is DPAPI protected (see `platform::secrets`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Credentials {
    /// Baidu only: the public half of the credential pair.
    pub app_id: String,
    /// API key, or Baidu's secret half.
    pub api_key: String,
}

impl Credentials {
    /// Whether either half is empty, which is what "not filled in yet" means.
    pub fn is_empty(&self) -> bool {
        self.app_id.trim().is_empty() || self.api_key.trim().is_empty()
    }

    fn trimmed(&self) -> Credentials {
        Credentials {
            app_id: self.app_id.trim().to_string(),
            api_key: self.api_key.trim().to_string(),
        }
    }

    /// The two fields under the names they carry in the JSON file, so the
    /// protect-and-reveal pass does not have to name them twice.
    fn fields(&mut self) -> [(&'static str, &mut String); 2] {
        [("appId", &mut self.app_id), ("apiKey", &mut self.api_key)]
    }
}

/// An OpenAI compatible endpoint the reader named, and the key it takes.
///
/// One entry covers every service that speaks the chat completions protocol:
/// OpenAI, DeepSeek, Zhipu, a model server on this machine. The address is kept
/// as the reader wrote it, with the path up to the version (`â€¦/v1`); the
/// completion path is added to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChatApi {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

/// Address a fresh install points the OpenAI compatible entry at.
pub const DEFAULT_CHAT_ENDPOINT: &str = "https://api.openai.com/v1";

impl Default for ChatApi {
    fn default() -> Self {
        ChatApi {
            endpoint: DEFAULT_CHAT_ENDPOINT.to_string(),
            api_key: String::new(),
            model: String::new(),
        }
    }
}

impl ChatApi {
    /// Whether this entry is filled in enough to be offered.
    ///
    /// The key and the model are what it takes: the address has a default, and
    /// a model the reader has not named is a request that cannot be made.
    pub fn is_empty(&self) -> bool {
        self.api_key.trim().is_empty() || self.model.trim().is_empty()
    }

    fn trimmed(&self) -> ChatApi {
        ChatApi {
            endpoint: self.endpoint.trim().trim_end_matches('/').to_string(),
            api_key: self.api_key.trim().to_string(),
            model: self.model.trim().to_string(),
        }
    }

    /// The completion URL this entry asks, which the client builds and the
    /// panel shows, so both mean the same request.
    pub fn completion_url(&self) -> String {
        format!("{}/chat/completions", self.endpoint.trim_end_matches('/'))
    }

    /// The one field of this entry that is a secret.
    ///
    /// The address and the model are configuration rather than credentials:
    /// protecting them would only make the file unreadable to the person who
    /// needs to see which service it points at.
    fn fields(&mut self) -> [(&'static str, &mut String); 1] {
        [("apiKey", &mut self.api_key)]
    }
}

/// The credentials the chosen entry translates with.
///
/// Borrowed rather than cloned: a caller that needs them is on the way to make
/// a request, and a key copied about the process is a key in more places than it
/// has to be.
pub enum ApiCredentials<'a> {
    Baidu(&'a Credentials),
    Chat(&'a ChatApi),
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

/// The folder the app keeps its own files in, under `%APPDATA%`.
///
/// It is derived rather than asked of Tauri because two callers need it before
/// there is an `AppHandle` to ask: the interface language is read before the app
/// is built, and a translation asks for the offline pack from the middle of the
/// translation path, where no window is in sight. `None` on a machine whose
/// `APPDATA` is not set, which is a machine the app cannot run on anyway.
pub fn data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join(IDENTIFIER))
}

/// The stored interface language preference, read without an `AppHandle`.
pub fn stored_ui_language() -> UiLanguage {
    #[derive(Deserialize)]
    struct Stored {
        #[serde(default, rename = "uiLang")]
        ui_lang: UiLanguage,
    }

    let Some(path) = data_dir().map(|dir| dir.join("settings.json")) else {
        return UiLanguage::default();
    };
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
            if crate::platform::desktop::user_locale()
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

/// Which colours the two windows are drawn in.
///
/// `Theme` decides light or dark; this decides the hues. The default is the
/// WinUI palette the stylesheet carries in `:root`, and every other entry is a
/// block of the same semantic tokens in `tokens.css` â€” so a palette changes the
/// colours and never the layering. Each one states both halves, which is why
/// the two settings are separate rather than one list of twelve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Palette {
    /// The WinUI colours, and the one the app has always used.
    #[default]
    Default,
    /// Warm paper, for reading under a lamp.
    Sepia,
    /// Cool blue-grey, for a bright office screen.
    Nord,
    /// The low-glare Solarized scheme.
    Solarized,
    /// The night scheme, purple where the accent goes.
    Dracula,
    /// True black, for an OLED panel.
    Amoled,
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

/// Normalizes the source-language list: canonical codes, no duplicates, no
/// separators left inside an entry.
pub fn language_codes(names: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for raw in names {
        for part in raw.split([',', ';', ' ', '\t', '\n', '\r']) {
            let code = crate::translate::normalize_lang_code(part);
            if !code.is_empty() && !result.contains(&code) {
                result.push(code);
            }
        }
    }
    result
}

/// True when a selection in `text` may still be translated.
///
/// Only the "translate these source languages" setting is consulted. An empty
/// list means every language, and a text whose language cannot be told apart is
/// let through, because a wrong guess must not swallow a selection the user
/// asked to translate.
pub fn allows_source(names: &[String], text: &str) -> bool {
    let wanted = language_codes(names);
    if wanted.is_empty() {
        return true;
    }
    match crate::lang::detect(text) {
        Some(detected) => wanted
            .iter()
            .any(|code| crate::lang::same_base(code, &detected)),
        None => true,
    }
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
    let locale = crate::platform::desktop::user_locale();
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
    /// Version of the format this file was written in, so a later release can
    /// tell what it has to migrate and an older one what it cannot read.
    pub format_version: u32,
    /// Master switch for the select-to-translate feature.
    pub enabled: bool,
    /// Trigger when the mouse drags across text.
    pub trigger_on_drag: bool,
    /// Trigger when a word is double clicked.
    pub trigger_on_double_click: bool,
    /// Language the selection is translated into.
    pub target_lang: String,
    /// Which service translates.
    ///
    /// One field for one choice. A `1.x` file spread it over `channel`,
    /// `cloudProvider`, `cloudVendor` and `provider`, which `migrate` folds
    /// into this.
    pub service: Service,
    /// Baidu credentials the reader filled in themselves.
    ///
    /// Empty until they do, and the entry that uses them is not offered until
    /// then: see [`Service::needs_credentials`].
    #[serde(default)]
    pub api_baidu: Credentials,
    /// The OpenAI compatible endpoint the reader named, on the same terms.
    #[serde(default)]
    pub api_openai: ChatApi,
    /// Random identifier of this installation, so the proxy can count the daily
    /// characters of one device. Derived from the machine rather than drawn at
    /// random, so installing the app again does not land on a new allowance.
    pub cloud_id: String,
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
    /// Source languages that still trigger a translation. Empty means every
    /// language, and a selection whose language cannot be told is translated as
    /// well, so a wrong guess never swallows a selection.
    #[serde(default, deserialize_with = "string_or_list")]
    pub source_langs: Vec<String>,
    /// Language of the interface itself.
    pub ui_lang: UiLanguage,
    /// Whether Glossy has never been started before. Only the very first
    /// launch opens the settings window; every later one goes straight to the
    /// notification area.
    pub first_run: bool,
    /// Whether the first-run walkthrough has been shown.
    ///
    /// It is written when the guide opens rather than when it is finished, for
    /// the same reason as [`Self::first_run`]: a window closed a second later
    /// has still shown it, and the way back to it is the button on the
    /// **General** page.
    pub guide_seen: bool,
    /// Colour scheme for both windows.
    pub theme: Theme,
    /// Which palette those colours come from; `theme` still decides whether the
    /// light or the dark half of it is used.
    pub palette: Palette,
    /// Accent colour the palette's own is replaced with, as `#rrggbb`. Empty
    /// means the palette decides, which is what a new install has.
    pub accent: String,
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
    /// Accelerator such as `Ctrl+Alt+G` that brings the settings window up.
    pub hotkey_settings: String,
    /// Accelerator such as `Ctrl+Alt+Q` that starts a screenshot translation.
    pub hotkey_ocr: String,
    /// Which languages the on-machine recogniser may read with.
    ///
    /// Each language is a downloadable pack of two files (the recogniser and
    /// its dictionary), and every language named here is tried on a screenshot:
    /// the reading each of them gives a line is weighed against the others and
    /// the best one is kept, so a screenshot holding Japanese and English comes
    /// back right without the user having to say which it is. The list always
    /// holds at least one language; a name this build does not offer is
    /// dropped.
    pub ocr_packs: Vec<String>,
    /// Whether the features that are still being built are on show.
    ///
    /// Nothing about it reaches a user: the switch is turned on by typing a key
    /// into its field, and a build in which it is off behaves exactly as it did
    /// before any of those features existed. What it hides is the **Subtitles**
    /// page, which reads a part of the screen over and over.
    pub developer_mode: bool,
    /// The language the subtitles are written in.
    pub subtitle_source_lang: String,
    /// The language the subtitle translation is read in.
    pub subtitle_target_lang: String,
    /// Size of the subtitle text in the translation box, in pixels.
    pub subtitle_font_size: u32,
    /// The one recognition language subtitle reading uses.
    ///
    /// It is a language of its own rather than [`Self::ocr_packs`] because the
    /// region is read again every second: reading it with one recogniser costs a
    /// fraction of reading it with every checked language, and a subtitle is
    /// only ever written in one of them.
    pub subtitle_pack: String,
    /// Ask the other services when the chosen one fails or rate-limits.
    pub fallback_enabled: bool,
    /// The services to try, in order, after the chosen one failed. The chosen
    /// one is not repeated here; it is always tried first.
    pub fallback_order: Vec<Service>,
    /// Show the sentence the selected word sits in, next to the word itself.
    pub word_sentence: bool,
    /// Pair the original with the translation sentence by sentence.
    pub sentence_pairs: bool,
    /// Draw the popup without the blocks that are only nice to have.
    pub compact_popup: bool,
    /// Speaking rate of the pronunciation buttons, -10 (slowest) to 10.
    pub speech_rate: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            format_version: FORMAT_VERSION,
            enabled: true,
            trigger_on_drag: true,
            trigger_on_double_click: true,
            target_lang: default_target_lang(),
            // A fresh install asks Glossy's own server, which needs nothing
            // filled in and works from mainland China.
            service: Service::CloudBaidu,
            api_baidu: Credentials::default(),
            api_openai: ChatApi::default(),
            cloud_id: crate::translate::new_install_id(),
            restore_clipboard: true,
            show_original: true,
            min_selection_len: MIN_SELECTION_LEN,
            units_enabled: true,
            ignored_apps: Vec::new(),
            source_langs: Vec::new(),
            ui_lang: UiLanguage::default(),
            first_run: true,
            guide_seen: false,
            theme: Theme::default(),
            palette: Palette::default(),
            accent: String::new(),
            font_scale: 100,
            popup_width: DEFAULT_POPUP_WIDTH,
            popup_opacity: DEFAULT_POPUP_OPACITY,
            auto_close_secs: 0,
            close_after_copy: false,
            autostart: false,
            check_updates: false,
            history_limit: 50,
            hotkey: "Ctrl+Alt+C".to_string(),
            hotkey_settings: "Ctrl+Alt+G".to_string(),
            hotkey_ocr: "Ctrl+Alt+Q".to_string(),
            ocr_packs: vec![crate::ocr::models::DEFAULT_PACK.to_string()],
            developer_mode: false,
            subtitle_source_lang: "auto".to_string(),
            subtitle_target_lang: default_target_lang(),
            subtitle_font_size: 26,
            subtitle_pack: crate::ocr::models::DEFAULT_PACK.to_string(),
            fallback_enabled: true,
            fallback_order: vec![Service::CloudYoudao, Service::Google],
            word_sentence: false,
            sentence_pairs: false,
            compact_popup: false,
            speech_rate: 0,
        }
    }
}

impl Settings {
    /// The entry of the service list this file names.
    pub fn service(&self) -> Service {
        self.service
    }

    /// Makes `service` the active one.
    ///
    /// One field, so there is nothing to keep in step: the same call used to
    /// write four of them, and a file that held them is folded into this one by
    /// `migrate` before anything reads it.
    pub fn set_service(&mut self, service: Service) {
        self.service = service;
    }

    /// Whether the credentials an entry needs have been filled in.
    ///
    /// The entries that need none are always ready. This is what decides whether
    /// the two of the reader's own are offered at all, and the window applies
    /// the same rule to its dropdown, so the list and the translation never
    /// disagree.
    pub fn credentials_filled(&self, service: Service) -> bool {
        match service {
            Service::ApiBaidu => !self.api_baidu.is_empty(),
            Service::ApiOpenAi => !self.api_openai.is_empty(),
            _ => true,
        }
    }

    /// The credentials the chosen entry translates with, or nothing when the
    /// choice needs none.
    pub fn api_credentials(&self) -> Option<ApiCredentials<'_>> {
        match self.service {
            Service::ApiBaidu if !self.api_baidu.is_empty() => {
                Some(ApiCredentials::Baidu(&self.api_baidu))
            }
            Service::ApiOpenAi if !self.api_openai.is_empty() => {
                Some(ApiCredentials::Chat(&self.api_openai))
            }
            _ => None,
        }
    }

    /// Replaces every protected credential with the key it protects.
    ///
    /// Answers whether the file has to be written again, which it has when a
    /// value was still plain text (an older file) or could not be unlocked at
    /// all â€” a key protected for another Windows login is a key this one cannot
    /// use, and leaving the unusable blob in the file would only mislead the
    /// next read. A key that is dropped takes its entry with it: the choice
    /// falls back on what needs nothing, which is what `sanitized` enforces.
    fn reveal_credentials(&mut self) -> bool {
        let mut rewrite = false;
        for (name, field, value) in self.stored_credentials() {
            if value.is_empty() {
                continue;
            }
            if !crate::platform::secrets::is_protected(value) {
                rewrite = true;
                continue;
            }
            match crate::platform::secrets::reveal(value) {
                Some(plain) => *value = plain,
                None => {
                    note!(
                        "glossy: the {field} of `{name}` was protected for another Windows \
                         login, so it has to be entered again"
                    );
                    *value = String::new();
                    rewrite = true;
                }
            }
        }
        // `parse` sanitized the blob it read, so the trimming and the fallback it
        // decides were applied to a protected value rather than to the key. Now
        // that the keys are plain text it is run again, which is also what turns a
        // key that could not be unlocked into the fallback it takes with it.
        *self = std::mem::take(self).sanitized();
        rewrite
    }

    /// The copy that goes to disk â€” and to an export, which is the same shape
    /// for the same reason: neither is a place to leave a key in the open.
    pub fn protected_for_storage(&self) -> Settings {
        let mut stored = self.clone();
        for (name, field, value) in stored.stored_credentials() {
            if value.is_empty() || crate::platform::secrets::is_protected(value) {
                continue;
            }
            match crate::platform::secrets::protect(value) {
                Ok(protected) => *value = protected,
                // Storing the key unprotected beats losing it; DPAPI is part of
                // Windows, so this only happens in a broken environment.
                Err(error) => {
                    note!("glossy: cannot protect the {field} of `{name}`: {error}")
                }
            }
        }
        stored
    }

    /// Every credential field, with the name it is reported under.
    fn stored_credentials(&mut self) -> Vec<(&'static str, &'static str, &mut String)> {
        let mut fields = Vec::new();
        for (field, value) in self.api_baidu.fields() {
            fields.push(("baidu", field, value));
        }
        for (field, value) in self.api_openai.fields() {
            fields.push(("openai", field, value));
        }
        fields
    }

    /// The services a translation may be asked of, the chosen one first.
    ///
    /// A fallback that repeats the choice or names a service twice would only
    /// cost a second attempt on the same backend, so the list is de-duplicated
    /// here.
    pub fn service_order(&self) -> Vec<Service> {
        let active = self.service();
        let mut order = vec![active];
        if self.fallback_enabled {
            for service in &self.fallback_order {
                if !order.contains(service) {
                    order.push(*service);
                }
            }
        }
        order
    }

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
                if Self::unreadable(&raw) {
                    // A copy is kept next to the file, because the next save
                    // overwrites it: a hand edit that broke the JSON, or a file
                    // written by a newer release, stays recoverable either way.
                    let kept = match Self::set_aside(&path) {
                        Some(backup) => format!(", kept as {}", backup.display()),
                        None => String::new(),
                    };
                    note!(
                        "glossy: {} is not in format {FORMAT_VERSION}{kept}, \
                         so Glossy starts from the defaults",
                        path.display()
                    );
                    return Settings::default();
                }
                let mut settings = Self::parse(&raw);
                // A file written before the install id was derived from the
                // machine carries no id, and the one this build computes has to
                // reach the disk: otherwise the proxy would see a new device on
                // every launch and hand out the daily allowance again.
                let mut rewrite = false;
                if !Self::stores(&raw, "cloudId") || settings.cloud_id.trim().is_empty() {
                    settings.cloud_id = crate::translate::new_install_id();
                    rewrite = true;
                }
                // The keys belong to this Windows login, which `parse` cannot
                // know: what it read is the protected blob, and what the rest of
                // the application needs is the key itself.
                if settings.reveal_credentials() {
                    rewrite = true;
                }
                if rewrite {
                    // The file was missing something this build computes, so
                    // write the result back; failing to do so only means the
                    // next save is the one that cleans the file up.
                    if let Err(error) = settings.save(app) {
                        note!("glossy: cannot rewrite the settings file: {error}");
                    }
                }
                settings
            }
            Err(_) => Settings::default(),
        }
    }

    /// Whether a stored file is in a shape this build cannot read, which means
    /// the defaults are used instead.
    ///
    /// Two cases. A file that is not a JSON object at all â€” a hand edit that
    /// broke it, or half a file after a crash â€” and one written by a *newer*
    /// format, whose settings this build would only misread: a key that means
    /// something else there would be taken as the value it used to have. A file
    /// that names no version is the one written before the freeze, which is
    /// version `0` and is migrated rather than set aside.
    fn unreadable(raw: &str) -> bool {
        let text = raw.trim_start_matches('\u{feff}');
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(serde_json::Value::Object(stored)) => stored
                .get("formatVersion")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|version| version > u64::from(FORMAT_VERSION)),
            // A file that is JSON but not an object holds no setting at all,
            // which `parse` answers with the defaults anyway; going through the
            // same path keeps the backup in one place.
            Ok(_) => true,
            Err(_) => true,
        }
    }

    /// Copies the settings file to `settings.backup-<unix seconds>.json` and
    /// answers with the copy, or with nothing when it could not be written.
    fn set_aside(path: &Path) -> Option<PathBuf> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        let backup = path.with_file_name(format!("settings.backup-{stamp}.json"));
        match std::fs::copy(path, &backup) {
            Ok(_) => Some(backup),
            Err(error) => {
                note!("glossy: cannot keep a copy of {}: {error}", path.display());
                None
            }
        }
    }

    /// Whether a stored file holds a given key at all, which `parse` cannot
    /// tell: it merges the file into the defaults, so a key that is missing
    /// there shows up as the default value.
    fn stores(raw: &str, key: &str) -> bool {
        let text = raw.trim_start_matches('\u{feff}');
        serde_json::from_str::<serde_json::Value>(text)
            .ok()
            .and_then(|value| value.as_object().map(|map| map.contains_key(key)))
            .unwrap_or(false)
    }

    /// Brings a stored file up to `FORMAT_VERSION`, one version at a time.
    ///
    /// Every step belongs here and none of them belongs in `parse`: a file
    /// written by an older release is read through this, and the merge that
    /// follows only has to deal with the current shape. A step is gated on the
    /// version it upgrades from, so the chain stays readable as it grows.
    fn migrate(stored: &mut serde_json::Map<String, serde_json::Value>, from: u32) {
        if from < 2 {
            // Version 1 spread the choice of service over four fields. They are
            // read into the one field this build writes, and the four are
            // removed: a key left behind would be merged back in by `parse` as
            // a value nothing reads.
            //
            // A version 0 file names only a provider, and one of those â€” the
            // free endpoint â€” is an entry of its own, which `from_legacy` knows.
            //
            // A file that already names `service` is left as it is: it is a
            // hand edit or a file whose version says nothing, and reading the
            // four fields it does not have would overwrite what it says.
            if !stored.contains_key("service") {
                let service = Service::from_legacy(
                    stored.get("channel").and_then(serde_json::Value::as_str),
                    stored
                        .get("cloudVendor")
                        .and_then(serde_json::Value::as_str),
                    stored.get("provider").and_then(serde_json::Value::as_str),
                );
                stored.insert(
                    "service".to_string(),
                    serde_json::to_value(service).unwrap_or(serde_json::json!("cloud-baidu")),
                );
            }
            for key in [
                "channel",
                "cloudProvider",
                "cloudVendor",
                "provider",
                // The address of the relay is part of the build, and the window
                // has had no field for it since 1.7; the credential map holds
                // keys for providers this build does not offer.
                "cloudEndpoint",
                "credentials",
                "apiKey",
                "appId",
            ] {
                stored.remove(key);
            }
        }
        // Whatever the file said, the shape read from here on is the current
        // one, and the next save writes that number back.
        stored.insert(
            "formatVersion".to_string(),
            serde_json::json!(FORMAT_VERSION),
        );

        // A file written before a screenshot could be read with more than one
        // language names exactly one, in `ocrPack`: it becomes the only language
        // that is checked, so the language the user chose is not quietly
        // forgotten and read as Chinese and English instead.
        if !stored.contains_key("ocrPacks") {
            if let Some(pack) = stored.remove("ocrPack") {
                stored.insert("ocrPacks".to_string(), serde_json::json!([pack]));
            }
        }
    }

    /// Reads persisted JSON, falling back to the defaults for anything unusable.
    ///
    /// The file is merged into the defaults one setting at a time: a value that
    /// does not fit its setting (a hand edit, a number stored as a string, a
    /// provider that no longer exists) costs only that setting instead of
    /// resetting the whole file - which the next save would then write back as
    /// defaults, losing everything the user had configured.
    ///
    /// What the file declares about itself is read first, by `migrate`: a file
    /// from an older release is brought up to `FORMAT_VERSION` before any of it
    /// is merged, and a file this build cannot read at all never reaches here
    /// (`load` sets it aside instead).
    fn parse(raw: &str) -> Settings {
        // Editors on Windows like to write a UTF-8 byte order mark.
        let text = raw.trim_start_matches('\u{feff}');
        let Ok(serde_json::Value::Object(mut stored)) = serde_json::from_str(text) else {
            return Settings::default().sanitized();
        };

        let mut merged = serde_json::to_value(Settings::default())
            .ok()
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();

        let from = stored
            .get("formatVersion")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as u32;
        Self::migrate(&mut stored, from);

        for (key, value) in stored {
            // A key this build does not write is dropped: `migrate` has already
            // turned everything an older version said into the current shape,
            // so what is left over is a hand edit or a key from a build that is
            // newer than this one.
            if !merged.contains_key(&key) {
                continue;
            }
            // Deserializing the single setting tells whether it still fits; the
            // other fields come from the defaults and cannot fail the probe.
            let probe = serde_json::json!({ key.clone(): value.clone() });
            match serde_json::from_value::<Settings>(probe) {
                Ok(_) => {
                    merged.insert(key, value);
                }
                Err(error) => note!("glossy: ignoring the stored setting `{key}`: {error}"),
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
        let settings = Self::parse(text);
        // The file was written by `export_settings`, which protects the keys the
        // same way the settings file does, so they are unlocked here. A key that
        // belongs to another Windows login cannot be unlocked, and what is left
        // of it is dropped rather than stored as a blob nothing can read.
        let mut settings = settings;
        settings.reveal_credentials();
        Ok(settings.sanitized())
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

    pub fn sanitized(mut self) -> Settings {
        if self.target_lang.trim().is_empty() {
            self.target_lang = default_target_lang();
        } else {
            // A code from another vendor (`jp`, `ZH-HANS`) or one written by
            // hand has to match an entry of the language menu, otherwise the
            // dropdown has nothing to show for it.
            self.target_lang = crate::translate::normalize_lang_code(&self.target_lang);
        }
        self.min_selection_len = self.min_selection_len.clamp(1, 40);
        // An accent is a colour or nothing: a half-typed or hand-edited value
        // would be set as an invalid custom property, which the stylesheet then
        // quietly ignores, and the picker would show a swatch that colours
        // nothing.
        let accent = self.accent.trim().to_ascii_lowercase();
        let is_colour = accent.len() == 7
            && accent.starts_with('#')
            && accent[1..].bytes().all(|byte| byte.is_ascii_hexdigit());
        self.accent = if is_colour { accent } else { String::new() };
        self.ignored_apps = ignored_processes(&self.ignored_apps);
        self.source_langs = language_codes(&self.source_langs);
        self.font_scale = self.font_scale.clamp(80, 160);
        self.popup_width = self.popup_width.clamp(280, 560);
        self.popup_opacity = self.popup_opacity.clamp(50, 100);
        self.auto_close_secs = self.auto_close_secs.min(600);
        self.history_limit = self.history_limit.min(crate::history::MAX_LIMIT);
        self.hotkey = self.hotkey.trim().to_string();
        self.hotkey_settings = self.hotkey_settings.trim().to_string();
        self.hotkey_ocr = self.hotkey_ocr.trim().to_string();
        // The languages the recogniser reads with: names this build does not
        // offer are dropped, a repeat is kept once, the order is the one the
        // page shows, and a list that ends up empty is the language that was
        // always there.
        self.ocr_packs = crate::ocr::models::checked(&self.ocr_packs);
        // The subtitles read one language and are translated into another, both
        // named by the user before the reading starts: a name this build cannot
        // read or write falls back to what everything else falls back to.
        self.subtitle_pack = crate::ocr::models::checked(std::slice::from_ref(&std::mem::take(
            &mut self.subtitle_pack,
        )))
        .first()
        .cloned()
        .unwrap_or_else(|| crate::ocr::models::DEFAULT_PACK.to_string());
        self.subtitle_source_lang = match self.subtitle_source_lang.trim() {
            "" => "auto".to_string(),
            code if code.eq_ignore_ascii_case("auto") => "auto".to_string(),
            code => crate::translate::normalize_lang_code(code),
        };
        self.subtitle_target_lang = self.subtitle_target_lang.trim().to_string();
        if self.subtitle_target_lang.is_empty() {
            self.subtitle_target_lang = default_target_lang();
        } else {
            self.subtitle_target_lang =
                crate::translate::normalize_lang_code(&self.subtitle_target_lang);
        }
        self.subtitle_font_size = self.subtitle_font_size.clamp(13, 40);
        if self.cloud_id.is_empty() {
            self.cloud_id = crate::translate::new_install_id();
        }
        if !self.trigger_on_drag && !self.trigger_on_double_click {
            self.trigger_on_drag = true;
        }
        // The reader's own credentials: trimmed here so nothing downstream has
        // to, and held against the same rule the window uses to decide whether
        // the entry is worth offering.
        self.api_baidu = self.api_baidu.trimmed();
        self.api_openai = self.api_openai.trimmed();
        // An entry that needs credentials is not offered without them, and a
        // file that names one anyway â€” a hand edit, or a key that belonged to
        // another Windows login â€” reads as the entry that needs nothing. The
        // window applies the same rule to its dropdown, so what it shows and
        // what translates are never two different engines.
        if self.service.needs_credentials() && !self.credentials_filled(self.service) {
            self.service = Service::CloudBaidu;
        }
        // The order is a list the window lets the user shuffle, so a hand edit
        // is as likely as a click; a duplicate would only be tried twice.
        //
        // `offline` is dropped from it as well: the models on this machine are a
        // choice the reader makes rather than a substitution the app makes for
        // them, and a fallback that quietly reaches for a 244 MB download is not
        // what turning the switch on asks for. A file that names it there loses
        // it the next time the settings are written.
        //
        // An entry of the reader's own goes the same way when its credentials
        // are not filled in: a fallback is a substitution the app makes, and one
        // that cannot be made is worse than none.
        let baidu = !self.api_baidu.is_empty();
        let chat = !self.api_openai.is_empty();
        let mut seen: Vec<Service> = Vec::new();
        self.fallback_order.retain(|service| {
            let usable = match service {
                Service::Offline => false,
                Service::ApiBaidu => baidu,
                Service::ApiOpenAi => chat,
                _ => true,
            };
            if !usable || seen.contains(service) {
                false
            } else {
                seen.push(*service);
                true
            }
        });
        self.speech_rate = self.speech_rate.clamp(-10, 10);
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
    fn one_unreadable_setting_keeps_the_rest_of_the_file() {
        // A file that names a credential map â€” one this build no longer has â€”
        // and one setting written by hand as a word: the map is dropped, and
        // only that setting falls back to its default.
        let settings = Settings::parse(
            "{\"service\":\"cloud-youdao\",\"fontScale\":\"big\",\
             \"credentials\":{\"baidu\":{\"appId\":\" 2024 \",\"apiKey\":\" secret \"}}}",
        );

        assert_eq!(settings.service(), Service::CloudYoudao);
        assert_eq!(settings.font_scale, 100);
    }

    #[test]
    fn ignores_settings_that_no_longer_exist() {
        let settings =
            Settings::parse("{\"unknownSetting\":42,\"targetLang\":\"fr\",\"provider\":\"gone\"}");

        assert_eq!(settings.target_lang, "fr");
        // A provider that was removed leaves the engine on the default one, and
        // the remaining settings are untouched.
        assert_eq!(settings.service(), Service::CloudBaidu);
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
    fn normalizes_the_source_languages() {
        let settings = Settings {
            source_langs: vec![
                " EN ".to_string(),
                "en".to_string(),
                "jp, zh-Hant".to_string(),
                "".to_string(),
            ],
            ..Settings::default()
        }
        .sanitized();

        assert_eq!(settings.source_langs, vec!["en", "ja", "zh-TW"]);
    }

    #[test]
    fn a_vendor_the_server_dropped_keeps_the_engine_it_was_served_by() {
        // `cloudVendor` is not part of the file any more; what it said is read
        // once, while a `1.x` file is migrated, and lands on the entry the
        // server can still ask.
        assert_eq!(
            Settings::parse("{\"channel\":\"cloud\",\"cloudVendor\":\" Youdao \"}").service(),
            Service::CloudYoudao
        );
        assert_eq!(
            Settings::parse("{\"channel\":\"cloud\",\"cloudVendor\":\"BAIDU\"}").service(),
            Service::CloudBaidu
        );
        assert_eq!(
            Settings::parse("{\"channel\":\"cloud\",\"cloudVendor\":\"deepl\"}").service(),
            Service::CloudBaidu
        );
        assert_eq!(
            Settings::parse("{\"channel\":\"cloud\"}").service(),
            Service::CloudBaidu
        );
    }

    #[test]
    fn the_languages_that_are_read_with_survive_a_settings_file() {
        // A name this build does not know â€” a hand edit, or a language a later
        // build dropped â€” is dropped from the list instead of being carried
        // into a download of a model that does not exist, and a list that is
        // left with nothing is the language that was always there.
        let packs = |values: &[&str]| {
            Settings {
                ocr_packs: values.iter().map(|value| value.to_string()).collect(),
                ..Settings::default()
            }
            .sanitized()
            .ocr_packs
        };

        assert_eq!(packs(&["ch"]), vec!["ch"]);
        assert_eq!(packs(&[" CH "]), vec!["ch"]);
        assert_eq!(packs(&["ko", "ch"]), vec!["ch", "ko"]);
        assert_eq!(packs(&["ja", "ja"]), vec!["ja"]);
        assert_eq!(packs(&["klingon"]), vec!["ch"]);
        assert_eq!(packs(&[]), vec!["ch"]);
    }

    #[test]
    fn the_language_a_file_written_before_this_one_names_is_kept() {
        // The key was `ocrPack` and held one language; it is read as the one
        // language that is checked, so the choice the user made is not quietly
        // read as Chinese and English instead.
        let settings = Settings::parse("{\"ocrPack\":\"ja\"}");
        assert_eq!(settings.ocr_packs, vec!["ja"]);
        // A file that names both â€” a hand edit â€” is read as the newer key.
        let settings = Settings::parse("{\"ocrPack\":\"ja\",\"ocrPacks\":[\"cht\"]}");
        assert_eq!(settings.ocr_packs, vec!["cht"]);
    }

    #[test]
    fn an_older_settings_file_has_every_source_language() {
        let settings = Settings::parse("{\"minSelectionLen\":3}");

        assert!(settings.source_langs.is_empty());
    }

    #[test]
    fn reads_the_source_languages_of_an_older_version() {
        let settings = Settings::parse("{\"sourceLangs\":\"en, jp\"}");

        assert_eq!(settings.source_langs, vec!["en", "ja"]);
    }

    #[test]
    fn imports_a_file_this_app_exported() {
        let raw = serde_json::to_string_pretty(&Settings::parse(
            "{\"targetLang\":\"ja\",\"service\":\"cloud-youdao\"}",
        ))
        .unwrap();

        let imported = Settings::import(&raw).unwrap();
        assert_eq!(imported.target_lang, "ja");
        assert_eq!(imported.service(), Service::CloudYoudao);
    }

    #[test]
    fn refuses_a_file_that_holds_no_setting() {
        assert!(Settings::import("not json at all").is_err());
        assert!(Settings::import("[1, 2, 3]").is_err());
        assert!(Settings::import("{\"somethingElse\":true}").is_err());
    }

    #[test]
    fn a_fresh_install_starts_on_the_built_in_engine() {
        let settings = Settings::default();

        assert_eq!(settings.service(), Service::CloudBaidu);
        assert_eq!(settings.service().id(), "cloud-baidu");
    }

    #[test]
    fn every_service_answers_to_its_own_id_and_to_nothing_else() {
        for service in Service::ALL {
            assert_eq!(Service::from_id(service.id()), Some(service));
        }
        // What the window sends for anything it no longer offers, and what a
        // stray id in a shortcut would look like.
        assert_eq!(Service::from_id(" deepl"), None);
        assert_eq!(Service::from_id(""), None);
        assert_eq!(Service::from_id("Cloud-Baidu"), None);
        // Enough slack for the window to send an id with a stray space in it.
        assert_eq!(Service::from_id(" google "), Some(Service::Google));
    }

    #[test]
    fn every_service_survives_a_round_trip_through_the_stored_shape() {
        for service in Service::ALL {
            let mut settings = Settings::default();
            settings.set_service(service);

            assert_eq!(settings.service(), service, "{} was lost", service.id());
        }
    }

    #[test]
    fn a_provider_this_build_dropped_reads_as_the_built_in_engine() {
        let settings = Settings::parse("{\"channel\":\"api\",\"provider\":\"deepl\"}");

        assert_eq!(settings.service(), Service::CloudBaidu);
    }

    #[test]
    fn a_file_that_named_the_local_model_reads_as_the_built_in_engine() {
        let settings = Settings::parse(
            "{\"channel\":\"cloud\",\"cloudProvider\":\"local\",\
             \"localEndpoint\":\"http://127.0.0.1:11434/v1\",\"localModel\":\"qwen2.5:7b\"}",
        );

        assert_eq!(settings.service(), Service::CloudBaidu);
    }

    #[test]
    fn the_free_endpoint_is_the_one_entry_that_left_the_cloud_channel() {
        let mut settings = Settings::default();
        settings.set_service(Service::Google);

        assert_eq!(settings.service(), Service::Google);
        // One field, so there is nothing left over that could disagree with it.
        let written = serde_json::to_value(&settings).unwrap();
        assert_eq!(written["service"], serde_json::json!("google"));
        for gone in ["channel", "provider", "cloudVendor", "cloudProvider"] {
            assert!(written.get(gone).is_none(), "{gone} is still written");
        }
    }

    #[test]
    fn the_fallback_order_uses_every_service_once() {
        let settings = Settings {
            fallback_order: vec![
                Service::Google,
                Service::CloudYoudao,
                Service::Google,
                Service::CloudBaidu,
            ],
            ..Settings::default()
        };
        let settings = settings.sanitized();

        assert_eq!(
            settings.fallback_order,
            vec![Service::Google, Service::CloudYoudao, Service::CloudBaidu]
        );
        // The chosen service comes first and is not repeated by the fallbacks.
        assert_eq!(
            settings.service_order(),
            vec![Service::CloudBaidu, Service::Google, Service::CloudYoudao,]
        );
    }

    #[test]
    fn the_fallback_order_never_reaches_for_the_offline_models() {
        let settings = Settings {
            fallback_order: vec![Service::Offline, Service::Google, Service::Offline],
            ..Settings::default()
        };
        let settings = settings.sanitized();

        assert_eq!(settings.fallback_order, vec![Service::Google]);
        assert_eq!(
            settings.service_order(),
            vec![Service::CloudBaidu, Service::Google]
        );
    }

    #[test]
    fn an_accent_is_a_colour_or_nothing_at_all() {
        for (stored, wanted) in [
            ("#0f6cbd", "#0f6cbd"),
            ("  #ABCDEF  ", "#abcdef"),
            ("", ""),
            ("   ", ""),
            ("blue", ""),
            ("#12345", ""),
            ("#12345g", ""),
            ("0f6cbd", ""),
            ("#0f6cbd00", ""),
        ] {
            let settings = Settings {
                accent: stored.to_string(),
                ..Settings::default()
            }
            .sanitized();

            assert_eq!(settings.accent, wanted, "{stored:?}");
        }
    }

    #[test]
    fn a_palette_and_an_accent_survive_a_file_and_an_older_one_still_loads() {
        let settings = Settings {
            palette: Palette::Nord,
            accent: "#4c6a92".to_string(),
            ..Settings::default()
        }
        .sanitized();
        let json = serde_json::to_string(&settings).expect("settings serialize");
        let reloaded = Settings::parse(&json);

        assert_eq!(reloaded.palette, Palette::Nord);
        assert_eq!(reloaded.accent, "#4c6a92");

        // A file written before either setting existed keeps the WinUI palette
        // and lets it choose the accent, which is what the defaults are.
        let older = Settings::parse("{\"formatVersion\":2,\"theme\":\"dark\",\"fontScale\":115}");
        assert_eq!(older.palette, Palette::Default);
        assert_eq!(older.accent, "");
        assert_eq!(older.font_scale, 115);

        // A palette this build does not know costs that one setting, not the
        // file: everything else in it is still read.
        let unknown =
            Settings::parse("{\"formatVersion\":2,\"palette\":\"solarized2\",\"fontScale\":130}");
        assert_eq!(unknown.palette, Palette::Default);
        assert_eq!(unknown.font_scale, 130);
    }

    #[test]
    fn switching_the_fallback_off_leaves_only_the_chosen_service() {
        let settings = Settings {
            fallback_enabled: false,
            ..Settings::default()
        };

        assert_eq!(settings.service_order(), vec![settings.service()]);
    }

    #[test]
    fn the_speaking_rate_stays_within_what_sapi_accepts() {
        let settings = Settings {
            speech_rate: 42,
            ..Settings::default()
        };

        assert_eq!(settings.sanitized().speech_rate, 10);
    }

    /// The frozen contract, as published in `contract/contract.json`.
    ///
    /// It is the promise the `2.x` line is built on: the shape of the settings
    /// file and the names of the IPC commands do not change under it. Editing
    /// either one deliberately means editing the contract in the same commit,
    /// which is what this file makes visible in review.
    fn contract() -> serde_json::Value {
        serde_json::from_str(include_str!("../../contract/contract.json"))
            .expect("the contract is JSON")
    }

    fn strings(value: &serde_json::Value) -> Vec<String> {
        value
            .as_array()
            .expect("an array")
            .iter()
            .map(|entry| entry.as_str().expect("a string").to_string())
            .collect()
    }

    #[test]
    fn the_stored_shape_is_the_one_the_contract_publishes() {
        let contract = contract();
        let written = serde_json::to_value(Settings::default()).expect("settings serialize");
        let keys: Vec<String> = written
            .as_object()
            .expect("an object")
            .keys()
            .cloned()
            .collect();

        assert_eq!(keys, strings(&contract["settingsKeys"]));
        assert_eq!(contract["formatVersion"], serde_json::json!(FORMAT_VERSION));
        assert_eq!(written["formatVersion"], serde_json::json!(FORMAT_VERSION));
    }

    #[test]
    fn a_settings_file_says_which_format_it_is_in() {
        let json = serde_json::to_string(&Settings::default()).expect("settings serialize");

        // The file is written by this build, so it carries this format.
        assert_eq!(Settings::parse(&json).format_version, FORMAT_VERSION);
        // A file written before the freeze names no version, and reading it
        // migrates it up to the current one rather than leaving it at `0`.
        assert_eq!(
            Settings::parse("{\"enabled\":true}").format_version,
            FORMAT_VERSION
        );
        assert_eq!(
            Settings::parse("{\"formatVersion\":0,\"enabled\":true}").format_version,
            FORMAT_VERSION
        );
    }

    #[test]
    fn a_file_from_before_the_freeze_is_migrated_before_it_is_merged() {
        // A file written before the channels existed names a provider and no
        // channel, and the migration is what keeps its intent: the free
        // endpoint stays the free endpoint instead of sliding into the cloud
        // channel, which is what a plain merge into the defaults would do.
        assert_eq!(
            Settings::parse("{\"provider\":\"google\"}").service(),
            Service::Google
        );

        // A provider that wanted an account of the user's is not offered any
        // more: the migration still reads what it meant, and the first save
        // replaces it with an entry this build has.
        assert_eq!(
            Settings::parse("{\"provider\":\"baidu\"}").service(),
            Service::CloudBaidu
        );
        assert_eq!(
            Settings::parse("{\"provider\":\"cloud\"}").service(),
            Service::CloudBaidu
        );

        // The rest of such a file is kept, including the settings the keys of
        // that era used to sit next to.
        let settings = Settings::parse(
            "{\"apiKey\":\"stored-key\",\"provider\":\"baidu\",\"targetLang\":\"ja\"}",
        );
        assert_eq!(settings.target_lang, "ja");
        assert_eq!(settings.service(), Service::CloudBaidu);
    }

    #[test]
    fn a_one_field_file_is_read_as_the_service_it_names() {
        for service in Service::ALL {
            // The two entries of the reader's own only exist with credentials
            // behind them, so a file that names one has to carry them too.
            let filled = match service {
                Service::ApiBaidu => Settings {
                    api_baidu: Credentials {
                        app_id: "2024".to_string(),
                        api_key: "key".to_string(),
                    },
                    ..Settings::default()
                },
                Service::ApiOpenAi => Settings {
                    api_openai: ChatApi {
                        api_key: "sk-x".to_string(),
                        model: "gpt-4o-mini".to_string(),
                        ..ChatApi::default()
                    },
                    ..Settings::default()
                },
                _ => Settings::default(),
            };

            let written = serde_json::to_string(&Settings { service, ..filled }).unwrap();

            assert_eq!(
                Settings::parse(&written).service(),
                service,
                "{}",
                service.id()
            );
        }
    }

    #[test]
    fn the_four_fields_of_a_one_x_file_are_folded_into_one() {
        // What 1.8 wrote for each entry: the file has to keep meaning the same
        // engine after the four fields are gone.
        let cases = [
            ("cloud", "builtin", "baidu", "baidu", Service::CloudBaidu),
            ("cloud", "builtin", "youdao", "baidu", Service::CloudYoudao),
            ("api", "builtin", "", "google", Service::Google),
            ("offline", "builtin", "", "baidu", Service::Offline),
        ];
        for (channel, cloud_provider, vendor, provider, expected) in cases {
            let raw = format!(
                "{{\"formatVersion\":1,\"channel\":\"{channel}\",\
                 \"cloudProvider\":\"{cloud_provider}\",\"cloudVendor\":\"{vendor}\",\
                 \"provider\":\"{provider}\",\"targetLang\":\"de\"}}"
            );
            let settings = Settings::parse(&raw);
            assert_eq!(settings.service(), expected, "{raw}");
            // Everything else in the file is kept.
            assert_eq!(settings.target_lang, "de");

            // And what is written back is the current shape, with nothing of
            // the four left behind for a later read to interpret.
            let written = serde_json::to_value(&settings).unwrap();
            assert_eq!(written["service"], serde_json::json!(expected.id()));
            assert_eq!(written["formatVersion"], serde_json::json!(FORMAT_VERSION));
            for gone in ["channel", "cloudProvider", "cloudVendor", "provider"] {
                assert!(written.get(gone).is_none(), "{gone} survived");
            }
        }
    }

    #[test]
    fn the_fields_nothing_reads_any_more_are_dropped_by_the_migration() {
        // A 1.8 file: the relay address and a credential map. Neither is a
        // setting this build has, so neither may survive into the next write.
        let raw = "{\"formatVersion\":1,\"channel\":\"api\",\"provider\":\"google\",\
                   \"cloudEndpoint\":\"https://glossy.example.workers.dev\",\
                   \"credentials\":{\"baidu\":{\"appId\":\"2024\",\"apiKey\":\"dpapi:AQAA\"}},\
                   \"apiKey\":\"plain-key\",\"appId\":\"2024\"}";
        let settings = Settings::parse(raw);

        assert_eq!(settings.service(), Service::Google);
        let written = serde_json::to_value(&settings).unwrap();
        for gone in ["cloudEndpoint", "credentials", "apiKey", "appId"] {
            assert!(written.get(gone).is_none(), "{gone} survived");
        }
    }

    #[test]
    fn an_entry_with_nothing_behind_it_is_not_offered() {
        // The rule the dropdown and the engine share: an entry of the reader's
        // own exists only once its credentials do.
        let empty = Settings::default();
        assert!(!empty.credentials_filled(Service::ApiBaidu));
        assert!(!empty.credentials_filled(Service::ApiOpenAi));
        // One half of a Baidu pair is not a credential.
        let half = Settings {
            api_baidu: Credentials {
                app_id: "2024".to_string(),
                api_key: String::new(),
            },
            ..Settings::default()
        };
        assert!(!half.credentials_filled(Service::ApiBaidu));
        // The key and the model are what an OpenAI compatible entry needs; the
        // address has a default.
        let keyed = Settings {
            api_openai: ChatApi {
                api_key: "sk-x".to_string(),
                model: "gpt-4o-mini".to_string(),
                ..ChatApi::default()
            },
            ..Settings::default()
        };
        assert!(keyed.credentials_filled(Service::ApiOpenAi));
        // And the four that need nothing are always ready.
        for service in Service::ALL {
            if !service.needs_credentials() {
                assert!(empty.credentials_filled(service), "{}", service.id());
            }
        }
    }

    #[test]
    fn a_choice_of_an_unfilled_entry_reads_as_the_entry_that_needs_nothing() {
        // A hand edited file, or a key that belonged to another Windows login:
        // the choice cannot stand, and the engine that answers is the one the
        // window can show.
        let settings = Settings::parse("{\"service\":\"api-baidu\"}");
        assert_eq!(settings.service(), Service::CloudBaidu);
        assert!(settings.api_credentials().is_none());

        // With the credentials in the file it stands.
        let filled = Settings::parse(
            "{\"service\":\"api-openai\",\"apiOpenai\":\
             {\"apiKey\":\"sk-x\",\"model\":\"gpt-4o-mini\"}}",
        );
        assert_eq!(filled.service(), Service::ApiOpenAi);
        assert!(matches!(
            filled.api_credentials(),
            Some(ApiCredentials::Chat(_))
        ));
    }

    #[test]
    fn an_unfilled_entry_is_dropped_from_the_fallback_order() {
        let settings = Settings::parse(
            "{\"fallbackEnabled\":true,\
             \"fallbackOrder\":[\"google\",\"api-baidu\",\"api-openai\",\"offline\"]}",
        );

        assert_eq!(settings.fallback_order, vec![Service::Google]);
    }

    #[test]
    fn the_key_reaches_the_disk_protected_and_comes_back_plain() {
        let settings = Settings {
            api_baidu: Credentials {
                app_id: " 2024 ".to_string(),
                api_key: "baidu-secret".to_string(),
            },
            api_openai: ChatApi {
                endpoint: "https://api.deepseek.com/v1/".to_string(),
                api_key: "sk-secret".to_string(),
                model: "deepseek-chat".to_string(),
            },
            ..Settings::default()
        };

        // What `save` writes, and what an export writes with it.
        let raw = serde_json::to_string(&settings.protected_for_storage())
            .expect("the settings serialize");
        assert!(!raw.contains("baidu-secret"), "{raw}");
        assert!(!raw.contains("sk-secret"), "{raw}");
        // The address and the model are not secrets and stay readable, so a
        // reader of the file can see which service it points at.
        assert!(raw.contains("https://api.deepseek.com/v1"), "{raw}");
        assert!(raw.contains("deepseek-chat"), "{raw}");

        // What `load` and `import` do to it. Everything in the file was already
        // protected, so nothing has to be written again.
        let mut read = Settings::parse(&raw);
        assert!(!read.reveal_credentials(), "the file is left alone");
        assert_eq!(read.api_baidu.app_id, "2024");
        assert_eq!(read.api_baidu.api_key, "baidu-secret");
        assert_eq!(read.api_openai.api_key, "sk-secret");
        assert_eq!(
            read.api_openai.completion_url(),
            "https://api.deepseek.com/v1/chat/completions"
        );

        // A value that is already protected is left exactly as it is, so a save
        // of an unmodified file does not re-encrypt it: the blob is stable and
        // the file is not rewritten on every launch.
        let stored = settings.protected_for_storage();
        let again = stored.protected_for_storage();
        assert_eq!(again.api_openai.api_key, stored.api_openai.api_key);
        assert_eq!(again.api_baidu.api_key, stored.api_baidu.api_key);

        // A revealed key is protected again, because that is what the object it
        // sits in holds from then on.
        let revealed = Settings::parse(&raw);
        let re_protected = revealed.protected_for_storage();
        assert!(crate::platform::secrets::is_protected(
            &re_protected.api_openai.api_key
        ));
    }
    #[test]
    fn a_key_from_another_windows_login_is_dropped_with_its_entry() {
        // A blob only `CryptUnprotectData` of the login that made it can open.
        // Nothing else in the app can do anything with it, so the entry goes
        // and the choice falls back on one that needs nothing.
        let mut settings = Settings {
            service: Service::ApiOpenAi,
            api_openai: ChatApi {
                api_key: "dpapi:AQAA-not-a-blob".to_string(),
                model: "gpt-4o-mini".to_string(),
                ..ChatApi::default()
            },
            ..Settings::default()
        };

        assert!(settings.reveal_credentials(), "the file is written again");
        assert!(settings.api_openai.api_key.is_empty());
        assert_eq!(settings.service(), Service::CloudBaidu);
        assert!(settings.api_credentials().is_none());
    }

    #[test]
    fn a_file_this_build_cannot_read_is_set_aside() {
        // A newer format can give a key a meaning this build does not know, so
        // none of it may be merged into the defaults.
        assert!(Settings::unreadable(
            "{\"formatVersion\":999,\"targetLang\":\"ja\"}"
        ));
        // Half a file after a crash, and a file that is JSON but holds nothing.
        assert!(Settings::unreadable("{\"targetLang\":"));
        assert!(Settings::unreadable("[]"));
        assert!(Settings::unreadable(""));

        // The current format, and every file written before the freeze.
        assert!(!Settings::unreadable(&format!(
            "{{\"formatVersion\":{FORMAT_VERSION}}}"
        )));
        assert!(!Settings::unreadable("{\"formatVersion\":1}"));
        assert!(!Settings::unreadable("{\"enabled\":true}"));
    }

    #[test]
    fn the_file_that_could_not_be_read_is_kept_next_to_the_settings() {
        let dir = std::env::temp_dir().join(format!(
            "glossy-settings-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the clock is set")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("the directory is writable");
        let path = dir.join("settings.json");
        std::fs::write(&path, "{\"targetLang\":").expect("the file is writable");

        let backup = Settings::set_aside(&path).expect("the copy is written");

        assert_eq!(backup.parent(), path.parent());
        let name = backup.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("settings.backup-"), "{name}");
        assert!(name.ends_with(".json"), "{name}");
        assert!(path.exists(), "the original file is still there");
        assert_eq!(
            std::fs::read_to_string(&backup).expect("the copy is readable"),
            "{\"targetLang\":"
        );
        std::fs::remove_dir_all(&dir).expect("the directory is removable");
    }
}
