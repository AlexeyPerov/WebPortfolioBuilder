use crate::config::widget_layout_children;
use crate::error::{CoreError, CoreResult};
use crate::html::configure_minijinja_html_escape;
use crate::html::{
    aria_default_for_social_preset, build_apps_showcase_subscribe_data,
    build_careers_split_widget_html, build_reference_panel_html, catalog_stat_line_or,
    catalog_store_url_warnings, external_link_attrs, html_escape, render_reference_panel_detail_html,
    render_vacancy_panel_html, resolve_catalog_store_entries,
    social_icon_preset_class, social_icon_preset_svg,
};
use crate::routing::{
    resolve_asset_href_for_page, resolve_internal_slug_reference, PageRoute, RouteIndex,
};
use crate::types::{CatalogApp, ConfigWarning, SiteConfig, Vacancy, WidgetNode};
use minijinja::Environment;
use serde_json::{json, Value as JsonValue};

use std::collections::HashMap;
use std::path::Path;

pub fn load_widget_env(template_dir: &Path) -> CoreResult<Environment<'static>> {
    let mut env = Environment::new();
    configure_minijinja_html_escape(&mut env);
    let widgets_dir = template_dir.join("widgets");
    for entry in std::fs::read_dir(&widgets_dir).map_err(|e| CoreError::msg(e.to_string()))? {
        let entry = entry.map_err(|e| CoreError::msg(e.to_string()))?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "html") {
            let name = path.file_stem().unwrap().to_string_lossy();
            let content = std::fs::read_to_string(&path)?;
            env.add_template_owned(format!("widgets/{name}"), content)
                .map_err(|e| CoreError::msg(e.to_string()))?;
        }
    }
    Ok(env)
}

pub struct WidgetRenderContext<'a> {
    pub page_path: &'a str,
    pub site: &'a SiteConfig,
    pub route: &'a PageRoute,
    pub routes: &'a RouteIndex,
    pub env: &'a Environment<'static>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PageScriptNeeds {
    pub scroll_reveal: bool,
    pub catalog_carousel: bool,
    pub split_widget: bool,
    pub reference_panel: bool,
    pub image_lightbox: bool,
}

impl PageScriptNeeds {
    pub fn needs_widgets_config(&self) -> bool {
        self.scroll_reveal || self.catalog_carousel || self.split_widget
    }
}

pub fn collect_page_script_needs(widgets: &[WidgetNode]) -> PageScriptNeeds {
    let mut needs = PageScriptNeeds::default();
    for w in widgets {
        accumulate_widget_script_needs(w, &mut needs);
    }
    needs
}

fn accumulate_widget_script_needs(widget: &WidgetNode, needs: &mut PageScriptNeeds) {
    if widget.enabled == Some(false) {
        return;
    }
    let t = widget.widget_type.trim();
    if matches!(t, "row" | "column" | "grid") {
        if let Ok(Some(children)) = widget_layout_children(&widget.props) {
            for child in &children {
                accumulate_widget_script_needs(child, needs);
            }
        }
        return;
    }
    match t {
        "intro" | "cover_banner" | "follow_us" | "info_grid" | "project_grid" => {
            needs.scroll_reveal = true;
        }
        "images_grid" => {
            needs.scroll_reveal = true;
            needs.image_lightbox = true;
        }
        "careers_tabs" => {
            needs.scroll_reveal = true;
            needs.split_widget = true;
        }
        "apps_showcase" | "media_swiper" => {
            needs.scroll_reveal = true;
            needs.catalog_carousel = true;
        }
        "reference_panel" => {
            needs.scroll_reveal = true;
            needs.reference_panel = true;
        }
        _ => {}
    }
}

pub fn render_widget_tree(
    ctx: &WidgetRenderContext<'_>,
    widgets: &[WidgetNode],
) -> CoreResult<(String, Vec<ConfigWarning>)> {
    let mut parts = Vec::new();
    let mut warnings = Vec::new();
    for (i, widget) in widgets.iter().enumerate() {
        let path = format!("{} -> widgets[{i}]", ctx.page_path);
        let (html, w, err) = render_widget(ctx, &path, widget);
        if let Some(e) = err {
            return Err(e);
        }
        warnings.extend(w);
        if !html.is_empty() {
            parts.push(html);
        }
    }
    Ok((join_widget_html(&parts), warnings))
}

/// Join rendered widget HTML matching Go `renderWidgetTree` spacing.
fn join_widget_html(parts: &[String]) -> String {
    let is_multiline = |html: &str| html.contains('\n');
    let mut b = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            let prev_multi = is_multiline(&parts[i - 1]);
            let curr_multi = is_multiline(part);
            let sep = match (prev_multi, curr_multi) {
                (false, false) => "\n",
                (false, true) => "\n\n",
                (true, false) => "\n",
                (true, true) => "\n\n",
            };
            b.push_str(sep);
        }
        b.push_str(part.trim_end_matches('\n'));
        if is_multiline(part) {
            b.push('\n');
        }
    }
    b
}

fn render_widget(
    ctx: &WidgetRenderContext<'_>,
    path: &str,
    widget: &WidgetNode,
) -> (String, Vec<ConfigWarning>, Option<CoreError>) {
    if widget.enabled == Some(false) {
        return (String::new(), vec![], None);
    }
    let widget_type = widget.widget_type.trim();
    if widget_type.is_empty() {
        return (
            String::new(),
            vec![],
            Some(CoreError::msg(format!(
                "{path}.type: required field missing"
            ))),
        );
    }
    if widget_type == "columns" {
        return (
            String::new(),
            vec![],
            Some(CoreError::msg(format!(
                "{path}.type: unknown widget type {widget_type:?}"
            ))),
        );
    }
    if matches!(widget_type, "row" | "column" | "grid") {
        return match render_layout_widget(ctx, path, widget_type, widget) {
            Ok(v) => (v, vec![], None),
            Err(e) => (String::new(), vec![], Some(e)),
        };
    }
    if is_leaf_widget(widget_type) {
        if widget.props.contains_key("children") {
            return (
                String::new(),
                vec![],
                Some(CoreError::msg(format!(
                    "{path}.props.children: only layout widgets (row, column, grid) may define children"
                ))),
            );
        }
        return match render_leaf_widget(ctx, path, widget_type, widget) {
            Ok((html, w)) => (html, w, None),
            Err(e) => (String::new(), vec![], Some(e)),
        };
    }
    (
        String::new(),
        vec![],
        Some(CoreError::msg(format!(
            "{path}.type: unknown widget type {widget_type:?}"
        ))),
    )
}

fn is_leaf_widget(t: &str) -> bool {
    matches!(
        t,
        "intro"
            | "apps_showcase"
            | "info_grid"
            | "images_grid"
            | "careers_tabs"
            | "follow_us"
            | "cover_banner"
            | "project_grid"
            | "media_swiper"
            | "reference_panel"
    )
}

fn render_layout_widget(
    ctx: &WidgetRenderContext<'_>,
    path: &str,
    widget_type: &str,
    widget: &WidgetNode,
) -> CoreResult<String> {
    let children_raw = widget.props.get("children").ok_or_else(|| {
        CoreError::msg(format!(
            "{path}.props.children: required for layout widget {widget_type:?}"
        ))
    })?;
    let children: Vec<WidgetNode> = serde_json::from_value(children_raw.clone()).map_err(|e| {
        CoreError::msg(format!(
            "{path}.props.children: invalid children array: {e}"
        ))
    })?;
    if children.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.children: must not be empty for layout widget {widget_type:?}"
        )));
    }
    let mut child_parts = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.props.children[{i}]");
        let (rendered, _, err) = render_widget(ctx, &child_path, child);
        if let Some(e) = err {
            return Err(e);
        }
        if !rendered.is_empty() {
            child_parts.push(rendered);
        }
    }
    if child_parts.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.children: rendered content must not be empty for layout widget {widget_type:?}"
        )));
    }
    let children_html = child_parts.join("\n");
    let widget_id = widget.id.trim();
    if !widget_id.is_empty() && !is_safe_html_id(widget_id) {
        return Err(CoreError::msg(format!(
            "{path}.id: invalid id {widget_id:?}"
        )));
    }

    let mut ctx_map = json!({ "ID": widget_id, "Children": children_html });
    match widget_type {
        "row" | "column" => {
            let gap = read_optional_gap(&widget.props, &format!("{path}.props.gap"))?;
            if !gap.is_empty() {
                ctx_map["Style"] = json!(format!("gap:{gap};"));
            }
            execute_widget_template(ctx.env, widget_type, ctx_map)
        }
        "grid" => {
            let min_cw = parse_grid_min_column_width(
                &widget.props,
                &format!("{path}.props.min_column_width"),
            )?;
            let gap = read_optional_gap(&widget.props, &format!("{path}.props.gap"))?;
            let grid_gap = if gap.is_empty() { "1.25rem" } else { &gap };
            ctx_map["InnerStyle"] = json!(format!("--widget-grid-min: {min_cw}; gap: {grid_gap};"));
            execute_widget_template(ctx.env, "grid", ctx_map)
        }
        _ => Err(CoreError::msg(format!(
            "{path}.type: unsupported layout widget {widget_type:?}"
        ))),
    }
}

fn execute_widget_template(
    env: &Environment<'static>,
    name: &str,
    data: JsonValue,
) -> CoreResult<String> {
    let tpl = env
        .get_template(&format!("widgets/{name}"))
        .map_err(|e| CoreError::msg(e.to_string()))?;
    tpl.render(data)
        .map_err(|e| CoreError::msg(format!("cannot execute widget template {name:?}: {e}")))
}

fn render_leaf_widget(
    ctx: &WidgetRenderContext<'_>,
    path: &str,
    widget_type: &str,
    widget: &WidgetNode,
) -> CoreResult<(String, Vec<ConfigWarning>)> {
    match widget_type {
        "intro" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            let data = parse_intro_props(ctx, widget, path, id)?;
            if data.is_none() {
                return Ok((String::new(), vec![]));
            }
            Ok((
                execute_widget_template(ctx.env, "intro", data.unwrap())?,
                vec![],
            ))
        }
        "cover_banner" => Ok((
            execute_widget_template(
                ctx.env,
                "cover_banner",
                parse_cover_banner_props(ctx, widget, path)?,
            )?,
            vec![],
        )),
        "follow_us" => {
            let data = build_follow_us_data(ctx, widget, path)?;
            if data.is_none() {
                return Ok((String::new(), vec![]));
            }
            Ok((
                execute_widget_template(ctx.env, "follow_us", data.unwrap())?,
                vec![],
            ))
        }
        "info_grid" => Ok((
            execute_widget_template(
                ctx.env,
                "info_grid",
                parse_info_grid_props(ctx, widget, path)?,
            )?,
            vec![],
        )),
        "images_grid" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            let (data, warnings) = parse_images_grid_props(ctx, widget, path, id)?;
            Ok((
                execute_widget_template(ctx.env, "images_grid", data)?,
                warnings,
            ))
        }
        "careers_tabs" => Ok((
            execute_widget_template(
                ctx.env,
                "careers_tabs",
                parse_careers_tabs_props(ctx, widget, path)?,
            )?,
            vec![],
        )),
        "apps_showcase" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            let (data, warnings) = parse_apps_showcase_props(ctx, widget, path, id)?;
            Ok((
                execute_widget_template(ctx.env, "apps_showcase", data)?,
                warnings,
            ))
        }
        "project_grid" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            Ok((
                execute_widget_template(
                    ctx.env,
                    "project_grid",
                    parse_project_grid_props(ctx, widget, path)?,
                )?,
                vec![],
            ))
        }
        "media_swiper" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            Ok((
                execute_widget_template(
                    ctx.env,
                    "media_swiper",
                    parse_media_swiper_props(ctx, widget, path)?,
                )?,
                vec![],
            ))
        }
        "reference_panel" => {
            let id = widget.id.trim();
            if !id.is_empty() && !is_safe_html_id(id) {
                return Err(CoreError::msg(format!("{path}.id: invalid id {id:?}")));
            }
            Ok((
                execute_widget_template(
                    ctx.env,
                    "reference_panel",
                    parse_reference_panel_props(widget, path, id)?,
                )?,
                vec![],
            ))
        }
        _ => Err(CoreError::msg(format!(
            "{path}.type: unsupported leaf widget {widget_type:?}"
        ))),
    }
}

fn parse_intro_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
    widget_id: &str,
) -> CoreResult<Option<JsonValue>> {
    #[derive(serde::Deserialize, Default)]
    struct CtaRaw {
        #[serde(default)]
        label: String,
        #[serde(default)]
        url: String,
    }
    #[derive(serde::Deserialize, Default)]
    struct LinkButtonItemRaw {
        #[serde(default)]
        label: String,
        #[serde(default)]
        url: String,
    }
    #[derive(serde::Deserialize, Default)]
    struct LinkButtonsRaw {
        #[serde(default)]
        title: String,
        #[serde(default)]
        items: Vec<LinkButtonItemRaw>,
    }
    #[derive(serde::Deserialize)]
    struct Raw {
        #[serde(default)]
        title: String,
        #[serde(default)]
        paragraphs: Vec<String>,
        #[serde(default)]
        pre: String,
        #[serde(default)]
        cta: Option<CtaRaw>,
        #[serde(default)]
        link_buttons: Option<LinkButtonsRaw>,
    }
    let p: Raw = props_to_struct(&widget.props, path)?;
    let title = p.title.trim();
    let paras: Vec<String> = p
        .paragraphs
        .into_iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let pre = p.pre.trim();
    if title.is_empty() && paras.is_empty() && pre.is_empty() {
        eprintln!("Warning: {path} -> intro: empty title, paragraphs, and pre; rendering nothing");
        return Ok(None);
    }
    let section_attr = if widget_id.is_empty() {
        String::new()
    } else {
        format!(r#" id="{}""#, html_escape(widget_id))
    };
    let mut data = json!({
        "SectionIDAttr": section_attr,
        "Title": title,
        "Paragraphs": paras,
        "Pre": pre,
        "HasPre": !pre.is_empty(),
    });
    if let Some(cta) = p.cta {
        let cta_url = cta.url.trim();
        if !cta_url.is_empty() {
            let resolved = resolve_project_grid_cta(ctx, cta_url)?;
            let cta_label = if cta.label.trim().is_empty() {
                "Learn more".to_string()
            } else {
                cta.label.trim().to_string()
            };
            data["CtaURL"] = json!(resolved);
            data["CtaLabel"] = json!(cta_label);
            data["CtaAttrs"] = json!(external_link_attrs(&resolved));
        }
    }
    if let Some(link_buttons) = p.link_buttons {
        let section_title = link_buttons.title.trim();
        let mut buttons = Vec::new();
        for (i, item) in link_buttons.items.iter().enumerate() {
            let label = item.label.trim();
            let url = item.url.trim();
            if label.is_empty() || url.is_empty() {
                continue;
            }
            let resolved = resolve_project_grid_cta(ctx, url).map_err(|e| {
                CoreError::msg(format!(
                    "{path}.props.link_buttons.items[{i}].url: {}",
                    e
                ))
            })?;
            buttons.push(format!(
                r#"<a class="intro__link-btn" href="{href}"{attrs}>{label}</a>"#,
                href = html_escape(&resolved),
                attrs = external_link_attrs(&resolved),
                label = html_escape(label),
            ));
        }
        if !buttons.is_empty() {
            data["HasLinkButtons"] = json!(true);
            data["LinkButtonsTitle"] = json!(if section_title.is_empty() {
                "Projects".to_string()
            } else {
                section_title.to_string()
            });
            data["LinkButtonsHTML"] = json!(buttons.join(""));
        }
    }
    Ok(Some(data))
}

fn parse_cover_banner_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<JsonValue> {
    #[derive(serde::Deserialize)]
    struct Raw {
        src: String,
        alt: String,
    }
    let p: Raw = props_to_struct(&widget.props, path)?;
    let src = p.src.trim();
    if src.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.src: required field missing"
        )));
    }
    let alt = if p.alt.trim().is_empty() {
        "Cover".to_string()
    } else {
        p.alt.trim().to_string()
    };
    Ok(json!({
        "Src": resolve_asset_href_for_page(src, ctx.route),
        "Alt": alt,
    }))
}

fn build_follow_us_data(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<Option<JsonValue>> {
    #[derive(serde::Deserialize, Default)]
    struct Raw {
        title: String,
    }
    let r: Raw = props_to_struct(&widget.props, path).unwrap_or_default();
    let title = if r.title.trim().is_empty() {
        "Follow us".to_string()
    } else {
        r.title.trim().to_string()
    };
    let mut buttons = Vec::new();
    for link in ctx.site.social.resolved_links() {
        let u = link.url.trim();
        if u.is_empty() {
            continue;
        }
        let img_path = link.icon_image.trim();
        let preset_svg = social_icon_preset_svg(&link.icon);
        if img_path.is_empty() && preset_svg.is_empty() {
            continue;
        }
        let aria = if !link.aria_label.trim().is_empty() {
            link.aria_label.trim().to_string()
        } else {
            aria_default_for_social_preset(&link.icon).to_string()
        };
        let (inner, btn_class) = if !img_path.is_empty() {
            let h = resolve_asset_href_for_page(img_path, ctx.route);
            (
                format!(
                    r#"<img class="follow-us__icon" src="{}" alt="" decoding="async">"#,
                    html_escape(&h)
                ),
                "follow-us__btn follow-us__btn--custom".to_string(),
            )
        } else {
            let mut cls = "follow-us__btn".to_string();
            let mod_cls = social_icon_preset_class(&link.icon);
            if !mod_cls.is_empty() {
                cls.push(' ');
                cls.push_str(mod_cls);
            }
            (preset_svg.to_string(), cls)
        };
        buttons.push(format!(
            r#"<a class="{btn_class}" href="{u}"{attrs} aria-label="{aria}">{inner}</a>"#,
            btn_class = btn_class,
            u = html_escape(u),
            attrs = external_link_attrs(u),
            aria = html_escape(&aria),
            inner = inner,
        ));
    }
    if buttons.is_empty() {
        eprintln!("Warning: {path} -> follow_us: no social links resolved; rendering nothing");
        return Ok(None);
    }
    Ok(Some(json!({
        "Title": title,
        "HasButtons": true,
        "Buttons": buttons.join(""),
    })))
}

fn parse_info_grid_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<JsonValue> {
    #[derive(serde::Deserialize)]
    struct ItemRaw {
        image: String,
        title: String,
        text: String,
    }
    #[derive(serde::Deserialize)]
    struct RawTop {
        title: String,
        items: Vec<ItemRaw>,
    }
    let p: RawTop = props_to_struct(&widget.props, path)?;
    if p.items.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.items: required and must not be empty"
        )));
    }
    let mut items = Vec::new();
    for (i, it) in p.items.iter().enumerate() {
        let title = it.title.trim();
        let text = it.text.trim();
        if title.is_empty() && text.is_empty() {
            return Err(CoreError::msg(format!(
                "{path}.props.items[{i}]: at least one of title or text is required"
            )));
        }
        let img = if it.image.trim().is_empty() {
            String::new()
        } else {
            resolve_asset_href_for_page(it.image.trim(), ctx.route)
        };
        let img_alt = if title.is_empty() {
            " ".into()
        } else {
            title.to_string()
        };
        items.push(json!({
            "Image": img,
            "ImageAlt": img_alt,
            "Title": title,
            "Text": text,
        }));
    }
    Ok(json!({ "Title": p.title.trim(), "Items": items }))
}

fn parse_images_grid_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
    widget_id: &str,
) -> CoreResult<(JsonValue, Vec<ConfigWarning>)> {
    let raw = widget.props.get("images").ok_or_else(|| {
        CoreError::msg(format!(
            "{path}.props.images: required and must not be empty"
        ))
    })?;
    let (entries, warnings) =
        normalize_images_grid_raw(ctx.page_path, raw, &format!("{path}.props.images"))?;
    if entries.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.images: required and must not be empty"
        )));
    }
    let images: Vec<JsonValue> = entries
        .iter()
        .map(|e| {
            json!({
                "Src": resolve_asset_href_for_page(&e.src, ctx.route),
                "Alt": e.alt,
            })
        })
        .collect();
    #[derive(serde::Deserialize, Default)]
    struct Top {
        title: String,
    }
    let top: Top = props_to_struct(&widget.props, path).unwrap_or_default();
    let section_id = if widget_id.is_empty() { "photos" } else { widget_id };
    let section_attr = format!(r#" id="{}""#, crate::html::html_escape(section_id));
    Ok((
        json!({
            "SectionIDAttr": section_attr,
            "Title": top.title.trim(),
            "Images": images,
        }),
        warnings,
    ))
}

struct ImagesGridEntry {
    src: String,
    alt: String,
}

fn normalize_images_grid_raw(
    page_path: &str,
    raw: &JsonValue,
    cfg_path: &str,
) -> CoreResult<(Vec<ImagesGridEntry>, Vec<ConfigWarning>)> {
    use crate::types::ConfigWarning;
    if let Some(arr) = raw.as_array() {
        if arr.first().is_some_and(|v| v.is_string()) {
            let mut out = Vec::new();
            let mut warnings = Vec::new();
            for (i, v) in arr.iter().enumerate() {
                let s = v.as_str().unwrap_or("").trim();
                if s.is_empty() {
                    return Err(CoreError::msg(format!(
                        "{cfg_path}[{i}]: empty string not allowed"
                    )));
                }
                let alt = format!("photo {}", i + 1);
                warnings.push(ConfigWarning::content(
                    page_path,
                    format!(
                        "{cfg_path}[{i}]: missing descriptive alt text (using generic fallback {alt:?})"
                    ),
                ));
                out.push(ImagesGridEntry {
                    src: s.to_string(),
                    alt,
                });
            }
            return Ok((out, warnings));
        }
        let mut out = Vec::new();
        let mut warnings = Vec::new();
        for (i, v) in arr.iter().enumerate() {
            let src = v.get("src").and_then(|s| s.as_str()).unwrap_or("").trim();
            if src.is_empty() {
                return Err(CoreError::msg(format!("{cfg_path}[{i}].src: required")));
            }
            let mut alt = v
                .get("alt")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if is_generic_images_grid_alt(&alt, i) {
                if alt.is_empty() {
                    alt = format!("photo {}", i + 1);
                }
                warnings.push(ConfigWarning::content(
                    page_path,
                    format!(
                        "{cfg_path}[{i}].alt: missing descriptive alt text (using generic fallback {alt:?})"
                    ),
                ));
            }
            out.push(ImagesGridEntry {
                src: src.to_string(),
                alt,
            });
        }
        return Ok((out, warnings));
    }
    Err(CoreError::msg(format!(
        "{cfg_path}: must be an array of strings or objects with src"
    )))
}

fn is_generic_images_grid_alt(alt: &str, index: usize) -> bool {
    let alt = alt.trim();
    alt.is_empty() || alt == format!("photo {}", index + 1)
}

fn parse_careers_tabs_props(
    _ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<JsonValue> {
    #[derive(serde::Deserialize)]
    struct Raw {
        title: String,
        vacancies: Vec<Vacancy>,
        #[serde(default)]
        labels: HashMap<String, String>,
    }
    let p: Raw = props_to_struct(&widget.props, path)?;
    if p.vacancies.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.vacancies: required and must not be empty"
        )));
    }
    let mut req = "Requirements for this position:".to_string();
    let mut resp = "Responsibilities:".to_string();
    let mut adv = "Advantages of working with us:".to_string();
    if let Some(v) = p
        .labels
        .get("requirements_title")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        req = v.to_string();
    }
    if let Some(v) = p
        .labels
        .get("responsibilities_title")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        resp = v.to_string();
    }
    if let Some(v) = p
        .labels
        .get("advantages_title")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        adv = v.to_string();
    }
    let mut entries = Vec::new();
    for (i, v) in p.vacancies.iter().enumerate() {
        if v.role.trim().is_empty() {
            return Err(CoreError::msg(format!(
                "{path}.props.vacancies[{i}].role: required field missing"
            )));
        }
        let body = render_vacancy_panel_html(v, &req, &resp, &adv);
        entries.push((v.role.trim().to_string(), body));
    }
    let split_html = build_careers_split_widget_html(&entries);
    Ok(json!({
        "SectionTitle": p.title.trim(),
        "SplitWidget": split_html,
    }))
}

fn parse_apps_showcase_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
    widget_id: &str,
) -> CoreResult<(JsonValue, Vec<ConfigWarning>)> {
    #[derive(serde::Deserialize)]
    struct Raw {
        #[serde(default)]
        section_title: String,
        #[serde(default)]
        apps: Vec<JsonValue>,
    }
    let p: Raw = props_to_struct(&widget.props, path)?;
    if p.apps.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.apps: required and must not be empty"
        )));
    }
    let icons = ctx.site.store_icons_with_defaults();
    let subscribe = build_apps_showcase_subscribe_data(&ctx.site.subscribe_block);
    let mut cards = Vec::new();
    let mut warnings = Vec::new();
    for (i, app_raw) in p.apps.iter().enumerate() {
        let app_path = format!("{path}.props.apps[{i}]");
        let app: CatalogApp = serde_json::from_value(app_raw.clone())
            .map_err(|e| CoreError::msg(format!("{app_path}: invalid app entry: {e}")))?;
        warnings.extend(catalog_store_url_warnings(
            ctx.page_path,
            &app_path,
            app_raw,
        ));
        cards.push(build_apps_showcase_card_data(
            ctx,
            &app,
            &app_path,
            &icons,
            subscribe.clone(),
        )?);
    }
    Ok((
        json!({
            "ID": widget_id,
            "SectionTitle": p.section_title.trim(),
            "Cards": cards,
        }),
        warnings,
    ))
}

fn build_apps_showcase_card_data(
    ctx: &WidgetRenderContext<'_>,
    app: &CatalogApp,
    app_path: &str,
    icons: &HashMap<String, String>,
    subscribe: Option<JsonValue>,
) -> CoreResult<JsonValue> {
    let image = app.image.trim();
    if image.is_empty() {
        return Err(CoreError::msg(format!(
            "{app_path}.image: required field missing"
        )));
    }
    let title = app.title.trim();
    let header_image = app.header_image.trim();
    let (header_href, header_alt, title_in_header, body_title) = if !header_image.is_empty() {
        let header_alt = if title.is_empty() {
            "App".to_string()
        } else {
            title.to_string()
        };
        (
            resolve_asset_href_for_page(header_image, ctx.route),
            header_alt,
            String::new(),
            if title.is_empty() {
                String::new()
            } else {
                title.to_string()
            },
        )
    } else if !title.is_empty() {
        (
            String::new(),
            String::new(),
            title.to_string(),
            String::new(),
        )
    } else {
        (String::new(), String::new(), String::new(), String::new())
    };

    let bg = if app.card_background.trim().is_empty() {
        "var(--widget-gradient)".to_string()
    } else {
        app.card_background.trim().to_string()
    };
    let card_style = format!(r#" style="background: {}""#, html_escape(&bg));
    let mut card_class = "offer-card catalog-app-card scroll-reveal".to_string();
    if !title_in_header.is_empty() {
        card_class.push_str(" catalog-app-card--title-in-header");
    }

    let mut slides = Vec::new();
    for (i, raw_src) in app.swiper_images.iter().enumerate() {
        let src = raw_src.trim();
        if src.is_empty() {
            return Err(CoreError::msg(format!(
                "{app_path}.swiper_images[{i}]: required field missing"
            )));
        }
        slides.push(json!({
            "Src": resolve_asset_href_for_page(src, ctx.route),
            "Alt": format!("{title} screenshot {}", i + 1),
        }));
    }

    let resolved_stores = resolve_catalog_store_entries(app, icons);
    let stores: Vec<JsonValue> = resolved_stores
        .iter()
        .filter(|s| !s.url.trim().is_empty())
        .map(|store| {
            json!({
                "ClassSuffix": store.class_suffix,
                "URL": store.url,
                "Attrs": external_link_attrs(&store.url),
                "AriaLabel": store.aria_label,
                "IconSrc": resolve_asset_href_for_page(&store.icon_src, ctx.route),
            })
        })
        .collect();

    let mut card = json!({
        "CardClass": card_class,
        "CardStyleAttr": card_style,
        "HeaderImage": header_href,
        "HeaderAlt": header_alt,
        "TitleInHeader": title_in_header,
        "BodyTitle": body_title,
        "IconSrc": resolve_asset_href_for_page(image, ctx.route),
        "IconAlt": title,
        "StatLeft1": catalog_stat_line_or(&app.stat_left_line_1, "1M+"),
        "StatLeft2": catalog_stat_line_or(&app.stat_left_line_2, "Downloads"),
        "StatRight1": catalog_stat_line_or(&app.stat_right_line_1, "4.8"),
        "StatRight2": catalog_stat_line_or(&app.stat_right_line_2, "on Google Play"),
        "Text1": app.text_1.trim(),
        "Text2": app.text_2.trim(),
        "Slides": slides,
        "Stores": stores,
    });
    if let Some(sub) = subscribe {
        card["Subscribe"] = sub;
    }
    Ok(card)
}

fn parse_project_grid_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<JsonValue> {
    #[derive(serde::Deserialize, Default)]
    struct CtaRaw {
        #[serde(default)]
        label: String,
        #[serde(default)]
        url: String,
    }
    #[derive(serde::Deserialize)]
    struct CardRaw {
        title: String,
        description: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        image: String,
        #[serde(default)]
        cta: CtaRaw,
        #[serde(default)]
        secondary_cta: Option<CtaRaw>,
        meta: JsonValue,
    }
    #[derive(serde::Deserialize)]
    struct TopRaw {
        #[serde(default)]
        heading: String,
        #[serde(default)]
        subheading: String,
        #[serde(default)]
        section_id: String,
        cards: Vec<CardRaw>,
    }
    let tr: TopRaw = props_to_struct(&widget.props, path)?;
    if tr.cards.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.cards: required and must not be empty"
        )));
    }
    let section_id = tr.section_id.trim();
    let section_attr = if section_id.is_empty() {
        String::new()
    } else {
        if !is_safe_html_id(section_id) {
            return Err(CoreError::msg(format!(
                "{path}.props.section_id: invalid id {section_id:?}"
            )));
        }
        format!(r#" id="{}""#, html_escape(section_id))
    };
    let _ = parse_min_card_column_width(
        &widget.props,
        &format!("{path}.props.min_card_column_width"),
    )?;
    let grid_style = String::new();
    let mut cards = Vec::new();
    for (i, c) in tr.cards.iter().enumerate() {
        let card_path = format!("{path}.props.cards[{i}]");
        let title = c.title.trim();
        if title.is_empty() {
            return Err(CoreError::msg(format!(
                "{card_path}.title: required field missing"
            )));
        }
        let desc = c.description.trim();
        if desc.is_empty() {
            return Err(CoreError::msg(format!(
                "{card_path}.description: required field missing"
            )));
        }
        let img = c.image.trim();
        let cta_url = c.cta.url.trim();
        if cta_url.is_empty() {
            return Err(CoreError::msg(format!(
                "{card_path}.cta.url: required field missing"
            )));
        }
        let cta_label = if c.cta.label.trim().is_empty() {
            "Learn more".to_string()
        } else {
            c.cta.label.trim().to_string()
        };
        let resolved_cta = resolve_project_grid_cta(ctx, cta_url)?;
        let (meta_line, meta_pairs) =
            parse_project_grid_card_meta(&c.meta, &format!("{card_path}.meta"))?;
        let tags: Vec<String> = c
            .tags
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        let mut card = json!({
            "HasImage": !img.is_empty(),
            "Title": title,
            "Description": desc,
            "Tags": tags,
            "MetaLine": meta_line,
            "MetaPairs": meta_pairs,
            "HasMetaPairs": !meta_pairs.is_empty(),
            "CtaLabel": cta_label,
            "CtaURL": resolved_cta,
            "CtaAttrs": external_link_attrs(&resolved_cta),
            "HasSecondaryCta": false,
        });
        if let Some(sec) = &c.secondary_cta {
            let sec_url = sec.url.trim();
            if !sec_url.is_empty() {
                let sec_label = if sec.label.trim().is_empty() {
                    "Learn more".to_string()
                } else {
                    sec.label.trim().to_string()
                };
                let resolved_sec = resolve_project_grid_cta(ctx, sec_url)?;
                card["HasSecondaryCta"] = json!(true);
                card["SecondaryCtaLabel"] = json!(sec_label);
                card["SecondaryCtaURL"] = json!(resolved_sec);
                card["SecondaryCtaAttrs"] = json!(external_link_attrs(&resolved_sec));
            }
        }
        if !img.is_empty() {
            card["ImageSrc"] = json!(resolve_asset_href_for_page(img, ctx.route));
            card["ImageAlt"] = json!(title);
        }
        cards.push(card);
    }
    Ok(json!({
        "SectionIDAttr": section_attr,
        "Heading": tr.heading.trim(),
        "Subheading": tr.subheading.trim(),
        "GridStyle": grid_style,
        "Cards": cards,
    }))
}

fn parse_media_swiper_props(
    ctx: &WidgetRenderContext<'_>,
    widget: &WidgetNode,
    path: &str,
) -> CoreResult<JsonValue> {
    let img_raw = widget.props.get("images").ok_or_else(|| {
        CoreError::msg(format!(
            "{path}.props.images: required and must not be empty"
        ))
    })?;
    #[derive(serde::Deserialize)]
    struct Slide {
        src: String,
        #[serde(default)]
        alt: String,
    }
    let slides: Vec<Slide> = serde_json::from_value(img_raw.clone())
        .map_err(|e| CoreError::msg(format!("{path}.props.images: invalid array: {e}")))?;
    if slides.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.images: required and must not be empty"
        )));
    }
    #[derive(serde::Deserialize, Default)]
    struct Top {
        aria_label: String,
    }
    let t: Top = props_to_struct(&widget.props, path).unwrap_or_default();
    let aria = if t.aria_label.trim().is_empty() {
        "Image carousel".to_string()
    } else {
        t.aria_label.trim().to_string()
    };
    let out_slides: Vec<JsonValue> = slides
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let src = s.src.trim();
            if src.is_empty() {
                return Err(CoreError::msg(format!(
                    "{path}.props.images[{i}].src: required field missing"
                )));
            }
            let alt = if s.alt.trim().is_empty() {
                format!("slide {}", i + 1)
            } else {
                s.alt.trim().to_string()
            };
            Ok(json!({
                "Src": resolve_asset_href_for_page(src, ctx.route),
                "Alt": alt,
            }))
        })
        .collect::<CoreResult<Vec<_>>>()?;
    Ok(json!({ "AriaLabel": aria, "Slides": out_slides }))
}

fn parse_reference_panel_props(
    widget: &WidgetNode,
    path: &str,
    widget_id: &str,
) -> CoreResult<JsonValue> {
    #[derive(serde::Deserialize, Default)]
    struct ArgRaw {
        name: String,
        #[serde(rename = "type", default)]
        arg_type: String,
        #[serde(default)]
        description: String,
    }
    #[derive(serde::Deserialize)]
    struct EntryRaw {
        label: String,
        #[serde(default)]
        method: String,
        #[serde(default)]
        signature: String,
        description: String,
        #[serde(default)]
        arguments: Vec<ArgRaw>,
        #[serde(default)]
        example: String,
    }
    #[derive(serde::Deserialize)]
    struct TopRaw {
        #[serde(default)]
        title: String,
        #[serde(default)]
        intro: String,
        entries: Vec<EntryRaw>,
    }
    let p: TopRaw = props_to_struct(&widget.props, path)?;
    if p.entries.is_empty() {
        return Err(CoreError::msg(format!(
            "{path}.props.entries: required and must not be empty"
        )));
    }
    let id_prefix = if widget_id.is_empty() {
        "ref-panel".to_string()
    } else {
        widget_id.to_string()
    };
    let aria_label = if p.title.trim().is_empty() {
        "Reference".to_string()
    } else {
        p.title.trim().to_string()
    };
    let mut built: Vec<(String, String)> = Vec::new();
    for (i, e) in p.entries.iter().enumerate() {
        let entry_path = format!("{path}.props.entries[{i}]");
        let label = e.label.trim();
        if label.is_empty() {
            return Err(CoreError::msg(format!(
                "{entry_path}.label: required field missing"
            )));
        }
        let description = e.description.trim();
        if description.is_empty() {
            return Err(CoreError::msg(format!(
                "{entry_path}.description: required field missing"
            )));
        }
        let method = e.method.trim();
        let signature = e.signature.trim();
        let example = e.example.trim();
        let mut args: Vec<(String, String, String)> = Vec::new();
        for (j, a) in e.arguments.iter().enumerate() {
            let arg_path = format!("{entry_path}.arguments[{j}]");
            let name = a.name.trim();
            if name.is_empty() {
                return Err(CoreError::msg(format!(
                    "{arg_path}.name: required field missing"
                )));
            }
            args.push((
                name.to_string(),
                a.arg_type.trim().to_string(),
                a.description.trim().to_string(),
            ));
        }
        let body = render_reference_panel_detail_html(
            method,
            signature,
            description,
            &args,
            example,
        );
        built.push((label.to_string(), body));
    }
    let section_attr = if widget_id.is_empty() {
        String::new()
    } else {
        format!(r#" id="{}""#, html_escape(widget_id))
    };
    let split_html = build_reference_panel_html(&aria_label, &id_prefix, &built);
    Ok(json!({
        "SectionIDAttr": section_attr,
        "Title": p.title.trim(),
        "Intro": p.intro.trim(),
        "HasIntro": !p.intro.trim().is_empty(),
        "SplitHTML": split_html,
    }))
}

fn resolve_project_grid_cta(ctx: &WidgetRenderContext<'_>, raw: &str) -> CoreResult<String> {
    let u = raw.trim();
    if u.is_empty() {
        return Ok(String::new());
    }
    if crate::routing::is_external_or_special_href(u) || u.starts_with('/') {
        return Ok(u.to_string());
    }
    resolve_internal_slug_reference(ctx.route, u, &ctx.routes.by_slug)
}

fn parse_project_grid_card_meta(
    raw: &JsonValue,
    meta_path: &str,
) -> CoreResult<(String, Vec<JsonValue>)> {
    if raw.is_null() {
        return Err(CoreError::msg(format!(
            "{meta_path}: required field missing"
        )));
    }
    if let Some(s) = raw.as_str() {
        let s = s.trim();
        if s.is_empty() {
            return Err(CoreError::msg(format!("{meta_path}: must not be empty")));
        }
        return Ok((s.to_string(), vec![]));
    }
    if let Some(obj) = raw.as_object() {
        let mut keys: Vec<_> = obj.keys().cloned().collect();
        keys.sort();
        if keys.is_empty() {
            return Err(CoreError::msg(format!(
                "{meta_path}: object must not be empty"
            )));
        }
        let mut pairs = Vec::new();
        for k in keys {
            let v = obj[&k].as_str().unwrap_or("").trim();
            if v.is_empty() {
                return Err(CoreError::msg(format!(
                    "{meta_path}.{k}: value must not be empty"
                )));
            }
            pairs.push(json!({ "Key": k, "Value": v }));
        }
        return Ok((String::new(), pairs));
    }
    Err(CoreError::msg(format!(
        "{meta_path}: must be a string or object with string values"
    )))
}

fn parse_grid_min_column_width(
    props: &HashMap<String, JsonValue>,
    cfg_path: &str,
) -> CoreResult<String> {
    const DEFAULT: &str = "260px";
    match props.get("min_column_width") {
        None => Ok(DEFAULT.into()),
        Some(v) => {
            let s = v.as_str().unwrap_or("").trim();
            if s.is_empty() {
                return Ok(DEFAULT.into());
            }
            if s.to_lowercase().contains("..") {
                return Err(CoreError::msg(format!("{cfg_path}: invalid value {s:?}")));
            }
            sanitize_css_gap_or_length(s, cfg_path)
        }
    }
}

fn parse_min_card_column_width(
    props: &HashMap<String, JsonValue>,
    cfg_path: &str,
) -> CoreResult<String> {
    const DEFAULT: &str = "260px";
    match props.get("min_card_column_width") {
        None => Ok(DEFAULT.into()),
        Some(v) => {
            let s = v.as_str().unwrap_or("").trim();
            if s.is_empty() {
                return Ok(DEFAULT.into());
            }
            if s.to_lowercase().contains("..") {
                return Err(CoreError::msg(format!("{cfg_path}: invalid value {s:?}")));
            }
            sanitize_css_gap_or_length(s, cfg_path)
        }
    }
}

fn read_optional_gap(props: &HashMap<String, JsonValue>, cfg_path: &str) -> CoreResult<String> {
    match props.get("gap") {
        None => Ok(String::new()),
        Some(v) => {
            let s = v.as_str().unwrap_or("").trim();
            sanitize_css_gap_or_length(s, cfg_path)
        }
    }
}

fn sanitize_css_gap_or_length(s: &str, cfg_path: &str) -> CoreResult<String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.len() > 40 {
        return Err(CoreError::msg(format!("{cfg_path}: value too long")));
    }
    for c in s.chars() {
        let ok = c.is_ascii_alphanumeric()
            || c.is_ascii_digit()
            || matches!(c, '%' | '.' | '-' | '_' | '/' | ',' | '(' | ')' | '#' | ' ')
            || c.is_alphabetic();
        if !ok {
            return Err(CoreError::msg(format!(
                "{cfg_path}: unsupported character in {s:?}"
            )));
        }
    }
    Ok(s.to_string())
}

fn is_safe_html_id(id: &str) -> bool {
    for (i, c) in id.chars().enumerate() {
        if i == 0 {
            if c != '_' && !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
                return false;
            }
        } else if c != '_' && c != '-' && !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
            return false;
        }
    }
    true
}

fn props_to_struct<T: serde::de::DeserializeOwned>(
    props: &HashMap<String, JsonValue>,
    path: &str,
) -> CoreResult<T> {
    serde_json::from_value(serde_json::Value::Object(
        props.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    ))
    .map_err(|e| CoreError::msg(format!("{path}.props: invalid shape: {e}")))
}
