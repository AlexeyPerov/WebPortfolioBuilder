//! Port of Go `discoverContentBundles` / `resolveSiteDir` / `resolveProjectRoot` cases.

mod common;

use common::{create_test_site_dir, workspace_root, write_json_file};
use portfoliowebsitebuilder_core::{
    discover_content_bundles, resolve_project_root, resolve_site_dir,
};
use std::fs;

#[test]
fn discover_content_bundles_finds_valid_sites() {
    let root = tempfile::tempdir().unwrap();
    let content = root.path().join("content");
    fs::create_dir_all(content.join("alpha/pages")).unwrap();
    fs::create_dir_all(content.join("beta/pages")).unwrap();
    fs::create_dir_all(content.join("assets-only")).unwrap();
    fs::create_dir_all(content.join("broken")).unwrap();

    write_json_file(
        &content.join("alpha/site.json"),
        r#"{"site_id":"alpha","output_folder":"Results/Alpha"}"#,
    );
    write_json_file(
        &content.join("beta/site.json"),
        r#"{"site_id":"beta","output_folder":"Results/Beta"}"#,
    );
    write_json_file(&content.join("broken/site.json"), "{not json");
    fs::create_dir_all(content.join("_template/pages")).unwrap();
    write_json_file(
        &content.join("_template/site.json"),
        r#"{"site_id":"template","output_folder":"Results/Template"}"#,
    );

    let got = discover_content_bundles(root.path()).unwrap();
    assert_eq!(got, vec!["content/alpha", "content/beta"]);
}

#[test]
fn resolve_site_dir_empty_uses_default_bundle() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    let default = project.join("content/kometa");
    fs::create_dir_all(default.join("pages")).unwrap();
    write_json_file(
        &default.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"Results/Kometa"}"#,
    );

    let got = resolve_site_dir(project, "");
    assert_eq!(got, default);
}

#[test]
fn resolve_site_dir_relative_under_project_root() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    let site_dir = create_test_site_dir(project);
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"x","output_folder":"Results/X"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[]}"#,
    );

    let got = resolve_site_dir(project, "content/test-site");
    assert_eq!(got, site_dir);
}

#[test]
fn resolve_site_dir_absolute_path() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    let abs = site_dir.canonicalize().unwrap_or(site_dir.clone());

    let got = resolve_site_dir(root.path(), abs.to_str().unwrap());
    assert_eq!(got, abs);
}

#[test]
fn resolve_project_root_prefers_cwd_with_kometa_marker() {
    let ws = workspace_root();
    let marker = ws.join("content/kometa/site.json");
    if !marker.is_file() {
        eprintln!("skip resolve_project_root: kometa marker not in workspace");
        return;
    }

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(&ws).unwrap();
    let got = resolve_project_root().unwrap();
    std::env::set_current_dir(original).unwrap();

    let expected = ws.canonicalize().unwrap_or(ws);
    let got_canon = got.canonicalize().unwrap_or(got);
    assert_eq!(got_canon, expected);
}

#[test]
fn discover_content_bundles_in_workspace_lists_kometa() {
    let ws = workspace_root();
    if !ws.join("content/kometa/site.json").is_file() {
        eprintln!("skip discover in workspace: kometa not present");
        return;
    }

    let bundles = discover_content_bundles(&ws).unwrap();
    assert!(
        bundles.iter().any(|b| b == "content/kometa"),
        "expected kometa in {bundles:?}"
    );
}
