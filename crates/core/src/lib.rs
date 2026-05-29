//! Core engine for Portfolio Website Builder (Phase 1).
//!
//! See [README.md](../README.md) for template engine choice and escaping policy.

pub mod assets;
pub mod bundles;
pub mod config;
pub mod error;
pub mod fs_util;
pub mod html;
pub mod render;
pub mod routing;
pub mod serve;
pub mod strict;
pub mod types;
pub mod widgets;

pub use assets::{check_referenced_site_assets, copy_referenced_site_assets};
pub use bundles::{discover_content_bundles, resolve_project_root, resolve_site_dir};
pub use config::{load_site_bundle, validated_output_folder, validated_output_folder_for};
pub use error::{CoreError, CoreResult};
pub use html::{
    build_footer_legal_row, build_footer_outer_html, HTML_TEMPLATE_FAILURE_MARKER,
};
pub use render::{
    build_rendered_page_data, generate_site, render_site_bundle, render_site_bundle_html,
    validate_site, validate_site_bundle, validate_site_bundle_only,
};
pub use routing::{
    build_route_index, normalized_slug, resolve_internal_slug_reference, resolve_nav_href,
    PageRoute, RouteIndex,
};
pub use strict::{enforce_strict_warnings, handle_config_warnings, print_config_warnings};
pub use types::{ConfigWarning, SiteBundle};
