use crate::core::profile::{Profile, ProfileManager};
use std::sync::Mutex;
use tauri::{command, AppHandle, State};
use tauri_plugin_android_fs::AndroidFsExt;

pub struct ProfileState(pub Mutex<ProfileManager>);

#[command]
pub fn list_profiles(state: State<ProfileState>) -> Result<Vec<Profile>, String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    Ok(mgr.list().to_vec())
}

#[command]
pub fn get_active_profile(state: State<ProfileState>) -> Result<Option<Profile>, String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    Ok(mgr.active().cloned())
}

#[command]
pub fn set_active_profile(id: String, state: State<ProfileState>) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.set_active(&id).map_err(|e| e.to_string())
}

#[command]
pub fn create_profile(name: String, root_uri: String, state: State<ProfileState>) -> Result<Profile, String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    let profile = Profile::new(name, root_uri);
    let cloned = profile.clone();
    mgr.add(profile).map_err(|e| e.to_string())?;
    Ok(cloned)
}

#[command]
pub fn delete_profile(id: String, state: State<ProfileState>) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.remove(&id).map_err(|e| e.to_string())
}

#[command]
pub async fn pick_game_folder(app: AppHandle) -> Result<Option<String>, String> {
    let api = app.android_fs();
    let selected = api.picker().pick_dir(None, true).map_err(|e| e.to_string())?;
    let Some(dir_uri) = selected else { return Ok(None); };
    let json = serde_json::to_string(&dir_uri).map_err(|e| e.to_string())?;
    Ok(Some(json))
}
