//! Translating a whole document, one piece at a time.
//!
//! The popup translates a selection; this is the same translation with a file
//! for a source. The file is split into the pieces a reader would recognize —
//! paragraphs, headings, the text of a subtitle cue — each piece is sent to the
//! translator in turn, and the answers are put back in the order they were
//! asked for. Everything that must not be translated (a fenced code block, the
//! number and the timing of a cue, the markers that carry the structure) is
//! copied through untouched.
//!
//! A piece never crosses [`MAX_SEGMENT_CHARS`]: longer text is cut after the
//! last sentence end that fits, so no request is refused for its size and no
//! cut lands in the middle of a sentence when it can be avoided. The page may
//! ask for a shorter piece than that, and may ask for the pieces that hold no
//! letters to be left as they are.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::platform;
use crate::settings::Settings;
use crate::state::AppState;
use crate::translate::{self, Languages};

/// Event the settings window follows while a document is translated.
const EVENT: &str = "glossy://document";

/// Longest piece sent in one request: under the 1800 the popup allows and the
/// 2000 the relay accepts, so a piece is always inside both. The Document page
/// can ask for a shorter one, down to [`MIN_SEGMENT_CHARS`].
const MAX_SEGMENT_CHARS: usize = 1500;

/// Shortest limit the page may ask for. Below this a request would carry a
/// sentence or two, which costs a round trip for every line of the file.
const MIN_SEGMENT_CHARS: usize = 200;

/// Longest document translated at all, in characters. A book would spend the
/// whole allowance of the day in one press, so it is refused with the size
/// instead.
const MAX_DOCUMENT_CHARS: usize = 60_000;

/// How much of the finished translation the settings window is shown. The whole
/// text stays in the backend until it is saved.
const MAX_PREVIEW_CHARS: usize = 600;

/// Document translation is not finished, so every request for it is refused and
/// the page stays closed until it is.
const UNDER_DEVELOPMENT: &str =
    "Document translation is still being built, so nothing is translated yet.";

/// Refuses a document request while the feature is being built. It is read at
/// the top of every entry point, so the work behind it stays compiled and the
/// page cannot reach it either way.
fn under_development() -> Result<(), String> {
    Err(UNDER_DEVELOPMENT.to_string())
}

/// A piece of the file: copied through, or asked for a translation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Keep(String),
    Translate(String),
}

/// What a file is, which decides how it is split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Markdown,
    Subtitle,
    Pdf,
    Word,
}

impl Format {
    /// The format of a file name, or `None` for one Glossy does not open.
    fn of(name: &str) -> Option<Self> {
        let extension = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "txt" | "text" => Some(Self::Text),
            "md" | "markdown" => Some(Self::Markdown),
            "srt" => Some(Self::Subtitle),
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Word),
            _ => None,
        }
    }

    /// The short name the settings window shows.
    fn id(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Subtitle => "srt",
            Self::Pdf => "pdf",
            Self::Word => "docx",
        }
    }

    /// The extension the translation of a file is written with. A PDF comes
    /// out as plain text; everything else keeps the extension it came in with.
    fn output(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Subtitle => "srt",
            Self::Pdf => "txt",
            Self::Word => "docx",
        }
    }

    fn split(self, text: &str, splitting: Splitting) -> Vec<Part> {
        match self {
            Self::Text => paragraphs(text, splitting),
            // The text of a PDF arrives as the lines the pages were printed
            // with, so the wrapped lines are joined back into paragraphs first.
            Self::Pdf => paragraphs(&crate::text::normalize(text), splitting),
            Self::Markdown => markdown(text, splitting),
            Self::Subtitle => subtitles(text, splitting),
            // A word document is not text: its pieces come from the runs of its
            // paragraphs, in `open_word`.
            Self::Word => Vec::new(),
        }
    }
}

/// A file that has been read and split, waiting to be translated.
#[derive(Clone)]
struct Plan {
    name: String,
    /// The extension the translation is written with.
    extension: String,
    bom: bool,
    parts: Vec<Part>,
    /// A Word document is written back into a copy of itself rather than into
    /// text, so the package and the paragraph each piece came from are kept
    /// with the plan.
    word: Option<Arc<Word>>,
}

/// Where a piece of a Word document came from, so the translation of it can be
/// written back into the paragraph it belongs to.
struct Word {
    document: crate::docx::Document,
    /// The paragraph of the document each piece of the plan belongs to, in the
    /// order the pieces are asked for.
    paragraphs: Vec<usize>,
}

impl Plan {
    /// How many pieces need a translation, which is how many requests it takes.
    fn segments(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| matches!(part, Part::Translate(_)))
            .count()
    }

    /// The characters that will be sent, which is what the allowance is spent
    /// on: the parts that are copied through cost nothing.
    fn chars(&self) -> usize {
        self.parts
            .iter()
            .map(|part| match part {
                Part::Translate(text) => text.chars().count(),
                Part::Keep(_) => 0,
            })
            .sum()
    }
}

/// A finished translation, kept for the Save button.
struct Finished {
    name: String,
    /// The extension the file is written with, which is the one the source came
    /// in with, except for a PDF, whose translation is text.
    extension: String,
    bom: bool,
    text: String,
    /// The rewritten package of a Word document, which is what is saved instead
    /// of the text.
    bytes: Option<Vec<u8>>,
    /// The language it was translated into, which is part of the file name.
    target: String,
}

/// The file the page opened, and the translation of it once it is done.
static PLAN: Mutex<Option<Plan>> = Mutex::new(None);
static FINISHED: Mutex<Option<Finished>> = Mutex::new(None);
/// Set by Cancel, read between two pieces.
static STOPPED: AtomicBool = AtomicBool::new(false);
/// How a file is split: the limit the page chose for one request, and whether
/// the pieces that hold no letters are left alone.
///
/// A file is split once, while it is opened, so the choices travel with the
/// call rather than sitting in a global: two files opened at the same time would
/// otherwise overwrite each other's, and one of them would be split by the
/// other's limit.
#[derive(Clone, Copy)]
struct Splitting {
    /// The longest piece one request may hold.
    limit: usize,
    /// Whether a piece that holds no letters is copied through instead.
    skip_plain: bool,
}

impl Splitting {
    /// The choices the page sent, brought inside the range the backend takes.
    fn asked(segment_chars: Option<usize>, skip_plain_parts: Option<bool>) -> Self {
        Self {
            limit: segment_chars
                .unwrap_or(MAX_SEGMENT_CHARS)
                .clamp(MIN_SEGMENT_CHARS, MAX_SEGMENT_CHARS),
            skip_plain: skip_plain_parts.unwrap_or(true),
        }
    }
}

/// What the page is told about a file it has just picked.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub name: String,
    /// `txt`, `md` or `srt`.
    pub format: String,
    /// Characters that will be sent to the translator.
    pub chars: usize,
    /// How many requests that takes.
    pub segments: usize,
    /// The beginning of what will be translated, so a file read in the wrong
    /// encoding is visible before the allowance is spent on it.
    pub sample: String,
}

/// What the settings window is told while a document is translated.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state")]
enum Progress {
    /// `done` of `total` pieces are translated.
    #[serde(rename = "progress")]
    Underway { done: usize, total: usize },
    #[serde(rename = "done")]
    Done {
        done: usize,
        total: usize,
        preview: String,
    },
    #[serde(rename = "failed")]
    Failed {
        message: String,
        done: usize,
        total: usize,
    },
    #[serde(rename = "cancelled")]
    Cancelled { done: usize, total: usize },
}

/// Refused while the feature is being built; the work itself is [`open`] below.
#[tauri::command]
pub fn document_open(
    name: String,
    data: String,
    segment_chars: Option<usize>,
    skip_plain_parts: Option<bool>,
) -> Result<Info, String> {
    under_development()?;
    let (info, _plan) = open(name, data, segment_chars, skip_plain_parts)?;
    Ok(info)
}

/// Reads the file the page picked and splits it, ready to be translated.
///
/// The piece limit and the skip of the parts that hold no letters belong to the
/// page: a file is split once, here, so they are read on the way in.
///
/// The bytes arrive as base64 rather than as text: a `.txt` or an `.srt`
/// written by an older Windows is often in the system code page, and only the
/// bytes let that be noticed and read properly. A `.pdf` and a `.docx` are not
/// text at all, so they are read into the paragraphs they hold instead.
///
/// The plan comes back with the summary instead of being read out of [`PLAN`]:
/// the file the app has open is one, and a caller that wants the plan of the
/// file it has just opened must not have to race another that is opening one.
fn open(
    name: String,
    data: String,
    segment_chars: Option<usize>,
    skip_plain_parts: Option<bool>,
) -> Result<(Info, Plan), String> {
    let bytes = STANDARD
        .decode(data.trim())
        .map_err(|error| format!("the file could not be read: {error}"))?;
    let format = Format::of(&name).ok_or_else(|| {
        format!(
            "Glossy translates text, Markdown, subtitle, PDF and Word (.docx) files; `{name}` \
             is none of those. An older Word file (.doc) has to be saved as .docx first."
        )
    })?;
    let splitting = Splitting::asked(segment_chars, skip_plain_parts);
    let plan = match format {
        Format::Word => open_word(name, &bytes, splitting)?,
        // A PDF is read into text: what is translated is the text of it, not
        // the pages, and the answer is written as a text file.
        Format::Pdf => Plan {
            name,
            extension: format.output().to_string(),
            bom: false,
            parts: format.split(&pdf_text(&bytes)?, splitting),
            word: None,
        },
        format => {
            let (text, bom) = decode(&bytes)?;
            Plan {
                name,
                extension: format.output().to_string(),
                bom,
                parts: format.split(&text, splitting),
                word: None,
            }
        }
    };
    if plan.segments() == 0 {
        return Err("There is no text to translate in this file.".to_string());
    }
    let chars = plan.chars();
    if chars > MAX_DOCUMENT_CHARS {
        return Err(format!(
            "This document holds {chars} characters to translate; Glossy takes at most \
             {MAX_DOCUMENT_CHARS} at a time."
        ));
    }
    let info = Info {
        name: plan.name.clone(),
        format: format.id().to_string(),
        chars,
        segments: plan.segments(),
        sample: sample_of(&plan),
    };
    // A translation of the file that was open is no longer what the page is
    // looking at, so a run still in flight is asked to stop.
    STOPPED.store(true, Ordering::SeqCst);
    *FINISHED.lock().unwrap() = None;
    *PLAN.lock().unwrap() = Some(plan.clone());
    Ok((info, plan))
}

/// Translates the file that is open, reporting after every piece.
///
/// A failure stops the run instead of skipping the piece: a translation with a
/// hole in it is worse than no translation, and the page says which piece it
/// stopped on.
#[tauri::command]
pub async fn document_start(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    source: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    under_development()?;
    let Some(plan) = PLAN.lock().unwrap().clone() else {
        return Err("No document is open.".to_string());
    };
    let mut settings: Settings = state.settings();
    // One request per piece, so the sentence pairing of the popup would only
    // add a second copy of every line to the answer.
    settings.sentence_pairs = false;
    let languages = Languages { source, target };
    // The file is named after the language it came out in: the one picked here,
    // or the one the settings translate into when the page left it to them.
    let target = languages
        .target_code()
        .map(str::to_string)
        .unwrap_or_else(|| settings.target_lang.clone());
    let total = plan.segments();
    STOPPED.store(false, Ordering::SeqCst);
    let mut done = 0usize;
    let mut text = String::new();
    // The translation of a Word paragraph is put together from the pieces it
    // was cut into, so it can be written back into the paragraph as one text.
    let mut translated: Vec<(usize, String)> = Vec::new();
    let mut asked = 0usize;
    for part in &plan.parts {
        let segment = match part {
            Part::Keep(kept) => {
                text.push_str(kept);
                continue;
            }
            Part::Translate(segment) => segment,
        };
        if STOPPED.load(Ordering::SeqCst) {
            emit(&app, Progress::Cancelled { done, total });
            return Ok(());
        }
        match translate::translate(segment, &settings, &languages).await {
            Ok(result) => match &plan.word {
                Some(word) => {
                    let paragraph = word.paragraphs.get(asked).copied().unwrap_or(usize::MAX);
                    match translated.last_mut() {
                        Some((index, buffer)) if *index == paragraph => {
                            buffer.push_str(&result.translation)
                        }
                        _ => translated.push((paragraph, result.translation)),
                    }
                }
                None => text.push_str(&result.translation),
            },
            Err(message) => {
                emit(
                    &app,
                    Progress::Failed {
                        message,
                        done,
                        total,
                    },
                );
                return Ok(());
            }
        }
        asked += 1;
        done += 1;
        emit(&app, Progress::Underway { done, total });
    }
    let (text, bytes) = match &plan.word {
        Some(word) => {
            let text = translated
                .iter()
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");
            let translations = translated.into_iter().collect();
            match word.document.rewrite(&translations) {
                Ok(bytes) => (text, Some(bytes)),
                Err(message) => {
                    emit(
                        &app,
                        Progress::Failed {
                            message,
                            done,
                            total,
                        },
                    );
                    return Ok(());
                }
            }
        }
        None => (text, None),
    };
    let preview = preview_of(&text);
    *FINISHED.lock().unwrap() = Some(Finished {
        name: plan.name.clone(),
        extension: plan.extension.clone(),
        bom: plan.bom,
        text,
        bytes,
        target,
    });
    emit(
        &app,
        Progress::Done {
            done,
            total,
            preview,
        },
    );
    Ok(())
}

/// Asks the translation in flight to stop after the piece it is on.
#[tauri::command]
pub fn document_cancel() {
    STOPPED.store(true, Ordering::SeqCst);
}

/// Writes the finished translation to the folder the page chose, the Desktop
/// when it chose none, and answers with the path it used.
#[tauri::command]
pub fn document_save(app: AppHandle, directory: Option<String>) -> Result<String, String> {
    under_development()?;
    let finished = FINISHED.lock().unwrap();
    let finished = finished
        .as_ref()
        .ok_or("There is no finished translation to save.")?;
    let directory = save_directory(&app, directory)?;
    let path = directory.join(output_name(
        &finished.name,
        &finished.target,
        &finished.extension,
    ));
    let bytes = match &finished.bytes {
        // A Word document goes out as the document it came in as, with the
        // translated paragraphs written into it, so its styles, its tables and
        // its pictures are the ones the reader already knows.
        Some(bytes) => bytes.clone(),
        None => {
            let mut bytes = Vec::new();
            // A file that came in with a byte order mark goes out with one, so
            // a reader that needed it still finds it.
            if finished.bom {
                bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            }
            bytes.extend_from_slice(finished.text.as_bytes());
            bytes
        }
    };
    std::fs::write(&path, bytes)
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    Ok(path.display().to_string())
}

/// Tells the settings window what happened. The window is the one that started
/// the translation, so a window that is gone simply receives nothing.
fn emit(app: &AppHandle, progress: Progress) {
    if let Some(window) = app.get_webview_window(crate::MAIN_LABEL) {
        let _ = window.emit(EVENT, progress);
    }
}

/// The folder a translation is written into: the one the page chose, or the
/// Desktop, which is where a file that was asked for usually belongs.
fn save_directory(app: &AppHandle, chosen: Option<String>) -> Result<PathBuf, String> {
    let chosen = chosen
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(chosen) = chosen {
        let path = PathBuf::from(&chosen);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(format!(
            "`{chosen}` is not a folder Glossy can save into; pick another one."
        ));
    }
    app.path()
        .desktop_dir()
        .or_else(|_| app.path().document_dir())
        .map_err(|error| {
            format!("neither the Desktop nor the Documents folder is available: {error}")
        })
}

/// Asks Windows for the folder translations should be saved into. The picker is
/// the one every program uses, so it looks like any other Save As.
#[tauri::command]
pub async fn document_pick_directory(app: AppHandle) -> Option<String> {
    // Closed while the feature is being built, so no folder picker opens.
    if under_development().is_err() {
        return None;
    }
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string())
}

/// The name of the file to write: the one it came from, the language it was
/// translated into, and the extension the translation is written with.
fn output_name(name: &str, target: &str, extension: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("document");
    let language = target.trim();
    let language = if language.is_empty() {
        "translated"
    } else {
        language
    };
    format!("{stem}.{language}.{extension}")
}

/// The text of a PDF, as far as it can be taken out of the pages without a
/// layout engine: the words in the order they are printed, with the lines a
/// paragraph was wrapped over joined back together.
fn pdf_text(bytes: &[u8]) -> Result<String, String> {
    let printed = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|error| format!("this PDF could not be read: {error}"))?;
    let text = crate::text::normalize(&printed);
    // A page of a scanned document holds a picture of the words, not the words,
    // and the allowance of the day would be spent on finding that out.
    if !text.chars().any(char::is_alphabetic) {
        return Err(
            "There is no text in this PDF to translate: it holds pictures of the \
                    words, as a scan does. Take a screenshot of the page instead."
                .to_string(),
        );
    }
    Ok(text)
}

/// Reads a Word package and lays out the paragraphs that need a translation.
///
/// A paragraph is one piece, cut further when it is longer than one request may
/// be, and every piece remembers the paragraph it came from so the translation
/// of it can be written back where the source was.
fn open_word(name: String, bytes: &[u8], splitting: Splitting) -> Result<Plan, String> {
    let document = crate::docx::Document::parse(bytes.to_vec())
        .map_err(|error| format!("`{name}` could not be read: {error}"))?;
    let paragraphs = document
        .paragraphs()
        .map_err(|error| format!("`{name}` could not be read: {error}"))?;
    let mut parts = Vec::new();
    let mut indices = Vec::new();
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if splitting.skip_plain && !worth_translating(paragraph) {
            continue;
        }
        let pieces = chunk(paragraph, splitting);
        indices.extend(std::iter::repeat(index).take(pieces.len()));
        parts.extend(pieces);
    }
    Ok(Plan {
        name,
        extension: Format::Word.output().to_string(),
        bom: false,
        parts,
        word: Some(Arc::new(Word {
            document,
            paragraphs: indices,
        })),
    })
}

/// Whether a paragraph is worth a request. One that holds only digits or
/// punctuation would come back unchanged, and the allowance of the day would
/// still be spent on it.
fn worth_translating(text: &str) -> bool {
    text.chars().any(char::is_alphabetic)
}

/// Decodes the bytes of a text file, and reports whether it carried a byte
/// order mark.
fn decode(bytes: &[u8]) -> Result<(String, bool), String> {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(rest.to_vec())
            .map(|text| (text, true))
            .map_err(|_| "This file is not text Glossy can read.".to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return Ok((utf16(rest, true), true));
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return Ok((utf16(rest, false), true));
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => Ok((text.to_string(), false)),
        // Not UTF-8, so it is in the code page this copy of Windows uses:
        // asking the system to read it is what keeps an older file usable.
        Err(_) => platform::encoding::from_system_code_page(bytes)
            .map(|text| (text, false))
            .ok_or_else(|| "This file is not text Glossy can read.".to_string()),
    }
}

/// Decodes UTF-16 without the mark that announced it.
fn utf16(bytes: &[u8], little_endian: bool) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16_lossy(&units)
}

/// The lines of `text`, each keeping the line ending that follows it, so the
/// pieces of the file can be put back together exactly as they came.
fn lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            out.push(&text[start..index + 1]);
            start = index + 1;
        }
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// The line without the ending that follows it.
fn body(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

fn is_blank(line: &str) -> bool {
    body(line).trim().is_empty()
}

/// A word or two of what will be translated, for the page to show.
fn sample_of(plan: &Plan) -> String {
    let text = plan
        .parts
        .iter()
        .find_map(|part| match part {
            Part::Translate(text) => Some(text.as_str()),
            Part::Keep(_) => None,
        })
        .unwrap_or("");
    let sample: String = text
        .chars()
        .take(160)
        .map(|ch| if ch.is_whitespace() { ' ' } else { ch })
        .collect();
    sample.trim().to_string()
}

/// The beginning of the translation, which is what the window shows.
fn preview_of(text: &str) -> String {
    let mut preview: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    if preview.chars().count() < text.chars().count() {
        preview.push('…');
    }
    preview
}

/// Ends the piece that is being collected, if there is one.
///
/// A piece that holds no letters is copied through instead of being sent when
/// the page asked for that: the answer would be the same text, and the
/// allowance of the day would still be spent on it.
fn flush(parts: &mut Vec<Part>, piece: &mut String, splitting: Splitting) {
    if piece.is_empty() {
        return;
    }
    if splitting.skip_plain && !worth_translating(piece) {
        parts.push(Part::Keep(piece.clone()));
    } else {
        parts.append(&mut chunk(piece, splitting));
    }
    piece.clear();
}

/// Splits a plain text file: a paragraph — a run of lines with no blank line
/// between them — is one piece, and the blank lines keep the shape of the file.
fn paragraphs(text: &str, splitting: Splitting) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut piece = String::new();
    for line in lines(text) {
        if is_blank(line) {
            flush(&mut parts, &mut piece, splitting);
            parts.push(Part::Keep(line.to_string()));
        } else {
            piece.push_str(line);
        }
    }
    flush(&mut parts, &mut piece, splitting);
    parts
}

/// Splits a Markdown file. Fenced code, the front matter and the markers that
/// carry the structure are copied through; the text of a heading, a list item,
/// a quote or a paragraph is translated.
fn markdown(text: &str, splitting: Splitting) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut piece = String::new();
    let mut fence: Option<&str> = None;
    let mut in_front_matter = text.starts_with("---\n") || text.starts_with("---\r\n");
    // The opening `---` is the first line, so only a later one ends the block.
    let mut front_matter_opener = in_front_matter;
    for line in lines(text) {
        let content = body(line);
        if let Some(marker) = fence {
            parts.push(Part::Keep(line.to_string()));
            if content.trim_start().starts_with(marker) {
                fence = None;
            }
            continue;
        }
        if in_front_matter {
            parts.push(Part::Keep(line.to_string()));
            if front_matter_opener {
                front_matter_opener = false;
            } else if content.trim_end() == "---" {
                in_front_matter = false;
            }
            continue;
        }
        if let Some(marker) = code_fence(content) {
            flush(&mut parts, &mut piece, splitting);
            parts.push(Part::Keep(line.to_string()));
            fence = Some(marker);
            continue;
        }
        if is_blank(line) || is_thematic_break(content) {
            flush(&mut parts, &mut piece, splitting);
            parts.push(Part::Keep(line.to_string()));
            continue;
        }
        let (marker, rest) = marker_of(content);
        if marker.is_empty() {
            piece.push_str(line);
            continue;
        }
        flush(&mut parts, &mut piece, splitting);
        parts.push(Part::Keep(marker.to_string()));
        parts.append(&mut chunk(rest, splitting));
        // The line ending that followed the text, so the file keeps its shape.
        if line.len() > content.len() {
            parts.push(Part::Keep(line[content.len()..].to_string()));
        }
    }
    flush(&mut parts, &mut piece, splitting);
    parts
}

/// The fence a line opens a code block with, if it opens one.
fn code_fence(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

/// Whether a line is a rule rather than text, such as `---` or `***`.
fn is_thematic_break(line: &str) -> bool {
    let trimmed: Vec<char> = line.chars().filter(|ch| !ch.is_whitespace()).collect();
    let Some(marker) = trimmed.first().copied() else {
        return false;
    };
    trimmed.len() >= 3
        && matches!(marker, '-' | '*' | '_')
        && trimmed.iter().all(|ch| *ch == marker)
}

/// The Markdown marker a line starts with, and the text after it. An empty
/// marker means the line is ordinary text, which is translated as it stands.
fn marker_of(line: &str) -> (&str, &str) {
    let bytes = line.as_bytes();
    match bytes.first() {
        Some(&ch @ (b'#' | b'>' | b'-' | b'*' | b'+')) => {
            let mut index = 0;
            while index < bytes.len() && bytes[index] == ch {
                index += 1;
            }
            // A run of the same character is a rule or the start of an
            // emphasised word rather than a marker, except for the quote and the
            // heading, which may nest.
            if index > 1 && ch != b'>' && ch != b'#' {
                return ("", line);
            }
            if ch == b'#' && index > 6 {
                return ("", line);
            }
            if index == bytes.len() {
                return ("", line);
            }
            if bytes[index] == b' ' {
                index += 1;
            } else {
                return ("", line);
            }
            (&line[..index], &line[index..])
        }
        Some(ch) if ch.is_ascii_digit() => {
            let mut index = 0;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if matches!(bytes.get(index), Some(b'.') | Some(b')'))
                && bytes.get(index + 1) == Some(&b' ')
            {
                return (&line[..index + 2], &line[index + 2..]);
            }
            ("", line)
        }
        // The first cell of a table row: the pipes that separate the cells stay
        // inside the text, so the table keeps its shape.
        Some(b'|') if bytes.get(1) == Some(&b' ') => ("| ", &line[2..]),
        Some(b'|') => ("|", &line[1..]),
        _ => ("", line),
    }
}

/// Splits an `.srt` file: the number and the timing line of a cue are copied
/// through, and the lines of its text are translated as one piece.
fn subtitles(text: &str, splitting: Splitting) -> Vec<Part> {
    let all = lines(text);
    let mut parts = Vec::new();
    let mut index = 0;
    while index < all.len() {
        if !is_blank(all[index]) && starts_cue(&all, index) {
            parts.push(Part::Keep(all[index].to_string()));
            parts.push(Part::Keep(all[index + 1].to_string()));
            index += 2;
            let mut cue = String::new();
            while index < all.len() && !is_blank(all[index]) && !starts_cue(&all, index) {
                cue.push_str(all[index]);
                index += 1;
            }
            if splitting.skip_plain && !worth_translating(&cue) {
                parts.push(Part::Keep(cue));
            } else {
                parts.append(&mut chunk(&cue, splitting));
            }
            continue;
        }
        parts.push(Part::Keep(all[index].to_string()));
        index += 1;
    }
    parts
}

/// Whether line `index` is the number of a cue and the line after it holds the
/// timings, which is what marks the start of a subtitle.
fn starts_cue(all: &[&str], index: usize) -> bool {
    let Some(timing) = all.get(index + 1) else {
        return false;
    };
    let number = body(all[index]).trim();
    !number.is_empty()
        && number.chars().all(|ch| ch.is_ascii_digit())
        && body(timing).contains("-->")
}

/// Splits a piece into the parts that fit in one request.
fn chunk(text: &str, splitting: Splitting) -> Vec<Part> {
    let limit = splitting.limit;
    let mut parts = Vec::new();
    let mut rest = text;
    while rest.chars().count() > limit {
        let cut = cut_point(rest, limit);
        let (head, tail) = rest.split_at(cut);
        if head.is_empty() {
            break;
        }
        parts.push(Part::Translate(head.to_string()));
        rest = tail;
    }
    if !rest.is_empty() {
        parts.push(Part::Translate(rest.to_string()));
    }
    parts
}

/// Where to cut a piece that is too long: after the last sentence end that
/// fits, or after the last space, or at the limit when it is one long run.
///
/// A cut is only taken from the second half of the piece, so a full stop near
/// the beginning cannot leave many one-word requests behind.
fn cut_point(text: &str, limit: usize) -> usize {
    let floor = limit / 2;
    let mut count = 0;
    let mut sentence = 0;
    let mut space = 0;
    for (index, ch) in text.char_indices() {
        count += 1;
        if count > limit {
            break;
        }
        let end = index + ch.len_utf8();
        if is_sentence_end(ch) {
            sentence = end;
        } else if ch.is_whitespace() {
            space = end;
        }
    }
    if sentence >= floor {
        return sentence;
    }
    if space >= floor {
        return space;
    }
    // One unbroken run, or a break too early to be worth taking: cut at the
    // limit, on a character boundary.
    text.char_indices()
        .nth(limit)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

/// Whether a character ends a sentence, in either of the scripts Glossy is
/// likely to be handed.
fn is_sentence_end(ch: char) -> bool {
    matches!(
        ch,
        '.' | '!' | '?' | ';' | '\n' | '。' | '！' | '？' | '；' | '…'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The choices a page that asked for nothing gets, which is what these
    /// tests exercise unless they say otherwise.
    fn defaults() -> Splitting {
        Splitting::asked(None, None)
    }

    /// Splitting a file and putting it back without translating anything has to
    /// give the file back exactly as it was.
    fn round_trip(format: Format, text: &str) -> String {
        format
            .split(text, defaults())
            .iter()
            .map(|part| match part {
                Part::Keep(text) | Part::Translate(text) => text.as_str(),
            })
            .collect()
    }

    fn parts(format: Format, text: &str) -> Vec<Part> {
        format.split(text, defaults())
    }

    fn translated(format: Format, text: &str) -> Vec<String> {
        parts(format, text)
            .into_iter()
            .filter_map(|part| match part {
                Part::Translate(text) => Some(text),
                Part::Keep(_) => None,
            })
            .collect()
    }

    #[test]
    fn a_text_file_keeps_its_blank_lines_and_translates_its_paragraphs() {
        let source = "First paragraph.\nStill the first.\n\nSecond paragraph.\n";
        assert_eq!(round_trip(Format::Text, source), source);
        assert_eq!(
            translated(Format::Text, source),
            vec![
                "First paragraph.\nStill the first.\n",
                "Second paragraph.\n"
            ]
        );
    }

    #[test]
    fn a_long_paragraph_becomes_several_requests_that_stay_whole_sentences() {
        let sentence = "This is one sentence of a longer paragraph. ";
        let source = sentence.repeat(120);
        let pieces = translated(Format::Text, &source);
        assert!(pieces.len() > 1);
        for piece in &pieces {
            assert!(piece.chars().count() <= MAX_SEGMENT_CHARS);
        }
        assert_eq!(pieces.concat(), source);
        assert!(pieces[0].trim_end().ends_with('.'));
    }

    #[test]
    fn a_run_without_spaces_is_cut_at_the_limit() {
        let source = "汉".repeat(MAX_SEGMENT_CHARS * 2 + 7);
        let pieces = translated(Format::Text, &source);
        assert_eq!(pieces.len(), 3);
        for piece in &pieces {
            assert!(piece.chars().count() <= MAX_SEGMENT_CHARS);
        }
        assert_eq!(pieces.concat(), source);
    }

    #[test]
    fn markdown_keeps_its_structure_and_translates_the_text_between_it() {
        let source = "---\ntitle: Notes\n---\n\n# A heading\n\n- one item\n- two items\n\n\
                      ```rust\nlet untranslated = 1;\n```\n\nA paragraph.\n";
        assert_eq!(round_trip(Format::Markdown, source), source);
        let pieces = translated(Format::Markdown, source);
        assert!(pieces.contains(&"A heading".to_string()));
        assert!(pieces.contains(&"one item".to_string()));
        assert!(pieces.contains(&"A paragraph.\n".to_string()));
        assert!(!pieces.iter().any(|piece| piece.contains("untranslated")));
        assert!(!pieces.iter().any(|piece| piece.contains("title:")));
    }

    #[test]
    fn markdown_keeps_the_numbering_of_an_ordered_list() {
        let source = "1. first\n2. second\n";
        assert_eq!(round_trip(Format::Markdown, source), source);
        assert_eq!(
            translated(Format::Markdown, source),
            vec!["first", "second"]
        );
    }

    #[test]
    fn markdown_does_not_take_emphasis_for_a_list() {
        let source = "**Bold** opening.\n\n---\n";
        assert_eq!(round_trip(Format::Markdown, source), source);
        assert_eq!(
            translated(Format::Markdown, source),
            vec!["**Bold** opening.\n"]
        );
    }

    #[test]
    fn subtitles_translate_only_the_text_of_a_cue() {
        let source = "1\n00:00:01,000 --> 00:00:03,500\nHello there.\nSecond line.\n\n2\n\
                      00:00:04,000 --> 00:00:06,000\nGoodbye.\n";
        assert_eq!(round_trip(Format::Subtitle, source), source);
        assert_eq!(
            translated(Format::Subtitle, source),
            vec!["Hello there.\nSecond line.\n", "Goodbye.\n"]
        );
    }

    #[test]
    fn a_cue_without_a_blank_line_after_it_is_still_found() {
        let source =
            "1\n00:00:01,000 --> 00:00:03,500\nOne.\n2\n00:00:04,000 --> 00:00:06,000\nTwo.\n";
        assert_eq!(round_trip(Format::Subtitle, source), source);
        assert_eq!(
            translated(Format::Subtitle, source),
            vec!["One.\n", "Two.\n"]
        );
    }

    #[test]
    fn the_format_comes_from_the_extension() {
        assert_eq!(Format::of("notes.md"), Some(Format::Markdown));
        assert_eq!(Format::of("NOTES.Markdown"), Some(Format::Markdown));
        assert_eq!(Format::of("captions.srt"), Some(Format::Subtitle));
        assert_eq!(Format::of("readme.txt"), Some(Format::Text));
        assert_eq!(Format::of("report.PDF"), Some(Format::Pdf));
        assert_eq!(Format::of("letter.docx"), Some(Format::Word));
        assert_eq!(Format::of("letter.doc"), None);
        assert_eq!(Format::of("no-extension"), None);
    }

    #[test]
    fn a_translation_is_saved_next_to_the_original() {
        assert_eq!(Format::Pdf.output(), "txt");
        assert_eq!(Format::Word.output(), "docx");
        assert_eq!(Format::Text.output(), "txt");
        assert_eq!(Format::Markdown.output(), "md");
        assert_eq!(Format::Subtitle.output(), "srt");
    }

    #[test]
    fn the_saved_name_names_the_language_it_was_translated_into() {
        assert_eq!(output_name("notes.md", "zh-CN", "md"), "notes.zh-CN.md");
        assert_eq!(
            output_name("C:\\Users\\me\\a.b.srt", "en", "srt"),
            "a.b.en.srt"
        );
        assert_eq!(output_name("notes.md", "  ", "md"), "notes.translated.md");
        assert_eq!(output_name("notes", "en", "txt"), "notes.en.txt");
        assert_eq!(output_name("report.pdf", "ja", "txt"), "report.ja.txt");
    }

    #[test]
    fn utf8_and_utf16_files_are_read_and_their_mark_is_remembered() {
        let (text, bom) = decode("héllo".as_bytes()).expect("plain UTF-8 reads");
        assert_eq!((text.as_str(), bom), ("héllo", false));

        let mut marked = vec![0xEF, 0xBB, 0xBF];
        marked.extend_from_slice("héllo".as_bytes());
        let (text, bom) = decode(&marked).expect("a UTF-8 mark reads");
        assert_eq!((text.as_str(), bom), ("héllo", true));

        let wide: Vec<u8> = "héllo".encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut marked = vec![0xFF, 0xFE];
        marked.extend_from_slice(&wide);
        let (text, bom) = decode(&marked).expect("UTF-16 reads");
        assert_eq!((text.as_str(), bom), ("héllo", true));
    }

    #[test]
    fn a_short_piece_is_one_request() {
        assert_eq!(
            chunk("Hello.\n", defaults()),
            vec![Part::Translate("Hello.\n".into())]
        );
        assert_eq!(chunk("", defaults()), Vec::<Part>::new());
    }

    #[test]
    fn the_page_is_refused_while_the_feature_is_being_built() {
        match document_open(
            "notes.md".to_string(),
            STANDARD.encode(b"hello"),
            None,
            None,
        ) {
            Err(error) => assert_eq!(error, UNDER_DEVELOPMENT),
            Ok(_) => panic!("the page is closed until the feature is finished"),
        }
    }

    /// Opens a file the way the page does, with the choices it sends, and gives
    /// back both the summary and the plan the file was split into.
    fn opened_plan(
        name: &str,
        bytes: &[u8],
        limit: Option<usize>,
        skip: Option<bool>,
    ) -> (Info, Plan) {
        open(name.to_string(), STANDARD.encode(bytes), limit, skip).expect("the file opens")
    }

    /// The longest piece a plan asks to translate.
    fn longest_piece(plan: &Plan) -> usize {
        plan.parts
            .iter()
            .filter_map(|part| match part {
                Part::Translate(segment) => Some(segment.chars().count()),
                Part::Keep(_) => None,
            })
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn the_page_can_ask_for_shorter_pieces() {
        let text = "One sentence here. ".repeat(30);
        let (info, _) = opened_plan(
            "notes.txt",
            text.as_bytes(),
            Some(MAX_SEGMENT_CHARS),
            Some(true),
        );
        assert_eq!(info.segments, 1, "the whole file fits in the default limit");

        let (info, plan) = opened_plan(
            "notes.txt",
            text.as_bytes(),
            Some(MIN_SEGMENT_CHARS),
            Some(true),
        );
        assert!(info.segments > 1, "a shorter limit cuts the file up");
        assert!(longest_piece(&plan) <= MIN_SEGMENT_CHARS);

        // A limit outside the range the backend takes is brought inside it
        // rather than refused, and the next file opens with the default again.
        let (_, plan) = opened_plan("notes.txt", text.as_bytes(), Some(1), Some(true));
        assert!(longest_piece(&plan) <= MIN_SEGMENT_CHARS);
        assert_eq!(
            opened("notes.txt", text.as_bytes())
                .expect("opens")
                .segments,
            1
        );
    }

    #[test]
    fn pieces_that_hold_no_letters_are_left_alone_when_the_page_asks() {
        let text = "12345\n\nHello.\n";
        let (info, plan) = opened_plan("numbers.txt", text.as_bytes(), None, Some(true));
        assert_eq!((info.segments, info.chars), (1, 7));
        assert!(plan.parts.contains(&Part::Keep("12345\n".to_string())));

        let (info, _) = opened_plan("numbers.txt", text.as_bytes(), None, Some(false));
        assert_eq!((info.segments, info.chars), (2, 13));
    }

    #[test]
    fn the_preview_stops_at_a_character_boundary() {
        let text = "汉".repeat(MAX_PREVIEW_CHARS + 10);
        let preview = preview_of(&text);
        assert_eq!(preview.chars().count(), MAX_PREVIEW_CHARS + 1);
        assert!(preview.ends_with('…'));
    }

    /// A PDF with one page whose single line of text says `words`, with the
    /// offsets of its cross reference table counted rather than guessed.
    fn pdf(words: &str) -> Vec<u8> {
        let stream = format!("BT /F1 24 Tf 72 720 Td ({words}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources \
             << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
                .to_string(),
            format!(
                "<< /Length {} >>\nstream\n{stream}\nendstream",
                stream.len()
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        let mut out = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (number, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.push_str(&format!("{} 0 obj\n{body}\nendobj\n", number + 1));
        }
        let table = out.len();
        out.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
        out.push_str("0000000000 65535 f \n");
        for offset in &offsets {
            out.push_str(&format!("{offset:010} 00000 n \n"));
        }
        out.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{table}\n%%EOF\n",
            objects.len() + 1
        ));
        out.into_bytes()
    }

    fn opened(name: &str, bytes: &[u8]) -> Result<Info, String> {
        open(name.to_string(), STANDARD.encode(bytes), None, None).map(|(info, _)| info)
    }

    fn refused(name: &str, bytes: &[u8]) -> String {
        match opened(name, bytes) {
            Ok(_) => panic!("`{name}` should have been refused"),
            Err(error) => error,
        }
    }

    fn segments_of(parts: &[Part]) -> Vec<String> {
        parts
            .iter()
            .filter_map(|part| match part {
                Part::Translate(text) => Some(text.clone()),
                Part::Keep(_) => None,
            })
            .collect()
    }

    #[test]
    fn a_pdf_is_opened_as_the_text_of_its_pages() {
        let (info, plan) = opened_plan("report.pdf", &pdf("Hello there."), None, None);
        assert_eq!(info.format, "pdf");
        assert!(info.sample.contains("Hello there."), "{}", info.sample);
        assert_eq!(plan.extension, "txt");
        assert_eq!(plan.segments(), 1);
        let segments = segments_of(&plan.parts);
        assert_eq!(segments[0].trim(), "Hello there.");
    }

    #[test]
    fn a_pdf_that_holds_a_picture_instead_of_words_is_refused() {
        let error = refused("scan.pdf", &pdf(""));
        assert!(error.contains("scan"), "{error}");
    }

    #[test]
    fn a_word_document_is_opened_paragraph_by_paragraph() {
        let bytes = crate::docx::tests::package_of(&[(
            "word/document.xml",
            "<?xml version=\"1.0\"?><w:document xmlns:w=\"w\"><w:body>\
             <w:p><w:r><w:t>First line.</w:t></w:r></w:p>\
             <w:p><w:r><w:t>7</w:t></w:r></w:p>\
             <w:p><w:r><w:t>Second line.</w:t></w:r></w:p></w:body></w:document>",
        )]);
        let (info, plan) = opened_plan("letter.docx", &bytes, None, None);
        assert_eq!(info.format, "docx");
        assert_eq!(info.segments, 2);
        assert_eq!(plan.extension, "docx");
        assert_eq!(
            segments_of(&plan.parts),
            vec!["First line.", "Second line."]
        );
        let word = plan.word.as_ref().expect("the document is kept");
        // The paragraph that only holds a number is passed over, so the second
        // translation belongs to the third paragraph.
        assert_eq!(word.paragraphs, vec![0, 2]);
    }

    #[test]
    fn a_file_of_an_unknown_kind_says_what_is_translated() {
        let error = refused("letter.doc", b"whatever");
        assert!(error.contains(".docx"), "{error}");
    }
}
