use crate::diagnostics::{
    config_warnings_to_diagnostics, strict_failure_diagnostics, BuildSiteResult,
    Diagnostic, PreviewServerInfo, ProjectRootInfo, ValidateSiteResult,
};
use crate::preview_server::PreviewServerState;
use portfoliowebsitebuilder_core::{
    copy_referenced_site_assets, discover_content_bundles, enforce_strict_warnings,
    load_site_bundle, render_site_bundle, resolve_project_root as core_resolve_project_root,
    resolve_site_dir, validate_site_bundle_only, validated_output_folder_for,
};
use portfoliowebsitebuilder_core::fs_util::{copy_template_static_assets, prepare_destination};
use std::path::{Path, PathBuf};
use tauri::State;

fn parse_project_root(project_root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(project_root);
    if !path.is_dir() {
        return Err(format!("project root is not a directory: {project_root}"));
    }
    Ok(path)
}

fn apply_strict_gate(
    warnings: &[portfoliowebsitebuilder_core::ConfigWarning],
    strict: bool,
    errors: &mut Vec<Diagnostic>,
) -> bool {
    if !strict {
        return true;
    }
    if enforce_strict_warnings(warnings).is_err() {
        errors.extend(strict_failure_diagnostics(warnings));
        return false;
    }
    true
}

fn run_validate(project_root: &Path, site_path: &str, strict: bool) -> ValidateSiteResult {
    let template_dir = project_root.join("Template");
    let site_dir = resolve_site_dir(project_root, site_path);

    let (bundle, load_warnings) = match load_site_bundle(&site_dir) {
        Ok(v) => v,
        Err(e) => {
            return ValidateSiteResult {
                ok: false,
                warnings: vec![],
                errors: vec![Diagnostic::error_from_core(e)],
            };
        }
    };

    let mut warnings = config_warnings_to_diagnostics(&load_warnings);
    let mut errors = Vec::new();

    if !apply_strict_gate(&load_warnings, strict, &mut errors) {
        return ValidateSiteResult {
            ok: false,
            warnings,
            errors,
        };
    }

    match validate_site_bundle_only(&bundle, &template_dir) {
        Ok(render_warnings) => {
            warnings.extend(config_warnings_to_diagnostics(&render_warnings));
            if !apply_strict_gate(&render_warnings, strict, &mut errors) {
                return ValidateSiteResult {
                    ok: false,
                    warnings,
                    errors,
                };
            }
            ValidateSiteResult {
                ok: errors.is_empty(),
                warnings,
                errors,
            }
        }
        Err(e) => ValidateSiteResult {
            ok: false,
            warnings,
            errors: vec![Diagnostic::error_from_core(e)],
        },
    }
}

fn run_build(project_root: &Path, site_path: &str, strict: bool) -> BuildSiteResult {
    let template_dir = project_root.join("Template");
    let site_dir = resolve_site_dir(project_root, site_path);

    let (bundle, load_warnings) = match load_site_bundle(&site_dir) {
        Ok(v) => v,
        Err(e) => {
            return BuildSiteResult {
                ok: false,
                output_dir: None,
                warnings: vec![],
                errors: vec![Diagnostic::error_from_core(e)],
            };
        }
    };

    let mut warnings = config_warnings_to_diagnostics(&load_warnings);
    let mut errors = Vec::new();

    if !apply_strict_gate(&load_warnings, strict, &mut errors) {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors,
        };
    }

    let output_folder = match validated_output_folder_for(
        &bundle.site.output_folder,
        &bundle.site_path,
    ) {
        Ok(f) => f,
        Err(e) => {
            return BuildSiteResult {
                ok: false,
                output_dir: None,
                warnings,
                errors: vec![Diagnostic::error_from_core(e)],
            };
        }
    };

    let target_dir = project_root.join(output_folder.replace('/', std::path::MAIN_SEPARATOR_STR));

    if let Err(e) = prepare_destination(&target_dir).map_err(portfoliowebsitebuilder_core::CoreError::from) {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors: vec![Diagnostic::error_from_core(e)],
        };
    }
    if let Err(e) =
        copy_template_static_assets(&template_dir, &target_dir).map_err(portfoliowebsitebuilder_core::CoreError::from)
    {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors: vec![Diagnostic::error_from_core(e)],
        };
    }
    if let Err(e) = copy_referenced_site_assets(&bundle, &target_dir) {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors: vec![Diagnostic::error_from_core(e)],
        };
    }

    let render_warnings = match render_site_bundle(&bundle, &target_dir, &template_dir) {
        Ok(w) => w,
        Err(e) => {
            return BuildSiteResult {
                ok: false,
                output_dir: None,
                warnings,
                errors: vec![Diagnostic::error_from_core(e)],
            };
        }
    };

    warnings.extend(config_warnings_to_diagnostics(&render_warnings));
    if !apply_strict_gate(&render_warnings, strict, &mut errors) {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors,
        };
    }

    let output_dir = target_dir.to_string_lossy().into_owned();
    BuildSiteResult {
        ok: true,
        output_dir: Some(output_dir),
        warnings,
        errors,
    }
}

#[tauri::command]
pub fn resolve_project_root() -> Result<ProjectRootInfo, String> {
    let project_root = core_resolve_project_root().map_err(|e| e.to_string())?;
    let template_dir = project_root.join("Template");
    Ok(ProjectRootInfo {
        project_root: project_root.to_string_lossy().into_owned(),
        template_dir: template_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn list_content_bundles(project_root: String) -> Result<Vec<String>, String> {
    let project_root = parse_project_root(&project_root)?;
    discover_content_bundles(&project_root).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_site(
    project_root: String,
    site_path: String,
    strict: bool,
) -> Result<ValidateSiteResult, String> {
    let project_root = parse_project_root(&project_root)?;
    Ok(run_validate(&project_root, &site_path, strict))
}

#[tauri::command]
pub fn build_site(
    project_root: String,
    site_path: String,
    strict: bool,
    preview: State<'_, PreviewServerState>,
) -> Result<BuildSiteResult, String> {
    preview.stop();
    let project_root = parse_project_root(&project_root)?;
    Ok(run_build(&project_root, &site_path, strict))
}

#[tauri::command]
pub fn start_preview_server(
    output_dir: String,
    port: u16,
    preview: State<'_, PreviewServerState>,
) -> Result<PreviewServerInfo, String> {
    if port == 0 {
        return Err("port must be greater than 0".into());
    }
    let output_dir = PathBuf::from(&output_dir);
    let (bound_port, url) = preview.start(&output_dir, port)?;
    Ok(PreviewServerInfo {
        url,
        port: bound_port,
        output_dir: output_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn stop_preview_server(preview: State<'_, PreviewServerState>) -> Result<(), String> {
    preview.stop();
    Ok(())
}

// Kept for compatibility with Task 1 scaffold until callers migrate.
#[tauri::command]
pub fn resolve_project_root_info() -> Result<ProjectRootInfo, String> {
    resolve_project_root()
}
