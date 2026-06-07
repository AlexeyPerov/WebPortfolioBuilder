//! Port of Go `widget_scripts_test.go`.

use portfoliowebsitebuilder_core::types::WidgetNode;
use portfoliowebsitebuilder_core::widgets::collect_page_script_needs;
use serde_json::json;

fn widget(widget_type: &str, props: serde_json::Value) -> WidgetNode {
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

#[test]
fn collect_page_script_needs_intro_only() {
    let needs = collect_page_script_needs(&[widget("intro", json!({}))]);
    assert!(needs.scroll_reveal, "intro should require scroll-reveal.js");
    assert!(
        !needs.catalog_carousel
            && !needs.split_widget
            && !needs.reference_panel
            && !needs.image_lightbox,
        "intro-only page should not need carousel/split/reference/lightbox scripts: {needs:?}"
    );
}

#[test]
fn collect_page_script_needs_reference_panel() {
    let needs = collect_page_script_needs(&[widget(
        "reference_panel",
        json!({
            "entries": [{
                "label": "RunAll",
                "description": "Runs all categories."
            }]
        }),
    )]);
    assert!(needs.reference_panel, "reference_panel should require reference-panel.js");
    assert!(needs.scroll_reveal);
}

#[test]
fn collect_page_script_needs_kometa_home_mix() {
    let needs = collect_page_script_needs(&[
        widget("intro", json!({})),
        widget("apps_showcase", json!({})),
        widget("images_grid", json!({})),
        widget("careers_tabs", json!({})),
    ]);
    assert!(
        needs.scroll_reveal && needs.catalog_carousel && needs.split_widget && needs.image_lightbox,
        "expected all interactive scripts for Kometa-like home: {needs:?}"
    );
    assert!(
        needs.needs_widgets_config(),
        "Kometa-like home should inject site-widgets-config"
    );
}

#[test]
fn collect_page_script_needs_nested_layout() {
    let needs = collect_page_script_needs(&[widget(
        "grid",
        json!({
            "children": [{"type": "media_swiper"}]
        }),
    )]);
    assert!(
        needs.catalog_carousel,
        "nested media_swiper should require catalog-carousel.js"
    );
}

#[test]
fn collect_page_script_needs_skips_disabled_widgets() {
    let mut disabled = widget("apps_showcase", json!({}));
    disabled.enabled = Some(false);
    let needs = collect_page_script_needs(&[widget("intro", json!({})), disabled]);
    assert!(needs.scroll_reveal);
    assert!(
        !needs.catalog_carousel,
        "disabled apps_showcase should not require catalog-carousel.js"
    );
}
