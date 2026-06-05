use crate::assets::{check_referenced_site_assets, copy_referenced_site_assets};
use crate::config::{load_site_bundle, validated_output_folder_for};
use crate::error::{CoreError, CoreResult};
use crate::fs_util::{copy_template_static_assets, prepare_destination};
use crate::html::{
    build_footer_outer_html, build_theme_css_variables, build_widgets_config_script,
    configure_minijinja_html_escape, escape_stylesheet_href, normalized_typography,
    resolved_canonical_url, resolved_open_graph_image, resolved_theme_color, resolved_twitter_card,
    HTML_TEMPLATE_FAILURE_MARKER,
};
use crate::routing::{
    asset_prefix_for_depth, build_route_index, resolve_asset_href_for_page,
    resolve_internal_slug_reference, PageRoute, RouteIndex,
};
use crate::types::{ConfigWarning, SiteBundle, SitePageFile};

fn normalize_background_effect(
    raw: &str,
    page_path: &str,
) -> (String, Option<ConfigWarning>) {
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return (String::new(), None);
    }
    match normalized.as_str() {
        "light_leak" | "magic_dust" => (normalized, None),
        _ => (
            String::new(),
            Some(ConfigWarning::content(
                page_path,
                format!(
                    "unknown layout.background_effect {:?}; ignored",
                    raw.trim()
                ),
            )),
        ),
    }
}

fn background_effect_css_class(effect: &str) -> &'static str {
    match effect {
        "light_leak" => "light-leak",
        "magic_dust" => "magic-dust",
        _ => "",
    }
}
use crate::widgets::{
    collect_page_script_needs, load_widget_env, render_widget_tree, WidgetRenderContext,
};
use minijinja::Environment;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn validate_site_bundle(
    bundle: &SiteBundle,
    template_dir: &Path,
) -> CoreResult<Vec<ConfigWarning>> {
    let (_, warnings) = render_site_bundle_internal(bundle, template_dir, None)?;
    Ok(warnings)
}

/// Renders all pages to HTML strings keyed by output path (e.g. `index.html`, `about/index.html`).
pub fn render_site_bundle_html(
    bundle: &SiteBundle,
    template_dir: &Path,
) -> CoreResult<(HashMap<String, String>, Vec<ConfigWarning>)> {
    render_site_bundle_internal(bundle, template_dir, None)
}

pub fn render_site_bundle(
    bundle: &SiteBundle,
    target_dir: &Path,
    template_dir: &Path,
) -> CoreResult<Vec<ConfigWarning>> {
    let (_, warnings) = render_site_bundle_internal(bundle, template_dir, Some(target_dir))?;
    Ok(warnings)
}

fn render_site_bundle_internal(
    bundle: &SiteBundle,
    template_dir: &Path,
    target_dir: Option<&Path>,
) -> CoreResult<(HashMap<String, String>, Vec<ConfigWarning>)> {
    let routes = build_route_index(bundle)?;
    let widget_env = load_widget_env(template_dir)?;

    let layout_content = fs::read_to_string(template_dir.join("layout.html"))?;
    let mut layout_env = Environment::new();
    configure_minijinja_html_escape(&mut layout_env);
    layout_env
        .add_template_owned("layout", layout_content)
        .map_err(|e| CoreError::msg(e.to_string()))?;

    let page_by_path: HashMap<&str, &SitePageFile> =
        bundle.pages.iter().map(|p| (p.path.as_str(), p)).collect();

    let mut warnings = Vec::new();
    let mut pages = HashMap::new();
    for route in &routes.ordered {
        let page_file = page_by_path[route.source_path.as_str()];
        let (data, page_warnings) =
            build_rendered_page_data(bundle, page_file, route, &routes, &widget_env)?;
        warnings.extend(page_warnings);

        let html = layout_env
            .get_template("layout")
            .map_err(|e| CoreError::msg(e.to_string()))?
            .render(data)
            .map_err(|e| {
                CoreError::msg(format!("cannot render page {:?}: {e}", route.source_path))
            })?;

        if html.contains(HTML_TEMPLATE_FAILURE_MARKER) {
            return Err(CoreError::msg(format!(
                "rendered HTML for page {:?} ({:?}) contains {:?}: unsafe substitution",
                route.source_path, route.output_rel_path, HTML_TEMPLATE_FAILURE_MARKER
            )));
        }

        pages.insert(route.output_rel_path.clone(), html.clone());

        if let Some(target) = target_dir {
            let dst = target.join(
                route
                    .output_rel_path
                    .replace('/', std::path::MAIN_SEPARATOR_STR),
            );
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dst, html)?;
        }
    }
    Ok((pages, warnings))
}

pub fn build_rendered_page_data(
    bundle: &SiteBundle,
    page_file: &SitePageFile,
    route: &PageRoute,
    routes: &RouteIndex,
    widget_env: &Environment<'static>,
) -> CoreResult<(serde_json::Value, Vec<ConfigWarning>)> {
    let page = &page_file.page;
    let title = if page.title.trim().is_empty() {
        bundle.site.site_id.trim().to_string()
    } else {
        page.title.trim().to_string()
    };

    let meta_description = page.seo.description.trim().to_string();
    let canonical_url = resolved_canonical_url(
        &bundle.site.base_url,
        &route.slug,
        page.seo.canonical_url.trim(),
    );
    let open_graph_image =
        resolved_open_graph_image(&bundle.site.base_url, page.seo.og_image.trim());
    let theme_color = resolved_theme_color(&bundle.site.theme);
    let twitter_card = resolved_twitter_card(&open_graph_image);
    let has_seo =
        !meta_description.is_empty() || !canonical_url.is_empty() || !open_graph_image.is_empty();

    let (fonts_href, font_heading, font_body) = normalized_typography(&bundle.site.typography);
    let fonts_href = escape_stylesheet_href(&fonts_href);
    let theme_css = build_theme_css_variables(&bundle.site.theme);

    let show_header = !page.layout.hide_header;
    let show_footer = bundle.site.footer.is_enabled() && !page.layout.hide_footer;

    let (background_effect, layout_warnings) =
        normalize_background_effect(&page.layout.background_effect, &page_file.path);
    let has_background_effect = !background_effect.is_empty();
    let background_effect_class = background_effect_css_class(&background_effect);
    let load_background_effects_script = background_effect == "magic_dust";

    let brand_href = resolve_internal_slug_reference(route, "", &routes.by_slug)
        .map_err(|e| CoreError::msg(format!("{} -> header.brand: {e}", bundle.site_path)))?;
    let header_brand_logo = resolve_asset_href_for_page(&bundle.site.header.brand.logo, route);
    let header_brand_text = bundle.site.header.brand.text.trim().to_string();
    let show_header_brand = !header_brand_logo.is_empty() || !header_brand_text.is_empty();
    let favicon_href = resolve_asset_href_for_page(&bundle.site.favicon, route);
    let site_icon_href = if !favicon_href.is_empty() {
        favicon_href
    } else {
        header_brand_logo.clone()
    };

    let header_nav = render_header_nav_for_page(bundle, route, routes)?;

    let wctx = WidgetRenderContext {
        page_path: &page_file.path,
        site: &bundle.site,
        route,
        routes,
        env: widget_env,
    };
    let (main_content, mut widget_warnings) = render_widget_tree(&wctx, &page.widgets)?;
    widget_warnings.extend(layout_warnings);

    let footer_html = if show_footer {
        build_footer_outer_html(&bundle.site.footer)
    } else {
        String::new()
    };

    let script_needs = collect_page_script_needs(&page.widgets);
    let asset_prefix = asset_prefix_for_depth(&route.dir_rel_path);

    let mut data = json!({
        "Title": title,
        "MetaDescription": meta_description,
        "CanonicalURL": canonical_url,
        "OpenGraphImage": open_graph_image,
        "ThemeColor": theme_color,
        "TwitterCard": twitter_card,
        "HasSEO": has_seo,
        "TypographyGoogleFonts": fonts_href,
        "TypographyFontHeading": font_heading,
        "TypographyFontBody": font_body,
        "ThemeCSSVariables": theme_css,
        "SiteIconHref": site_icon_href,
        "ShowHeader": show_header,
        "ShowFooter": show_footer,
        "ShowHeaderBrand": show_header_brand,
        "HeaderBrandHref": brand_href,
        "HeaderBrandLogo": header_brand_logo,
        "HeaderBrandText": header_brand_text,
        "HeaderNav": header_nav,
        "MainContentHTML": main_content,
        "FooterHTML": footer_html,
        "LoadScrollRevealScript": script_needs.scroll_reveal,
        "LoadCatalogCarouselScript": script_needs.catalog_carousel,
        "LoadSplitWidgetScript": script_needs.split_widget,
        "LoadImageLightboxScript": script_needs.image_lightbox,
        "LoadWidgetsConfig": script_needs.needs_widgets_config(),
        "StylesHref": format!("{}styles.css", asset_prefix),
        "ScrollRevealScriptHref": format!("{}scroll-reveal.js", asset_prefix),
        "CatalogCarouselScriptHref": format!("{}catalog-carousel.js", asset_prefix),
        "SplitWidgetScriptHref": format!("{}split-widget.js", asset_prefix),
        "NavScriptHref": format!("{}nav.js", asset_prefix),
        "ImageLightboxScriptHref": format!("{}image-lightbox.js", asset_prefix),
        "BackgroundEffect": background_effect,
        "BackgroundEffectClass": background_effect_class,
        "HasBackgroundEffect": has_background_effect,
        "LoadBackgroundEffectsScript": load_background_effects_script,
        "BackgroundEffectsScriptHref": format!("{}background-effects.js", asset_prefix),
    });

    if script_needs.needs_widgets_config() {
        data["WidgetsConfigScript"] = json!(build_widgets_config_script(&bundle.site.widgets));
    } else {
        data["WidgetsConfigScript"] = json!("");
    }

    Ok((data, widget_warnings))
}

pub fn render_header_nav_for_page(
    bundle: &SiteBundle,
    route: &PageRoute,
    routes: &RouteIndex,
) -> CoreResult<Vec<serde_json::Value>> {
    let mut out = Vec::new();
    for (i, item) in bundle.site.header.nav.iter().enumerate() {
        let label = item.label.trim();
        if label.is_empty() {
            continue;
        }
        let resolved = crate::routing::resolve_nav_href(route, &item.href, &routes.by_slug)
            .map_err(|e| {
                CoreError::msg(format!("{} -> header.nav[{i}].href: {e}", bundle.site_path))
            })?;
        let open_new_tab = item.open_in_new_tab && resolved.to_lowercase().starts_with("http");
        let is_active = crate::routing::is_nav_item_active(route, &item.href)?;
        out.push(json!({
            "Label": label,
            "Href": resolved,
            "OpenNewTab": open_new_tab,
            "IsActive": is_active,
        }));
    }
    Ok(out)
}

pub fn validate_site_bundle_only(
    bundle: &SiteBundle,
    template_dir: &Path,
) -> CoreResult<Vec<ConfigWarning>> {
    check_referenced_site_assets(bundle)?;
    validate_site_bundle(bundle, template_dir)
}

pub fn generate_site(
    project_root: &Path,
    site_input: &str,
    strict: bool,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> CoreResult<String> {
    use crate::bundles::resolve_site_dir;
    use crate::strict::handle_config_warnings;

    let template_dir = project_root.join("Template");
    let site_dir = resolve_site_dir(project_root, site_input);
    let (bundle, warnings) = load_site_bundle(&site_dir)?;
    handle_config_warnings(stderr, &warnings, strict)?;

    let output_folder = validated_output_folder_for(&bundle.site.output_folder, &bundle.site_path)?;
    let target_dir = project_root.join(output_folder.replace('/', std::path::MAIN_SEPARATOR_STR));

    prepare_destination(&target_dir)?;
    copy_template_static_assets(&template_dir, &target_dir)?;
    copy_referenced_site_assets(&bundle, &target_dir)?;

    let render_warnings = render_site_bundle(&bundle, &target_dir, &template_dir)?;
    handle_config_warnings(stderr, &render_warnings, strict)?;

    writeln!(stdout, "Website generated successfully.")?;
    writeln!(stdout, "Output: {}", target_dir.display())?;
    Ok(target_dir.to_string_lossy().into_owned())
}

pub fn validate_site(
    project_root: &Path,
    site_input: &str,
    template_dir: &Path,
    strict: bool,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> CoreResult<()> {
    use crate::bundles::resolve_site_dir;
    use crate::strict::handle_config_warnings;

    let site_dir = resolve_site_dir(project_root, site_input);
    let (bundle, warnings) = load_site_bundle(&site_dir)?;
    handle_config_warnings(stderr, &warnings, strict)?;
    let render_warnings = validate_site_bundle_only(&bundle, template_dir)?;
    handle_config_warnings(stderr, &render_warnings, strict)?;
    writeln!(stdout, "Validation passed: {}", bundle.site_dir)?;
    Ok(())
}
