use crate::models::{InstalledPack, PackType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io;
use tauri::AppHandle;
use tauri_plugin_android_fs::{AndroidFsExt, Entry, FileAccessMode, FsUri};

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("IO error: {0}")] Io(#[from] io::Error),
    #[error("Profile not found: {0}")] NotFound(String),
    #[error("Profile already exists: {0}")] AlreadyExists(String),
    #[error("JSON error: {0}")] Json(#[from] serde_json::Error),
    #[error("Android FS error: {0}")] AndroidFs(String),
}

impl From<tauri_plugin_android_fs::Error> for ProfileError {
    fn from(e: tauri_plugin_android_fs::Error) -> Self { ProfileError::AndroidFs(e.to_string()) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub root_uri: String,
    pub active_packs: HashSet<String>,
    pub settings: serde_json::Value,
    pub last_played: Option<i64>,
}

impl Profile {
    pub fn new(name: impl Into<String>, root_uri: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            root_uri,
            active_packs: HashSet::new(),
            settings: serde_json::json!({}),
            last_played: None,
        }
    }
    pub fn set_pack_active(&mut self, uuid: &str, active: bool) {
        if active { self.active_packs.insert(uuid.to_string()); } else { self.active_packs.remove(uuid); }
    }
    pub fn is_pack_active(&self, uuid: &str) -> bool { self.active_packs.contains(uuid) }
}

pub struct ProfileManager {
    profiles: Vec<Profile>,
    active_id: Option<String>,
    store_path: std::path::PathBuf,
}
impl ProfileManager {
    pub fn load(store_path: std::path::PathBuf) -> Result<Self, ProfileError> {
        if store_path.exists() {
            let data = std::fs::read_to_string(&store_path)?;
            let (profiles, active_id): (Vec<Profile>, Option<String>) = serde_json::from_str(&data)?;
            Ok(Self { profiles, active_id, store_path })
        } else {
            Ok(Self { profiles: Vec::new(), active_id: None, store_path })
        }
    }
    pub fn save(&self) -> Result<(), ProfileError> {
        let data = serde_json::to_string_pretty(&(&self.profiles, &self.active_id))?;
        if let Some(parent) = self.store_path.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::write(&self.store_path, data)?;
        Ok(())
    }
    pub fn add(&mut self, profile: Profile) -> Result<(), ProfileError> {
        if self.profiles.iter().any(|p| p.name == profile.name) {
            return Err(ProfileError::AlreadyExists(profile.name));
        }
        self.profiles.push(profile);
        self.save()
    }
    pub fn remove(&mut self, id: &str) -> Result<(), ProfileError> {
        let idx = self.profiles.iter().position(|p| p.id == id)
            .ok_or_else(|| ProfileError::NotFound(id.to_string()))?;
        self.profiles.remove(idx);
        if self.active_id.as_deref() == Some(id) { self.active_id = None; }
        self.save()
    }
    pub fn set_active(&mut self, id: &str) -> Result<(), ProfileError> {
        if !self.profiles.iter().any(|p| p.id == id) {
            return Err(ProfileError::NotFound(id.to_string()));
        }
        self.active_id = Some(id.to_string());
        self.save()
    }
    pub fn active(&self) -> Option<&Profile> {
        self.active_id.as_ref().and_then(|id| self.profiles.iter().find(|p| &p.id == id))
    }
    pub fn list(&self) -> &[Profile] { &self.profiles }
    pub fn get(&self, id: &str) -> Option<&Profile> { self.profiles.iter().find(|p| p.id == id) }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Profile> { self.profiles.iter_mut().find(|p| p.id == id) }

    pub fn scan_packs(&self, app: &AppHandle, profile_id: &str) -> Result<Vec<InstalledPack>, ProfileError> {
        let profile = self.get(profile_id).ok_or_else(|| ProfileError::NotFound(profile_id.to_string()))?;
        let api = app.android_fs();
        let root_uri: FsUri = serde_json::from_str(&profile.root_uri)?;

        let resource_uri = api.create_dir_all(&root_uri, "resource_packs")?;
        let behavior_uri = api.create_dir_all(&root_uri, "behavior_packs")?;

        let mut result = Vec::new();
        for (dir_uri, pack_type) in [(resource_uri, PackType::Resources), (behavior_uri, PackType::Data)] {
            let entries = match api.read_dir(&dir_uri) { Ok(e) => e, Err(_) => continue };
            for entry in entries {
                let pack_dir_uri = match entry { Entry::Dir { uri, .. } => uri, _ => continue };
                let manifest_uri = api.resolve_dir_uri(&pack_dir_uri, "manifest.json")?;
                let file = match api.open_file(&manifest_uri, FileAccessMode::Read) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let manifest: crate::models::PackManifest = serde_json::from_reader(file)?;
                result.push(InstalledPack {
                    manifest,
                    pack_type: pack_type.clone(),
                    uri: serde_json::to_string(&pack_dir_uri).unwrap_or_default(),
                    source_archive: String::new(),
                });
            }
        }
        Ok(result)
    }
            }
