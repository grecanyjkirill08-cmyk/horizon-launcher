mod android;
mod commands;
mod core;
mod models;

use commands::profile_cmds::ProfileState;
use core::profile::ProfileManager;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_android_fs::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("Cannot get app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let store_path = data_dir.join("profiles.json");
            let mgr = ProfileManager::load(store_path)?;
            app.manage(ProfileState(Mutex::new(mgr)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::game_cmds::check_minecraft_installed,
            commands::game_cmds::get_minecraft_version,
            commands::game_cmds::launch_game,
            commands::download_cmds::download_addon,
            commands::profile_cmds::list_profiles,
            commands::profile_cmds::get_active_profile,
            commands::profile_cmds::set_active_profile,
            commands::profile_cmds::create_profile,
            commands::profile_cmds::delete_profile,
            commands::profile_cmds::pick_game_folder,
            commands::addon_cmds::list_packs,
            commands::addon_cmds::toggle_pack,
            commands::addon_cmds::install_addon_from_uri,
            commands::addon_cmds::pick_and_install_addon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
