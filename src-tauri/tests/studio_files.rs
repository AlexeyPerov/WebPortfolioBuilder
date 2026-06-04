//! Bundle file path safety and listing.

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
    assert!(imported.starts_with("assets/logo"));
    assert!(imported.ends_with(".png"));

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

    app_lib::studio_files::delete_bundle_asset(&root_str, site, &imported).expect("delete");
    assert!(!dest.is_file());
    app_lib::studio_files::delete_bundle_asset(&root_str, site, &imported2).expect("delete 2");
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
