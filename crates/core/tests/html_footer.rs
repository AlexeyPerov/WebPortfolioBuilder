//! Port of Go `html_test.go` (footer HTML helpers).

use portfoliowebsitebuilder_core::types::{FooterConfig, FooterContact};
use portfoliowebsitebuilder_core::{build_footer_legal_row, build_footer_outer_html};

#[test]
fn build_footer_outer_html_no_scroll_reveal() {
    let html = build_footer_outer_html(&FooterConfig {
        enabled: Some(true),
        section_title: "Contacts".into(),
        contact: FooterContact {
            email: "hello@example.com".into(),
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(!html.is_empty(), "expected footer HTML");
    assert!(
        !html.contains("scroll-reveal"),
        "footer must not use scroll-reveal, got: {html}"
    );
    assert!(html.contains(r#"id="footer""#));
    assert!(html.contains(r#"id="footer-contact""#));
}

#[test]
fn build_footer_legal_row_omits_empty_terms_url() {
    let row = build_footer_legal_row(&FooterConfig {
        privacy_url: "https://example.com/privacy".into(),
        privacy_label: "Privacy Policy".into(),
        terms_url: String::new(),
        terms_label: "Terms of Service".into(),
        ..Default::default()
    });
    assert!(row.contains("Privacy Policy"));
    assert!(
        !row.contains("Terms of Service"),
        "expected no terms link when URL empty, got: {row}"
    );
}
