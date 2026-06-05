mod commands;
mod diagnostics;
mod preview_server;
mod settings;
mod site_ops;
pub mod site_template;
pub mod studio_files;

use preview_server::PreviewServerState;
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::{Emitter, Manager};

fn setup_studio_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let save = MenuItem::with_id(app, "file_save", "Save", true, Some("CmdOrCtrl+S"))?;
    let save_all = MenuItem::with_id(app, "file_save_all", "Save All", true, None::<&str>)?;
    let file = Submenu::with_items(app, "File", true, &[&save, &save_all])?;
    let menu = Menu::with_items(app, &[&file])?;
    app.set_menu(menu)?;

    let handle = app.handle().clone();
    app.on_menu_event(move |_app, event| {
        let event_name = match event.id().as_ref() {
            "file_save" => Some("studio-save"),
            "file_save_all" => Some("studio-save-all"),
            _ => None,
        };
        if let Some(name) = event_name {
            let _ = handle.emit(name, ());
        }
    });

    Ok(())
}

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
            commands::get_studio_settings,
            commands::save_studio_settings,
            commands::project_info_for_root,
            commands::list_bundle_files_cmd,
            commands::read_bundle_file_cmd,
            commands::read_bundle_image_cmd,
            commands::write_bundle_file_cmd,
            commands::import_bundle_asset_cmd,
            commands::delete_bundle_asset_cmd,
            commands::rename_bundle_asset_cmd,
            commands::create_site_from_template,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            setup_studio_menu(app)?;
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
