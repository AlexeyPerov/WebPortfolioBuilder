use portfoliowebsitebuilder_core::{ConfigWarning, CoreError};
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Serialize)]
pub struct Diagnostic {
    pub file_path: String,
    pub message: String,
    pub severity: Severity,
}

impl Diagnostic {
    pub fn warning(w: &ConfigWarning) -> Self {
        Self {
            file_path: w.file_path.clone(),
            message: warning_message(w),
            severity: Severity::Warning,
        }
    }

    pub fn error_from_core(err: CoreError) -> Self {
        Self {
            file_path: String::new(),
            message: err.to_string(),
            severity: Severity::Error,
        }
    }

    pub fn strict_error(w: &ConfigWarning) -> Self {
        Self {
            file_path: w.file_path.clone(),
            message: warning_message(w),
            severity: Severity::Error,
        }
    }
}

pub fn warning_message(w: &ConfigWarning) -> String {
    if !w.detail.is_empty() {
        w.detail.clone()
    } else {
        format!("unknown key: {}", w.key)
    }
}

pub fn is_strict_eligible_warning(w: &ConfigWarning) -> bool {
    if !w.key.is_empty() {
        return true;
    }
    w.detail.contains("unknown props key:")
}

pub fn strict_failure_diagnostics(warnings: &[ConfigWarning]) -> Vec<Diagnostic> {
    warnings
        .iter()
        .filter(|w| is_strict_eligible_warning(w))
        .map(Diagnostic::strict_error)
        .collect()
}

pub fn config_warnings_to_diagnostics(warnings: &[ConfigWarning]) -> Vec<Diagnostic> {
    warnings.iter().map(Diagnostic::warning).collect()
}

#[derive(Serialize)]
pub struct ValidateSiteResult {
    pub ok: bool,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct BuildSiteResult {
    pub ok: bool,
    pub output_dir: Option<String>,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct ProjectRootInfo {
    pub project_root: String,
    pub template_dir: String,
}

#[derive(Serialize)]
pub struct PreviewServerInfo {
    pub url: String,
    pub port: u16,
    pub output_dir: String,
}
