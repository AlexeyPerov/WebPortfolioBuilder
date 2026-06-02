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
