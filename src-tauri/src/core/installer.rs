use crate::core::manifest::{determine_pack_type, parse_manifest_from_zip, ManifestError};
use crate::models::{InstalledPack, PackType};
use std::io;
use tauri::AppHandle;
use tauri_plugin_android_fs::{AndroidFsExt, Entry, FileAccessMode, FileUri};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Manifest error: {0}")] Manifest(#[from] ManifestError),
    #[error("IO error: {0}")] Io(#[from] io::Error),
    #[error("Android FS error: {0}")] AndroidFs(String),
}

impl From<tauri_plugin_android_fs::Error> for InstallError {
    fn from(e: tauri_plugin_android_fs::Error) -> Self { InstallError::AndroidFs(e.to_string()) }
}

pub fn install_addon(app: &AppHandle, archive_uri: &FileUri, target_dir_uri: &FileUri) -> Result<InstalledPack, InstallError> {
    let api = app.android_fs();
    let file = api.open_file(archive_uri, FileAccessMode::Read)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let manifest = parse_manifest_from_zip(&mut archive)?;
    let pack_type = determine_pack_type(&manifest)?;
    let pack_dir_uri = api.create_dir_all(target_dir_uri, &manifest.header.uuid)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();
        if entry_name.ends_with('/') {
            let dir_path = entry_name.trim_end_matches('/');
            let _ = api.create_dir_all(&pack_dir_uri, dir_path);
            continue;
        }
        let new_file_uri = api.create_file(&pack_dir_uri, &entry_name, None)?;
        let mut buf = Vec::new();
        io::copy(&mut entry, &mut buf)?;
        api.write(&new_file_uri, &buf)?;
    }

    Ok(InstalledPack {
        manifest,
        pack_type,
        uri: pack_dir_uri.to_string(),
        source_archive: archive_uri.to_string(),
    })
}
pub fn install_mcaddon(
    app: &AppHandle,
    archive_uri: &FileUri,
    resource_packs_uri: &FileUri,
    behavior_packs_uri: &FileUri,
) -> Result<Vec<InstalledPack>, InstallError> {
    let api = app.android_fs();
    let file = api.open_file(archive_uri, FileAccessMode::Read)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let temp_dir_uri = api.get_private_storage().cache_dir()
        .create_dir_all(&format!("mcaddon_{}", uuid::Uuid::new_v4()))?;
    let mut installed = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with(".mcpack") || name.ends_with(".zip") {
            let temp_file_uri = api.create_file(&temp_dir_uri, &name, None)?;
            let mut buf = Vec::new();
            io::copy(&mut entry, &mut buf)?;
            api.write(&temp_file_uri, &buf)?;

            let probe_file = api.open_file(&temp_file_uri, FileAccessMode::Read)?;
            let mut probe_archive = zip::ZipArchive::new(probe_file)?;
            let manifest = parse_manifest_from_zip(&mut probe_archive)?;
            let pack_type = determine_pack_type(&manifest)?;
            let target = match pack_type {
                PackType::Resources => resource_packs_uri,
                PackType::Data => behavior_packs_uri,
            };
            match install_addon(app, &temp_file_uri, target) {
                Ok(pack) => installed.push(pack),
                Err(e) => eprintln!("Failed to install nested pack {}: {}", name, e),
            }
        }
    }
    let _ = api.remove_dir_all(&temp_dir_uri);
    if installed.is_empty() {
        return Err(InstallError::Manifest(ManifestError::NotFound));
    }
    Ok(installed)
}
