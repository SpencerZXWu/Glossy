//! Update checks, through `tauri-plugin-updater`.
//!
//! The skeleton ships with a placeholder public key in `tauri.conf.json`, so a
//! check cannot succeed until a real key pair is generated
//! (`cargo tauri signer generate -w ~/.tauri/glossy.key`) and the public half is
//! put in place. `configured` recognises the placeholder so the settings window
//! can say so instead of reporting a signature error, and the release script has
//! to export `TAURI_SIGNING_PRIVATE_KEY` and turn
//! `bundle.createUpdaterArtifacts` on before an update can be published.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// The placeholder in `tauri.conf.json`, which is not a usable key.
const PLACEHOLDER: &str = "PLACEHOLDER_";

/// What the settings window shows when a newer release exists.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
}

/// Whether a signing key was filled in.
pub fn configured(app: &AppHandle) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|value| value.get("pubkey"))
        .and_then(|value| value.as_str())
        .map(|key| !key.starts_with(PLACEHOLDER) && key.len() > 40)
        .unwrap_or(false)
}

fn unconfigured() -> String {
    "this build has no update signing key yet".to_string()
}

/// Asks the endpoint in `tauri.conf.json` whether a newer release exists.
pub async fn check(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    if !configured(app) {
        return Err(unconfigured());
    }
    let updater = app.updater().map_err(|error| error.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            notes: update.body.clone().unwrap_or_default(),
        })),
        Ok(None) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

/// Downloads the pending release, installs it and restarts Glossy.
async fn install(app: &AppHandle) -> Result<(), String> {
    if !configured(app) {
        return Err(unconfigured());
    }
    let updater = app.updater().map_err(|error| error.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "no update is waiting".to_string())?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| error.to_string())?;
    // The new version only runs after a restart; the installer has replaced the
    // files by now.
    app.restart()
}

/// Checks for a release in the background, for a start that asked for it.
///
/// Failures stay silent: an offline machine is not worth a message on every
/// login, and the settings window has a button that reports them on demand.
pub fn check_in_background(app: &AppHandle) {
    if !configured(app) {
        return;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match check(&handle).await {
            Ok(Some(info)) => {
                let _ = handle.emit("glossy://update", info);
            }
            Ok(None) => {}
            Err(error) => eprintln!("Glossy could not check for updates: {error}"),
        }
    });
}

/// Whether this build can update at all, for the settings window to show.
#[tauri::command]
pub fn update_capability(app: AppHandle) -> bool {
    configured(&app)
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    check(&app).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    install(&app).await
}
