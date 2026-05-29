mod commands;
mod diagnostics;
mod preview_server;

use preview_server::PreviewServerState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PreviewServerState::default())
        .invoke_handler(tauri::generate_handler![
            commands::resolve_project_root,
            commands::resolve_project_root_info,
            commands::list_content_bundles,
            commands::validate_site,
            commands::build_site,
            commands::start_preview_server,
            commands::stop_preview_server,
        ])
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
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
