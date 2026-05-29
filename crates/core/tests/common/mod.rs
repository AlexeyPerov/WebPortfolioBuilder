//! Shared helpers for config / strict / bundle integration tests.

use std::fs;
use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn create_test_site_dir(root: &Path) -> PathBuf {
    let site_dir = root.join("content").join("test-site");
    let pages_dir = site_dir.join("pages");
    fs::create_dir_all(&pages_dir).expect("create pages dir");
    site_dir
}

pub fn write_json_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, content).expect("write json");
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let from = src.join(&name);
        let to = dst.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Normalize HTML for golden parity comparison (see tests/README.md).
pub fn normalize_html_for_test(raw: &str) -> String {
    let normalized_endings = raw.replace("\r\n", "\n");
    let lines: Vec<String> = normalized_endings
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    let mut joined = lines.join("\n");
    while joined.ends_with("\n\n\n") {
        joined.pop();
    }
    if !joined.ends_with('\n') {
        joined.push('\n');
    }
    let trimmed = joined.trim();
    if !trimmed.starts_with("<!DOCTYPE") && !trimmed.starts_with("<html") {
        panic!(
            "normalize_html_for_test: output does not start with <!DOCTYPE or <html after normalization"
        );
    }
    format!("{trimmed}\n")
}

pub fn golden_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

pub fn template_dir() -> Option<PathBuf> {
    let t = workspace_root().join("Template");
    if t.join("layout.html").is_file() {
        Some(t)
    } else {
        None
    }
}
