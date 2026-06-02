use crate::error::{CoreError, CoreResult};
use crate::types::ConfigWarning;
use std::io::Write;

pub fn enforce_strict_warnings(warnings: &[ConfigWarning]) -> CoreResult<()> {
    let mut failures: Vec<String> = warnings
        .iter()
        .filter(|w| is_strict_eligible_warning(w))
        .map(|w| w.to_string())
        .collect();
    failures.sort();
    match failures.len() {
        0 => Ok(()),
        1 => Err(CoreError::msg(failures[0].clone())),
        n => Err(CoreError::msg(format!(
            "strict validation failed ({n} issues):\n  - {}",
            failures.join("\n  - ")
        ))),
    }
}

fn is_strict_eligible_warning(w: &ConfigWarning) -> bool {
    if !w.key.is_empty() {
        return true;
    }
    w.detail.contains("unknown props key:")
}

pub fn print_config_warnings(stderr: &mut dyn Write, warnings: &[ConfigWarning]) -> CoreResult<()> {
    for w in warnings {
        writeln!(stderr, "Warning: {}", w.to_string())?;
    }
    Ok(())
}

pub fn handle_config_warnings(
    stderr: &mut dyn Write,
    warnings: &[ConfigWarning],
    strict: bool,
) -> CoreResult<()> {
    print_config_warnings(stderr, warnings)?;
    if strict {
        enforce_strict_warnings(warnings)?;
    }
    Ok(())
}

use crate::types::WidgetNode;
use std::collections::HashSet;

fn widget_allowed_keys(widget_type: &str) -> Option<HashSet<&'static str>> {
    let keys: &[&str] = match widget_type {
        "intro" => &["title", "paragraphs"],
        "cover_banner" => &["src", "alt"],
        "follow_us" => &["title"],
        "info_grid" => &["title", "items"],
        "images_grid" => &["title", "images"],
        "careers_tabs" => &["title", "vacancies", "labels"],
        "apps_showcase" => &["section_title", "apps"],
        "project_grid" => &[
            "heading",
            "subheading",
            "section_id",
            "cards",
            "min_card_column_width",
        ],
        "media_swiper" => &["images", "aria_label"],
        "row" => &["children", "gap"],
        "column" => &["children"],
        "grid" => &["children", "min_column_width", "gap"],
        _ => return None,
    };
    Some(keys.iter().copied().collect())
}

pub fn unknown_widget_prop_key_warnings(
    page_path: &str,
    widgets: &[WidgetNode],
) -> Vec<ConfigWarning> {
    let mut warnings = Vec::new();
    walk_widget_prop_keys("widgets", page_path, widgets, &mut warnings);
    warnings
}

fn walk_widget_prop_keys(
    prefix: &str,
    page_path: &str,
    widgets: &[WidgetNode],
    warnings: &mut Vec<ConfigWarning>,
) {
    for (i, w) in widgets.iter().enumerate() {
        let wpath = format!("{prefix}[{i}]");
        let widget_type = w.widget_type.trim();
        if let Some(allowed) = widget_allowed_keys(widget_type) {
            for key in w.props.keys() {
                if !allowed.contains(key.as_str()) {
                    warnings.push(ConfigWarning::content(
                        page_path,
                        format!("{wpath}.props: unknown props key: {key:?}"),
                    ));
                }
            }
        }
        if matches!(widget_type, "row" | "column" | "grid") {
            if let Ok(Some(children)) = crate::config::widget_layout_children(&w.props) {
                if !children.is_empty() {
                    walk_widget_prop_keys(
                        &format!("{wpath}.props.children"),
                        page_path,
                        &children,
                        warnings,
                    );
                }
            }
        }
    }
}
