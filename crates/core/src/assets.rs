use crate::error::{CoreError, CoreResult};
use crate::routing::is_external_or_special_href;
use crate::types::{SiteBundle, WidgetNode};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct BundleAssetReference {
    web_path: String,
    config_path: String,
}

pub fn check_referenced_site_assets(bundle: &SiteBundle) -> CoreResult<()> {
    let seen = dedupe_bundle_asset_references(bundle)?;
    let mut paths: Vec<_> = seen.keys().cloned().collect();
    paths.sort();

    for web_path in paths {
        let config_path = &seen[&web_path];
        let src_abs = resolve_asset_under_site_bundle(&bundle.site_dir, &web_path)?.0;
        if !src_abs.is_file() {
            return Err(CoreError::msg(format!(
                "{config_path}: referenced asset does not exist: {web_path:?}"
            )));
        }
    }
    Ok(())
}

pub fn copy_referenced_site_assets(bundle: &SiteBundle, target_dir: &Path) -> CoreResult<()> {
    let seen = dedupe_bundle_asset_references(bundle)?;
    let mut paths: Vec<_> = seen.keys().cloned().collect();
    paths.sort();

    for web_path in paths {
        let config_path = &seen[&web_path];
        let (src_abs, rel_out) = resolve_asset_under_site_bundle(&bundle.site_dir, &web_path)?;
        if !src_abs.is_file() {
            return Err(CoreError::msg(format!(
                "{config_path}: referenced asset does not exist: {web_path:?}"
            )));
        }
        let dst = target_dir.join(rel_out.replace('/', std::path::MAIN_SEPARATOR_STR));
        crate::fs_util::copy_file(&src_abs, &dst).map_err(|e| {
            CoreError::msg(format!(
                "{config_path}: cannot copy asset {web_path:?}: {e}"
            ))
        })?;
    }
    Ok(())
}

fn dedupe_bundle_asset_references(bundle: &SiteBundle) -> CoreResult<HashMap<String, String>> {
    let refs = collect_bundle_asset_references(bundle)?;
    let mut seen = HashMap::new();
    for r in refs {
        seen.entry(r.web_path).or_insert(r.config_path);
    }
    Ok(seen)
}

fn collect_bundle_asset_references(bundle: &SiteBundle) -> CoreResult<Vec<BundleAssetReference>> {
    let mut refs = Vec::new();

    add_known(
        &mut refs,
        &bundle.site.header.brand.logo,
        &format!("{} -> header.brand.logo", bundle.site_path),
    )?;
    for (icon_key, icon_path) in &bundle.site.store_icons {
        add_known(
            &mut refs,
            icon_path,
            &format!("{} -> store_icons.{icon_key}", bundle.site_path),
        )?;
    }
    for (i, link) in bundle.site.social.links.iter().enumerate() {
        add_known(
            &mut refs,
            &link.icon_image,
            &format!("{} -> social.links[{i}].icon_image", bundle.site_path),
        )?;
    }

    for page_file in &bundle.pages {
        add_known(
            &mut refs,
            &page_file.page.seo.og_image,
            &format!("{} -> seo.og_image", page_file.path),
        )?;
        collect_asset_refs_from_widgets(
            &page_file.page.widgets,
            &format!("{} -> widgets", page_file.path),
            &mut refs,
        )?;
    }
    Ok(refs)
}

fn add_known(
    refs: &mut Vec<BundleAssetReference>,
    value: &str,
    config_path: &str,
) -> CoreResult<()> {
    if let Some(r) = normalize_asset_reference(value, config_path)? {
        refs.push(r);
    }
    Ok(())
}

fn collect_asset_refs_from_widgets(
    widgets: &[WidgetNode],
    base_path: &str,
    refs: &mut Vec<BundleAssetReference>,
) -> CoreResult<()> {
    for (i, widget) in widgets.iter().enumerate() {
        let widget_path = format!("{base_path}[{i}]");
        for (key, raw) in &widget.props {
            let prop_path = format!("{widget_path}.props.{key}");
            if key == "children" {
                let children: Vec<WidgetNode> =
                    serde_json::from_value(raw.clone()).map_err(|e| {
                        CoreError::msg(format!("{prop_path}: invalid children array: {e}"))
                    })?;
                collect_asset_refs_from_widgets(&children, &prop_path, refs)?;
                continue;
            }
            collect_asset_refs_from_any(raw, &prop_path, key, refs)?;
        }
    }
    Ok(())
}

fn collect_asset_refs_from_any(
    value: &JsonValue,
    path: &str,
    key: &str,
    refs: &mut Vec<BundleAssetReference>,
) -> CoreResult<()> {
    match value {
        JsonValue::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            for child_key in keys {
                let child_path = format!("{path}.{child_key}");
                collect_asset_refs_from_any(&map[&child_key], &child_path, &child_key, refs)?;
            }
        }
        JsonValue::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                collect_asset_refs_from_any(item, &format!("{path}[{i}]"), key, refs)?;
            }
        }
        JsonValue::String(s) if looks_like_asset_field(key) => {
            if let Some(r) = normalize_asset_reference(s, path)? {
                refs.push(r);
            }
        }
        _ => {}
    }
    Ok(())
}

fn looks_like_asset_field(key: &str) -> bool {
    let k = key.trim().to_lowercase();
    if k.is_empty() {
        return false;
    }
    k.contains("image")
        || k.contains("icon")
        || k.contains("logo")
        || k.contains("photo")
        || k.contains("asset")
        || k == "src"
        || k == "cover"
}

fn normalize_asset_reference(
    value: &str,
    config_path: &str,
) -> CoreResult<Option<BundleAssetReference>> {
    let raw = value.trim();
    if raw.is_empty() || is_external_or_special_href(raw) {
        return Ok(None);
    }
    let normalized = raw.trim_start_matches("./").replace('\\', "/");
    if !normalized.starts_with("assets/") {
        return Err(CoreError::msg(format!(
            r#"{config_path}: local asset path must start with "assets/" (got {raw:?})"#
        )));
    }
    if normalized == "assets/" || normalized == "assets" {
        return Err(CoreError::msg(format!(
            r#"{config_path}: local asset path must reference a file under "assets/" (got {raw:?})"#
        )));
    }
    Ok(Some(BundleAssetReference {
        web_path: normalized,
        config_path: config_path.to_string(),
    }))
}

fn resolve_asset_under_site_bundle(
    site_dir: &str,
    web_path: &str,
) -> CoreResult<(PathBuf, String)> {
    let p = web_path.trim().trim_start_matches("./").replace('\\', "/");
    if p.contains("..") {
        return Err(CoreError::msg(format!("invalid asset path {web_path:?}")));
    }
    if !p.starts_with("assets/") {
        return Err(CoreError::msg(format!(
            r#"invalid asset path {web_path:?} (must start with "assets/")"#
        )));
    }

    let assets_root = PathBuf::from(site_dir).join("assets").canonicalize()?;
    let local_rel = p.trim_start_matches("assets/");
    let clean = Path::new(local_rel);
    if clean.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::RootDir
        )
    }) {
        return Err(CoreError::msg(format!("invalid asset path {web_path:?}")));
    }
    let src_abs = assets_root.join(clean);
    let check_rel = src_abs.strip_prefix(&assets_root).map_err(|_| {
        CoreError::msg(format!("asset path escapes site assets root: {web_path:?}"))
    })?;
    if check_rel.as_os_str().is_empty() {
        return Err(CoreError::msg(format!("invalid asset path {web_path:?}")));
    }
    let rel_out = format!("assets/{}", check_rel.to_string_lossy().replace('\\', "/"));
    Ok((src_abs, rel_out))
}
