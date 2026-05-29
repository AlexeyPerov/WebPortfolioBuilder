use crate::content_watcher::ContentWatcherState;
use crate::diagnostics::{BuildSiteResult, PreviewServerInfo, ProjectRootInfo, ValidateSiteResult};
use crate::preview_server::PreviewServerState;
use crate::settings::{load_settings, save_settings, StudioSettings};
use crate::site_ops::{run_build, run_validate};
use crate::studio_files::{
    list_bundle_files, project_info_at, read_bundle_file, write_bundle_file, BundleFileEntry,
};
use portfoliowebsitebuilder_core::{
    discover_content_bundles, resolve_project_root as core_resolve_project_root,
};
use std::path::PathBuf;
use tauri::{AppHandle, State};

fn parse_project_root(project_root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(project_root);
    if !path.is_dir() {
        return Err(format!("project root is not a directory: {project_root}"));
    }
    Ok(path)
}

#[tauri::command]
pub fn resolve_project_root() -> Result<ProjectRootInfo, String> {
    let project_root = core_resolve_project_root().map_err(|e| e.to_string())?;
    let template_dir = project_root.join("Template");
    Ok(ProjectRootInfo {
        project_root: project_root.to_string_lossy().into_owned(),
        template_dir: template_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn list_content_bundles(project_root: String) -> Result<Vec<String>, String> {
    let project_root = parse_project_root(&project_root)?;
    discover_content_bundles(&project_root).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_site(
    project_root: String,
    site_path: String,
    strict: bool,
) -> Result<ValidateSiteResult, String> {
    let project_root = parse_project_root(&project_root)?;
    Ok(run_validate(&project_root, &site_path, strict))
}

#[tauri::command]
pub fn build_site(
    project_root: String,
    site_path: String,
    strict: bool,
    preview: State<'_, PreviewServerState>,
) -> Result<BuildSiteResult, String> {
    preview.stop();
    let project_root = parse_project_root(&project_root)?;
    Ok(run_build(&project_root, &site_path, strict))
}

#[tauri::command]
pub fn start_preview_server(
    output_dir: String,
    port: u16,
    preview: State<'_, PreviewServerState>,
) -> Result<PreviewServerInfo, String> {
    if port == 0 {
        return Err("port must be greater than 0".into());
    }
    let output_dir = PathBuf::from(&output_dir);
    let (bound_port, url) = preview.start(&output_dir, port)?;
    Ok(PreviewServerInfo {
        url,
        port: bound_port,
        output_dir: output_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn stop_preview_server(preview: State<'_, PreviewServerState>) -> Result<(), String> {
    preview.stop();
    Ok(())
}

// Kept for compatibility with Task 1 scaffold until callers migrate.
#[tauri::command]
pub fn resolve_project_root_info() -> Result<ProjectRootInfo, String> {
    resolve_project_root()
}

#[tauri::command]
pub fn get_studio_settings(app: tauri::AppHandle) -> Result<StudioSettings, String> {
    load_settings(&app)
}

#[tauri::command]
pub fn save_studio_settings(app: tauri::AppHandle, settings: StudioSettings) -> Result<(), String> {
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn project_info_for_root(project_root: String) -> Result<ProjectRootInfo, String> {
    project_info_at(&project_root)
}

#[tauri::command]
pub fn list_bundle_files_cmd(
    project_root: String,
    site_path: String,
) -> Result<Vec<BundleFileEntry>, String> {
    list_bundle_files(&project_root, &site_path)
}

#[tauri::command]
pub fn read_bundle_file_cmd(
    project_root: String,
    site_path: String,
    relative_path: String,
) -> Result<String, String> {
    read_bundle_file(&project_root, &site_path, &relative_path)
}

#[tauri::command]
pub fn set_auto_rebuild(
    app: AppHandle,
    enabled: bool,
    project_root: String,
    site_path: String,
    strict: bool,
    preview_port: u16,
    watcher: State<'_, ContentWatcherState>,
) -> Result<(), String> {
    if !enabled {
        watcher.stop();
        return Ok(());
    }
    let project_root = parse_project_root(&project_root)?;
    watcher.start(app, project_root, site_path, strict, preview_port)
}

#[tauri::command]
pub fn write_bundle_file_cmd(
    project_root: String,
    site_path: String,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    write_bundle_file(&project_root, &site_path, &relative_path, &content)
}
