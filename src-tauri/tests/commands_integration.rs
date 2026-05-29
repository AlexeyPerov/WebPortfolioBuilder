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
