use portfoliowebsitebuilder_core::{load_site_bundle, resolve_site_dir};
use serde_json::Value;
use std::fs;
use std::path::Path;

const TEMPLATE_REL: &str = "content/_template";
const CONTENT_ROOT: &str = "content";

pub fn validate_site_id(site_id: &str) -> Result<(), String> {
    let id = site_id.trim();
    if id.is_empty() {
        return Err("site id cannot be empty".into());
    }
    if id != site_id {
        return Err("site id must not have leading or trailing whitespace".into());
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("site id must be a single folder name (no path separators)".into());
    }
    if id.starts_with('-') || id.ends_with('-') {
        return Err("site id cannot start or end with a hyphen".into());
    }
    for c in id.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
            continue;
        }
        return Err(format!(
            "invalid site id {id:?}: use lowercase letters, digits, and hyphens only"
        ));
    }
    Ok(())
}

/// `Results/DemoWebsite` style path from a bundle folder name.
pub fn output_folder_for_site_id(site_id: &str) -> String {
    let title: String = site_id
        .split('-')
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
            }
        })
        .collect();
    format!("Results/{title}Website")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let from = src.join(&name);
        let to = dst.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn patch_site_json(site_json_path: &Path, site_id: &str) -> Result<(), String> {
    let raw = fs::read_to_string(site_json_path)
        .map_err(|e| format!("read {}: {e}", site_json_path.display()))?;
    let mut value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse site.json: {e}"))?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "site.json must be a JSON object".to_string())?;
    obj.insert("site_id".to_string(), Value::String(site_id.to_string()));
    obj.insert(
        "output_folder".to_string(),
        Value::String(output_folder_for_site_id(site_id)),
    );
    let patched = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(site_json_path, format!("{patched}\n"))
        .map_err(|e| format!("write {}: {e}", site_json_path.display()))?;
    Ok(())
}

/// Copy `content/_template/` to `content/<site_id>/`, patch `site.json`, verify bundle loads.
pub fn create_site_from_template(project_root: &Path, site_id: &str) -> Result<String, String> {
    validate_site_id(site_id)?;
    let site_id = site_id.trim();

    let template_dir = project_root.join(TEMPLATE_REL);
    if !template_dir.is_dir() {
        return Err(format!(
            "template not found at {TEMPLATE_REL}; add the skeleton under content/_template/"
        ));
    }

    let target_dir = project_root.join(CONTENT_ROOT).join(site_id);
    if target_dir.exists() {
        return Err(format!("site already exists: content/{site_id}"));
    }

    copy_dir_recursive(&template_dir, &target_dir).map_err(|e| format!("copy template: {e}"))?;

    let site_json_path = target_dir.join("site.json");
    patch_site_json(&site_json_path, site_id)?;

    let bundle_rel = format!("{CONTENT_ROOT}/{site_id}");
    let site_dir = resolve_site_dir(project_root, &bundle_rel);
    let (bundle, _) = load_site_bundle(&site_dir).map_err(|e| e.to_string())?;
    if bundle.site.site_id != site_id {
        return Err(format!(
            "site.json site_id {:?} does not match folder name {site_id:?}",
            bundle.site.site_id
        ));
    }

    Ok(bundle_rel.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_site_id_rejects_invalid() {
        assert!(validate_site_id("").is_err());
        assert!(validate_site_id("My Site").is_err());
        assert!(validate_site_id("-bad").is_err());
        assert!(validate_site_id("bad-").is_err());
        assert!(validate_site_id("a/b").is_err());
        assert!(validate_site_id("ok-site").is_ok());
    }

    #[test]
    fn output_folder_for_site_id_formats() {
        assert_eq!(output_folder_for_site_id("demo"), "Results/DemoWebsite");
        assert_eq!(
            output_folder_for_site_id("my-studio"),
            "Results/MyStudioWebsite"
        );
    }
}
