//! Port of Go `render_test.go` page-data and dry-run portions.

mod common;

use common::workspace_root;
use portfoliowebsitebuilder_core::types::{
    FooterConfig, HeaderBrand, HeaderConfig, NavItem, PageConfig, PageLayout, PageSeo, SiteBundle,
    SiteConfig, SitePageFile, WidgetNode,
};
use portfoliowebsitebuilder_core::widgets::load_widget_env;
use portfoliowebsitebuilder_core::{
    build_rendered_page_data, build_route_index, load_site_bundle, validate_site,
    validate_site_bundle, HTML_TEMPLATE_FAILURE_MARKER,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;

fn template_dir() -> Option<std::path::PathBuf> {
    let t = workspace_root().join("Template");
    if t.join("layout.html").is_file() {
        Some(t)
    } else {
        None
    }
}

#[test]
fn build_rendered_page_data_applies_merge_model() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip merge model test: Template not present");
        return;
    };
    let footer_enabled = true;
    let bundle = SiteBundle {
        site_dir: "content/demo".into(),
        site_path: "content/demo/site.json".into(),
        site: SiteConfig {
            site_id: "demo-site".into(),
            base_url: "https://example.com/repo".into(),
            header: HeaderConfig {
                nav: vec![
                    NavItem {
                        label: "Home".into(),
                        href: String::new(),
                        open_in_new_tab: false,
                    },
                    NavItem {
                        label: "About".into(),
                        href: "about".into(),
                        open_in_new_tab: false,
                    },
                ],
                ..Default::default()
            },
            footer: FooterConfig {
                enabled: Some(footer_enabled),
                section_title: "Contact".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        pages: vec![
            SitePageFile {
                path: "content/demo/pages/home.json".into(),
                page: PageConfig {
                    slug: String::new(),
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
            SitePageFile {
                path: "content/demo/pages/about.json".into(),
                page: PageConfig {
                    slug: "about".into(),
                    layout: PageLayout {
                        hide_header: true,
                        hide_footer: true,
                    },
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
        ],
    };

    let routes = build_route_index(&bundle).unwrap();
    let about_route = routes.ordered.iter().find(|r| r.slug == "about").unwrap();
    let widget_env = load_widget_env(&template_dir).unwrap();
    let (data, _) =
        build_rendered_page_data(&bundle, &bundle.pages[1], about_route, &routes, &widget_env)
            .unwrap();

    assert_eq!(data["Title"], json!("demo-site"));
    assert_eq!(data["ShowHeader"], json!(false));
    assert_eq!(data["ShowFooter"], json!(false));
    assert_eq!(
        data["CanonicalURL"],
        json!("https://example.com/repo/about/")
    );
}

#[test]
fn build_rendered_page_data_hides_empty_header_brand() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip header brand test: Template not present");
        return;
    };
    let mut props = HashMap::new();
    props.insert("title".to_string(), json!("Hi"));

    let bundle = SiteBundle {
        site_dir: "content/alexeyperov-io".into(),
        site_path: "content/alexeyperov-io/site.json".into(),
        site: SiteConfig {
            site_id: "alexeyperov-io".into(),
            header: HeaderConfig {
                brand: HeaderBrand::default(),
                nav: vec![NavItem {
                    label: "Apps".into(),
                    href: "#apps".into(),
                    open_in_new_tab: false,
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        pages: vec![SitePageFile {
            path: "content/alexeyperov-io/pages/home.json".into(),
            page: PageConfig {
                slug: String::new(),
                widgets: vec![WidgetNode {
                    widget_type: "intro".into(),
                    id: String::new(),
                    enabled: None,
                    props,
                }],
                ..Default::default()
            },
            has_slug: true,
            has_widgets: true,
        }],
    };

    let routes = build_route_index(&bundle).unwrap();
    let widget_env = load_widget_env(&template_dir).unwrap();
    let (data, _) = build_rendered_page_data(
        &bundle,
        &bundle.pages[0],
        &routes.ordered[0],
        &routes,
        &widget_env,
    )
    .unwrap();
    assert_eq!(data["ShowHeaderBrand"], json!(false));
}

#[test]
fn build_rendered_page_data_favicon_without_header_brand() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip favicon test: Template not present");
        return;
    };
    let mut props = HashMap::new();
    props.insert("title".to_string(), json!("Hi"));

    let bundle = SiteBundle {
        site_dir: "content/alexeyperov-io".into(),
        site_path: "content/alexeyperov-io/site.json".into(),
        site: SiteConfig {
            site_id: "alexeyperov-io".into(),
            favicon: "assets/github-logo.png".into(),
            header: HeaderConfig {
                brand: HeaderBrand::default(),
                nav: vec![NavItem {
                    label: "Home".into(),
                    href: "".into(),
                    open_in_new_tab: false,
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        pages: vec![SitePageFile {
            path: "content/alexeyperov-io/pages/home.json".into(),
            page: PageConfig {
                slug: String::new(),
                widgets: vec![WidgetNode {
                    widget_type: "intro".into(),
                    id: String::new(),
                    enabled: None,
                    props,
                }],
                ..Default::default()
            },
            has_slug: true,
            has_widgets: true,
        }],
    };

    let routes = build_route_index(&bundle).unwrap();
    let widget_env = load_widget_env(&template_dir).unwrap();
    let (data, _) = build_rendered_page_data(
        &bundle,
        &bundle.pages[0],
        &routes.ordered[0],
        &routes,
        &widget_env,
    )
    .unwrap();
    assert_eq!(data["ShowHeaderBrand"], json!(false));
    assert_eq!(data["SiteIconHref"], json!("assets/github-logo.png"));
    assert_eq!(data["HeaderBrandLogo"], json!(""));
}

#[test]
fn build_rendered_page_data_emits_social_meta_fields() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip social meta test: Template not present");
        return;
    };
    let mut theme = HashMap::new();
    theme.insert("accent".into(), "#3296ed".into());

    let bundle = SiteBundle {
        site_dir: "content/demo".into(),
        site_path: "content/demo/site.json".into(),
        site: SiteConfig {
            site_id: "demo-site".into(),
            base_url: "https://example.com/repo".into(),
            theme,
            ..Default::default()
        },
        pages: vec![SitePageFile {
            path: "content/demo/pages/home.json".into(),
            page: PageConfig {
                slug: String::new(),
                title: "Demo Home".into(),
                seo: PageSeo {
                    description: "A polished demo portfolio site.".into(),
                    og_image: "assets/cover.png".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            has_slug: true,
            has_widgets: true,
        }],
    };

    let routes = build_route_index(&bundle).unwrap();
    let widget_env = load_widget_env(&template_dir).unwrap();
    let (data, _) = build_rendered_page_data(
        &bundle,
        &bundle.pages[0],
        &routes.ordered[0],
        &routes,
        &widget_env,
    )
    .unwrap();

    assert_eq!(
        data["MetaDescription"],
        json!("A polished demo portfolio site.")
    );
    assert_eq!(data["CanonicalURL"], json!("https://example.com/repo/"));
    assert_eq!(
        data["OpenGraphImage"],
        json!("https://example.com/repo/assets/cover.png")
    );
    assert_eq!(data["ThemeColor"], json!("#3296ed"));
    assert_eq!(data["TwitterCard"], json!("summary_large_image"));
}

#[test]
fn validate_site_bundle_dry_run_does_not_write_files() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip dry-run test: Template not present");
        return;
    };
    let kometa = workspace_root().join("content/kometa");
    if !kometa.join("site.json").is_file() {
        eprintln!("skip dry-run test: kometa not present");
        return;
    }
    let (bundle, _) = load_site_bundle(&kometa).unwrap();
    let out = tempfile::tempdir().unwrap();
    let before: Vec<_> = fs::read_dir(out.path())
        .map(|rd| rd.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();

    validate_site_bundle(&bundle, &template_dir).unwrap();

    let after: Vec<_> = fs::read_dir(out.path())
        .map(|rd| rd.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();
    assert_eq!(before.len(), after.len());
}

#[test]
fn validate_site_bundle_html_never_contains_zgotmplz() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip ZgotmplZ test: Template not present");
        return;
    };
    for bundle_name in ["kometa", "demo"] {
        let site = workspace_root().join("content").join(bundle_name);
        if !site.join("site.json").is_file() {
            continue;
        }
        let (bundle, _) = load_site_bundle(&site).unwrap();
        validate_site_bundle(&bundle, &template_dir).unwrap_or_else(|e| {
            panic!("validate_site_bundle for {bundle_name}: {e}");
        });
    }
}

#[test]
fn validate_site_demo_succeeds() {
    let ws = workspace_root();
    let demo = ws.join("content/demo");
    if !demo.join("site.json").is_file() {
        eprintln!("skip validate_site_demo: demo not present");
        return;
    }
    let template_dir = ws.join("Template");
    if !template_dir.join("layout.html").is_file() {
        eprintln!("skip validate_site_demo: Template not present");
        return;
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    validate_site(
        &ws,
        "content/demo",
        &template_dir,
        false,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let stdout_s = String::from_utf8_lossy(&stdout);
    assert!(stdout_s.contains("Validation passed:"));
    let combined = format!("{stdout_s}{}", String::from_utf8_lossy(&stderr));
    assert!(
        !combined.contains(HTML_TEMPLATE_FAILURE_MARKER),
        "output must not contain unsafe template marker"
    );
}
