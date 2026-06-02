//! Port of Go `strict_test.go`.

mod common;

use common::{copy_dir_recursive, create_test_site_dir, workspace_root, write_json_file};
use portfoliowebsitebuilder_core::{
    enforce_strict_warnings, load_site_bundle, validate_site, ConfigWarning,
};

#[test]
fn enforce_strict_warnings_fails_on_unknown_top_level_key() {
    let warnings = vec![ConfigWarning {
        file_path: "site.json".into(),
        key: "mystery".into(),
        detail: String::new(),
    }];
    let err = enforce_strict_warnings(&warnings).unwrap_err();
    assert!(
        err.to_string().contains("unknown key: mystery"),
        "unexpected error: {err}"
    );
}

#[test]
fn enforce_strict_warnings_ignores_legacy_hero_warning() {
    let warnings = vec![ConfigWarning::content(
        "home.json",
        r#"legacy key "hero" is not supported; use widgets (e.g. intro, cover_banner) for page heroes"#,
    )];
    enforce_strict_warnings(&warnings).unwrap();
}

#[test]
fn load_site_bundle_warns_on_unknown_widget_prop_key() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi","typo":true}}]}"#,
    );

    let (_, warnings) = load_site_bundle(&site_dir).unwrap();
    assert_eq!(warnings.len(), 1, "warnings: {warnings:?}");
    assert!(
        warnings[0].to_string().contains("unknown props key"),
        "got: {}",
        warnings[0].to_string()
    );
}

#[test]
fn validate_site_strict_fails_on_unknown_top_level_key() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    let template_src = workspace_root().join("Template");
    if !template_src.join("layout.html").is_file() {
        eprintln!("skip validate_site_strict_fails: Template not present");
        return;
    }
    copy_dir_recursive(&template_src, &project.join("Template")).unwrap();

    let site_dir = create_test_site_dir(project);
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"test","output_folder":"Results/Test","mystery":"x"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let err = validate_site(
        project,
        "content/test-site",
        &project.join("Template"),
        true,
        &mut stdout,
        &mut stderr,
    )
    .unwrap_err();

    let stderr_s = String::from_utf8_lossy(&stderr);
    let msg = format!("{err} {stderr_s}");
    assert!(
        msg.contains("unknown key: mystery"),
        "expected unknown key error, got: {msg}"
    );
}

#[test]
fn validate_site_strict_works_with_validate_only() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    let template_src = workspace_root().join("Template");
    if !template_src.join("layout.html").is_file() {
        eprintln!("skip validate_site_strict_works: Template not present");
        return;
    }
    copy_dir_recursive(&template_src, &project.join("Template")).unwrap();

    let site_dir = project.join("content/clean");
    std::fs::create_dir_all(site_dir.join("pages")).unwrap();
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"test","output_folder":"Results/Test"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    validate_site(
        project,
        "content/clean",
        &project.join("Template"),
        true,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();

    let stdout_s = String::from_utf8_lossy(&stdout);
    assert!(
        stdout_s.contains("Validation passed:"),
        "expected validation success, got stdout={stdout_s:?}"
    );
}
