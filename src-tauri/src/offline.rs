//! Translation read on this machine.
//!
//! The fourth service is not a service at all: the text never leaves the
//! machine. A direction — Chinese into English, English into Chinese — is two
//! ONNX models and a piece table, downloaded from the Resources page like the
//! recognition engine, and the same runtime the reader uses runs them.
//!
//! The models are OPUS-MT's `opus-mt-zh-en` and `opus-mt-en-zh` (Helsinki-NLP,
//! CC-BY-4.0), exported to ONNX by Xenova and quantized to int8. A direction is
//! read the way Marian reads: the piece table turns the text into pieces, the
//! encoder turns the pieces into one vector per piece, and the decoder writes
//! the translation a piece at a time, always taking the piece it is most sure
//! of — the search the model was published with is a beam of six, which is
//! several times the work for a difference a reader does not notice.
//!
//! Everything about a direction is one file: the download is skippable, the
//! removal is a delete, and a machine with the pack translates without a
//! connection and without a quota.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use ort::value::Tensor;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::classify;
use crate::translate::{self, TranslationResult, USER_AGENT};

/// Folder under the app's data directory that holds the pack, next to the one
/// the recognition engine lives in.
const FOLDER: &str = "translate";

/// Emitted whenever a download starts, makes progress, ends or fails.
pub const EVENT: &str = "glossy://offline-model";

/// The name the card shows as the engine that answered.
pub const PROVIDER: &str = "offline";

/// One file of the pack.
struct Asset {
    /// Name it is stored under, inside [`FOLDER`].
    name: &'static str,
    url: &'static str,
    /// Size of the download, which is what the progress bar counts.
    bytes: u64,
    /// Lowercase hex SHA-256 of the file that ends up on the disk, not of the
    /// archive it came out of.
    sha256: &'static str,
    /// Path of the file inside the download, for the one asset that arrives
    /// inside an archive; the runtime is published as a wheel, which is a zip
    /// holding the library among a lot of Python the app has no use for.
    entry: Option<&'static str>,
}

/// The runtime the models are read through, which every direction shares.
///
/// The same library the recognition engine downloads, kept here as well rather
/// than borrowed from it: the two packs are downloaded and removed on their own
/// rows, and one of them being gone must not take the other with it.
static RUNTIME: Asset = Asset {
    name: "onnxruntime.dll",
    url: "https://files.pythonhosted.org/packages/9f/10/3d946d5d5f2cdcc3c8da36cae63190c516d16349edaffd944bda60ca4c3e/onnxruntime-1.28.0-cp311-cp311-win_amd64.whl",
    bytes: 17_766_712,
    sha256: "14e186627f109a15f28f0b36f6e8ea07c79eac666818e63c39a9792f415bf649",
    entry: Some("onnxruntime/capi/onnxruntime.dll"),
};

/// One direction: the two halves of the model and the piece table it reads and
/// writes with.
struct Pair {
    /// Name the settings and the log lines agree on, and the way round it goes.
    id: &'static str,
    /// Language the model takes in.
    source: &'static str,
    /// Language it writes out.
    target: &'static str,
    encoder: Asset,
    decoder: Asset,
    pieces: Asset,
}

/// The two directions of the pack.
///
/// `opus-mt` is one model per direction, so the two are separate downloads
/// rather than one model asked twice; both are int8, which brings a direction
/// down to about 114 MB where the float weights are 445. They are also
/// downloaded one at a time, because half of the pack is a working translator
/// for whoever reads only one way round.
///
/// The files come from the mirror the recognition engine already downloads
/// from, which carries byte-for-byte the same artifacts as the original
/// `Xenova` repositories on Hugging Face — every digest below was checked
/// against both — and is several times faster from a mainland connection than
/// the other mirrors are.
static PAIRS: [Pair; 2] = [
    Pair {
        id: "zh-en",
        source: "zh-CN",
        target: "en",
        encoder: Asset {
            name: "zh-en_encoder.onnx",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=onnx/encoder_model_quantized.onnx",
            bytes: 52_899_742,
            sha256: "84d5e171b626bc8b6b220d022ac58696e9528c25deeacca62b5cbf4364547a99",
            entry: None,
        },
        decoder: Asset {
            name: "zh-en_decoder.onnx",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=onnx/decoder_model_quantized.onnx",
            bytes: 59_842_102,
            sha256: "debcc3054b9ff9aaed972bd6e251a459abe1e2f1402da1efa63163dc2f46ef73",
            entry: None,
        },
        pieces: Asset {
            name: "zh-en_tokenizer.json",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=tokenizer.json",
            bytes: 6_381_339,
            sha256: "b306d0301cf280bfd647d7067b5ade2a97b987e6d678df110703c002433643ff",
            entry: None,
        },
    },
    Pair {
        id: "en-zh",
        source: "en",
        target: "zh-CN",
        encoder: Asset {
            name: "en-zh_encoder.onnx",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=onnx/encoder_model_quantized.onnx",
            bytes: 52_899_742,
            sha256: "d3b7912bf6a9bd27e4c074c2df91d4ff3d5b4bc5f7f6c8d7cc9c805c98fbafee",
            entry: None,
        },
        decoder: Asset {
            name: "en-zh_decoder.onnx",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=onnx/decoder_model_quantized.onnx",
            bytes: 59_842_102,
            sha256: "2c66a3981099b40edbb3a0d65e015d05964d42fecb9780f8422776eff5939112",
            entry: None,
        },
        pieces: Asset {
            name: "en-zh_tokenizer.json",
            url: "https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=tokenizer.json",
            bytes: 6_380_952,
            sha256: "d0c7da27056e8f42adce9e76d8e792e5daa64e15f5acd2e7aabf0121877dd4c1",
            entry: None,
        },
    },
];

impl Pair {
    /// The three files of one direction, in the order they are fetched.
    ///
    /// The directions live in a `static` table, so the files they hand back are
    /// static too; saying so is what lets the caller keep them for as long as a
    /// download runs.
    fn assets(&'static self) -> Vec<&'static Asset> {
        vec![&self.encoder, &self.decoder, &self.pieces]
    }

    /// What this direction is, in bytes.
    fn bytes(&self) -> u64 {
        self.encoder.bytes + self.decoder.bytes + self.pieces.bytes
    }
}

/// The direction a downloaded id names; `None` for an id this build does not
/// offer, which is answered by downloading nothing rather than by guessing.
fn resolve(id: Option<&str>) -> Option<&'static Pair> {
    let wanted = id.map(str::trim).filter(|wanted| !wanted.is_empty())?;
    PAIRS.iter().find(|pair| pair.id == wanted)
}

/// Every file of the pack, the runtime first: what a download fetches when
/// nothing of it is on the disk yet.
fn every_asset() -> Vec<&'static Asset> {
    let mut all: Vec<&'static Asset> = vec![&RUNTIME];
    for pair in &PAIRS {
        all.push(&pair.encoder);
        all.push(&pair.decoder);
        all.push(&pair.pieces);
    }
    all
}

/// The whole pack, in bytes.
fn whole() -> u64 {
    every_asset().iter().map(|asset| asset.bytes).sum()
}

/// What the settings page is told about the pack.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Whether every file of the pack is on the disk.
    pub installed: bool,
    pub downloading: bool,
    pub bytes: u64,
    pub total: u64,
    pub error: Option<String>,
    /// What the pack takes up now, in bytes.
    pub size: u64,
    /// What the whole pack is, in bytes, so the page can say what a download
    /// would cost before it is started.
    pub full: u64,
    pub folder: String,
    /// Which direction is being fetched, or `runtime` while the shared library
    /// is; `None` when nothing is.
    pub target: Option<String>,
    /// The directions, and whether each one is ready.
    pub pairs: Vec<PairStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairStatus {
    pub id: &'static str,
    pub source: &'static str,
    pub target: &'static str,
    pub installed: bool,
    pub size: u64,
    /// Whether this one is the direction being downloaded right now.
    pub downloading: bool,
}

/// How far a download has come, where it is, and why it stopped.
#[derive(Default)]
struct Progress {
    bytes: u64,
    total: u64,
    target: Option<String>,
    error: Option<String>,
}

static PROGRESS: Mutex<Progress> = Mutex::new(Progress {
    bytes: 0,
    total: 0,
    target: None,
    error: None,
});

static DOWNLOADING: AtomicBool = AtomicBool::new(false);

/// What the progress of the shared runtime is reported under.
const RUNTIME_TARGET: &str = "runtime";

/// The folder the pack lives in, created if it is not there yet.
///
/// It is the directory beside the settings file, which is what the application
/// hands the pages, so nothing here needs an `AppHandle`: a translation asks
/// for the pack from the middle of the translation path, where there is none.
pub fn dir() -> Result<PathBuf, String> {
    let data = crate::settings::data_dir()
        .ok_or_else(|| "The application's data folder is not available.".to_string())?;
    let dir = data.join(FOLDER);
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("The offline translation folder could not be created: {error}"))?;
    Ok(dir)
}

/// Whether a file is the one it should be.
///
/// The digest is only checked while downloading; afterwards the size is enough,
/// because a file of the right size in the app's own folder is one this module
/// wrote.
fn present(path: &Path, asset: &Asset) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.len() == asset.bytes)
        .unwrap_or(false)
}

/// Whether everything the pack is made of is on the disk.
pub fn installed() -> bool {
    let Ok(dir) = dir() else {
        return false;
    };
    every_asset()
        .iter()
        .all(|asset| present(&dir.join(asset.name), asset))
}

/// Whether one direction can be read right now.
fn pair_ready_in(dir: &Path, pair: &Pair) -> bool {
    [&pair.encoder, &pair.decoder, &pair.pieces]
        .iter()
        .all(|asset| present(&dir.join(asset.name), asset))
}

/// Whether the pack translates this pair of languages at all.
///
/// The two directions are the whole of it, so a text and a target that name one
/// of them in either order is what it answers yes to. Anything else is a
/// channel that would have to refuse the translation anyway.
pub fn serves(source: &str, target: &str) -> bool {
    pair_for(source, target).is_some()
}

/// The direction a pair of language codes names.
///
/// The source is allowed to be "auto": a direction is picked by the language
/// the reader wants, and the model is the one that writes it. That is what
/// makes the automatic source work at all offline, where there is nothing to
/// ask which language a text is in.
fn pair_for(source: &str, target: &str) -> Option<&'static Pair> {
    let source = translate::normalize_lang_code(source);
    let target = translate::normalize_lang_code(target);
    let auto = source.is_empty() || source.eq_ignore_ascii_case("auto");
    let chinese = |code: &str| {
        code.eq_ignore_ascii_case("zh")
            || code.eq_ignore_ascii_case("zh-cn")
            || code.eq_ignore_ascii_case("zh-tw")
    };

    PAIRS.iter().find(|pair| {
        let wanted = pair.target.eq_ignore_ascii_case(&target);
        let given = auto
            || pair.source.eq_ignore_ascii_case(&source)
            || (pair.source == "zh-CN" && chinese(&source));
        wanted && given
    })
}

/// The address requests go to, as the downloader's user agent needs it.
///
/// The model host answers 403 to a request that carries no `User-Agent` header
/// at all, which is what a bare reqwest request is.
///
/// There is no whole-request timeout on purpose. A model is a hundred megabytes
/// over a connection that may be slow rather than broken, and a deadline that
/// cuts that off would fail a download that was still making progress — which
/// is the second half of why the first attempt at this took an hour. What is
/// watched instead is the gap between two reads: half a minute with nothing
/// arriving is a connection that has died, and that is reported.
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(30))
        .read_timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| format!("The downloader could not be set up: {error}"))
}

pub fn status() -> Status {
    let folder = dir()
        .map(|dir| dir.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = Path::new(&folder);
    let (bytes, total, target, error) = PROGRESS
        .lock()
        .map(|guard| {
            (
                guard.bytes,
                guard.total,
                guard.target.clone(),
                guard.error.clone(),
            )
        })
        .unwrap_or((0, 0, None, None));
    let downloading = DOWNLOADING.load(Ordering::SeqCst);
    let pairs: Vec<PairStatus> = PAIRS
        .iter()
        .map(|pair| PairStatus {
            id: pair.id,
            source: pair.source,
            target: pair.target,
            installed: pair_ready_in(dir, pair),
            size: pair.bytes(),
            downloading: downloading && target.as_deref() == Some(pair.id),
        })
        .collect();
    let size = every_asset()
        .iter()
        .filter(|asset| present(&dir.join(asset.name), asset))
        .map(|asset| asset.bytes)
        .sum();
    Status {
        // The runtime alone is of no use, so the pack counts as installed only
        // when a direction can actually be read.
        installed: pairs.iter().all(|pair| pair.installed),
        downloading,
        bytes: if downloading { bytes } else { 0 },
        total: if downloading { total } else { 0 },
        error,
        size,
        full: whole(),
        folder,
        target: if downloading { target } else { None },
        pairs,
    }
}

fn remember(bytes: u64, target: Option<&str>, total: u64, error: Option<String>) {
    if let Ok(mut guard) = PROGRESS.lock() {
        guard.bytes = bytes;
        guard.target = target.map(str::to_string);
        guard.total = total;
        guard.error = error;
    }
}

fn publish(app: &AppHandle) {
    let _ = app.emit(EVENT, status());
}

/// Downloads whatever of the pack is missing, and checks every file it wrote.
///
/// `id` names one direction, or is `None` for both of them. A direction is the
/// unit the page offers, because each one is a whole translator on its own:
/// somebody who reads only one way round never has to wait for the other half.
/// The runtime is shared and of no use by itself, so it goes with whichever
/// direction is asked for first.
///
/// What is already on the disk is skipped, so a download that was interrupted
/// picks up where it left off at the granularity of whole files.
///
/// The second caller does not start a second download: it is told one is
/// running, and can ask [`status`] until it is done.
pub async fn download(app: &AppHandle, id: Option<&str>) -> Result<(), String> {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return Err("The offline translation pack is still being downloaded.".to_string());
    }
    // The last failure is cleared here; what is being fetched is announced by
    // `install`, once it knows which group it actually has to fetch.
    remember(0, None, 0, None);
    publish(app);
    let outcome = install(app, id).await;
    DOWNLOADING.store(false, Ordering::SeqCst);
    // A failure keeps the name of the group it happened in, so the page can put
    // the message on the row that was downloading instead of on the whole pack.
    let last = PROGRESS.lock().ok().and_then(|guard| guard.target.clone());
    match &outcome {
        Ok(()) => remember(0, None, 0, None),
        Err(error) => remember(0, last.as_deref(), 0, Some(error.clone())),
    }
    publish(app);
    outcome
}

async fn install(app: &AppHandle, id: Option<&str>) -> Result<(), String> {
    let dir = dir()?;
    let client = client()?;

    let wanted: Vec<&'static Pair> = match resolve(id) {
        Some(pair) => vec![pair],
        None => PAIRS.iter().collect(),
    };

    // The runtime is shared by both directions, so it is fetched with whichever
    // one is being asked for rather than on a row of its own.
    let mut groups: Vec<(&'static str, Vec<&'static Asset>)> = Vec::new();
    if !present(&dir.join(RUNTIME.name), &RUNTIME) {
        groups.push((RUNTIME_TARGET, vec![&RUNTIME]));
    }
    for pair in wanted {
        groups.push((pair.id, pair.assets()));
    }

    for (target, assets) in groups {
        // What is already there is not announced: there would be nothing to
        // watch, and the row would look busy for a download that is not one.
        if assets
            .iter()
            .all(|asset| present(&dir.join(asset.name), asset))
        {
            continue;
        }
        let span: u64 = assets.iter().map(|asset| asset.bytes).sum();
        let mut done = 0u64;
        remember(done, Some(target), span, None);
        publish(app);
        for asset in assets {
            let path = dir.join(asset.name);
            if present(&path, asset) {
                done += asset.bytes;
                remember(done, Some(target), span, None);
                publish(app);
                continue;
            }
            let raw = fetch(&client, asset, done, target, span, app).await?;
            let bytes = unpack(asset, raw)?;
            if sha256(&bytes) != asset.sha256 {
                return Err(format!(
                    "{} did not arrive intact. Please try again.",
                    asset.name
                ));
            }
            write(&dir.join(asset.name), &bytes)?;
            done += asset.bytes;
            remember(done, Some(target), span, None);
            publish(app);
        }
    }
    Ok(())
}

/// Reads one file, telling the page how far along it is as it arrives.
async fn fetch(
    client: &reqwest::Client,
    asset: &Asset,
    done: u64,
    target: &str,
    span: u64,
    app: &AppHandle,
) -> Result<Vec<u8>, String> {
    let mut response = client
        .get(asset.url)
        .send()
        .await
        .map_err(|error| format!("{} could not be downloaded: {error}", asset.name))?;
    if !response.status().is_success() {
        return Err(format!(
            "{} could not be downloaded: the server answered {}.",
            asset.name,
            response.status().as_u16()
        ));
    }

    let mut bytes = Vec::with_capacity(asset.bytes as usize);
    let mut announced = 0u64;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("{} was cut short: {error}", asset.name))?
    {
        bytes.extend_from_slice(&chunk);
        if bytes.len() as u64 - announced >= 512 * 1024 {
            announced = bytes.len() as u64;
            remember(done + announced, Some(target), span, None);
            publish(app);
        }
    }
    Ok(bytes)
}

/// The file the asset is, which is the download itself except for the runtime.
fn unpack(asset: &Asset, raw: Vec<u8>) -> Result<Vec<u8>, String> {
    let Some(entry) = asset.entry else {
        return Ok(raw);
    };
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(raw))
        .map_err(|error| format!("The offline runtime download could not be read: {error}"))?;
    let mut file = archive
        .by_name(entry)
        .map_err(|error| format!("The offline runtime download holds no {entry}: {error}"))?;
    let mut bytes = Vec::with_capacity(file.size() as usize);
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|error| format!("The offline runtime library could not be read: {error}"))?;
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Writes the file only once it is whole, so a download that dies leaves the
/// pack missing rather than half-installed.
fn write(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let partial = target.with_extension("part");
    std::fs::write(&partial, bytes)
        .map_err(|error| format!("The offline files could not be written: {error}"))?;
    std::fs::rename(&partial, target)
        .map_err(|error| format!("The offline files could not be written: {error}"))
}

/// Takes one direction off the disk, or the whole pack with `None`.
///
/// Removing one direction leaves the other alone, along with the shared
/// runtime: the files are per direction, so the other way round keeps working.
///
/// The runtime library cannot always go: Windows keeps it while the process has
/// it loaded, so it is left behind until the app is restarted, which is not a
/// removal that failed. The models are what is read, and they do go.
pub fn remove(id: Option<&str>) -> Result<(), String> {
    let dir = dir()?;
    // The models are held open by the sessions, so those are let go of first.
    forget();

    let mut files: Vec<&Asset> = Vec::new();
    match resolve(id) {
        Some(pair) => files.extend(pair.assets()),
        None => {
            files.push(&RUNTIME);
            for pair in &PAIRS {
                files.extend(pair.assets());
            }
        }
    }

    let mut failure = None;
    for asset in files {
        if let Err(error) = std::fs::remove_file(dir.join(asset.name)) {
            if error.kind() != std::io::ErrorKind::NotFound {
                failure = Some(format!("{} could not be removed: {error}", asset.name));
            }
        }
    }
    if dir.join(RUNTIME.name).exists() {
        remember(0, None, 0, failure.clone());
    } else {
        remember(0, None, 0, None);
        failure = None;
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/* ---- The model, and the text it answers with ---------------------------- */

/// The decoder starts on the pad piece, which is what Marian was trained with.
const PAD_ID: i64 = 65_000;
/// The piece that ends a translation.
const EOS_ID: i64 = 0;
/// Longest translation the decoder is allowed to write, in pieces.
///
/// A sentence of the length the app accepts is well under this, and a decoder
/// that never writes the end piece has to be stopped by something.
const MAX_OUTPUT: usize = 200;
/// Longest text the encoder is handed, in pieces. The models were trained on
/// 512, and a piece table reading of the longest text the app accepts stays
/// under this.
const MAX_INPUT: usize = 500;

/// One direction, loaded and ready.
struct Model {
    encoder: ort::session::Session,
    decoder: ort::session::Session,
    pieces: Vocabulary,
}

/// The models that were read off the disk, kept between translations.
///
/// Opening a direction reads about 113 MB and sets up two runtimes, which is a
/// second or two, so a direction is held for as long as the app runs. Every
/// entry is behind one lock because running a model needs it to be the only one
/// using it.
static LOADED: Mutex<Vec<(&'static str, Model)>> = Mutex::new(Vec::new());

/// Forgets the loaded models, so the next translation opens them again. Called
/// when the files they were built from are removed.
pub fn forget() {
    if let Ok(mut guard) = LOADED.lock() {
        guard.clear();
    }
}

/// Translates `text` with the offline pack.
///
/// Blocking, and told to be: a translation is a second of arithmetic on this
/// machine, and the caller puts it on a thread that is allowed to take it.
pub fn translate(text: &str, source: &str, target: &str) -> Result<TranslationResult, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Nothing was selected.".to_string());
    }
    let pair = pair_for(source, target)
        .ok_or_else(|| "The offline pack translates Chinese and English only.".to_string())?;
    let dir = dir()?;
    if !pair_ready_in(&dir, pair) {
        return Err(
            "The offline translation pack is not downloaded yet. It is on the Resources \
             page of the settings window."
                .to_string(),
        );
    }

    let mut loaded = LOADED
        .lock()
        .map_err(|_| "The offline engine is not usable right now.".to_string())?;
    if !loaded.iter().any(|(id, _)| *id == pair.id) {
        loaded.push((pair.id, Model::open(&dir, pair)?));
    }
    let model = loaded
        .iter_mut()
        .find(|(id, _)| *id == pair.id)
        .map(|(_, model)| model)
        .ok_or_else(|| "The offline engine could not be opened.".to_string())?;

    let translation = model.run(text)?;
    let mut result = TranslationResult::new(
        classify::classify(text),
        PROVIDER,
        text,
        &translate::normalize_lang_code(target),
    );
    result.translation = translation;
    result.source_lang = translate::normalize_lang_code(pair.source);
    Ok(result)
}

impl Model {
    /// Opens one direction: the runtime, the two halves of the model, and the
    /// piece table they are read with.
    fn open(dir: &Path, pair: &Pair) -> Result<Model, String> {
        // The library arrives by path rather than being linked in, so the first
        // model opened decides which one the process uses; every later call
        // finds it already loaded and leaves it alone.
        ort::init_from(dir.join(RUNTIME.name))
            .map_err(|error| format!("The offline runtime is not usable: {error}"))?
            .commit();
        let open = |asset: &Asset, which: &str| -> Result<ort::session::Session, String> {
            ort::session::Session::builder()
                .map_err(|error| format!("The offline engine could not be set up: {error}"))?
                .commit_from_file(dir.join(asset.name))
                .map_err(|error| format!("The offline {which} model could not be opened: {error}"))
        };
        Ok(Model {
            encoder: open(&pair.encoder, "encoder")?,
            decoder: open(&pair.decoder, "decoder")?,
            pieces: Vocabulary::load(&dir.join(pair.pieces.name))?,
        })
    }

    /// Writes the translation of `text`, one piece at a time.
    ///
    /// The decoder is the one without a cache: every step is handed the whole
    /// translation so far and answers with the piece that comes after it. That
    /// is more arithmetic than keeping the attention of the earlier pieces
    /// around, and it is the version that cannot go wrong on a model whose
    /// inputs are named differently — a sentence is a second either way.
    fn run(&mut self, text: &str) -> Result<String, String> {
        let mut ids = self.pieces.encode(text);
        // `encode` gives the pieces of the text alone; a Marian encoder expects
        // the sentence to end with the end piece, which the post-processor of
        // the shipped file would have added.
        ids.push(EOS_ID);
        ids.truncate(MAX_INPUT);

        let length = ids.len();
        let input = Tensor::from_array(([1usize, length], ids)).map_err(|error| {
            format!("The text could not be handed to the offline engine: {error}")
        })?;
        let mask = Tensor::from_array(([1usize, length], vec![1i64; length])).map_err(|error| {
            format!("The text could not be handed to the offline engine: {error}")
        })?;
        let read = self
            .encoder
            .run(ort::inputs!["input_ids" => input, "attention_mask" => mask])
            .map_err(|error| format!("The offline engine could not read the text: {error}"))?;
        let (shape, memory) = read["last_hidden_state"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("The offline engine answered unexpectedly: {error}"))?;
        let steps = shape[1] as usize;
        let width = shape[2] as usize;
        let memory = memory.to_vec();

        let mut written: Vec<i64> = Vec::new();
        let mut tokens: Vec<i64> = vec![PAD_ID];
        for _ in 0..MAX_OUTPUT {
            let so_far = tokens.len();
            let input =
                Tensor::from_array(([1usize, so_far], tokens.clone())).map_err(|error| {
                    format!("The translation could not be handed to the offline engine: {error}")
                })?;
            let mask =
                Tensor::from_array(([1usize, steps], vec![1i64; steps])).map_err(|error| {
                    format!("The translation could not be handed to the offline engine: {error}")
                })?;
            let memory =
                Tensor::from_array(([1usize, steps, width], memory.clone())).map_err(|error| {
                    format!("The translation could not be handed to the offline engine: {error}")
                })?;
            let written_so_far = self
                .decoder
                .run(ort::inputs![
                    "input_ids" => input,
                    "encoder_attention_mask" => mask,
                    "encoder_hidden_states" => memory,
                ])
                .map_err(|error| {
                    format!("The offline engine could not translate the text: {error}")
                })?;
            let (shape, logits) = written_so_far["logits"]
                .try_extract_tensor::<f32>()
                .map_err(|error| format!("The offline engine answered unexpectedly: {error}"))?;
            let classes = shape[2] as usize;
            let row = &logits[(so_far - 1) * classes..so_far * classes];

            // The pad piece is never a translation, and the model was published
            // with it banned from the search for exactly that reason.
            let mut pick = 0usize;
            let mut best = f32::NEG_INFINITY;
            for (id, score) in row.iter().enumerate() {
                if id as i64 == PAD_ID || !score.is_finite() {
                    continue;
                }
                if *score > best {
                    best = *score;
                    pick = id;
                }
            }
            if pick as i64 == EOS_ID {
                break;
            }
            tokens.push(pick as i64);
            written.push(pick as i64);
        }
        Ok(self.pieces.decode(&written))
    }
}

/// The piece table of one direction, read from the `tokenizer.json` that ships
/// beside the models.
///
/// It is the piece table of a Marian model exported for the browser: a unigram
/// table, which is what SentencePiece is underneath, plus the two things the
/// model itself does not carry — that a space becomes the piece `▁`, and that
/// the piece table is asked for the most likely reading of the text rather than
/// the longest one.
struct Vocabulary {
    /// Every piece, by id.
    pieces: Vec<String>,
    /// The unigram score of each piece, by id.
    scores: Vec<f32>,
    /// Where a piece is in the table.
    index: HashMap<String, u32>,
    /// Length of the longest piece, in characters, which bounds a lookup.
    longest: usize,
    /// The score of the piece that stands in for a character the table never
    /// had, which is what the model was trained to read it as.
    unknown: f32,
}

impl Vocabulary {
    fn load(path: &Path) -> Result<Vocabulary, String> {
        #[derive(serde::Deserialize)]
        struct File {
            model: Unigram,
        }
        #[derive(serde::Deserialize)]
        struct Unigram {
            vocab: Vec<(String, f64)>,
        }

        let raw = std::fs::read(path)
            .map_err(|error| format!("The offline piece table could not be read: {error}"))?;
        let file: File = serde_json::from_slice(&raw)
            .map_err(|error| format!("The offline piece table could not be understood: {error}"))?;

        let mut pieces = Vec::with_capacity(file.model.vocab.len());
        let mut scores = Vec::with_capacity(file.model.vocab.len());
        let mut index = HashMap::with_capacity(file.model.vocab.len());
        for (id, (piece, score)) in file.model.vocab.into_iter().enumerate() {
            index.insert(piece.clone(), id as u32);
            pieces.push(piece);
            scores.push(score as f32);
        }
        let longest = pieces
            .iter()
            .map(|piece| piece.chars().count())
            .max()
            .unwrap_or(1);
        // What a character the table never had costs. It is priced a little
        // below the worst piece in the table rather than at the score the
        // `<unk>` piece itself carries: the three pieces that carry no text
        // (`<unk>`, `</s>` and `<pad>`) score 0.0, which is the *best* score in
        // a unigram table, and a reading that cost nothing would be chosen over
        // every real piece of the word it sits in.
        let unknown = scores.iter().copied().fold(f32::INFINITY, f32::min) - UNK_PENALTY;
        Ok(Vocabulary {
            pieces,
            scores,
            index,
            longest,
            unknown,
        })
    }

    /// Reads `text` the way the model reads it: the most likely reading of it,
    /// piece by piece.
    ///
    /// This is the Viterbi of a unigram table — the reading whose pieces add up
    /// to the highest score — which is what SentencePiece does and what the
    /// pieces were trained to be read with. Reading it greedily from the left
    /// would fit the longest pieces instead, and the model would be handed a
    /// reading it has never seen.
    fn encode(&self, text: &str) -> Vec<i64> {
        let marked = mark(text);
        // Where each character starts, so a piece is taken as a slice of the
        // marked text and no candidate has to be built to be looked up.
        let mut bounds: Vec<usize> = marked.char_indices().map(|(at, _)| at).collect();
        bounds.push(marked.len());
        let count = bounds.len() - 1;

        let mut best = vec![f32::NEG_INFINITY; count + 1];
        let mut taken = vec![(u32::MAX, 0usize); count + 1];
        best[0] = 0.0;
        for at in 1..=count {
            let lowest = at.saturating_sub(self.longest);
            for start in lowest..at {
                if !best[start].is_finite() {
                    continue;
                }
                let piece = &marked[bounds[start]..bounds[at]];
                if let Some(&id) = self.index.get(piece) {
                    let score = best[start] + self.scores[id as usize];
                    if score > best[at] {
                        best[at] = score;
                        taken[at] = (id, start);
                    }
                }
            }
            if !best[at].is_finite() {
                // Nothing in the table covers this position: one character is
                // written as the unknown piece, which is what the table has for
                // a character it never met.
                let start = at - 1;
                best[at] = best[start] + self.unknown;
                taken[at] = (UNKNOWN_ID, start);
            }
        }

        let mut ids = Vec::new();
        let mut at = count;
        while at > 0 {
            let (id, start) = taken[at];
            ids.push(id as i64);
            at = start;
        }
        ids.reverse();
        ids
    }

    /// Writes the pieces back as text.
    fn decode(&self, ids: &[i64]) -> String {
        let mut text = String::new();
        for id in ids {
            let Some(piece) = usize::try_from(*id).ok().and_then(|id| self.pieces.get(id)) else {
                continue;
            };
            if *id == EOS_ID || *id == PAD_ID || piece == UNKNOWN_PIECE {
                continue;
            }
            text.push_str(piece);
        }
        tidy(&text.replace('▁', " "))
    }
}

/// The piece a space is written as, and the piece a character the table never
/// had is written as.
const SPACE: char = '▁';
const UNKNOWN_PIECE: &str = "<unk>";
/// Where the unknown piece sits in every one of these tables.
const UNKNOWN_ID: u32 = 1;
/// How far below the worst piece in the table a character the table never had
/// is priced. It only has to be low enough that any reading made of real pieces
/// is preferred to one that gives up on a character.
const UNK_PENALTY: f32 = 10.0;

/// Puts the space marks into `text`, which is how the model sees a space.
///
/// One mark at the front, one mark for every run of whitespace inside, and none
/// at the end: the reading of "a b" is the piece `▁a` followed by `▁b`, which
/// is the same reading SentencePiece takes.
fn mark(text: &str) -> String {
    let mut marked = String::with_capacity(text.len() + 1);
    marked.push(SPACE);
    let mut marked_space = true;
    for character in text.chars() {
        if character.is_whitespace() {
            if !marked_space {
                marked.push(SPACE);
                marked_space = true;
            }
        } else {
            marked.push(character);
            marked_space = false;
        }
    }
    if marked_space && marked.len() > 1 {
        marked.pop();
    }
    marked
}

/// Punctuation the cleanup takes a space away from, and the ones English puts
/// no space before at all.
const TIDY_MARKS: [&str; 10] = [",", ".", "!", "?", ";", ":", "%", ")", "]", "}"];

/// Takes the marks out and clears the spaces English puts before punctuation.
fn tidy(text: &str) -> String {
    let mut clean = text.trim().to_string();
    for mark in TIDY_MARKS {
        clean = clean.replace(&format!(" {mark}"), mark);
    }
    clean
}

/* ---- The commands the settings page calls ------------------------------- */

/// What the settings page is told about the pack.
#[tauri::command]
pub fn offline_model_status() -> Status {
    status()
}

/// Downloads whatever of the pack is not on the disk yet.
///
/// `pack` names one direction (`zh-en`, `en-zh`); without it both are fetched,
/// one after the other.
#[tauri::command]
pub async fn offline_model_download(
    app: AppHandle,
    pack: Option<String>,
) -> Result<Status, String> {
    let wanted = pack.as_deref();
    // The button can be pressed twice while a download runs; the second one
    // waits for the one already going rather than starting another.
    if DOWNLOADING.load(Ordering::SeqCst) {
        wait_for_install().await?;
    } else {
        download(&app, wanted).await?;
    }
    Ok(status())
}

/// Waits until a download that is already running is done.
async fn wait_for_install() -> Result<(), String> {
    // Half a minute is longer than a retry needs and shorter than a download
    // takes; the page polls the status itself either way, so what is waited for
    // here is only the caller that asked twice.
    for _ in 0..150 {
        if !DOWNLOADING.load(Ordering::SeqCst) {
            let error = PROGRESS.lock().ok().and_then(|guard| guard.error.clone());
            return match error {
                Some(error) => Err(error),
                None => Ok(()),
            };
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err("The offline pack is still being downloaded.".to_string())
}

/// Takes one direction off the disk, or everything, after the page has asked.
#[tauri::command]
pub fn offline_model_remove(pack: Option<String>) -> Status {
    // A file the engine still holds open is not a reason to report the removal
    // as failed: the state that is answered below is what the page draws.
    let _ = remove(pack.as_deref());
    status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_asset_has_a_name_of_its_own() {
        let names: Vec<&str> = every_asset().iter().map(|asset| asset.name).collect();
        for name in &names {
            assert!(!name.contains('/') && !name.contains('\\'), "{name}");
        }
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn the_pack_is_two_directions_and_both_ways_round() {
        assert_eq!(PAIRS.len(), 2);
        assert!(serves("zh-CN", "en"));
        assert!(serves("en", "zh-CN"));
        // The source may be left to the app, which picks the model by what the
        // reader wants to read.
        assert!(serves("auto", "en"));
        assert!(serves("", "zh-CN"));
        // Anything else is a translation this pack cannot make.
        assert!(!serves("ja", "en"));
        assert!(!serves("en", "ja"));
        assert!(!serves("zh-CN", "zh-TW"));
    }

    #[test]
    fn a_space_is_one_mark_and_the_marks_are_where_they_belong() {
        assert_eq!(mark("a b"), format!("{SPACE}a{SPACE}b"));
        assert_eq!(mark("  a   b  "), format!("{SPACE}a{SPACE}b"));
        assert_eq!(mark("你好"), format!("{SPACE}你好"));
        assert_eq!(mark("   "), "");
        assert_eq!(mark(""), "");
    }

    /// A piece table small enough to write out, with the same shape as the one
    /// the download carries.
    fn table(pieces: &[(&str, f32)]) -> Vocabulary {
        let mut index = HashMap::new();
        let mut names = Vec::new();
        let mut scores = Vec::new();
        for (id, (piece, score)) in pieces.iter().enumerate() {
            index.insert(piece.to_string(), id as u32);
            names.push(piece.to_string());
            scores.push(*score);
        }
        let longest = names
            .iter()
            .map(|piece| piece.chars().count())
            .max()
            .unwrap_or(1);
        Vocabulary {
            pieces: names,
            scores,
            index,
            longest,
            unknown: -100.0,
        }
    }

    #[test]
    fn the_reading_is_the_most_likely_one_rather_than_the_longest() {
        // Two readings of `▁ab`: one piece of both letters, or the two letters
        // one at a time. The one with the better score is the one that is read,
        // which is the whole point of the table being a table of likelihoods.
        let split = table(&[
            ("</s>", 0.0),
            ("<unk>", 0.0),
            ("▁", -1.0),
            ("a", -1.0),
            ("b", -1.0),
        ]);
        assert_eq!(split.encode("ab"), vec![2, 3, 4]);
        let whole = table(&[
            ("</s>", 0.0),
            ("<unk>", 0.0),
            ("▁ab", -0.5),
            ("a", -1.0),
            ("b", -1.0),
        ]);
        assert_eq!(whole.encode("ab"), vec![2]);
    }

    #[test]
    fn a_character_the_table_never_had_is_read_as_unknown() {
        let vocabulary = table(&[("</s>", 0.0), ("<unk>", 0.0), ("▁a", -1.0)]);
        assert_eq!(vocabulary.encode("a×"), vec![2, 1]);
    }

    #[test]
    fn the_text_is_written_back_as_it_was_said() {
        let vocabulary = table(&[
            ("</s>", 0.0),
            ("<unk>", 0.0),
            ("▁Hello", -1.0),
            ("▁world", -1.0),
            (".", -1.0),
        ]);
        assert_eq!(vocabulary.decode(&[2, 3]), "Hello world");
        // The space English writes before a full stop is not part of the text.
        assert_eq!(vocabulary.decode(&[2, 3, 4]), "Hello world.");
        // The end piece is not part of the text either.
        assert_eq!(vocabulary.decode(&[2, 3, 0]), "Hello world");
    }
}
