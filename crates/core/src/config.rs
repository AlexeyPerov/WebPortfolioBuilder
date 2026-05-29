use crate::error::{CoreError, CoreResult};
use crate::strict::unknown_widget_prop_key_warnings;
use crate::types::{ConfigWarning, PageConfig, SiteBundle, SiteConfig, SitePageFile, WidgetNode};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const SITE_TOP_LEVEL_KEYS: &[&str] = &[
    "site_id",
    "output_folder",
    "theme",
    "typography",
    "store_icons",
    "subscribe_block",
    "social",
    "header",
    "footer",
    "base_url",
    "widgets",
];

const PAGE_TOP_LEVEL_KEYS: &[&str] = &["slug", "widgets", "title", "seo", "layout"];

const LEGACY_PAGE_KEYS: &[&str] = &["hero"];

pub fn load_site_bundle(site_dir: &Path) -> CoreResult<(SiteBundle, Vec<ConfigWarning>)> {
    let clean = site_dir.canonicalize().unwrap_or_else(|_| site_dir.to_path_buf());
    let site_path = clean.join("site.json");
    let (site, site_warnings) = load_site_config(&site_path)?;

    let pages_dir = clean.join("pages");
    let mut page_paths: Vec<PathBuf> = fs::read_dir(&pages_dir)
        .map_err(|e| {
            CoreError::msg(format!(
                "cannot list page configs in {:?}: {e}",
                pages_dir
            ))
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    page_paths.sort();

    let mut warnings = site_warnings;
    let mut pages = Vec::new();
    for page_path in page_paths {
        let (page_file, page_warnings) = load_page_config(&page_path)?;
        warnings.extend(page_warnings);
        pages.push(page_file);
    }

    let bundle = SiteBundle {
        site_dir: clean.to_string_lossy().into_owned(),
        site_path: site_path.to_string_lossy().into_owned(),
        site,
        pages,
    };

    validate_site_bundle(&bundle)?;
    for page_file in &bundle.pages {
        warnings.extend(unknown_widget_prop_key_warnings(
            &page_file.path,
            &page_file.page.widgets,
        ));
    }

    Ok((bundle, warnings))
}

fn load_site_config(path: &Path) -> CoreResult<(SiteConfig, Vec<ConfigWarning>)> {
    let (raw_keys, site) = decode_json_object_file(path)?;
    let warnings = unknown_top_level_key_warnings(path, &raw_keys, SITE_TOP_LEVEL_KEYS);
    Ok((site, warnings))
}

fn load_page_config(path: &Path) -> CoreResult<(SitePageFile, Vec<ConfigWarning>)> {
    let (raw_keys, page) = decode_json_object_file::<PageConfig>(path)?;
    let page_file = SitePageFile {
        path: path.to_string_lossy().into_owned(),
        page,
        has_slug: raw_keys.contains_key("slug"),
        has_widgets: raw_keys.contains_key("widgets"),
    };
    let mut warnings = legacy_page_key_warnings(path, &raw_keys);
    warnings.extend(unknown_top_level_key_warnings(
        path,
        &raw_keys,
        PAGE_TOP_LEVEL_KEYS,
    ));
    Ok((page_file, warnings))
}

fn decode_json_object_file<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> CoreResult<(HashMap<String, JsonValue>, T)> {
    let data = fs::read_to_string(path)
        .map_err(|e| CoreError::msg(format!("cannot read config {:?}: {e}", path)))?;
    let raw_keys: HashMap<String, JsonValue> = serde_json::from_str(&data)
        .map_err(|e| CoreError::msg(format!("invalid config json {:?}: {e}", path)))?;
    let target: T = serde_json::from_str(&data)
        .map_err(|e| CoreError::msg(format!("invalid config json {:?}: {e}", path)))?;
    Ok((raw_keys, target))
}

fn legacy_page_key_warnings(path: &Path, raw_keys: &HashMap<String, JsonValue>) -> Vec<ConfigWarning> {
    let path_s = path.to_string_lossy();
    if raw_keys.contains_key("hero") {
        vec![ConfigWarning::content(
            path_s,
            r#"legacy key "hero" is not supported; use widgets (e.g. intro, cover_banner) for page heroes"#,
        )]
    } else {
        vec![]
    }
}

fn unknown_top_level_key_warnings(
    path: &Path,
    raw_keys: &HashMap<String, JsonValue>,
    allowed: &[&str],
) -> Vec<ConfigWarning> {
    let allowed: HashSet<&str> = allowed.iter().copied().collect();
    let legacy: HashSet<&str> = LEGACY_PAGE_KEYS.iter().copied().collect();
    let path_s = path.to_string_lossy().into_owned();
    let mut unknown: Vec<String> = raw_keys
        .keys()
        .filter(|k| !allowed.contains(k.as_str()) && !legacy.contains(k.as_str()))
        .cloned()
        .collect();
    unknown.sort();
    unknown
        .into_iter()
        .map(|key| ConfigWarning {
            file_path: path_s.clone(),
            key,
            detail: String::new(),
        })
        .collect()
}

pub fn validate_site_bundle(bundle: &SiteBundle) -> CoreResult<()> {
    if bundle.site.site_id.trim().is_empty() {
        return Err(CoreError::msg(format!(
            r#"{} -> "site_id" is required and must not be empty"#,
            bundle.site_path
        )));
    }
    validated_output_folder_for(&bundle.site.output_folder, &bundle.site_path)?;

    let mut slug_owner: HashMap<String, String> = HashMap::new();
    for page_file in &bundle.pages {
        if !page_file.has_slug {
            return Err(CoreError::msg(format!(
                r#"{} -> "slug" is required"#,
                page_file.path
            )));
        }
        if !page_file.has_widgets {
            return Err(CoreError::msg(format!(
                r#"{} -> "widgets" is required"#,
                page_file.path
            )));
        }
        let slug = page_file.page.slug.clone();
        if let Some(other) = slug_owner.get(&slug) {
            return Err(CoreError::msg(format!(
                r#"{} -> duplicate slug {:?} (already used in {})"#,
                page_file.path, slug, other
            )));
        }
        slug_owner.insert(slug, page_file.path.clone());
        validate_duplicate_project_grid_section_ids(&page_file.path, &page_file.page.widgets)?;
    }
    Ok(())
}

pub fn validated_output_folder(name: &str) -> CoreResult<String> {
    validated_output_folder_for(name, "config.json")
}

pub fn validated_output_folder_for(name: &str, source: &str) -> CoreResult<String> {
    let s = name.trim();
    if s.is_empty() {
        return Err(CoreError::msg(format!(
            r#"{source}: "output_folder" is required and must not be empty"#
        )));
    }
    if Path::new(s).is_absolute() {
        return Err(CoreError::msg(format!(
            r#"{source}: "output_folder" must be a relative path (got {:?})"#,
            s
        )));
    }
    let normalized = s.replace('\\', "/").trim_matches('/').to_string();
    if normalized.is_empty() {
        return Err(CoreError::msg(format!(
            r#"{source}: "output_folder" is required and must not be empty"#
        )));
    }
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(CoreError::msg(format!(
                r#"{source}: invalid "output_folder" {:?}"#,
                name
            )));
        }
    }
    Ok(normalized)
}

fn validate_duplicate_project_grid_section_ids(
    page_path: &str,
    widgets: &[WidgetNode],
) -> CoreResult<()> {
    let mut seen: HashMap<String, String> = HashMap::new();
    walk_project_grid_ids("widgets", page_path, widgets, &mut seen)
}

fn walk_project_grid_ids(
    prefix: &str,
    page_path: &str,
    widgets: &[WidgetNode],
    seen: &mut HashMap<String, String>,
) -> CoreResult<()> {
    for (i, w) in widgets.iter().enumerate() {
        let wpath = format!("{prefix}[{i}]");
        match w.widget_type.trim() {
            "project_grid" => {
                if let Some(sid) = project_grid_section_id_raw(&w.props) {
                    if let Some(prev) = seen.get(&sid) {
                        return Err(CoreError::msg(format!(
                            r#"{page_path} -> duplicate project_grid props.section_id {:?} (also declared at {prev})"#,
                            sid
                        )));
                    }
                    seen.insert(sid, format!("{wpath} ({page_path})"));
                }
            }
            "row" | "column" | "grid" => {
                if let Some(children) = widget_layout_children(&w.props)? {
                    if !children.is_empty() {
                        walk_project_grid_ids(
                            &format!("{wpath}.props.children"),
                            page_path,
                            &children,
                            seen,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn widget_layout_children(
    props: &HashMap<String, JsonValue>,
) -> CoreResult<Option<Vec<WidgetNode>>> {
    match props.get("children") {
        None => Ok(None),
        Some(raw) => {
            let children: Vec<WidgetNode> = serde_json::from_value(raw.clone())
                .map_err(|_| CoreError::msg("invalid layout children"))?;
            Ok(Some(children))
        }
    }
}

fn project_grid_section_id_raw(props: &HashMap<String, JsonValue>) -> Option<String> {
    props.get("section_id").and_then(|v| {
        v.as_str()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    })
}
