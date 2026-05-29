mod commands;
mod content_watcher;
mod diagnostics;
mod preview_server;
mod settings;
mod site_ops;
pub mod studio_files;

use content_watcher::ContentWatcherState;
use preview_server::PreviewServerState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PreviewServerState::default())
        .manage(ContentWatcherState::default())
        .invoke_handler(tauri::generate_handler![
            commands::resolve_project_root,
            commands::resolve_project_root_info,
            commands::list_content_bundles,
            commands::validate_site,
            commands::build_site,
            commands::start_preview_server,
            commands::stop_preview_server,
            commands::get_studio_settings,
            commands::save_studio_settings,
            commands::project_info_for_root,
            commands::list_bundle_files_cmd,
            commands::read_bundle_file_cmd,
            commands::write_bundle_file_cmd,
            commands::set_auto_rebuild,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(preview) = window.try_state::<PreviewServerState>() {
                    preview.stop();
                }
                if let Some(watcher) = window.try_state::<ContentWatcherState>() {
                    watcher.stop();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
