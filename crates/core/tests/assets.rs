//! Port of Go `asset_pipeline_test.go`.

mod common;

use common::{workspace_root, write_json_file};
use portfoliowebsitebuilder_core::types::{
    HeaderBrand, HeaderConfig, PageConfig, PageSeo, SiteConfig, SitePageFile, WidgetNode,
};
use portfoliowebsitebuilder_core::{
    check_referenced_site_assets, copy_referenced_site_assets, load_site_bundle,
    validate_site_bundle_only, SiteBundle,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn write_test_asset(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, b"asset").unwrap();
}

#[test]
fn collect_bundle_asset_references_rejects_non_assets_prefix() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = root.path();
    let bundle = SiteBundle {
        site_dir: site_dir.to_string_lossy().into_owned(),
        site_path: site_dir.join("site.json").to_string_lossy().into_owned(),
        site: SiteConfig {
            header: HeaderConfig {
                brand: HeaderBrand {
                    logo: "Images/logo.png".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        pages: vec![],
    };

    let err = check_referenced_site_assets(&bundle).unwrap_err();
    assert!(
        err.to_string().contains("header.brand.logo"),
        "unexpected error: {err}"
    );
}

#[test]
fn copy_referenced_site_assets_rejects_escape_path() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = root.path();
    fs::create_dir_all(site_dir.join("assets")).unwrap();
    let page_path = site_dir.join("pages/home.json");
    fs::create_dir_all(page_path.parent().unwrap()).unwrap();

    let mut props = HashMap::new();
    props.insert("images".to_string(), json!([]));
    props.insert("title".to_string(), json!(""));
    props.insert("image".to_string(), json!("assets/../secrets.png"));

    let bundle = SiteBundle {
        site_dir: site_dir.to_string_lossy().into_owned(),
        site_path: site_dir.join("site.json").to_string_lossy().into_owned(),
        pages: vec![SitePageFile {
            path: page_path.to_string_lossy().into_owned(),
            page: PageConfig {
                slug: String::new(),
                widgets: vec![WidgetNode {
                    widget_type: "images_grid".into(),
                    id: String::new(),
                    enabled: None,
                    props,
                }],
                ..Default::default()
            },
            has_slug: true,
            has_widgets: true,
        }],
        site: Default::default(),
    };

    let err = copy_referenced_site_assets(&bundle, &root.path().join("out")).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("widgets[0].props.image") || msg.contains("invalid asset path"),
        "unexpected error: {err}"
    );
}

#[test]
fn copy_referenced_site_assets_copies_deduped_recursive_references() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = root.path();
    let assets_dir = site_dir.join("assets");
    write_test_asset(&assets_dir.join("cover.png"));
    write_test_asset(&assets_dir.join("icons/badge.png"));
    write_test_asset(&assets_dir.join("icons/googleplay.png"));

    let child_props = {
        let mut m = HashMap::new();
        m.insert("children".to_string(), json!([]));
        m.insert("icon_image".to_string(), json!("assets/icons/badge.png"));
        m
    };
    let root_props = {
        let mut m = HashMap::new();
        m.insert(
            "children".to_string(),
            json!([{
                "type": "column",
                "props": child_props
            }]),
        );
        m.insert("image".to_string(), json!("assets/cover.png"));
        m.insert("cards".to_string(), json!([{"image": "assets/cover.png"}]));
        m
    };

    let page_path = site_dir.join("pages/home.json");
    fs::create_dir_all(page_path.parent().unwrap()).unwrap();

    let mut store_icons = HashMap::new();
    store_icons.insert("google_play".into(), "assets/icons/googleplay.png".into());

    let bundle = SiteBundle {
        site_dir: site_dir.to_string_lossy().into_owned(),
        site_path: site_dir.join("site.json").to_string_lossy().into_owned(),
        site: SiteConfig {
            store_icons,
            ..Default::default()
        },
        pages: vec![SitePageFile {
            path: page_path.to_string_lossy().into_owned(),
            page: PageConfig {
                slug: String::new(),
                seo: PageSeo {
                    og_image: "assets/cover.png".into(),
                    ..Default::default()
                },
                widgets: vec![WidgetNode {
                    widget_type: "row".into(),
                    id: String::new(),
                    enabled: None,
                    props: root_props,
                }],
                ..Default::default()
            },
            has_slug: true,
            has_widgets: true,
        }],
    };

    let out_dir = root.path().join("out");
    copy_referenced_site_assets(&bundle, &out_dir).unwrap();

    assert!(out_dir.join("assets/cover.png").is_file());
    assert!(out_dir.join("assets/icons/badge.png").is_file());
    assert!(out_dir.join("assets/icons/googleplay.png").is_file());
}

#[test]
fn validate_site_bundle_only_fails_on_missing_asset_before_render() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = root.path().join("content/mini");
    fs::create_dir_all(site_dir.join("assets")).unwrap();
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"mini","output_folder":"Results/Mini"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}],"seo":{"og_image":"assets/missing.png"}}"#,
    );

    let template = workspace_root().join("Template");
    if !template.join("layout.html").is_file() {
        eprintln!("skip validate missing asset: Template not present");
        return;
    }

    let (bundle, _) = load_site_bundle(&site_dir).unwrap();
    let err = validate_site_bundle_only(&bundle, &template).unwrap_err();
    assert!(
        err.to_string().contains("referenced asset does not exist"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_site_bundle_only_fails_on_unknown_nav_slug() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = root.path().join("content/nav-bad");
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"t","output_folder":"Results/T","header":{"nav":[{"label":"X","href":"missing-page"}]}}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    );

    let template = workspace_root().join("Template");
    if !template.join("layout.html").is_file() {
        eprintln!("skip validate unknown slug: Template not present");
        return;
    }

    let (bundle, _) = load_site_bundle(&site_dir).unwrap();
    let err = validate_site_bundle_only(&bundle, &template).unwrap_err();
    assert!(
        err.to_string().contains("unknown internal slug"),
        "unexpected error: {err}"
    );
}
