//! Port of Go asset pipeline helpers in `fs.go` (`prepareDestination`, `copyTemplateStaticAssets`).

use portfoliowebsitebuilder_core::fs_util::{copy_template_static_assets, prepare_destination};
use std::fs;
#[test]
fn prepare_destination_wipes_existing_tree() {
    let root = tempfile::tempdir().unwrap();
    let dest = root.path().join("Results/Site");
    fs::create_dir_all(&dest.join("nested")).unwrap();
    fs::write(dest.join("nested/old.html"), b"old").unwrap();
    fs::write(dest.join("stale.css"), b"stale").unwrap();

    prepare_destination(&dest).unwrap();

    assert!(
        !dest.exists(),
        "destination dir should be removed before rebuild"
    );
}

#[test]
fn copy_template_static_assets_skips_html_and_copies_other_files() {
    let root = tempfile::tempdir().unwrap();
    let src = root.path().join("Template");
    let dst = root.path().join("out");
    fs::create_dir_all(src.join("widgets")).unwrap();
    fs::write(src.join("layout.html"), b"<html></html>").unwrap();
    fs::write(src.join("widgets/partial.html"), b"<div></div>").unwrap();
    fs::write(src.join("styles.css"), b"body{}").unwrap();
    fs::create_dir_all(src.join("js")).unwrap();
    fs::write(src.join("js/app.js"), b"console.log(1)").unwrap();

    copy_template_static_assets(&src, &dst).unwrap();

    assert!(!dst.join("layout.html").exists());
    assert!(!dst.join("widgets/partial.html").exists());
    assert_eq!(
        fs::read_to_string(dst.join("styles.css")).unwrap(),
        "body{}"
    );
    assert_eq!(
        fs::read_to_string(dst.join("js/app.js")).unwrap(),
        "console.log(1)"
    );
}

mod common;

#[test]
fn generate_site_rebuild_clears_previous_output() {
    use common::{copy_dir_recursive, write_json_file};
    use portfoliowebsitebuilder_core::generate_site;

    let ws_template = common::workspace_root().join("Template");
    if !ws_template.join("layout.html").is_file() {
        eprintln!("skip generate_site_rebuild: Template not present");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    copy_dir_recursive(&ws_template, &project.join("Template")).unwrap();

    let site_dir = project.join("content/mini");
    write_json_file(
        &site_dir.join("site.json"),
        r#"{"site_id":"mini","output_folder":"Results/Mini"}"#,
    );
    write_json_file(
        &site_dir.join("pages/home.json"),
        r#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    generate_site(project, "content/mini", false, &mut stdout, &mut stderr).unwrap();

    let out_dir = project.join("Results/Mini");
    fs::write(out_dir.join("stale.txt"), b"leftover").unwrap();

    generate_site(project, "content/mini", false, &mut stdout, &mut stderr).unwrap();

    assert!(
        out_dir.join("index.html").is_file(),
        "expected regenerated index.html"
    );
    assert!(
        !out_dir.join("stale.txt").exists(),
        "prepare_destination should wipe stale files from prior build"
    );
}
