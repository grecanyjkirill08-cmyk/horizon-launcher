use crate::core::downloader;
use std::path::PathBuf;
use tauri::{command, AppHandle};

#[command]
pub async fn download_addon(app: AppHandle, url: String, dest: String) -> Result<String, String> {
    let path = downloader::download_file(app, &url, &PathBuf::from(&dest)).await.map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
