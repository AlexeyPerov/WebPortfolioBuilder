use crate::error::{CoreError, CoreResult};
use crate::types::SiteBundle;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRoute {
    pub slug: String,
    pub source_path: String,
    pub output_rel_path: String,
    pub dir_rel_path: String,
}

#[derive(Debug, Clone)]
pub struct RouteIndex {
    pub ordered: Vec<PageRoute>,
    pub by_slug: HashMap<String, PageRoute>,
}

pub fn build_route_index(bundle: &SiteBundle) -> CoreResult<RouteIndex> {
    let mut routes = Vec::with_capacity(bundle.pages.len());
    let mut by_slug = HashMap::with_capacity(bundle.pages.len());

    for page_file in &bundle.pages {
        let slug = normalized_slug(&page_file.page.slug)?;
        let route = PageRoute {
            slug: slug.clone(),
            source_path: page_file.path.clone(),
            output_rel_path: output_rel_path_for_slug(&slug),
            dir_rel_path: output_dir_rel_path_for_slug(&slug),
        };
        by_slug.insert(slug, route.clone());
        routes.push(route);
    }

    routes.sort_by(|a, b| a.output_rel_path.cmp(&b.output_rel_path));

    Ok(RouteIndex {
        ordered: routes,
        by_slug,
    })
}

fn output_rel_path_for_slug(slug: &str) -> String {
    if slug.is_empty() {
        "index.html".into()
    } else {
        format!("{slug}/index.html")
    }
}

fn output_dir_rel_path_for_slug(slug: &str) -> String {
    if slug.is_empty() {
        String::new()
    } else {
        slug.to_string()
    }
}

pub fn normalized_slug(slug: &str) -> CoreResult<String> {
    let s = slug.trim().trim_matches('/');
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.contains("..") || s.contains('\\') || s.contains("//") {
        return Err(CoreError::msg(format!("invalid slug {slug:?}")));
    }
    Ok(s.replace('\\', "/"))
}

pub fn resolve_nav_href(
    current: &PageRoute,
    raw_href: &str,
    routes_by_slug: &HashMap<String, PageRoute>,
) -> CoreResult<String> {
    let href = raw_href.trim();
    if href.is_empty() {
        return resolve_internal_slug_reference(current, "", routes_by_slug);
    }
    if href.starts_with('#') {
        if current.slug.is_empty() {
            return Ok(href.to_string());
        }
        let home_prefix = resolve_internal_slug_reference(current, "", routes_by_slug)?;
        return Ok(format!("{home_prefix}{href}"));
    }
    resolve_internal_slug_reference(current, href, routes_by_slug)
}

pub fn resolve_internal_slug_reference(
    current: &PageRoute,
    raw_href: &str,
    routes_by_slug: &HashMap<String, PageRoute>,
) -> CoreResult<String> {
    let href = raw_href.trim();
    if is_external_or_special_href(href) {
        return Ok(href.to_string());
    }

    let (target_slug, fragment) = parse_slug_reference(href)?;
    let target_route = routes_by_slug.get(&target_slug).ok_or_else(|| {
        CoreError::msg(format!("unknown internal slug reference {target_slug:?}"))
    })?;

    let base = relative_dir_link(&current.dir_rel_path, &target_route.dir_rel_path);
    Ok(format!("{base}{fragment}"))
}

fn parse_slug_reference(href: &str) -> CoreResult<(String, String)> {
    let (href, frag) = if let Some(i) = href.find('#') {
        (&href[..i], href[i..].to_string())
    } else {
        (href, String::new())
    };
    let href = href.trim();
    if href.is_empty() || href == "/" {
        return Ok((String::new(), frag));
    }
    let slug = normalized_slug(href)?;
    Ok((slug, frag))
}

pub fn relative_dir_link(from_dir: &str, to_dir: &str) -> String {
    let from_path = dir_path_components(from_dir);
    let to_path = dir_path_components(to_dir);

    let rel = path_relative(&from_path, &to_path);
    if rel == "." {
        "./".into()
    } else if rel.ends_with('/') {
        rel
    } else {
        format!("{rel}/")
    }
}

fn dir_path_components(dir: &str) -> PathBuf {
    let trimmed = dir.trim().trim_matches('/');
    if trimmed.is_empty() {
        PathBuf::new()
    } else {
        trimmed.split('/').filter(|s| !s.is_empty()).collect()
    }
}

fn path_relative(from: &Path, to: &Path) -> String {
    let from_comps: Vec<_> = from.components().collect();
    let to_comps: Vec<_> = to.components().collect();
    let mut i = 0;
    while i < from_comps.len() && i < to_comps.len() && from_comps[i] == to_comps[i] {
        i += 1;
    }
    let mut result = String::new();
    for _ in i..from_comps.len() {
        if !result.is_empty() {
            result.push('/');
        }
        result.push_str("..");
    }
    for (j, comp) in to_comps.iter().enumerate().skip(i) {
        if !result.is_empty() {
            result.push('/');
        }
        if let Component::Normal(s) = comp {
            result.push_str(&s.to_string_lossy());
        }
        let _ = j;
    }
    if result.is_empty() {
        ".".into()
    } else {
        result
    }
}

pub fn asset_prefix_for_depth(dir_rel_path: &str) -> String {
    if dir_rel_path.is_empty() {
        return String::new();
    }
    let segments = dir_rel_path.split('/').filter(|s| !s.is_empty()).count();
    "../".repeat(segments)
}

pub fn resolve_asset_href_for_page(raw: &str, route: &PageRoute) -> String {
    let href = raw.trim();
    if href.is_empty() {
        return String::new();
    }
    if is_external_or_special_href(href) || href.starts_with('/') {
        return href.to_string();
    }
    let href = href.trim_start_matches("./");
    format!("{}{href}", asset_prefix_for_depth(&route.dir_rel_path))
}

pub fn is_external_or_special_href(href: &str) -> bool {
    if href.starts_with('#') {
        return true;
    }
    let lowered = href.to_lowercase();
    lowered.starts_with("http://")
        || lowered.starts_with("https://")
        || lowered.starts_with("mailto:")
        || lowered.starts_with("tel:")
        || lowered.starts_with("data:")
        || lowered.starts_with("javascript:")
}
