use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP error: {0}")] Http(#[from] reqwest::Error),
    #[error("IO error: {0}")] Io(#[from] std::io::Error),
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub url: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: f64,
}

pub async fn download_file(app: AppHandle, url: &str, dest: &Path) -> Result<PathBuf, DownloadError> {
    let client = reqwest::Client::builder().user_agent("MinecraftLauncher/1.0").build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length();
    if let Some(parent) = dest.parent() { tokio::fs::create_dir_all(parent).await?; }
    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    let _ = app.emit("download-progress", DownloadProgress { url: url.to_string(), downloaded: 0, total, percent: 0.0 });
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        let percent = match total { Some(t) if t > 0 => (downloaded as f64 / t as f64) * 100.0, _ => 0.0 };
        let _ = app.emit("download-progress", DownloadProgress { url: url.to_string(), downloaded, total, percent });
    }
    file.flush().await?;
    Ok(dest.to_path_buf())
}

pub async fn download_profile_bundle(app: AppHandle, urls: Vec<String>, dest_dir: &Path) -> Result<Vec<PathBuf>, DownloadError> {
    tokio::fs::create_dir_all(dest_dir).await?;
    let mut paths = Vec::with_capacity(urls.len());
    for (i, url) in urls.iter().enumerate() {
        let default_name = format!("bundle_part_{}.bin", i);
        let filename = url.rsplit('/').next().unwrap_or(&default_name);
        let dest = dest_dir.join(filename);
        paths.push(download_file(app.clone(), url, &dest).await?);
    }
    Ok(paths)
}
