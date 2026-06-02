use portfoliowebsitebuilder_core::resolve_site_dir;
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Serialize)]
pub struct BundleFileEntry {
    pub relative_path: String,
    pub name: String,
    pub is_dir: bool,
}

fn parse_project_root(project_root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(project_root);
    if !path.is_dir() {
        return Err(format!("project root is not a directory: {project_root}"));
    }
    Ok(path)
}

fn bundle_root(project_root: &Path, site_path: &str) -> PathBuf {
    resolve_site_dir(project_root, site_path)
}

/// Resolve `relative_path` under the bundle directory; reject path traversal.
fn resolve_bundle_relative(bundle_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let rel = Path::new(relative_path.trim_start_matches(['/', '\\']));
    if rel.as_os_str().is_empty() {
        return Err("empty relative path".into());
    }
    for component in rel.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(format!("invalid path component in {relative_path}")),
        }
    }
    let joined = bundle_root.join(rel);
    let bundle_canon = bundle_root
        .canonicalize()
        .map_err(|e| format!("bundle directory: {e}"))?;
    let parent = joined
        .parent()
        .ok_or_else(|| "invalid file path".to_string())?;
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file_canon = joined
        .canonicalize()
        .or_else(|_| {
            // New file not yet on disk: canonicalize parent + file name
            let name = joined
                .file_name()
                .ok_or_else(|| "invalid file path".to_string())?;
            parent
                .canonicalize()
                .map(|p| p.join(name))
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
    if !file_canon.starts_with(&bundle_canon) {
        return Err("path escapes content bundle".into());
    }
    Ok(file_canon)
}

fn is_editable_json(relative_path: &str) -> bool {
    let rel = relative_path.replace('\\', "/");
    rel == "site.json" || rel.starts_with("pages/") && rel.ends_with(".json")
}

fn is_image_file(relative_path: &str) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "avif"];
    Path::new(relative_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn mime_for_image(relative_path: &str) -> &'static str {
    match Path::new(relative_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("avif") => "image/avif",
        _ => "application/octet-stream",
    }
}

#[derive(Clone, Serialize)]
pub struct BundleImagePreview {
    pub relative_path: String,
    pub data_url: String,
}

pub fn list_bundle_files(
    project_root: &str,
    site_path: &str,
) -> Result<Vec<BundleFileEntry>, String> {
    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    if !bundle.is_dir() {
        return Err(format!("bundle not found: {}", bundle.display()));
    }

    let mut entries = Vec::new();

    let site_json = bundle.join("site.json");
    if site_json.is_file() {
        entries.push(BundleFileEntry {
            relative_path: "site.json".into(),
            name: "site.json".into(),
            is_dir: false,
        });
    }

    let pages_dir = bundle.join("pages");
    if pages_dir.is_dir() {
        let mut pages: Vec<_> = fs::read_dir(&pages_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                BundleFileEntry {
                    relative_path: format!("pages/{name}"),
                    name,
                    is_dir: false,
                }
            })
            .collect();
        pages.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        entries.extend(pages);
    }

    let assets_dir = bundle.join("assets");
    entries.push(BundleFileEntry {
        relative_path: "assets".into(),
        name: "assets".into(),
        is_dir: true,
    });
    if assets_dir.is_dir() {
        let mut asset_names: Vec<_> = fs::read_dir(&assets_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        asset_names.sort();
        for name in asset_names {
            let path = assets_dir.join(&name);
            entries.push(BundleFileEntry {
                relative_path: format!("assets/{name}"),
                name,
                is_dir: path.is_dir(),
            });
        }
    }

    Ok(entries)
}

pub fn read_bundle_file(
    project_root: &str,
    site_path: &str,
    relative_path: &str,
) -> Result<String, String> {
    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let path = resolve_bundle_relative(&bundle, relative_path)?;
    if path.is_dir() {
        return Err("cannot read a directory".into());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn read_bundle_image(
    project_root: &str,
    site_path: &str,
    relative_path: &str,
) -> Result<BundleImagePreview, String> {
    if !is_image_file(relative_path) {
        return Err("not an image file".into());
    }
    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let path = resolve_bundle_relative(&bundle, relative_path)?;
    if path.is_dir() {
        return Err("cannot preview a directory".into());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let encoded = STANDARD.encode(bytes);
    let mime = mime_for_image(relative_path);
    Ok(BundleImagePreview {
        relative_path: relative_path.replace('\\', "/"),
        data_url: format!("data:{mime};base64,{encoded}"),
    })
}

pub fn write_bundle_file(
    project_root: &str,
    site_path: &str,
    relative_path: &str,
    content: &str,
) -> Result<(), String> {
    if !is_editable_json(relative_path) {
        return Err("only site.json and pages/*.json are writable".into());
    }
    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let path = resolve_bundle_relative(&bundle, relative_path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn project_info_at(project_root: &str) -> Result<crate::diagnostics::ProjectRootInfo, String> {
    let project_root = parse_project_root(project_root)?;
    let template_dir = project_root.join("Template");
    if !template_dir.is_dir() {
        return Err(format!(
            "Template/ not found under {}",
            project_root.display()
        ));
    }
    let content = project_root.join("content");
    if !content.is_dir() {
        return Err(format!(
            "content/ not found under {}",
            project_root.display()
        ));
    }
    Ok(crate::diagnostics::ProjectRootInfo {
        project_root: project_root.to_string_lossy().into_owned(),
        template_dir: template_dir.to_string_lossy().into_owned(),
    })
}
