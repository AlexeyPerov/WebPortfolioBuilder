use crate::routing::is_external_or_special_href;
use crate::theme::enrich_theme_with_social_defaults;
use crate::types::{
    CatalogApp, FooterConfig, FooterContact, StoreLink, SubscribeBlock, WidgetsConfig,
};
use serde_json::json;
use std::collections::HashMap;

const DEFAULT_GOOGLE_FONTS_HREF: &str =
    "https://fonts.googleapis.com/css?family=Quicksand:700|Roboto:400,400i,700&display=swap";

pub fn normalized_typography(t: &crate::types::TypographyConfig) -> (String, String, String) {
    let href = t.google_fonts_stylesheet_href.trim();
    let href = if href.is_empty() {
        DEFAULT_GOOGLE_FONTS_HREF
    } else {
        href
    };
    let heading = t.font_family_heading.trim();
    let heading = if heading.is_empty() {
        r#""Quicksand", sans-serif"#
    } else {
        heading
    };
    let body = t.font_family_body.trim();
    let body = if body.is_empty() {
        r#""Roboto", sans-serif"#
    } else {
        body
    };
    (href.to_string(), heading.to_string(), body.to_string())
}

pub fn build_footer_outer_html(f: &FooterConfig) -> String {
    if !f.is_enabled() {
        return String::new();
    }
    let inner = build_footer_section(f);
    format!(
        r#"<footer class="footer section section-gradient" id="footer"><div class="container">{inner}</div></footer>"#
    )
}

pub fn build_widgets_config_script(w: &WidgetsConfig) -> String {
    #[derive(serde::Serialize)]
    struct ScrollRevealExport {
        respect_reduced_motion: bool,
        root_margin: String,
        threshold: f64,
    }
    #[derive(serde::Serialize)]
    struct CarouselExport {
        swipe_threshold_px: i64,
        keyboard_navigation: bool,
    }
    #[derive(serde::Serialize)]
    struct SplitWidgetExport {
        keyboard_navigation: bool,
    }
    #[derive(serde::Serialize)]
    struct WidgetsExport {
        scroll_reveal: ScrollRevealExport,
        carousel: CarouselExport,
        split_widget: SplitWidgetExport,
    }

    let root_margin = if w.scroll_reveal.root_margin.trim().is_empty() {
        "0px 0px -5% 0px".to_string()
    } else {
        w.scroll_reveal.root_margin.trim().to_string()
    };

    let export = WidgetsExport {
        scroll_reveal: ScrollRevealExport {
            respect_reduced_motion: w.scroll_reveal.respect_reduced_motion.unwrap_or(true),
            root_margin,
            threshold: w.scroll_reveal.threshold.unwrap_or(0.12),
        },
        carousel: CarouselExport {
            swipe_threshold_px: w
                .carousel
                .swipe_threshold_px
                .filter(|&v| v > 0)
                .unwrap_or(30),
            keyboard_navigation: w.carousel.keyboard_navigation.unwrap_or(true),
        },
        split_widget: SplitWidgetExport {
            keyboard_navigation: w.split_widget.keyboard_navigation.unwrap_or(false),
        },
    };

    let mut safe = serde_json::to_string(&export).unwrap_or_else(|_| "{}".into());
    safe = safe.replace('<', "\\u003c");
    format!(r#"<script type="application/json" id="site-widgets-config">{safe}</script>"#)
}

pub fn catalog_stat_line_or(value: &str, fallback: &str) -> String {
    if !value.trim().is_empty() {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

pub fn external_link_attrs(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        r#" target="_blank" rel="noopener noreferrer""#.into()
    } else {
        String::new()
    }
}

pub fn build_footer_section(f: &FooterConfig) -> String {
    let contact_row = build_footer_contact_row(&f.contact);
    let legal_row = build_footer_legal_row(f);
    if f.section_title.trim().is_empty()
        && contact_row.is_empty()
        && legal_row.is_empty()
        && f.copyright.trim().is_empty()
        && f.cookie_notice.trim().is_empty()
    {
        return String::new();
    }
    let mut b = String::new();
    if !f.section_title.trim().is_empty() {
        b.push_str(&format!(
            r#"<h2 class="center footer-heading">{}</h2>"#,
            html_escape(&f.section_title)
        ));
    }
    b.push_str(&contact_row);
    b.push_str(&legal_row);
    if !f.copyright.trim().is_empty() {
        b.push_str(&format!(
            r#"<p class="footer-copyright">{}</p>"#,
            html_escape(&f.copyright)
        ));
    }
    if !f.cookie_notice.trim().is_empty() {
        b.push_str(&format!(
            r#"<p class="footer-cookie">{}</p>"#,
            html_escape(&f.cookie_notice)
        ));
    }
    b
}

fn build_footer_contact_row(c: &FooterContact) -> String {
    let text_parts: Vec<String> = c
        .paragraphs
        .iter()
        .filter_map(|p| {
            let p = p.trim();
            if p.is_empty() {
                None
            } else {
                Some(html_escape(p))
            }
        })
        .collect();
    let text_joined = text_parts.join(" ");
    let email = c.email.trim();
    if text_joined.is_empty() && email.is_empty() {
        return String::new();
    }
    let mut b = String::from(r#"<div class="footer-contact-row" id="footer-contact">"#);
    b.push_str(r#"<div class="footer-contact-row__text">"#);
    b.push_str(&text_joined);
    b.push_str("</div>");
    if !email.is_empty() {
        b.push_str(&format!(
            r#"<a class="footer-contact-row__email" href="mailto:{e}">{e}</a>"#,
            e = html_escape(email)
        ));
    }
    b.push_str("</div>");
    b
}

pub fn build_footer_legal_row(f: &FooterConfig) -> String {
    let privacy_url = f.privacy_url.trim();
    let terms_url = f.terms_url.trim();
    if privacy_url.is_empty() && terms_url.is_empty() {
        return String::new();
    }
    let privacy_label = if f.privacy_label.trim().is_empty() {
        "Privacy Policy"
    } else {
        f.privacy_label.trim()
    };
    let terms_label = if f.terms_label.trim().is_empty() {
        "Terms of Service"
    } else {
        f.terms_label.trim()
    };
    let mut b = String::from(r#"<div class="footer-legal-links">"#);
    if !privacy_url.is_empty() {
        b.push_str(&format!(
            r#"<a class="footer-legal-links__left" href="{u}"{attrs}>{l}</a>"#,
            u = html_escape(privacy_url),
            attrs = external_link_attrs(privacy_url),
            l = html_escape(privacy_label)
        ));
    }
    if !terms_url.is_empty() {
        b.push_str(&format!(
            r#"<a class="footer-legal-links__right" href="{u}"{attrs}>{l}</a>"#,
            u = html_escape(terms_url),
            attrs = external_link_attrs(terms_url),
            l = html_escape(terms_label)
        ));
    }
    b.push_str("</div>");
    b
}

pub fn build_list(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    items
        .iter()
        .map(|item| format!("<li>{}</li>", html_escape(item)))
        .collect()
}

const SVG_GITHUB: &str = r#"<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/></svg>"#;

const SVG_LINKEDIN: &str = r#"<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>"#;

const SVG_FACEBOOK: &str = r#"<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M24 12.073C24 5.446 18.627.073 12 .073S0 5.446 0 12.073c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/></svg>"#;

pub fn social_icon_preset_class(icon: &str) -> &'static str {
    match icon.trim().to_lowercase().as_str() {
        "github" => "follow-us__btn--github",
        "linkedin" => "follow-us__btn--linkedin",
        "facebook" => "follow-us__btn--facebook",
        _ => "",
    }
}

pub fn social_icon_preset_svg(icon: &str) -> &'static str {
    match icon.trim().to_lowercase().as_str() {
        "github" => SVG_GITHUB,
        "linkedin" => SVG_LINKEDIN,
        "facebook" => SVG_FACEBOOK,
        _ => "",
    }
}

pub fn aria_default_for_social_preset(icon: &str) -> &'static str {
    match icon.trim().to_lowercase().as_str() {
        "github" => "GitHub",
        "linkedin" => "LinkedIn",
        "facebook" => "Facebook",
        _ => "Link",
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Text-node escaping matching Go `html/template` (includes `+` → `&#43;`).
pub fn go_template_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '+' => out.push_str("&#43;"),
            _ => out.push(c),
        }
    }
    out
}

/// Matches Go `html/template` URL escaping for stylesheet `href` attributes.
pub fn escape_stylesheet_href(url: &str) -> String {
    url.replace('|', "%7c").replace('&', "&amp;")
}

/// Configure Minijinja environments for Go `html/template`-compatible escaping.
pub fn configure_minijinja_html_escape(env: &mut minijinja::Environment) {
    use minijinja::{escape_formatter, AutoEscape};

    env.set_auto_escape_callback(|name| {
        if name == "layout" || name.starts_with("widgets/") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });

    env.set_formatter(|out, state, value| {
        if state.auto_escape() == AutoEscape::Html && !value.is_safe() {
            let owned;
            let s = if let Some(s) = value.as_str() {
                s
            } else {
                owned = value.to_string();
                owned.as_str()
            };
            return out
                .write_str(&go_template_text_escape(s))
                .map_err(minijinja::Error::from);
        }
        escape_formatter(out, state, value)
    });
}

pub struct ResolvedStoreEntry {
    pub url: String,
    pub aria_label: String,
    pub icon_src: String,
    pub class_suffix: String,
}

pub fn catalog_store_url_warnings(
    page_path: &str,
    app_path: &str,
    app_raw: &serde_json::Value,
) -> Vec<crate::types::ConfigWarning> {
    use crate::types::ConfigWarning;
    let obj = match app_raw.as_object() {
        Some(o) => o,
        None => return vec![],
    };
    let mut warnings = Vec::new();
    for (json_key, preset_key) in [
        ("google_play_url", "google_play"),
        ("app_store_url", "app_store"),
        ("amazon_store_url", "amazon"),
        ("galaxy_store_url", "galaxy"),
    ] {
        if let Some(raw) = obj.get(json_key) {
            if raw.as_str().is_some_and(|s| s.trim().is_empty()) {
                warnings.push(ConfigWarning::content(
                    page_path,
                    format!(
                        "{app_path}.{json_key}: store preset {preset_key:?} referenced but URL is empty"
                    ),
                ));
            }
        }
    }
    if let Some(raw_links) = obj.get("store_links") {
        if let Ok(links) = serde_json::from_value::<Vec<StoreLink>>(raw_links.clone()) {
            for (i, link) in links.iter().enumerate() {
                if !link.url.trim().is_empty() {
                    continue;
                }
                let icon_key = normalize_store_icon_key(&link.icon);
                let icon_image = link.icon_image.trim();
                if icon_key.is_empty() && icon_image.is_empty() {
                    continue;
                }
                let preset = if icon_key.is_empty() {
                    "custom"
                } else {
                    &icon_key
                };
                warnings.push(ConfigWarning::content(
                    page_path,
                    format!(
                        "{app_path}.store_links[{i}]: store preset {preset:?} referenced but URL is empty"
                    ),
                ));
            }
        }
    }
    warnings
}

pub fn resolve_catalog_store_entries(
    app: &CatalogApp,
    icons: &HashMap<String, String>,
) -> Vec<ResolvedStoreEntry> {
    if !app.store_links.is_empty() {
        let mut out = Vec::new();
        for link in &app.store_links {
            let u = link.url.trim();
            if u.is_empty() {
                continue;
            }
            let img_path = link.icon_image.trim();
            let icon_key = normalize_store_icon_key(&link.icon);
            let icon_src = if !img_path.is_empty() {
                img_path.to_string()
            } else if !icon_key.is_empty() {
                icons
                    .get(&icon_key)
                    .cloned()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            } else {
                String::new()
            };
            if icon_src.is_empty() {
                continue;
            }
            let mut aria = link.aria_label.trim().to_string();
            if aria.is_empty() {
                aria = default_aria_for_store_icon_key(&icon_key)
                    .unwrap_or("Store link")
                    .to_string();
            }
            out.push(ResolvedStoreEntry {
                url: u.to_string(),
                aria_label: aria,
                icon_src,
                class_suffix: store_class_suffix_from_link(&icon_key, !img_path.is_empty()),
            });
        }
        return out;
    }
    let slots = [
        (
            &app.google_play_url,
            "google_play",
            "googleplay",
            "Get it on Google Play",
        ),
        (
            &app.app_store_url,
            "app_store",
            "appstore",
            "Download on the App Store",
        ),
        (
            &app.amazon_store_url,
            "amazon",
            "amazon",
            "Available at Amazon Appstore",
        ),
        (
            &app.galaxy_store_url,
            "galaxy",
            "galaxy",
            "Available on Galaxy Store",
        ),
    ];
    let mut out = Vec::new();
    for (url, icon_key, class_suffix, def_aria) in slots {
        let u = url.trim();
        if u.is_empty() {
            continue;
        }
        let icon_src = icons
            .get(icon_key)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if icon_src.is_empty() {
            continue;
        }
        out.push(ResolvedStoreEntry {
            url: u.to_string(),
            aria_label: def_aria.to_string(),
            icon_src,
            class_suffix: class_suffix.to_string(),
        });
    }
    out
}

fn normalize_store_icon_key(icon: &str) -> String {
    icon.trim().to_lowercase().replace(' ', "_")
}

fn store_class_suffix_from_link(icon_key: &str, has_custom_image: bool) -> String {
    match icon_key {
        "google_play" => "googleplay".into(),
        "app_store" => "appstore".into(),
        "amazon" => "amazon".into(),
        "galaxy" => "galaxy".into(),
        "" if has_custom_image => "custom".into(),
        "" => "custom".into(),
        other => sanitize_icon_class_key(other),
    }
}

fn sanitize_icon_class_key(s: &str) -> String {
    let mut out = String::new();
    for c in s.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c == '_' || c == '-' {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "custom".into()
    } else {
        out
    }
}

fn default_aria_for_store_icon_key(icon_key: &str) -> Option<&'static str> {
    match icon_key {
        "google_play" => Some("Get it on Google Play"),
        "app_store" => Some("Download on the App Store"),
        "amazon" => Some("Available at Amazon Appstore"),
        "galaxy" => Some("Available on Galaxy Store"),
        _ => None,
    }
}

pub fn build_apps_showcase_subscribe_data(s: &SubscribeBlock) -> Option<serde_json::Value> {
    let links: Vec<_> = s
        .links
        .iter()
        .filter_map(|link| {
            let u = link.url.trim();
            if u.is_empty() {
                return None;
            }
            let label = if link.label.trim().is_empty() {
                u.to_string()
            } else {
                link.label.trim().to_string()
            };
            Some(json!({
                "URL": u,
                "Attrs": external_link_attrs(u),
                "Label": label,
            }))
        })
        .collect();
    if links.is_empty() {
        return None;
    }
    let title = if s.title.trim().is_empty() {
        "Subscribe for news"
    } else {
        s.title.trim()
    };
    Some(json!({ "Title": title, "Links": links }))
}

pub fn build_careers_split_widget_html(entries: &[(String, String)]) -> String {
    build_careers_split_widget("Open positions", "vacancy", entries)
}

fn build_careers_split_widget(
    aria_label: &str,
    id_prefix: &str,
    entries: &[(String, String)],
) -> String {
    if entries.is_empty() {
        return String::new();
    }
    if entries.len() == 1 {
        return format!(
            r#"<div class="split-widget split-widget--single"><div class="split-widget__panels"><div class="split-widget__panel is-active">{}</div></div></div>"#,
            entries[0].1
        );
    }
    let mut list = String::new();
    let mut panels = String::new();
    for (i, (label, body)) in entries.iter().enumerate() {
        let id = format!("{id_prefix}-{i}");
        let tab_class = if i == 0 {
            "split-widget__tab is-active"
        } else {
            "split-widget__tab"
        };
        let panel_class = if i == 0 {
            "split-widget__panel is-active"
        } else {
            "split-widget__panel"
        };
        let aria_sel = if i == 0 { "true" } else { "false" };
        list.push_str(&format!(
            r#"<li><button type="button" class="{tab_class}" data-target="{id}" role="tab" aria-selected="{aria_sel}">{}</button></li>"#,
            html_escape(label)
        ));
        panels.push_str(&format!(
            r#"<div id="{id}" class="{panel_class}" role="tabpanel">{body}</div>"#
        ));
    }
    format!(
        r#"<div class="split-widget" data-split-widget aria-label="{al}"><div class="split-widget__layout"><nav class="split-widget__nav" aria-label="{al}"><ul class="split-widget__list">{list}</ul></nav><div class="split-widget__panels">{panels}</div></div></div>"#,
        al = html_escape(aria_label)
    )
}

pub fn render_vacancy_panel_html(
    v: &crate::types::Vacancy,
    req_title: &str,
    resp_title: &str,
    adv_title: &str,
) -> String {
    let mut b = String::from(r#"<div class="vacancy-detail">"#);
    b.push_str(&format!("<h3>{}</h3>", html_escape(&v.role)));
    b.push_str(&format!(
        "<h4>{req_title}</h4><ul>{}</ul>",
        build_list(&v.requirements)
    ));
    b.push_str(&format!(
        "<h4>{resp_title}</h4><ul>{}</ul>",
        build_list(&v.responsibilities)
    ));
    b.push_str(&format!(
        "<h4>{adv_title}</h4><ul>{}</ul>",
        build_list(&v.advantages)
    ));
    if !v.apply_url.trim().is_empty() {
        let label = if v.apply_label.trim().is_empty() {
            "Apply for job"
        } else {
            v.apply_label.trim()
        };
        b.push_str(&format!(
            r#"<p class="vacancy-apply"><a class="vacancy-apply-btn" href="{}">{}</a></p>"#,
            html_escape(v.apply_url.trim()),
            html_escape(label)
        ));
    }
    b.push_str("</div>");
    b
}

pub fn build_theme_css_variables(theme: &HashMap<String, String>) -> String {
    let theme = enrich_theme_with_social_defaults(theme);
    let mut keys: Vec<_> = theme.keys().cloned().collect();
    keys.sort();
    let mut lines = Vec::new();
    for key in keys {
        let val = theme[&key].trim();
        if val.is_empty() {
            continue;
        }
        let css_key = key.replace('_', "-");
        lines.push(format!("      --{css_key}: {val};"));
    }
    lines.join("\n")
}

pub fn resolved_theme_color(theme: &HashMap<String, String>) -> String {
    if let Some(accent) = theme
        .get("accent")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return accent.to_string();
    }
    theme
        .get("page_bg")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn resolved_twitter_card(open_graph_image: &str) -> String {
    if open_graph_image.trim().is_empty() {
        "summary".into()
    } else {
        "summary_large_image".into()
    }
}

pub fn resolved_canonical_url(base_url: &str, slug: &str, explicit: &str) -> String {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        return explicit.to_string();
    }
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return String::new();
    }
    if slug.is_empty() {
        format!("{base}/")
    } else {
        format!("{base}/{slug}/")
    }
}

pub fn resolved_open_graph_image(base_url: &str, raw_image: &str) -> String {
    let raw_image = raw_image.trim();
    if raw_image.is_empty() {
        return String::new();
    }
    if is_external_or_special_href(raw_image) || raw_image.starts_with('/') {
        return raw_image.to_string();
    }
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        raw_image.to_string()
    } else {
        format!("{base}/{}", raw_image.trim_start_matches("./"))
    }
}

pub const HTML_TEMPLATE_FAILURE_MARKER: &str = "ZgotmplZ";
