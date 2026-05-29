use crate::error::{CoreError, CoreResult};
use std::fs;
use std::path::{Path, PathBuf};

const CONTENT_ROOT_REL: &str = "content";
const DEFAULT_CONTENT_BUNDLE_REL: &str = "content/kometa";

pub fn resolve_project_root() -> CoreResult<PathBuf> {
    let wd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if content_bundle_marker_exists(&wd) {
        return Ok(wd);
    }
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(Path::new(".")).to_path_buf();
        if content_bundle_marker_exists(&exe_dir) {
            return Ok(exe_dir);
        }
    }
    Ok(wd)
}

fn content_bundle_marker_exists(root: &Path) -> bool {
    root.join(DEFAULT_CONTENT_BUNDLE_REL)
        .join("site.json")
        .is_file()
}

pub fn discover_content_bundles(project_root: &Path) -> CoreResult<Vec<String>> {
    let content_dir = project_root.join(CONTENT_ROOT_REL);
    let entries = match fs::read_dir(&content_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => {
            return Err(CoreError::msg(format!(
                "cannot read {:?}: {e}",
                content_dir
            )))
        }
    };

    let mut bundles = Vec::new();
    for ent in entries.flatten() {
        if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir_name = ent.file_name();
        let dir_name = dir_name.to_string_lossy();
        if dir_name.starts_with('_') {
            continue;
        }
        let rel = format!("{}/{}", CONTENT_ROOT_REL, dir_name);
        let site_json = project_root.join(&rel).join("site.json");
        if is_valid_site_json(&site_json) {
            bundles.push(rel.replace('\\', "/"));
        }
    }
    bundles.sort();
    Ok(bundles)
}

fn is_valid_site_json(path: &Path) -> bool {
    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let raw: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return false,
    };
    raw.contains_key("site_id") && raw.contains_key("output_folder")
}

pub fn resolve_site_dir(project_root: &Path, site_input: &str) -> PathBuf {
    let site_input = site_input.trim();
    if site_input.is_empty() {
        return project_root.join(DEFAULT_CONTENT_BUNDLE_REL);
    }
    let p = Path::new(site_input);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        project_root.join(site_input.replace('/', std::path::MAIN_SEPARATOR_STR))
    }
}
