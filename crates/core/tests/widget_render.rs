//! Port of Go `widget_render_test.go`.

mod common;

use common::workspace_root;
use portfoliowebsitebuilder_core::types::{
    ConfigWarning, SiteConfig, SubscribeBlock, SubscribeLink, WidgetNode,
};
use portfoliowebsitebuilder_core::widgets::{
    load_widget_env, render_widget_tree, WidgetRenderContext,
};
use portfoliowebsitebuilder_core::{PageRoute, RouteIndex};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

const PAGE_PATH: &str = "content/demo/pages/home.json";

fn template_dir() -> Option<std::path::PathBuf> {
    let t = workspace_root().join("Template");
    if t.join("widgets").is_dir() {
        Some(t)
    } else {
        None
    }
}

fn default_route() -> PageRoute {
    PageRoute {
        slug: String::new(),
        source_path: PAGE_PATH.into(),
        output_rel_path: "index.html".into(),
        dir_rel_path: String::new(),
    }
}

fn default_routes(route: &PageRoute) -> RouteIndex {
    let mut by_slug = HashMap::new();
    by_slug.insert(String::new(), route.clone());
    RouteIndex {
        ordered: vec![route.clone()],
        by_slug,
    }
}

fn render_widgets(
    page_path: &str,
    site: &SiteConfig,
    widgets: &[WidgetNode],
) -> portfoliowebsitebuilder_core::CoreResult<(String, Vec<ConfigWarning>)> {
    let template_dir = template_dir().expect("Template/widgets required for widget render tests");
    let env = load_widget_env(&template_dir)?;
    let route = default_route();
    let routes = default_routes(&route);
    let ctx = WidgetRenderContext {
        page_path,
        site,
        route: &route,
        routes: &routes,
        env: &env,
    };
    render_widget_tree(&ctx, widgets)
}

fn widget(widget_type: &str, props: JsonValue) -> WidgetNode {
    let props = props
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    WidgetNode {
        widget_type: widget_type.into(),
        id: String::new(),
        enabled: None,
        props,
    }
}

fn widget_disabled(widget_type: &str, props: JsonValue) -> WidgetNode {
    let mut w = widget(widget_type, props);
    w.enabled = Some(false);
    w
}

#[test]
fn render_widget_tree_unknown_type_fails_with_path() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(PAGE_PATH, &site, &[widget("unknown", json!({}))]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("widgets[0].type"),
        "expected widget path in error, got: {msg}"
    );
}

#[test]
fn render_widget_tree_rejects_columns_alias() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(PAGE_PATH, &site, &[widget("columns", json!({}))]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains(r#"unknown widget type "columns""#),
        "unexpected error: {msg}"
    );
}

#[test]
fn render_widget_tree_leaf_children_fails() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(
        PAGE_PATH,
        &site,
        &[widget(
            "intro",
            json!({
                "children": [{"type": "intro"}]
            }),
        )],
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("only layout widgets"),
        "unexpected error: {msg}"
    );
}

#[test]
fn render_widget_tree_layout_needs_children() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(PAGE_PATH, &site, &[widget("row", json!({}))]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(".props.children"), "unexpected error: {msg}");
}

#[test]
fn render_widget_tree_layout_recurses() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "row",
        json!({
            "children": [{
                "type": "column",
                "props": {
                    "children": [{
                        "type": "intro",
                        "id": "intro1",
                        "props": { "title": "About us" }
                    }]
                }
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains(r#"class="widget-row section""#),
        "expected widget-row wrapper, got: {html}"
    );
    assert!(
        html.contains(r#"class="container widget-row-inner""#),
        "expected widget-row-inner container, got: {html}"
    );
    assert!(
        html.contains(r#"class="widget-column""#),
        "expected widget-column wrapper, got: {html}"
    );
    assert!(
        html.contains(r#"class="intro section"#),
        "expected intro section, got: {html}"
    );
    assert!(
        html.contains(r#"id="intro_title""#),
        "expected intro heading id parity, got: {html}"
    );
}

#[test]
fn grid_renders_custom_min_column_width() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "grid",
        json!({
            "min_column_width": "312px",
            "children": [{
                "type": "intro",
                "props": { "title": "Hi" }
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains("312px"),
        "expected min column width in output, got: {html}"
    );
}

#[test]
fn cover_banner_requires_src() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(PAGE_PATH, &site, &[widget("cover_banner", json!({}))]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("src"),
        "expected src required error, got: {msg}"
    );
}

#[test]
fn careers_tabs_single_vacancy_no_tabs() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "careers_tabs",
        json!({
            "title": "Careers",
            "vacancies": [{
                "role": "Designer",
                "requirements": ["Portfolio"],
                "responsibilities": ["UI work"],
                "advantages": ["Remote"],
                "apply_url": "https://example.com/apply"
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        !html.contains("data-split-widget"),
        "single vacancy should not emit data-split-widget: {html}"
    );
    assert!(
        !html.contains("split-widget__tab"),
        "single vacancy should not render tab buttons: {html}"
    );
    assert!(
        html.contains("split-widget--single") && html.contains("vacancy-detail"),
        "expected single-vacancy panel markup: {html}"
    );
}

#[test]
fn careers_tabs_multi_vacancy_has_tabs() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "careers_tabs",
        json!({
            "title": "Careers",
            "vacancies": [
                {
                    "role": "Designer",
                    "requirements": ["Portfolio"],
                    "responsibilities": ["UI work"],
                    "advantages": ["Remote"],
                    "apply_url": "https://example.com/apply"
                },
                {
                    "role": "Engineer",
                    "requirements": ["Go experience"],
                    "responsibilities": ["Backend work"],
                    "advantages": ["Flexible hours"],
                    "apply_url": "https://example.com/apply2"
                }
            ]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains("data-split-widget"),
        "expected split widget marker: {html}"
    );
    assert!(
        html.contains(r#"data-target="vacancy-0""#) && html.contains(r#"id="vacancy-0""#),
        "expected vacancy tab wiring: {html}"
    );
    assert!(
        html.contains(r#"data-target="vacancy-1""#),
        "expected second vacancy tab: {html}"
    );
}

#[test]
fn apps_showcase_renders_cards_and_swiper_and_stores() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig {
        store_icons: HashMap::from([("google_play".into(), "assets/icons/googleplay.png".into())]),
        subscribe_block: SubscribeBlock {
            title: "Stay updated".into(),
            links: vec![SubscribeLink {
                label: "Telegram".into(),
                url: "https://t.me/example".into(),
            }],
        },
        ..Default::default()
    };
    let widgets = vec![widget(
        "apps_showcase",
        json!({
            "section_title": "Our apps",
            "apps": [
                {
                    "image": "assets/icons/app.png",
                    "header_image": "assets/headers/app-header.png",
                    "title": "Kometa",
                    "text_1": "First paragraph",
                    "text_2": "Second paragraph",
                    "swiper_images": ["assets/swiper/1.png"],
                    "google_play_url": "https://play.google.com/store/apps/details?id=kometa"
                },
                {
                    "image": "assets/icons/app2.png",
                    "title": "Kometa 2",
                    "store_links": [{
                        "url": "https://example.com/store",
                        "aria_label": "Open store",
                        "icon": "google_play"
                    }]
                }
            ]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains(r#"data-widget-type="apps_showcase""#),
        "expected apps_showcase section marker, got: {html}"
    );
    assert!(
        html.contains(r#"class="catalog-carousel" data-catalog-carousel"#),
        "expected catalog carousel contract markup, got: {html}"
    );
    assert!(
        html.contains(r#"class="catalog-store-btn catalog-store-btn--googleplay""#),
        "expected store badge button, got: {html}"
    );
    assert!(
        html.contains(r#"class="catalog-app-card__subscribe""#),
        "expected subscribe block, got: {html}"
    );
}

#[test]
fn apps_showcase_hides_subscribe_when_no_links() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig {
        store_icons: HashMap::from([("google_play".into(), "assets/icons/googleplay.png".into())]),
        subscribe_block: SubscribeBlock {
            title: "Subscribe for news".into(),
            links: vec![],
        },
        ..Default::default()
    };
    let widgets = vec![widget(
        "apps_showcase",
        json!({
            "apps": [{
                "image": "assets/icons/app.png",
                "swiper_images": ["assets/swiper/1.png"],
                "google_play_url": "https://play.google.com/store/apps/details?id=example"
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        !html.contains(r#"class="catalog-app-card__subscribe""#),
        "expected no subscribe block when links empty, got: {html}"
    );
}

#[test]
fn apps_showcase_empty_store_url_warns_and_skips() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let page_path = "content/kometa/pages/home.json";
    let site = SiteConfig {
        store_icons: HashMap::from([
            ("google_play".into(), "assets/gp-store-icon.png".into()),
            ("app_store".into(), "assets/appstore-store-icon.png".into()),
            ("amazon".into(), "assets/amazon-store-icon.png".into()),
            ("galaxy".into(), "assets/galaxy-store-icon.png".into()),
        ]),
        ..Default::default()
    };
    let widgets = vec![widget(
        "apps_showcase",
        json!({
            "apps": [{
                "image": "assets/icon.png",
                "swiper_images": ["assets/slide.png"],
                "google_play_url": "https://play.google.com/store/apps/details?id=one",
                "amazon_store_url": "",
                "galaxy_store_url": ""
            }]
        }),
    )];
    let (html, warnings) = render_widgets(page_path, &site, &widgets).unwrap();
    assert!(
        !html.contains("catalog-store-btn--amazon") && !html.contains("catalog-store-btn--galaxy"),
        "expected no amazon/galaxy badges for empty URLs, got: {html}"
    );
    assert_eq!(
        warnings.len(),
        2,
        "expected 2 store URL warnings, got {}: {:?}",
        warnings.len(),
        warnings
    );
    for w in &warnings {
        assert!(
            w.to_string().contains("referenced but URL is empty"),
            "unexpected warning: {}",
            w.to_string()
        );
    }
}

#[test]
fn apps_showcase_requires_apps() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(PAGE_PATH, &site, &[widget("apps_showcase", json!({}))]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("widgets[0].props.apps"),
        "expected path-aware apps error, got: {msg}"
    );
}

#[test]
fn apps_showcase_app_image_required_path() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let err = render_widgets(
        PAGE_PATH,
        &site,
        &[widget(
            "apps_showcase",
            json!({
                "apps": [{ "title": "No image" }]
            }),
        )],
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("widgets[0].props.apps[0].image"),
        "expected path-aware image error, got: {msg}"
    );
}

#[test]
fn render_widget_tree_skips_disabled_widgets() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![
        widget("intro", json!({ "title": "Shown" })),
        widget_disabled("intro", json!({ "title": "Hidden" })),
    ];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(html.contains("Shown"), "expected enabled intro: {html}");
    assert!(
        !html.contains("Hidden"),
        "disabled widget should not render: {html}"
    );
}

#[test]
fn render_widget_tree_recognizes_media_swiper() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "media_swiper",
        json!({
            "images": [{ "src": "assets/pic1.png" }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains(r#"data-widget-type="media_swiper""#),
        "expected media_swiper marker: {html}"
    );
    assert!(
        html.contains("data-catalog-carousel")
            && html.contains(r#"class="catalog-carousel__slide"#),
        "expected catalog-carousel-compatible markup: {html}"
    );
}

#[test]
fn project_grid_renders_cards() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "project_grid",
        json!({
            "heading": "Projects",
            "subheading": "Selected work",
            "section_id": "portfolio",
            "min_card_column_width": "280px",
            "cards": [{
                "title": "Alpha",
                "description": "Short description.",
                "tags": ["Go", "Web"],
                "image": "assets/pic1.png",
                "meta": { "year": "2024", "platform": "Web" },
                "cta": { "label": "Open", "url": "https://example.com/p" }
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    for needle in [
        r#"data-widget-type="project_grid""#,
        r#"id="portfolio""#,
        "--project-grid-min:280px",
        "project-card__tags",
        "<dt>platform</dt>",
        "<dt>year</dt>",
        "project-card__cta",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in output, got: {html}"
        );
    }
}

#[test]
fn project_grid_optional_image() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "project_grid",
        json!({
            "cards": [{
                "title": "Text only",
                "description": "No thumbnail.",
                "tags": ["Go"],
                "meta": "Sample",
                "cta": { "url": "https://example.com/t" }
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        !html.contains("project-card__media"),
        "expected no media block: {html}"
    );
    assert!(
        html.contains("project-card--no-media"),
        "expected no-media modifier: {html}"
    );
}

#[test]
fn project_grid_meta_string() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "project_grid",
        json!({
            "cards": [{
                "title": "Beta",
                "description": "Desc",
                "tags": [],
                "image": "assets/pic2.png",
                "meta": "Highlighted release",
                "cta": { "url": "#intro_title" }
            }]
        }),
    )];
    let (html, _) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert!(
        html.contains("Highlighted release"),
        "expected meta line: {html}"
    );
}

#[test]
fn images_grid_warns_on_generic_alt_fallback() {
    let Some(_) = template_dir() else {
        eprintln!("skip: Template not present");
        return;
    };
    let site = SiteConfig::default();
    let widgets = vec![widget(
        "images_grid",
        json!({
            "images": ["assets/pic1.png"]
        }),
    )];
    let (_, warnings) = render_widgets(PAGE_PATH, &site, &widgets).unwrap();
    assert_eq!(
        warnings.len(),
        1,
        "expected 1 warning, got {}: {:?}",
        warnings.len(),
        warnings
    );
    assert!(
        warnings[0]
            .to_string()
            .contains("missing descriptive alt text"),
        "unexpected warning: {}",
        warnings[0].to_string()
    );
}
