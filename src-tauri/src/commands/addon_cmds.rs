use crate::commands::profile_cmds::ProfileState;
use crate::core::installer::{install_addon, install_mcaddon};
use crate::models::InstalledPack;
use tauri::{command, AppHandle, State};
use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FileUri};

#[command]
pub fn list_packs(app: AppHandle, profile_id: String, state: State<ProfileState>) -> Result<Vec<InstalledPack>, String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.scan_packs(&app, &profile_id).map_err(|e| e.to_string())
}

#[command]
pub fn toggle_pack(profile_id: String, uuid: String, active: bool, state: State<ProfileState>) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    {
        let profile = mgr.get_mut(&profile_id).ok_or_else(|| "Profile not found".to_string())?;
        profile.set_pack_active(&uuid, active);
    }
    mgr.save().map_err(|e| e.to_string())
}

#[command]
pub fn install_addon_from_uri(app: AppHandle, profile_id: String, archive_uri: String, state: State<ProfileState>) -> Result<Vec<InstalledPack>, String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    let profile = mgr.get(&profile_id).ok_or_else(|| "Profile not found".to_string())?;

    let api = app.android_fs();
    let archive_uri: FileUri = archive_uri.into();
    let root_uri: FileUri = profile.root_uri.clone().into();
    let resource_uri = api.create_dir_all(&root_uri, "resource_packs").map_err(|e| e.to_string())?;
    let behavior_uri = api.create_dir_all(&root_uri, "behavior_packs").map_err(|e| e.to_string())?;

    let file = api.open_file(&archive_uri, FileAccessMode::Read).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let has_manifest_in_root = archive.file_names().any(|n| n == "manifest.json");
    let is_mcaddon = !has_manifest_in_root && archive.file_names().any(|n| n.ends_with(".mcpack") || n.ends_with(".zip"));

    let result = if is_mcaddon {
        install_mcaddon(&app, &archive_uri, &resource_uri, &behavior_uri)
    } else {
        let manifest = crate::core::manifest::parse_manifest_from_zip(&mut archive).map_err(|e| e.to_string())?;
        let pack_type = crate::core::manifest::determine_pack_type(&manifest).map_err(|e| e.to_string())?;
        let target = match pack_type {
            crate::models::PackType::Resources => &resource_uri,
            crate::models::PackType::Data => &behavior_uri,
        };
        install_addon(&app, &archive_uri, target).map(|p| vec![p])
    };
    result.map_err(|e| e.to_string())
}

#[command]
pub fn pick_and_install_addon(app: AppHandle, profile_id: String, state: State<ProfileState>) -> Result<Vec<InstalledPack>, String> {
    let api = app.android_fs();
    let selected = api.show_open_file_dialog(&["application/zip", "application/octet-stream", "*/*"], false).map_err(|e| e.to_string())?;
    let Some(uris) = selected else { return Ok(Vec::new()); };
    let Some(archive_uri) = uris.into_iter().next() else { return Ok(Vec::new()); };
    install_addon_from_uri(app, profile_id, archive_uri.to_string(), state)
}
