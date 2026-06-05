use portfoliowebsitebuilder_core::{discover_content_bundles, resolve_site_dir};
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

#[derive(Clone, Serialize)]
pub struct RenameBundleAssetResult {
    pub new_relative_path: String,
    pub updated_sites: Vec<String>,
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

fn is_deletable_asset(relative_path: &str) -> bool {
    let rel = relative_path.replace('\\', "/");
    rel.starts_with("assets/") && is_image_file(relative_path)
}

fn import_asset_filename(source: &Path) -> String {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    let folder_name = source
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if folder_name.is_empty() {
        file_name.to_string()
    } else {
        format!("{folder_name}_{file_name}")
    }
}

fn unique_asset_filename(assets_dir: &Path, original_name: &str) -> String {
    let path = Path::new(original_name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    let mut candidate = original_name.to_string();
    let mut n = 1u32;
    while assets_dir.join(&candidate).exists() {
        candidate = format!("{stem}-{n}{ext}");
        n += 1;
    }
    candidate
}

pub fn import_bundle_asset(
    project_root: &str,
    site_path: &str,
    source_path: &str,
) -> Result<String, String> {
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err(format!("source is not a file: {source_path}"));
    }
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "invalid source file name".to_string())?;
    if !is_image_file(file_name) {
        return Err("source is not a supported image file".into());
    }

    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let assets_dir = bundle.join("assets");
    fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

    let dest_name = unique_asset_filename(&assets_dir, &import_asset_filename(&source));
    let dest_path = assets_dir.join(&dest_name);
    fs::copy(&source, &dest_path).map_err(|e| e.to_string())?;

    Ok(format!("assets/{dest_name}"))
}

fn replace_asset_path_in_text(content: &str, old_path: &str, new_path: &str) -> String {
    content.replace(old_path, new_path)
}

fn update_bundle_asset_references(
    bundle: &Path,
    old_relative_path: &str,
    new_relative_path: &str,
) -> Result<bool, String> {
    let old_norm = old_relative_path.replace('\\', "/");
    let new_norm = new_relative_path.replace('\\', "/");
    let mut changed = false;

    let site_json = bundle.join("site.json");
    if site_json.is_file() {
        let content = fs::read_to_string(&site_json).map_err(|e| e.to_string())?;
        let updated = replace_asset_path_in_text(&content, &old_norm, &new_norm);
        if updated != content {
            fs::write(&site_json, updated).map_err(|e| e.to_string())?;
            changed = true;
        }
    }

    let pages_dir = bundle.join("pages");
    if pages_dir.is_dir() {
        for entry in fs::read_dir(&pages_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if !path.extension().is_some_and(|ext| ext == "json") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let updated = replace_asset_path_in_text(&content, &old_norm, &new_norm);
            if updated != content {
                fs::write(&path, updated).map_err(|e| e.to_string())?;
                changed = true;
            }
        }
    }

    Ok(changed)
}

pub fn rename_bundle_asset(
    project_root: &str,
    site_path: &str,
    relative_path: &str,
    new_name: &str,
) -> Result<RenameBundleAssetResult, String> {
    if !is_deletable_asset(relative_path) {
        return Err("only image files under assets/ can be renamed".into());
    }

    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err("new name is empty".into());
    }
    if new_name.contains('/') || new_name.contains('\\') {
        return Err("name must not contain path separators".into());
    }
    if !is_image_file(new_name) {
        return Err("new name must be a supported image file".into());
    }

    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let old_path = resolve_bundle_relative(&bundle, relative_path)?;
    if !old_path.is_file() {
        return Err(format!("file not found: {relative_path}"));
    }

    let old_norm = relative_path.replace('\\', "/");
    let new_relative_path = format!("assets/{new_name}");
    if new_relative_path == old_norm {
        return Ok(RenameBundleAssetResult {
            new_relative_path,
            updated_sites: vec![],
        });
    }

    let new_path = resolve_bundle_relative(&bundle, &new_relative_path)?;
    if new_path.exists() {
        return Err(format!("asset already exists: {new_relative_path}"));
    }

    let site_norm = site_path.replace('\\', "/");
    let bundles = discover_content_bundles(&project_root).map_err(|e| e.to_string())?;
    let mut updated_sites = Vec::new();

    for site in &bundles {
        let site_bundle = bundle_root(&project_root, site);
        let old_file = site_bundle.join(&old_norm);
        let new_file = site_bundle.join(&new_relative_path);
        let mut touched = false;

        if old_file.is_file() {
            if new_file.exists() {
                return Err(format!("asset already exists in {site}: {new_relative_path}"));
            }
            fs::rename(&old_file, &new_file).map_err(|e| e.to_string())?;
            touched = true;
        } else if site == &site_norm {
            return Err(format!("file not found: {relative_path}"));
        }

        if update_bundle_asset_references(&site_bundle, &old_norm, &new_relative_path)? {
            touched = true;
        }

        if touched {
            updated_sites.push(site.clone());
        }
    }

    Ok(RenameBundleAssetResult {
        new_relative_path,
        updated_sites,
    })
}

pub fn delete_bundle_asset(
    project_root: &str,
    site_path: &str,
    relative_path: &str,
) -> Result<(), String> {
    if !is_deletable_asset(relative_path) {
        return Err("only image files under assets/ can be deleted".into());
    }
    let project_root = parse_project_root(project_root)?;
    let bundle = bundle_root(&project_root, site_path);
    let path = resolve_bundle_relative(&bundle, relative_path)?;
    if path.is_dir() {
        return Err("cannot delete a directory".into());
    }
    if !path.is_file() {
        return Err(format!("file not found: {relative_path}"));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())
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
