//! Integration tests for studio invoke command logic (run from repo root).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("content/kometa/site.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            panic!("repo root not found (no content/kometa/site.json above manifest dir)");
        }
    }
}

#[test]
fn discover_kometa_bundle() {
    let root = repo_root();
    let bundles =
        portfoliowebsitebuilder_core::discover_content_bundles(&root).expect("list bundles");
    assert!(
        bundles.iter().any(|b| b == "content/kometa"),
        "bundles: {bundles:?}"
    );
}

#[test]
fn validate_kometa_non_strict_ok() {
    let root = repo_root();
    let site_dir = portfoliowebsitebuilder_core::resolve_site_dir(&root, "content/kometa");
    let template_dir = root.join("Template");
    let (bundle, load_warnings) =
        portfoliowebsitebuilder_core::load_site_bundle(&site_dir).expect("load");
    assert!(
        portfoliowebsitebuilder_core::enforce_strict_warnings(&load_warnings).is_ok()
    );
    let render_warnings =
        portfoliowebsitebuilder_core::validate_site_bundle_only(&bundle, &template_dir)
            .expect("validate render");
    assert!(
        portfoliowebsitebuilder_core::enforce_strict_warnings(&render_warnings).is_ok()
    );
}

#[test]
fn template_bundle_not_listed() {
    let root = repo_root();
    let bundles =
        portfoliowebsitebuilder_core::discover_content_bundles(&root).expect("list bundles");
    assert!(
        !bundles.iter().any(|b| b.contains("_template")),
        "bundles: {bundles:?}"
    );
}

#[test]
fn create_site_from_template_in_temp() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    std::fs::create_dir_all(project.join("Template")).unwrap();
    std::fs::create_dir_all(project.join("content/_template/pages")).unwrap();
    std::fs::copy(
        root.join("content/_template/site.json"),
        project.join("content/_template/site.json"),
    )
    .unwrap();
    std::fs::copy(
        root.join("content/_template/pages/home.json"),
        project.join("content/_template/pages/home.json"),
    )
    .unwrap();

    let site_path = app_lib::site_template::create_site_from_template(project, "test-new-site")
        .expect("create site");
    assert_eq!(site_path, "content/test-new-site");

    let bundles =
        portfoliowebsitebuilder_core::discover_content_bundles(project).expect("list");
    assert!(bundles.contains(&site_path));

    let err = app_lib::site_template::create_site_from_template(project, "test-new-site")
        .expect_err("duplicate");
    assert!(err.contains("already exists"));

    let err = app_lib::site_template::create_site_from_template(project, "Bad Id")
        .expect_err("invalid id");
    assert!(err.contains("invalid site id"));
}
