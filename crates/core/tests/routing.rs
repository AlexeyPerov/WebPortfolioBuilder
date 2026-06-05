//! Port of Go `routing_test.go` plus route-index expectations for kometa/demo.

mod common;

use common::workspace_root;
use portfoliowebsitebuilder_core::{
    build_route_index, is_nav_item_active, load_site_bundle, normalized_slug,
    resolve_internal_slug_reference, resolve_nav_href, validated_output_folder_for, PageRoute,
    SiteBundle,
};
use std::collections::HashMap;

fn sample_routes() -> HashMap<String, PageRoute> {
    let mut routes = HashMap::new();
    routes.insert(
        String::new(),
        PageRoute {
            slug: String::new(),
            source_path: "pages/home.json".into(),
            output_rel_path: "index.html".into(),
            dir_rel_path: String::new(),
        },
    );
    routes.insert(
        "about".into(),
        PageRoute {
            slug: "about".into(),
            source_path: "pages/about.json".into(),
            output_rel_path: "about/index.html".into(),
            dir_rel_path: "about".into(),
        },
    );
    routes
}

#[test]
fn build_route_index_maps_home_and_slug() {
    let bundle = SiteBundle {
        site_dir: "/tmp/site".into(),
        site_path: "site.json".into(),
        site: Default::default(),
        pages: vec![
            portfoliowebsitebuilder_core::types::SitePageFile {
                path: "pages/home.json".into(),
                page: portfoliowebsitebuilder_core::types::PageConfig {
                    slug: String::new(),
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
            portfoliowebsitebuilder_core::types::SitePageFile {
                path: "pages/about.json".into(),
                page: portfoliowebsitebuilder_core::types::PageConfig {
                    slug: "about".into(),
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
        ],
    };

    let index = build_route_index(&bundle).unwrap();
    let home = index.by_slug.get("").unwrap();
    assert_eq!(home.output_rel_path, "index.html");
    let about = index.by_slug.get("about").unwrap();
    assert_eq!(about.output_rel_path, "about/index.html");
}

#[test]
fn resolve_internal_slug_reference_uses_relative_paths() {
    let routes = sample_routes();
    let from_home = resolve_internal_slug_reference(&routes[""], "about", &routes).unwrap();
    assert_eq!(from_home, "about/");
    let from_about = resolve_internal_slug_reference(&routes["about"], "", &routes).unwrap();
    assert_eq!(from_about, "../");
}

#[test]
fn resolve_nav_href_prefixes_hash_links_from_nested_page() {
    let routes = sample_routes();
    let from_home = resolve_nav_href(&routes[""], "#intro_title", &routes).unwrap();
    assert_eq!(from_home, "#intro_title");
    let from_about = resolve_nav_href(&routes["about"], "#intro_title", &routes).unwrap();
    assert_eq!(from_about, "../#intro_title");
}

#[test]
fn resolve_nav_href_empty_href_is_home() {
    let routes = sample_routes();
    let from_about = resolve_nav_href(&routes["about"], "", &routes).unwrap();
    assert_eq!(from_about, "../");
}

#[test]
fn is_nav_item_active_matches_page_slug() {
    let routes = sample_routes();
    assert!(is_nav_item_active(&routes["about"], "about").unwrap());
    assert!(!is_nav_item_active(&routes["about"], "").unwrap());
    assert!(is_nav_item_active(&routes[""], "").unwrap());
    assert!(!is_nav_item_active(&routes[""], "about").unwrap());
    assert!(!is_nav_item_active(&routes[""], "#intro").unwrap());
}

#[test]
fn resolve_internal_slug_reference_rejects_unknown_slug() {
    let routes = sample_routes();
    assert!(resolve_internal_slug_reference(&routes[""], "missing", &routes).is_err());
}

#[test]
fn normalized_slug_rejects_parent_segments() {
    assert!(normalized_slug("..").is_err());
    assert!(normalized_slug("about/../x").is_err());
}

#[test]
fn validated_output_folder_rejects_dot_and_parent_segments() {
    assert!(validated_output_folder_for("Results/../out", "site.json").is_err());
    assert!(validated_output_folder_for("Results/./site", "site.json").is_err());
}

#[test]
fn kometa_route_index_single_home() {
    let kometa = workspace_root().join("content/kometa");
    if !kometa.join("site.json").is_file() {
        eprintln!("skip kometa_route_index: bundle not present");
        return;
    }
    let (bundle, _) = load_site_bundle(&kometa).unwrap();
    let index = build_route_index(&bundle).unwrap();
    assert_eq!(index.ordered.len(), 1);
    assert_eq!(index.by_slug[""].output_rel_path, "index.html");
}

#[test]
fn demo_route_index_multi_page_paths() {
    let demo = workspace_root().join("content/demo");
    if !demo.join("site.json").is_file() {
        eprintln!("skip demo_route_index: bundle not present");
        return;
    }
    let (bundle, _) = load_site_bundle(&demo).unwrap();
    let index = build_route_index(&bundle).unwrap();
    assert_eq!(index.ordered.len(), 6);
    assert_eq!(index.by_slug[""].output_rel_path, "index.html");
    for slug in ["about", "apps", "careers", "gallery", "layouts"] {
        let route = index
            .by_slug
            .get(slug)
            .unwrap_or_else(|| panic!("missing slug {slug}"));
        assert_eq!(route.output_rel_path, format!("{slug}/index.html"));
    }
}
