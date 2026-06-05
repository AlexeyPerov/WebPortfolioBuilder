//! Bundle file path safety and listing.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("content/kometa/site.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            panic!("repo root not found");
        }
    }
}

#[test]
fn list_kometa_bundle_files() {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    let entries =
        app_lib::studio_files::list_bundle_files(&root_str, "content/kometa").expect("list");
    assert!(entries.iter().any(|e| e.relative_path == "site.json"));
    assert!(entries.iter().any(|e| e.relative_path == "pages/home.json"));
}

#[test]
fn reject_path_traversal() {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    let err = app_lib::studio_files::read_bundle_file(&root_str, "content/kometa", "../site.json")
        .expect_err("traversal");
    assert!(err.contains("invalid") || err.contains("escapes"));
}

#[test]
fn read_kometa_image_preview() {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    let preview =
        app_lib::studio_files::read_bundle_image(&root_str, "content/kometa", "assets/logo.png")
            .expect("image preview");
    assert_eq!(preview.relative_path, "assets/logo.png");
    assert!(preview.data_url.starts_with("data:image/png;base64,"));
}

#[test]
fn import_and_delete_bundle_asset() {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    let site = "content/kometa";
    let source = root.join("content/kometa/assets/logo.png");
    assert!(source.is_file());

    let imported = app_lib::studio_files::import_bundle_asset(
        &root_str,
        site,
        &source.to_string_lossy(),
    )
    .expect("import");
    assert_eq!(imported, "assets/assets_logo.png");

    let dest = root.join(site).join(&imported);
    assert!(dest.is_file());

    // Second import should get a suffixed name
    let imported2 = app_lib::studio_files::import_bundle_asset(
        &root_str,
        site,
        &source.to_string_lossy(),
    )
    .expect("import again");
    assert_ne!(imported, imported2);
    assert!(imported2.contains('-'));

    let renamed = app_lib::studio_files::rename_bundle_asset(
        &root_str,
        site,
        &imported,
        "renamed-logo.png",
    )
    .expect("rename");
    assert_eq!(renamed.new_relative_path, "assets/renamed-logo.png");
    assert!(!dest.is_file());
    assert!(root.join(site).join(&renamed.new_relative_path).is_file());

    app_lib::studio_files::delete_bundle_asset(&root_str, site, &renamed.new_relative_path)
        .expect("delete");
    app_lib::studio_files::delete_bundle_asset(&root_str, site, &imported2).expect("delete 2");
}

#[test]
fn rename_updates_asset_across_sites() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let root_str = project.to_string_lossy();

    for site in ["site-a", "site-b"] {
        let bundle = project.join("content").join(site);
        fs::create_dir_all(bundle.join("assets")).unwrap();
        fs::create_dir_all(bundle.join("pages")).unwrap();
        fs::write(bundle.join("assets/shared.png"), b"png").unwrap();
        fs::write(
            bundle.join("site.json"),
            r#"{"site_id":"x","output_folder":"out","seo":{"og_image":"assets/shared.png"}}"#,
        )
        .unwrap();
        fs::write(
            bundle.join("pages/home.json"),
            r#"{"slug":"","widgets":[{"type":"intro","props":{"image":"assets/shared.png"}}]}"#,
        )
        .unwrap();
    }

    let result = app_lib::studio_files::rename_bundle_asset(
        &root_str,
        "content/site-a",
        "assets/shared.png",
        "shared-renamed.png",
    )
    .expect("rename across sites");

    assert_eq!(result.new_relative_path, "assets/shared-renamed.png");
    assert_eq!(result.updated_sites.len(), 2);

    for site in ["site-a", "site-b"] {
        let bundle = project.join("content").join(site);
        assert!(!bundle.join("assets/shared.png").exists());
        assert!(bundle.join("assets/shared-renamed.png").is_file());

        let site_json = fs::read_to_string(bundle.join("site.json")).unwrap();
        assert!(site_json.contains("assets/shared-renamed.png"));
        assert!(!site_json.contains("assets/shared.png"));

        let page_json = fs::read_to_string(bundle.join("pages/home.json")).unwrap();
        assert!(page_json.contains("assets/shared-renamed.png"));
        assert!(!page_json.contains("assets/shared.png"));
    }
}

#[test]
fn reject_delete_non_asset() {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    let err = app_lib::studio_files::delete_bundle_asset(
        &root_str,
        "content/kometa",
        "site.json",
    )
    .expect_err("reject");
    assert!(err.contains("assets"));
}
