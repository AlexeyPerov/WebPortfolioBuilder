//! Port of Go `config_loader_test.go`.

mod common;

use portfoliowebsitebuilder_core::{load_site_bundle, validated_output_folder_for};
use common::{create_test_site_dir, workspace_root, write_json_file};

#[test]
fn validated_output_folder_allows_nested_relative_path() {
    let got = validated_output_folder_for("Results/KometaWebsite", "site.json").unwrap();
    assert_eq!(got, "Results/KometaWebsite");
}

#[test]
fn validated_output_folder_rejects_absolute_path() {
    let err = validated_output_folder_for("/tmp/out", "site.json").unwrap_err();
    assert!(
        err.to_string().contains("must be a relative path"),
        "unexpected error: {err}"
    );
}

#[test]
fn load_site_bundle_rejects_duplicate_slugs() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[]}"#,
    );
    write_json_file(
        &site_dir.join("pages/about.json"),
        r#"{"slug":"","widgets":[]}"#,
    );

    let err = load_site_bundle(&site_dir).unwrap_err();
    assert!(
        err.to_string().contains("duplicate slug"),
        "unexpected error: {err}"
    );
}

#[test]
fn load_site_bundle_rejects_invalid_output_folder() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"../bad"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[]}"#,
    );

    assert!(load_site_bundle(&site_dir).is_err());
}

#[test]
fn load_site_bundle_warns_on_legacy_hero_key() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[],"hero":{"image":"assets/cover.png"}}"#,
    );

    let (_, warnings) = load_site_bundle(&site_dir).unwrap();
    assert_eq!(warnings.len(), 1, "warnings: {warnings:?}");
    assert!(
        warnings[0].to_string().contains(r#"legacy key "hero""#),
        "got: {}",
        warnings[0].to_string()
    );
}

#[test]
fn load_site_bundle_warns_on_unknown_top_level_keys() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite","mystery":"x"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[],"unused_flag":true}"#,
    );

    let (_, warnings) = load_site_bundle(&site_dir).unwrap();
    assert_eq!(warnings.len(), 2, "warnings: {warnings:?}");
}

#[test]
fn load_site_bundle_rejects_missing_widgets_field() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(&site_dir.join("pages/home.json"), r#"{"slug":""}"#);

    let err = load_site_bundle(&site_dir).unwrap_err();
    assert!(
        err.to_string().contains(r#""widgets" is required"#),
        "unexpected error: {err}"
    );
}

#[test]
fn load_site_bundle_rejects_duplicate_project_grid_section_ids() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[
        {"type":"project_grid","props":{"section_id":"box","cards":[]}},
        {"type":"project_grid","props":{"section_id":"box","cards":[]}}
    ]}"#,
    );

    let err = load_site_bundle(&site_dir).unwrap_err();
    assert!(
        err.to_string().contains("duplicate project_grid"),
        "unexpected error: {err}"
    );
}

#[test]
fn load_site_bundle_rejects_nested_duplicate_project_grid_section_ids() {
    let root = tempfile::tempdir().unwrap();
    let site_dir = create_test_site_dir(root.path());
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"kometa","output_folder":"KometaWebsite"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[
        {"type":"row","props":{"children":[
            {"type":"project_grid","props":{"section_id":"same","cards":[]}},
            {"type":"project_grid","props":{"section_id":"same","cards":[]}}
        ]}}
    ]}"#,
    );

    assert!(load_site_bundle(&site_dir).is_err());
}

#[test]
fn kometa_site_bundle_loads() {
    let kometa = workspace_root().join("content/kometa");
    if !kometa.join("site.json").is_file() {
        eprintln!("skip kometa_site_bundle_loads: bundle not present");
        return;
    }

    let (_, warnings) = load_site_bundle(&kometa).unwrap();
    assert!(
        warnings.is_empty(),
        "unexpected warnings: {:?}",
        warnings.iter().map(|w| w.to_string()).collect::<Vec<_>>()
    );
}
