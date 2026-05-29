use crate::diagnostics::{
    config_warnings_to_diagnostics, strict_failure_diagnostics, BuildSiteResult, Diagnostic,
    ValidateSiteResult,
};
use portfoliowebsitebuilder_core::{
    copy_referenced_site_assets, enforce_strict_warnings, load_site_bundle, render_site_bundle,
    resolve_site_dir, validate_site_bundle_only, validated_output_folder_for,
};
use portfoliowebsitebuilder_core::fs_util::{copy_template_static_assets, prepare_destination};
use std::path::Path;

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

pub fn run_validate(project_root: &Path, site_path: &str, strict: bool) -> ValidateSiteResult {
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

pub fn run_build(project_root: &Path, site_path: &str, strict: bool) -> BuildSiteResult {
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

    if let Err(e) =
        prepare_destination(&target_dir).map_err(portfoliowebsitebuilder_core::CoreError::from)
    {
        return BuildSiteResult {
            ok: false,
            output_dir: None,
            warnings,
            errors: vec![Diagnostic::error_from_core(e)],
        };
    }
    if let Err(e) = copy_template_static_assets(&template_dir, &target_dir)
        .map_err(portfoliowebsitebuilder_core::CoreError::from)
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
