use crate::android::jni_bridge;
use tauri::command;

#[command]
pub async fn check_minecraft_installed() -> Result<bool, String> { jni_bridge::is_minecraft_installed().map_err(|e| e.to_string()) }
#[command]
pub async fn get_minecraft_version() -> Result<String, String> { jni_bridge::get_minecraft_version().map_err(|e| e.to_string()) }
#[command]
pub async fn launch_game() -> Result<(), String> { jni_bridge::launch_minecraft().map_err(|e| e.to_string()) }
