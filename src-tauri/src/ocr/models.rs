//! The files the local OCR engine is made of, and putting them on the disk.
//!
//! The engine is the ONNX Runtime library, one detector, and one recogniser
//! together with the dictionary it writes from. None of it ships with the app —
//! that would grow an installer of a few megabytes by about 33 MB — so it is
//! fetched the first time the user actually asks for a screenshot translation,
//! and only after they agreed to it. Every file is checked against the digest it
//! was published with before it is used, and a download that does not match is
//! thrown away rather than run.
//!
//! The runtime and the detector read any language, so they are downloaded once
//! and shared; the recogniser does not, and a language is a pack of its own —
//! the recogniser plus its dictionary — that can be downloaded and removed
//! without touching the rest. The languages the engine reads with are the
//! `ocrPacks` setting: every one of them reads the same screenshot and the best
//! reading of each line is the one that is kept. One pack is not a download of
//! its own: `ch` is the Chinese-and-English recogniser the app has always used,
//! so a fresh install and a settings file written before packs existed both
//! land on it.
//!
//! The models come from ModelScope — the ONNX distribution of PaddleOCR's
//! PP-OCR, published by RapidOCR — and the runtime from PyPI: both are plain
//! HTTPS downloads, and both are reachable from where the app is used, which the
//! release pages of the ONNX Runtime are not. All of them are Apache-2.0, and
//! the Resources page names their source for that reason.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use crate::translate;

/// Folder under the app's data directory that holds everything the engine
/// reads. Removing it removes the engine, which is what the settings button
/// does.
const FOLDER: &str = "ocr";

/// Emitted whenever a download starts, makes progress, ends or fails, so the
/// settings page can show where it is without asking again.
pub const EVENT: &str = "glossy://ocr-model";

/// The pack a fresh install starts on, and the one that reads the two languages
/// the app has always read: Chinese and English.
pub const DEFAULT_PACK: &str = "ch";

/// One file the engine needs.
struct Asset {
    /// Name it is stored under, inside [`FOLDER`].
    name: &'static str,
    url: &'static str,
    /// Path of the file inside the download, for the one asset that arrives
    /// inside an archive. The runtime is published as a wheel, which is a zip
    /// holding the library among a lot of Python the app has no use for.
    entry: Option<&'static str>,
    /// Size of the download, which is what the progress bar counts.
    bytes: u64,
    /// Lowercase hex SHA-256 of the file that ends up on the disk, not of the
    /// archive it came out of.
    sha256: &'static str,
}

/// The files every pack reads: the runtime that runs the models, and the
/// detector, which finds the lines of text whatever they are written in.
static CORE: [Asset; 2] = [
    Asset {
        name: "onnxruntime.dll",
        url: "https://files.pythonhosted.org/packages/9f/10/3d946d5d5f2cdcc3c8da36cae63190c516d16349edaffd944bda60ca4c3e/onnxruntime-1.28.0-cp311-cp311-win_amd64.whl",
        entry: Some("onnxruntime/capi/onnxruntime.dll"),
        bytes: 17_766_712,
        sha256: "14e186627f109a15f28f0b36f6e8ea07c79eac666818e63c39a9792f415bf649",
    },
    Asset {
        name: "ch_PP-OCRv4_det_mobile.onnx",
        url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/det/ch_PP-OCRv4_det_mobile.onnx",
        entry: None,
        bytes: 4_745_517,
        sha256: "d2a7720d45a54257208b1e13e36a8479894cb74155a5efe29462512d42f49da9",
    },
];

/// One recognition language pack: the recogniser and the dictionary of
/// characters it can write, which are the two files a language needs.
struct Pack {
    /// Name the settings hold and the page shows. It is never translated: it is
    /// what the file names, the log lines and a settings file all agree on.
    id: &'static str,
    rec: Asset,
    keys: Asset,
    /// Whether the language is offered on the Resources page.
    ///
    /// This is the one word that takes a language off the page, for a build
    /// whose recognition has not been checked on a real screen. Every language
    /// in the table below has been: each one was read over a picture written in
    /// its own script, and each one read it back — so they are all offered, and
    /// taking one off again is this word and nothing else.
    offered: bool,
}

/// The languages that can be read on this machine.
///
/// Every one of them is a PP-OCR mobile recogniser, which is the same shape of
/// model as the one the app shipped with: the same 48-pixel input, the same CTC
/// output, and a dictionary with one character per line. A pack is therefore
/// nothing but two more files, and `ch` is byte for byte what the app has been
/// downloading all along.
///
/// One pack reads one script and nothing else: `ch` writes no kana at all, so a
/// Japanese screenshot read with it loses every kana and comes back wrong. That
/// is why the language has to be picked rather than guessed.
static PACKS: [Pack; 6] = [
    Pack {
        id: DEFAULT_PACK,
        rec: Asset {
            name: "ch_PP-OCRv4_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/ch_PP-OCRv4_rec_mobile.onnx",
            entry: None,
            bytes: 10_857_958,
            sha256: "48fc40f24f6d2a207a2b1091d3437eb3cc3eb6b676dc3ef9c37384005483683b",
        },
        keys: Asset {
            name: "ppocr_keys_v1.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/ch_PP-OCRv4_rec_mobile/ppocr_keys_v1.txt",
            entry: None,
            bytes: 26_249,
            sha256: "28b2362ad4ab2dc38769aa72feb535e3a9ddb3fd2a7585a05920e6393b1dc7f7",
        },
        offered: true,
    },
    Pack {
        id: "ja",
        rec: Asset {
            name: "japan_PP-OCRv4_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/japan_PP-OCRv4_rec_mobile.onnx",
            entry: None,
            bytes: 9_753_335,
            sha256: "e1075a67dba758ecfc7ebc78a10ae61c95ac8fb66a9c86fab5541e33f085cb7a",
        },
        keys: Asset {
            name: "japan_dict.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/japan_PP-OCRv4_rec_mobile/japan_dict.txt",
            entry: None,
            bytes: 17_332,
            sha256: "1dcfcb41eec90576a945b3084f22ade11ced506e24f14879245b071698f308e8",
        },
        offered: true,
    },
    Pack {
        id: "cht",
        rec: Asset {
            name: "chinese_cht_PP-OCRv3_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/chinese_cht_PP-OCRv3_rec_mobile.onnx",
            entry: None,
            bytes: 11_152_536,
            sha256: "779656d044ce388045e02ea9244724616194e63928606436cdfc6dc3c9528cc6",
        },
        keys: Asset {
            name: "chinese_cht_dict.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/chinese_cht_PP-OCRv3_rec_mobile/chinese_cht_dict.txt",
            entry: None,
            bytes: 33_443,
            sha256: "832551fee1f2fbc97508772d81ebdc8dba12c00de97a35c71c9ddf43ddac1a83",
        },
        offered: true,
    },
    Pack {
        id: "latin",
        rec: Asset {
            name: "latin_PP-OCRv3_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/latin_PP-OCRv3_rec_mobile.onnx",
            entry: None,
            bytes: 8_978_191,
            sha256: "e9d7a33667e8aaa702862975186adf2012e3f390cc0f9422865957125f8071cf",
        },
        keys: Asset {
            name: "latin_dict.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/latin_PP-OCRv3_rec_mobile/latin_dict.txt",
            entry: None,
            bytes: 468,
            sha256: "8e6d4e3629788c35c31f7e530287d6147b549bb7a265bd6708bb281134429e2c",
        },
        offered: true,
    },
    Pack {
        id: "cyrillic",
        rec: Asset {
            name: "cyrillic_PP-OCRv3_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/cyrillic_PP-OCRv3_rec_mobile.onnx",
            entry: None,
            bytes: 8_972_413,
            sha256: "1efb65bdc460af1c0e8733d005b20952b17ca5aac10ddb56c968333791c5eaa3",
        },
        keys: Asset {
            name: "cyrillic_dict.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/cyrillic_PP-OCRv3_rec_mobile/cyrillic_dict.txt",
            entry: None,
            bytes: 410,
            sha256: "369a82c6c8c479784a5d726448b83b1eafb5fef0a4129a5eaa3929625ddcd132",
        },
        offered: true,
    },
    Pack {
        id: "ko",
        rec: Asset {
            name: "korean_PP-OCRv4_rec_mobile.onnx",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/korean_PP-OCRv4_rec_mobile.onnx",
            entry: None,
            bytes: 24_067_780,
            sha256: "ab151ba9065eccd98f884cf4d927db091be86137276392072edd4f9d43ad7426",
        },
        keys: Asset {
            name: "korean_dict.txt",
            url: "https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/korean_PP-OCRv4_rec_mobile/korean_dict.txt",
            entry: None,
            bytes: 14_480,
            sha256: "aa1fdc8ae8f7cd40a0ec4edb472eb0421e11427e6ccfee9915440742c18b0a20",
        },
        offered: true,
    },
];

impl Pack {
    /// The two files of the pack, in the order they are downloaded.
    fn assets(&self) -> [&Asset; 2] {
        [&self.rec, &self.keys]
    }

    /// What the pack takes up once it is on the disk.
    fn bytes(&self) -> u64 {
        self.rec.bytes + self.keys.bytes
    }
}

/// The pack a stored name stands for, or the default one when it names none.
///
/// A settings file holding a pack this build does not know — a hand edit, or a
/// language a later build dropped — reads as the one that was always there
/// rather than as an error the user cannot act on.
fn resolve(id: &str) -> &'static Pack {
    PACKS
        .iter()
        .find(|pack| pack.offered && pack.id == id)
        .unwrap_or_else(|| {
            PACKS
                .iter()
                .find(|pack| pack.id == DEFAULT_PACK)
                .expect("the default pack is in the table")
        })
}

/// Whether `id` names a language the Resources page offers.
///
/// The settings, the page and the commands all go by this rather than by what
/// is in the table: a language that is not offered cannot be picked, downloaded
/// or removed, and a settings file naming one reads as the default.
pub fn offered(id: &str) -> bool {
    PACKS.iter().any(|pack| pack.id == id && pack.offered)
}

/// Everything the runtime and the detector take up, once they are on the disk.
fn core_bytes() -> u64 {
    CORE.iter().map(|asset| asset.bytes).sum()
}

/// What a download of `id` still has to fetch, in bytes.
///
/// Files that are already on the disk are not counted, so the number the
/// first-use prompt names is what the user is actually about to wait for — a
/// second language is a few megabytes rather than a second whole engine.
pub fn missing_bytes(app: &AppHandle, id: &str) -> u64 {
    let pack = resolve(id);
    let Ok(dir) = dir(app) else {
        return whole(pack);
    };
    let mut missing = 0;
    for asset in CORE.iter().chain(pack.assets()) {
        if !present(&dir.join(asset.name), asset) {
            missing += asset.bytes;
        }
    }
    missing
}

/// What the settings page and the consent prompt are told about one pack.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackStatus {
    /// Name the pack is stored and asked for under.
    pub id: String,
    /// Both files of the pack are on the disk.
    pub installed: bool,
    /// Bytes the pack takes up or would take up.
    pub size: u64,
    /// This is the pack the current download is fetching.
    pub downloading: bool,
}

/// What the settings page and the consent prompt are told about the engine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The runtime and the detector are on the disk: some pack can be used.
    pub installed: bool,
    /// A download is running right now.
    pub downloading: bool,
    /// Bytes of the group named by `target` that are in place, and the number
    /// that group will reach: the shared files and a language are two spans, and
    /// neither row ever shows a number that mixes them.
    pub bytes: u64,
    pub total: u64,
    /// Why the last download failed, if it did.
    pub error: Option<String>,
    /// Bytes the runtime, the detector and every offered pack take up.
    pub size: u64,
    /// Bytes the runtime and the detector take up on their own, which is what
    /// the shared half of the page is about.
    pub core_size: u64,
    /// Folder the files are kept in, so the user can find them.
    pub folder: String,
    /// Every language this build offers, whether or not it is on the disk.
    pub packs: Vec<PackStatus>,
    /// What the download that is running — or the one that last ran — was
    /// fetching: `core` for the runtime and the detector, or the name of a
    /// language. It stays behind after a failure so the message can be put on
    /// the row the download belonged to.
    pub target: Option<String>,
}

/// Progress of the download that is running, or of the one that last ran.
#[derive(Default)]
struct Progress {
    bytes: u64,
    /// What is being fetched, and how much of it there is.
    target: Option<String>,
    total: u64,
    error: Option<String>,
}

static PROGRESS: Mutex<Progress> = Mutex::new(Progress {
    bytes: 0,
    target: None,
    total: 0,
    error: None,
});

/// Held for as long as a download is running, so a second request — the button
/// clicked twice, or a screenshot taken while one is going — joins the one
/// already going instead of fetching the same files twice.
static DOWNLOADING: AtomicBool = AtomicBool::new(false);

/// Everything a download of `pack` fetches when none of it is on the disk yet:
/// the shared files plus the language. What the first-use prompt falls back on
/// when it cannot look at the folder to see what is already there.
fn whole(pack: &Pack) -> u64 {
    core_bytes() + pack.bytes()
}

/// The folder the engine lives in, created if it is not there yet.
pub fn dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("The application's data folder is not available: {error}"))?
        .join(FOLDER);
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("The OCR folder could not be created: {error}"))?;
    Ok(dir)
}

/// Whether a file is the one it should be. The digest is only checked while
/// downloading; afterwards the size is enough, because a file of the right size
/// in the app's own folder is one this module wrote.
fn present(path: &Path, asset: &Asset) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.len() == asset.bytes)
        .unwrap_or(false)
}

/// Whether the runtime and the detector are on the disk.
pub fn installed(app: &AppHandle) -> bool {
    let Ok(dir) = dir(app) else {
        return false;
    };
    CORE.iter()
        .all(|asset| present(&dir.join(asset.name), asset))
}

/// Whether one pack's own two files are on the disk.
pub fn pack_installed(app: &AppHandle, id: &str) -> bool {
    let pack = resolve(id);
    let Ok(dir) = dir(app) else {
        return false;
    };
    pack.assets()
        .iter()
        .all(|asset| present(&dir.join(asset.name), asset))
}

/// Whether a pack can be used right now: the shared files and its own are all
/// there.
pub fn ready(app: &AppHandle, id: &str) -> bool {
    installed(app) && pack_installed(app, id)
}

/// Where each file of the engine is on the disk.
pub struct Paths {
    pub dll: PathBuf,
    pub det: PathBuf,
    pub rec: PathBuf,
    pub keys: PathBuf,
}

/// The paths of the installed files, for the engine to open.
pub fn paths(app: &AppHandle, id: &str) -> Result<Paths, String> {
    let pack = resolve(id);
    let dir = dir(app)?;
    if !ready(app, id) {
        return Err("The local OCR files are not downloaded yet.".to_string());
    }
    Ok(Paths {
        dll: dir.join(CORE[0].name),
        det: dir.join(CORE[1].name),
        rec: dir.join(pack.rec.name),
        keys: dir.join(pack.keys.name),
    })
}

/// The language a set of paths was built for, so the engine can tell whether the
/// model it holds is still the one being asked for.
pub fn pack_of(paths: &Paths) -> &'static str {
    PACKS
        .iter()
        .find(|pack| paths.rec.ends_with(pack.rec.name))
        .map(|pack| pack.id)
        .unwrap_or(DEFAULT_PACK)
}

pub fn status(app: &AppHandle) -> Status {
    let core = installed(app);
    let folder = dir(app)
        .map(|dir| dir.to_string_lossy().to_string())
        .unwrap_or_default();
    let (progress, error, target, total) = PROGRESS
        .lock()
        .map(|guard| {
            (
                guard.bytes,
                guard.error.clone(),
                guard.target.clone(),
                guard.total,
            )
        })
        .unwrap_or((0, None, None, 0));
    let downloading = DOWNLOADING.load(Ordering::SeqCst);
    let packs: Vec<PackStatus> = PACKS
        .iter()
        .filter(|pack| pack.offered)
        .map(|pack| PackStatus {
            id: pack.id.to_string(),
            installed: core && pack_ready_in(&folder, pack),
            size: pack.bytes(),
            downloading: downloading && target.as_deref() == Some(pack.id),
        })
        .collect();
    let size = if core {
        core_bytes()
            + PACKS
                .iter()
                .filter(|pack| pack.offered && pack_ready_in(&folder, pack))
                .map(Pack::bytes)
                .sum::<u64>()
    } else {
        0
    };
    Status {
        installed: core,
        downloading,
        bytes: if downloading { progress } else { 0 },
        total: if downloading { total } else { 0 },
        error,
        size,
        core_size: core_bytes(),
        folder,
        packs,
        target,
    }
}

/// The languages of a stored list that this build offers, in the order the page
/// shows them, with the default one when nothing is left.
///
/// The stored list is what the Resources page checked, so it is read rather
/// than trusted: a name this build does not know — a hand edit, a language a
/// later build dropped — is dropped, a repeated one is kept once, and a list
/// that ends up empty falls back to the language that was always there, because
/// a screenshot that could not be read at all is worse than one read in the
/// wrong script.
pub fn checked(ids: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = PACKS
        .iter()
        .filter(|pack| pack.offered && ids.iter().any(|id| id.trim().eq_ignore_ascii_case(pack.id)))
        .map(|pack| pack.id.to_string())
        .collect();
    if kept.is_empty() {
        kept.push(DEFAULT_PACK.to_string());
    }
    kept
}

/// Whether a pack's files are in `folder`, without asking for the directory
/// again — the status builds the answer for every pack at once.
fn pack_ready_in(folder: &str, pack: &Pack) -> bool {
    let dir = Path::new(folder);
    pack.assets()
        .iter()
        .all(|asset| present(&dir.join(asset.name), asset))
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
    // The page draws the same status the event carries.
    let _ = app.emit(EVENT, status(app));
}

/// Downloads whatever is missing and checks every file it wrote.
///
/// `id` names the pack that has to end up ready. The runtime and the detector
/// come with it when they are not there yet, because the pack is of no use
/// without them; files that are already present are skipped, so a second pack
/// is a download of its own two files.
///
/// The second caller does not start a second download: it is told one is
/// running, and can ask [`status`] until it is done.
pub async fn download(app: &AppHandle, id: &str) -> Result<(), String> {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return Err("The local OCR files are still being downloaded.".to_string());
    }
    let pack = resolve(id);
    // The last failure is cleared here; what is being fetched is announced by
    // `install`, once it knows which group it actually has to fetch.
    remember(0, None, 0, None);
    publish(app);
    let outcome = install(app, pack).await;
    DOWNLOADING.store(false, Ordering::SeqCst);
    // A failure keeps the name of the group it happened in, so the page can put
    // the message on the row that was downloading instead of on the shared one.
    let last = PROGRESS.lock().ok().and_then(|guard| guard.target.clone());
    match &outcome {
        Ok(()) => remember(0, None, 0, None),
        Err(error) => remember(0, last.as_deref(), 0, Some(error.clone())),
    }
    publish(app);
    outcome
}

/// What the progress of the shared files is reported under.
const CORE_TARGET: &str = "core";

async fn install(app: &AppHandle, pack: &'static Pack) -> Result<(), String> {
    let dir = dir(app)?;
    let client = client()?;

    // The shared files and the language are fetched one group at a time, and
    // each group is announced with the size of its own files: the engine row
    // counts the runtime and the detector, the language's row counts that
    // language. A group that is already complete is not announced at all, since
    // there would be nothing to watch.
    let mut groups: Vec<(&'static str, Vec<&'static Asset>)> =
        vec![(CORE_TARGET, CORE.iter().collect())];
    groups.push((pack.id, pack.assets().into_iter().collect()));

    for (target, assets) in groups {
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
            write(&path, &bytes)?;
            done += asset.bytes;
            remember(done, Some(target), span, None);
            publish(app);
        }
    }
    Ok(())
}

/// The client every download goes through.
///
/// The agent is not decoration: the ModelScope download host answers 403 to a
/// request that carries no `User-Agent` header at all, which is what a bare
/// reqwest request is.
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(translate::USER_AGENT)
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|error| format!("The downloader could not be set up: {error}"))
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
        .map_err(|error| format!("The ONNX Runtime download could not be read: {error}"))?;
    let mut file = archive
        .by_name(entry)
        .map_err(|error| format!("The ONNX Runtime download holds no {entry}: {error}"))?;
    let mut bytes = Vec::with_capacity(file.size() as usize);
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|error| format!("The ONNX Runtime library could not be read: {error}"))?;
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
/// engine missing rather than half-installed.
fn write(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let partial = target.with_extension("part");
    std::fs::write(&partial, bytes)
        .map_err(|error| format!("The OCR files could not be written: {error}"))?;
    std::fs::rename(&partial, target)
        .map_err(|error| format!("The OCR files could not be written: {error}"))
}

/// Takes a language pack off the disk, or everything with `None`.
///
/// Removing one pack leaves the runtime, the detector and every other pack
/// alone: the files are per pack, so the others keep working.
///
/// The runtime library cannot always go: Windows keeps it while the process has
/// it loaded, so it is left behind until the app is restarted. The models are
/// what the engine reads, and they do go.
pub fn remove(app: &AppHandle, id: Option<&str>) -> Result<(), String> {
    let dir = dir(app)?;
    // The engine holds the files open, so it has to be let go of before the
    // files it was built from are taken away.
    super::pipeline::forget();

    let mut files: Vec<&Asset> = Vec::new();
    match id {
        Some(id) => files.extend(resolve(id).assets()),
        None => {
            files.extend(CORE.iter());
            for pack in &PACKS {
                files.extend(pack.assets());
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
    // Only the runtime library is ever held open, and the models are gone: that
    // is a working removal as far as the user is concerned, so it is not
    // reported as a failure.
    if dir.join(CORE[0].name).exists() {
        remember(0, None, 0, failure.clone());
    } else {
        remember(0, None, 0, None);
        failure = None;
    }
    publish(app);
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file the app can download, shared ones first.
    fn every_asset() -> Vec<&'static Asset> {
        let mut all: Vec<&'static Asset> = CORE.iter().collect();
        for pack in &PACKS {
            all.extend(pack.assets());
        }
        all
    }

    #[test]
    fn every_asset_is_a_file_the_engine_opens() {
        // The names are what [`paths`] hands the engine, so they have to be
        // distinct and free of anything a path would read as a folder. One
        // folder holds every pack, so two packs may not share a file name.
        let names: Vec<&str> = every_asset().iter().map(|asset| asset.name).collect();
        for name in &names {
            assert!(!name.contains('/') && !name.contains('\\'));
        }
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn every_pack_is_two_files_and_the_default_one_exists() {
        assert_eq!(PACKS.len(), 6);
        assert!(PACKS.iter().any(|pack| pack.id == DEFAULT_PACK));
        for pack in &PACKS {
            assert!(!pack.id.is_empty() && !pack.id.contains('/'));
            assert!(pack.rec.name.ends_with(".onnx"));
            assert!(pack.keys.name.ends_with(".txt"));
        }
        let mut ids: Vec<&str> = PACKS.iter().map(|pack| pack.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PACKS.len());
    }

    #[test]
    fn every_language_in_the_table_is_offered() {
        // Each one was read over a picture in its own script on a real screen
        // before it was put back on the page; the ids are what the settings and
        // the file names agree on, so a language that is not offered cannot be
        // picked, downloaded or removed by anything.
        let offered: Vec<&str> = PACKS
            .iter()
            .filter(|pack| pack.offered)
            .map(|pack| pack.id)
            .collect();
        assert_eq!(offered, vec!["ch", "ja", "cht", "latin", "cyrillic", "ko"]);
        for pack in &PACKS {
            assert!(super::offered(pack.id), "{}", pack.id);
        }
    }

    #[test]
    fn a_pack_that_does_not_exist_reads_as_the_default_one() {
        // A settings file from before packs existed holds no name at all, and
        // one naming a language that is not in the table — a hand edit, a
        // pack this build dropped — may not leave the engine without a model,
        // nor be downloaded either.
        assert_eq!(resolve("").id, DEFAULT_PACK);
        assert_eq!(resolve("klingon").id, DEFAULT_PACK);
        assert_eq!(resolve("korean").id, DEFAULT_PACK);
        assert!(offered("ja"));
        assert!(!offered("klingon"));
    }

    #[test]
    fn the_languages_that_were_checked_are_the_ones_read_with() {
        // What the page checked, in the order the page shows them; a name this
        // build does not know and a repeat are dropped.
        assert_eq!(checked(&["ja".to_string()]), vec!["ja"]);
        assert_eq!(
            checked(&["ko".to_string(), "ch".to_string()]),
            vec!["ch", "ko"]
        );
        assert_eq!(
            checked(&[" ja ".to_string(), "ja".to_string(), "klingon".to_string()]),
            vec!["ja"]
        );
        // A list left with nothing is the language that was always there: a
        // screenshot that could not be read at all is worse than one read in
        // the wrong script.
        assert_eq!(checked(&[]), vec![DEFAULT_PACK]);
        assert_eq!(checked(&["klingon".to_string()]), vec![DEFAULT_PACK]);
    }

    #[test]
    fn one_download_is_the_shared_files_plus_the_language() {
        // The shared files plus the default pack are what a first download
        // fetches, and every file is a real download with a real digest.
        let pack = resolve(DEFAULT_PACK);
        assert_eq!(whole(pack), 33_396_436);
        assert_eq!(core_bytes(), 22_512_229);
        assert!(every_asset().iter().all(|asset| asset.bytes > 0));
        assert!(every_asset()
            .iter()
            .all(|asset| asset.sha256.len() == 64 && asset.url.starts_with("https://")));
        // Every model comes from the host the app already reaches, and only the
        // runtime comes from PyPI.
        assert!(every_asset()
            .iter()
            .filter(|asset| asset.name.ends_with(".onnx") || asset.name.ends_with(".txt"))
            .all(|asset| asset.url.starts_with("https://www.modelscope.cn/")));
    }

    #[test]
    fn only_the_runtime_arrives_inside_an_archive() {
        let zipped: Vec<&str> = every_asset()
            .iter()
            .filter(|asset| asset.entry.is_some())
            .map(|asset| asset.name)
            .collect();
        assert_eq!(zipped, vec!["onnxruntime.dll"]);
    }

    #[test]
    fn a_pack_costs_what_its_two_files_cost() {
        for pack in &PACKS {
            assert_eq!(pack.bytes(), pack.rec.bytes + pack.keys.bytes);
            assert!(pack.rec.bytes > pack.keys.bytes);
        }
    }

    #[test]
    fn the_notices_name_every_file_that_is_downloaded() {
        // Apache-2.0 asks for the source to be named, and a notices file that
        // has drifted from this table would name the wrong thing with
        // confidence — which is worse than naming nothing.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("THIRD_PARTY_NOTICES.md");
        let notices = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} could not be read: {error}", path.display()));
        for asset in every_asset() {
            assert!(
                notices.contains(asset.name),
                "{} is not named in the notices",
                asset.name
            );
            assert!(
                notices.contains(asset.url),
                "{} is not credited with the address it comes from",
                asset.name
            );
            assert!(
                notices.contains(asset.sha256),
                "{} is not listed with the digest it is checked against",
                asset.name
            );
        }
        assert!(notices.contains("Apache License 2.0"));
        assert!(notices.contains("PaddleOCR"));
        assert!(notices.contains("ONNX Runtime"));
    }

    #[test]
    fn a_file_of_the_wrong_size_is_not_the_one_that_was_downloaded() {
        let dir = std::env::temp_dir().join("glossy-ocr-model-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("sample.bin");
        std::fs::write(&path, b"123").unwrap();
        let asset = Asset {
            name: "sample.bin",
            url: "https://example.invalid",
            entry: None,
            bytes: 3,
            sha256: "",
        };
        assert!(present(&path, &asset));
        let other = Asset { bytes: 4, ..asset };
        assert!(!present(&path, &other));
        assert!(!present(&dir.join("nothing.bin"), &asset));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_digest_is_the_lowercase_hex_sha256_of_the_bytes() {
        // The published digest of an empty file, which every tool agrees on.
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// Reads one picture with two languages at once, which is what checking
    /// more than one on the Resources page does.
    ///
    /// `GLOSSY_OCR_CHECK` is the folder holding the downloaded files,
    /// `GLOSSY_OCR_SAMPLES` one holding `<name>-<width>x<height>.bgra`
    /// pictures, and the two languages are `GLOSSY_OCR_PACKS`.
    #[test]
    #[ignore = "needs the downloaded models and sample pictures"]
    fn two_languages_read_one_picture_together() {
        let files = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_CHECK").unwrap());
        let samples = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_SAMPLES").unwrap());
        let wanted: Vec<String> = std::env::var("GLOSSY_OCR_PACKS")
            .unwrap()
            .split(',')
            .map(str::to_string)
            .collect();
        let paths: Vec<Paths> = wanted
            .iter()
            .map(|id| {
                let pack = PACKS.iter().find(|pack| pack.id == id).unwrap();
                Paths {
                    dll: files.join(CORE[0].name),
                    det: files.join(CORE[1].name),
                    rec: files.join(pack.rec.name),
                    keys: files.join(pack.keys.name),
                }
            })
            .collect();

        for entry in std::fs::read_dir(&samples).unwrap() {
            let path = entry.unwrap().path();
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let (name, size) = stem.rsplit_once('-').unwrap();
            let (width, height) = size.split_once('x').unwrap();
            let pixels = std::fs::read(&path).unwrap();
            let started = std::time::Instant::now();
            let lines = crate::ocr::pipeline::recognize(
                &paths,
                &pixels,
                width.parse().unwrap(),
                height.parse().unwrap(),
            )
            .unwrap();
            let read = lines
                .iter()
                .map(|line| format!("{:.3} {}", line.score, line.text))
                .collect::<Vec<_>>()
                .join(" | ");
            println!(
                "sample {name:<4} with {} {:>5} ms -> {read}",
                wanted.join("+"),
                started.elapsed().as_millis()
            );
        }
    }

    /// Reads sample pictures with every pack, for a manual check of how well
    /// each language is recognised.
    ///
    /// Not part of the suite: it needs the models and the pictures on the disk,
    /// which only a check on a real machine has. `GLOSSY_OCR_CHECK` is the
    /// folder holding the downloaded files, `GLOSSY_OCR_SAMPLES` one holding
    /// `<name>-<width>x<height>.bgra` pictures, and `GLOSSY_OCR_PACKS` the packs
    /// to read them with, defaulting to all of them.
    #[test]
    #[ignore = "needs the downloaded models and sample pictures"]
    fn samples_are_read_with_every_pack() {
        let files = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_CHECK").unwrap());
        let samples = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_SAMPLES").unwrap());
        let wanted: Vec<String> = match std::env::var("GLOSSY_OCR_PACKS") {
            Ok(list) => list.split(',').map(str::to_string).collect(),
            Err(_) => PACKS.iter().map(|pack| pack.id.to_string()).collect(),
        };
        let mut pictures: Vec<(String, u32, u32, Vec<u8>)> = Vec::new();
        for entry in std::fs::read_dir(&samples).unwrap() {
            let path = entry.unwrap().path();
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let (name, size) = stem.rsplit_once('-').unwrap();
            let (width, height) = size.split_once('x').unwrap();
            pictures.push((
                name.to_string(),
                width.parse().unwrap(),
                height.parse().unwrap(),
                std::fs::read(&path).unwrap(),
            ));
        }
        pictures.sort_by(|a, b| a.0.cmp(&b.0));
        for id in &wanted {
            let pack = PACKS.iter().find(|pack| pack.id == id).unwrap();
            let paths = Paths {
                dll: files.join(CORE[0].name),
                det: files.join(CORE[1].name),
                rec: files.join(pack.rec.name),
                keys: files.join(pack.keys.name),
            };
            for (name, width, height, pixels) in &pictures {
                let started = std::time::Instant::now();
                let read = crate::ocr::pipeline::recognize(
                    std::slice::from_ref(&paths),
                    pixels,
                    *width,
                    *height,
                )
                .map(|lines| {
                    lines
                        .iter()
                        .map(|line| format!("{:.3} {}", line.score, line.text))
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .unwrap_or_else(|error| format!("<{error}>"));
                println!(
                    "pack {id:<9} sample {name:<4} {:>5} ms -> {read}",
                    started.elapsed().as_millis()
                );
            }
        }
    }
}
