//! Reads a translation out loud with the voices Windows already ships.
//!
//! SAPI is a COM API and a COM object belongs to the apartment that created it,
//! so a single hidden worker thread owns the voice for the whole run: it
//! initialises COM, creates the voice and speaks whatever the interface asks
//! for. Speaking asynchronously keeps the interface responsive, and every new
//! request first purges the queue, so pressing play twice does not queue two
//! readings behind each other but replaces the one that is running.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Condvar, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Media::Speech::{
    ISpObjectToken, ISpObjectTokenCategory, ISpVoice, SpObjectTokenCategory, SpVoice, SPCAT_VOICES,
    SPF_ASYNC, SPF_PURGEBEFORESPEAK,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    COINIT_APARTMENTTHREADED,
};

/// How long the interface waits for the worker to report that it has a voice.
const READY_TIMEOUT: Duration = Duration::from_millis(1500);

/// What the interface asks the voice to do.
enum Command {
    Speak {
        text: String,
        rate: i32,
        language: Option<String>,
    },
    Stop,
}

/// The command queue, created with the worker thread it feeds.
fn commands() -> &'static Mutex<Sender<Command>> {
    static COMMANDS: OnceLock<Mutex<Sender<Command>>> = OnceLock::new();
    COMMANDS.get_or_init(|| {
        let (sender, receiver) = channel();
        if let Err(error) = thread::Builder::new()
            .name("glossy-speech".to_string())
            .spawn(move || worker(receiver))
        {
            eprintln!("Glossy could not start its speech thread: {error}");
        }
        Mutex::new(sender)
    })
}

/// Set by the worker once COM and the voice are ready, so `speak` can report a
/// machine that has no voice at all instead of failing silently.
type Readiness = (Mutex<Option<Result<(), String>>>, Condvar);

fn readiness() -> &'static Readiness {
    static READY: OnceLock<Readiness> = OnceLock::new();
    READY.get_or_init(|| (Mutex::new(None), Condvar::new()))
}

fn send(command: Command) {
    if let Ok(sender) = commands().lock() {
        let _ = sender.send(command);
    }
}

/// Starts reading `text` out loud, replacing whatever is still being read.
///
/// `language` is the language of the translation, used to pick a voice that
/// pronounces it; the default voice is kept when the machine has no voice for
/// it. `rate` is SAPI's own -10..10 adjustment, where 0 is normal speed.
pub fn speak(text: &str, rate: i32, language: Option<&str>) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("there is nothing to read".to_string());
    }
    send(Command::Speak {
        text: text.to_string(),
        rate,
        language: language.map(str::to_string),
    });
    wait_until_ready()
}

/// Stops the reading that is in progress.
pub fn stop() {
    // Stopping does not need a voice, so a machine without one is not an error.
    send(Command::Stop);
}

/// Blocks until the worker has reported whether it has a voice.
fn wait_until_ready() -> Result<(), String> {
    let (ready, signal) = readiness();
    let guard = ready
        .lock()
        .map_err(|_| "the speech thread failed".to_string())?;
    if let Some(result) = guard.clone() {
        return result;
    }
    let (guard, timeout) = signal
        .wait_timeout_while(guard, READY_TIMEOUT, |ready| ready.is_none())
        .map_err(|_| "the speech thread failed".to_string())?;
    if timeout.timed_out() && guard.is_none() {
        return Err("the speech voice did not answer".to_string());
    }
    guard.clone().unwrap_or(Ok(()))
}

/// Owns the voice for as long as the application runs.
fn worker(receiver: Receiver<Command>) {
    let voice = unsafe { create_voice() };
    {
        let (ready, signal) = readiness();
        if let Ok(mut slot) = ready.lock() {
            *slot = Some(voice.as_ref().map(|_| ()).map_err(|error| error.clone()));
        }
        signal.notify_all();
    }

    // Without a voice there is nothing to wait for.
    let Ok(voice) = voice else {
        return;
    };
    let mut voices: HashMap<String, ISpObjectToken> = HashMap::new();
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Speak {
                text,
                rate,
                language,
            } => {
                if let Err(error) =
                    unsafe { speak_with(&voice, &mut voices, &text, rate, language.as_deref()) }
                {
                    eprintln!("Glossy could not read the translation out loud: {error}");
                }
            }
            // A null pointer with `SPF_PURGEBEFORESPEAK` drops everything that
            // is still queued, which is how SAPI stops.
            Command::Stop => {
                if let Err(error) = unsafe { voice.Speak(PCWSTR::null(), purge_flag(), None) } {
                    eprintln!("Glossy could not stop reading aloud: {error}");
                }
            }
        }
    }
    unsafe { CoUninitialize() };
}

/// The `dwflags` value `ISpVoice::Speak` expects, which the crate hands over as
/// a newtype.
fn purge_flag() -> u32 {
    SPF_PURGEBEFORESPEAK.0 as u32
}

/// Flags that queue the text and drop what is queued before it.
fn speak_flags() -> u32 {
    (SPF_ASYNC.0 | SPF_PURGEBEFORESPEAK.0) as u32
}

/// Initialises COM on this thread and creates the voice.
unsafe fn create_voice() -> Result<ISpVoice, String> {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    // The default voice is created with the voice itself, so no token is chosen
    // here: `SpVoice` is simply the voice the user configured in Windows.
    CoCreateInstance(&SpVoice, None, CLSCTX_ALL)
        .map_err(|error| format!("no speech voice is available ({error})"))
}

/// Speaks one piece of text, choosing a voice for `language` when possible.
unsafe fn speak_with(
    voice: &ISpVoice,
    cache: &mut HashMap<String, ISpObjectToken>,
    text: &str,
    rate: i32,
    language: Option<&str>,
) -> windows::core::Result<()> {
    if let Some(language) = language {
        if let Some(token) = cached_voice(cache, language)? {
            voice.SetVoice(&token)?;
        }
    }
    voice.SetRate(rate.clamp(-10, 10))?;
    // Drop the previous reading before queueing this one.
    voice.Speak(PCWSTR::null(), purge_flag(), None)?;
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    voice.Speak(PCWSTR(wide.as_ptr()), speak_flags(), None)
}

/// Finds the voice for `language` once and remembers it, so the token list is
/// only enumerated when a new language appears.
unsafe fn cached_voice(
    cache: &mut HashMap<String, ISpObjectToken>,
    language: &str,
) -> windows::core::Result<Option<ISpObjectToken>> {
    if let Some(token) = cache.get(language) {
        return Ok(Some(token.clone()));
    }
    // The machine has no voice for this language: the one that is already
    // installed is a better answer than silence.
    let Some(token) = voice_for(language)? else {
        return Ok(None);
    };
    cache.insert(language.to_string(), token.clone());
    Ok(Some(token))
}

/// The installed voice whose language matches `language`, if there is one.
unsafe fn voice_for(language: &str) -> windows::core::Result<Option<ISpObjectToken>> {
    let Some(wanted) = primary_language_id(language) else {
        return Ok(None);
    };
    let category: ISpObjectTokenCategory =
        CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL)?;
    category.SetId(SPCAT_VOICES, false)?;
    let tokens = category.EnumTokens(PCWSTR::null(), PCWSTR::null())?;
    let mut count = 0u32;
    tokens.GetCount(&mut count)?;
    for index in 0..count {
        let token = tokens.Item(index)?;
        let Ok(value) = token.GetStringValue(windows::core::w!("Language")) else {
            continue;
        };
        if take_pwstr(value).is_some_and(|text| lcid_matches(&text, wanted)) {
            return Ok(Some(token));
        }
    }
    Ok(None)
}

/// Copies a string the API allocated and releases it again.
unsafe fn take_pwstr(value: PWSTR) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let text = value.to_string().ok();
    CoTaskMemFree(Some(value.0 as *const core::ffi::c_void));
    text
}

/// True when the `Language` attribute of a voice, a hexadecimal LCID or a list
/// of them, names the language `wanted`.
fn lcid_matches(attribute: &str, wanted: u16) -> bool {
    attribute
        .split([',', ';'])
        .any(|value| parse_lcid(value) == Some(wanted))
}

/// The primary language part of a hexadecimal LCID, e.g. `409` -> English.
fn parse_lcid(value: &str) -> Option<u16> {
    let value = value.trim();
    let lcid = u16::from_str_radix(value, 16).ok()?;
    Some(lcid & 0x3FF)
}

/// The primary language id Windows uses for a language code such as `zh-CN`.
fn primary_language_id(language: &str) -> Option<u16> {
    let primary = language
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    Some(match primary.as_str() {
        "ar" => 0x01,
        "zh" => 0x04,
        "cs" => 0x05,
        "da" => 0x06,
        "de" => 0x07,
        "el" => 0x08,
        "en" => 0x09,
        "es" => 0x0A,
        "fi" => 0x0B,
        "fr" => 0x0C,
        "he" => 0x0D,
        "hu" => 0x0E,
        "is" => 0x0F,
        "it" => 0x10,
        "ja" => 0x11,
        "ko" => 0x12,
        "nl" => 0x13,
        "no" | "nb" => 0x14,
        "pl" => 0x15,
        "pt" => 0x16,
        "ro" => 0x18,
        "ru" => 0x19,
        "hr" | "sr" => 0x1A,
        "sk" => 0x1B,
        "sq" => 0x1C,
        "sv" => 0x1D,
        "th" => 0x1E,
        "tr" => 0x1F,
        "ur" => 0x20,
        "id" => 0x21,
        "uk" => 0x22,
        "be" => 0x23,
        "sl" => 0x24,
        "et" => 0x25,
        "lv" => 0x26,
        "lt" => 0x27,
        "fa" => 0x29,
        "vi" => 0x2A,
        "hy" => 0x2B,
        "az" => 0x2C,
        "eu" => 0x2D,
        "mk" => 0x2F,
        "af" => 0x36,
        "ka" => 0x37,
        "fo" => 0x38,
        "hi" => 0x39,
        "ms" => 0x3E,
        "kk" => 0x3F,
        "sw" => 0x41,
        "uz" => 0x43,
        "bn" => 0x45,
        "pa" => 0x46,
        "gu" => 0x47,
        "or" => 0x48,
        "ta" => 0x49,
        "te" => 0x4A,
        "kn" => 0x4B,
        "ml" => 0x4C,
        "mr" => 0x4E,
        "mn" => 0x50,
        "cy" => 0x52,
        "gl" => 0x56,
        "ne" => 0x61,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_voice_is_matched_by_the_language_of_the_translation() {
        assert_eq!(primary_language_id("en"), Some(0x09));
        assert_eq!(primary_language_id("en-US"), Some(0x09));
        assert_eq!(primary_language_id("zh-CN"), Some(0x04));
        assert_eq!(primary_language_id("pt_BR"), Some(0x16));
        assert_eq!(primary_language_id("JA"), Some(0x11));
        // A language Windows has no id for leaves the default voice in place.
        assert_eq!(primary_language_id("xx"), None);
        assert_eq!(primary_language_id(""), None);
    }

    #[test]
    fn the_language_attribute_of_a_voice_is_read_as_a_hexadecimal_lcid() {
        assert!(lcid_matches("409", 0x09));
        assert!(lcid_matches("804", 0x04));
        // Some voices list more than one language.
        assert!(lcid_matches("409,809", 0x09));
        assert!(lcid_matches(" 411 ", 0x11));
        assert!(!lcid_matches("409", 0x04));
        assert!(!lcid_matches("", 0x09));
    }
}
