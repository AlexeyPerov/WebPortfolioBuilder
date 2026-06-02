//! Theme token enrichment for generated CSS (social / follow-us contrast on dark sites).

use std::collections::HashMap;

/// Light pill behind social icons when the page theme is dark.
pub const DEFAULT_SOCIAL_BUTTON_BG_DARK: &str = "#f0f2f8";

const DEFAULT_SOCIAL_ICON_GITHUB_ON_DARK_BG: &str = "#f0f2f8";
const DEFAULT_SOCIAL_ICON_LINKEDIN_ON_DARK_BG: &str = "#5eb9ff";
const DEFAULT_SOCIAL_ICON_FACEBOOK_ON_DARK_BG: &str = "#6bb6ff";

/// Relative luminance below this ⇒ treated as a dark background (WCAG sRGB).
const LUMINANCE_DARK_THRESHOLD: f64 = 0.35;

/// Returns a copy of `theme` with social follow-us tokens filled when missing.
pub fn enrich_theme_with_social_defaults(
    theme: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = theme.clone();
    let page_dark = theme
        .get("page_bg")
        .map(|s| is_dark_css_color(s))
        .unwrap_or(false);

    if !page_dark {
        return out;
    }

    let btn_key = "social_button_background";
    if theme_value_empty(out.get(btn_key)) {
        out.insert(
            btn_key.to_string(),
            DEFAULT_SOCIAL_BUTTON_BG_DARK.to_string(),
        );
    }

    let button_dark = out
        .get(btn_key)
        .map(|s| is_dark_css_color(s))
        .unwrap_or(true);

    if button_dark {
        fill_if_empty(
            &mut out,
            "social_icon_github",
            DEFAULT_SOCIAL_ICON_GITHUB_ON_DARK_BG,
        );
        fill_if_empty(
            &mut out,
            "social_icon_linkedin",
            DEFAULT_SOCIAL_ICON_LINKEDIN_ON_DARK_BG,
        );
        fill_if_empty(
            &mut out,
            "social_icon_facebook",
            DEFAULT_SOCIAL_ICON_FACEBOOK_ON_DARK_BG,
        );
    }

    out
}

fn fill_if_empty(theme: &mut HashMap<String, String>, key: &str, value: &str) {
    if theme_value_empty(theme.get(key)) {
        theme.insert(key.to_string(), value.to_string());
    }
}

fn theme_value_empty(value: Option<&String>) -> bool {
    value.map(|s| s.trim().is_empty()).unwrap_or(true)
}

/// Whether a CSS color value (hex or gradient containing hex) reads as dark.
pub fn is_dark_css_color(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    if let Some((r, g, b)) = parse_hex_color(value) {
        return relative_luminance(r, g, b) < LUMINANCE_DARK_THRESHOLD;
    }
    for hex in extract_hex_colors(value) {
        if let Some((r, g, b)) = parse_hex_digits(&hex) {
            if relative_luminance(r, g, b) < LUMINANCE_DARK_THRESHOLD {
                return true;
            }
        }
    }
    false
}

fn parse_hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let digits = value.strip_prefix('#')?;
    parse_hex_digits(digits)
}

fn parse_hex_digits(digits: &str) -> Option<(u8, u8, u8)> {
    match digits.len() {
        3 => {
            let r = u8::from_str_radix(&digits[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&digits[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&digits[2..3].repeat(2), 16).ok()?;
            Some((r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
            let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
            let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
            Some((r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
            let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
            let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

fn extract_hex_colors(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let len = j - start;
            if len == 3 || len == 6 || len == 8 {
                out.push(value[start..j].to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    let r = srgb_channel_to_linear(f64::from(r) / 255.0);
    let g = srgb_channel_to_linear(f64::from(g) / 255.0);
    let b = srgb_channel_to_linear(f64::from(b) / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_channel_to_linear(c: f64) -> f64 {
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_page_gets_light_social_button_default() {
        let mut theme = HashMap::new();
        theme.insert("page_bg".into(), "#12141f".into());
        let enriched = enrich_theme_with_social_defaults(&theme);
        assert_eq!(
            enriched.get("social_button_background").map(String::as_str),
            Some(DEFAULT_SOCIAL_BUTTON_BG_DARK)
        );
        assert!(!enriched.contains_key("social_icon_github"));
    }

    #[test]
    fn light_page_unchanged() {
        let mut theme = HashMap::new();
        theme.insert("page_bg".into(), "#f8fafc".into());
        let enriched = enrich_theme_with_social_defaults(&theme);
        assert!(!enriched.contains_key("social_button_background"));
    }

    #[test]
    fn dark_page_with_dark_social_button_gets_light_icon_defaults() {
        let mut theme = HashMap::new();
        theme.insert("page_bg".into(), "#12141f".into());
        theme.insert("social_button_background".into(), "#252836".into());
        let enriched = enrich_theme_with_social_defaults(&theme);
        assert_eq!(
            enriched.get("social_icon_github").map(String::as_str),
            Some(DEFAULT_SOCIAL_ICON_GITHUB_ON_DARK_BG)
        );
    }

    #[test]
    fn explicit_theme_values_are_not_overwritten() {
        let mut theme = HashMap::new();
        theme.insert("page_bg".into(), "#12141f".into());
        theme.insert("social_button_background".into(), "#ffffff".into());
        theme.insert("social_icon_github".into(), "#181717".into());
        let enriched = enrich_theme_with_social_defaults(&theme);
        assert_eq!(
            enriched.get("social_button_background").map(String::as_str),
            Some("#ffffff")
        );
        assert_eq!(
            enriched.get("social_icon_github").map(String::as_str),
            Some("#181717")
        );
    }

    #[test]
    fn is_dark_detects_hex_and_gradient() {
        assert!(is_dark_css_color("#12141f"));
        assert!(!is_dark_css_color("#f0f2f8"));
        assert!(is_dark_css_color(
            "linear-gradient(to bottom, #1a1d2e, #12141f)"
        ));
    }
}
