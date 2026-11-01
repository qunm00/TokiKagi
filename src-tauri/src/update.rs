use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Check the update endpoint and install a newer release in place.
///
/// The plugin verifies a minisign signature over the download against the
/// public key in `tauri.conf.json` before it touches the installed app.
/// Returns `true` when a new version was installed; the caller restarts.
#[cfg_attr(debug_assertions, allow(dead_code))]
pub async fn check_and_install(app: &AppHandle) -> Result<bool, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Ok(false);
    };
    update
        .download_and_install(|_chunk: usize, _total: Option<u64>| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}
