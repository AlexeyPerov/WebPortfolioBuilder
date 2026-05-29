//! CLI integration tests (port of Go `cli_test.go` discover/list/validate/build portions).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_portfoliowebsitebuilder"))
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
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

#[test]
fn cli_validate_demo() {
    let ws = workspace_root();
    if !ws.join("content/demo/site.json").is_file() {
        eprintln!("skip cli_validate_demo: bundle not present");
        return;
    }

    let output = Command::new(bin())
        .args(["--validate", "--site", "content/demo"])
        .current_dir(&ws)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Validation passed:"),
        "expected validation success, got stdout={stdout:?}"
    );
}

#[test]
fn cli_validate_kometa() {
    let ws = workspace_root();
    if !ws.join("content/kometa/site.json").is_file() {
        eprintln!("skip cli_validate_kometa: bundle not present");
        return;
    }

    let output = Command::new(bin())
        .args(["--validate", "--site", "content/kometa"])
        .current_dir(&ws)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Validation passed:"),
        "expected validation success, got stdout={stdout:?}"
    );
}

#[test]
fn cli_list_sites_includes_kometa() {
    let ws = workspace_root();
    if !ws.join("content/kometa/site.json").is_file() {
        eprintln!("skip cli_list_sites: bundle not present");
        return;
    }

    let output = Command::new(bin())
        .arg("--list-sites")
        .current_dir(&ws)
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("content/kometa"),
        "expected kometa in list, got {stdout:?}"
    );
}

#[test]
fn cli_validate_strict_fails_on_unknown_key() {
    let ws = workspace_root();
    let template_src = ws.join("Template");
    if !template_src.join("layout.html").is_file() {
        eprintln!("skip cli_validate_strict_fails: Template not present");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    copy_dir_recursive(&template_src, &project.join("Template")).unwrap();

    let site_dir = project.join("content/strict-test");
    fs::create_dir_all(site_dir.join("pages")).unwrap();
    fs::write(
        site_dir.join("site.json"),
        br#"{"site_id":"test","output_folder":"Results/Test","mystery":"x"}"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("pages/home.json"),
        br#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    )
    .unwrap();

    let output = Command::new(bin())
        .args([
            "--validate",
            "--strict",
            "--site",
            "content/strict-test",
        ])
        .current_dir(project)
        .output()
        .expect("run cli");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown key: mystery"),
        "expected unknown key error, got stderr={stderr:?}"
    );
}

#[test]
fn cli_validate_strict_clean_bundle() {
    let ws = workspace_root();
    let template_src = ws.join("Template");
    if !template_src.join("layout.html").is_file() {
        eprintln!("skip cli_validate_strict_clean: Template not present");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    copy_dir_recursive(&template_src, &project.join("Template")).unwrap();

    let site_dir = project.join("content/clean");
    fs::create_dir_all(site_dir.join("pages")).unwrap();
    fs::write(
        site_dir.join("site.json"),
        br#"{"site_id":"test","output_folder":"Results/Test"}"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("pages/home.json"),
        br#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    )
    .unwrap();

    let output = Command::new(bin())
        .args(["--validate", "--strict", "--site", "content/clean"])
        .current_dir(project)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Validation passed:"),
        "expected validation success, got stdout={stdout:?}"
    );
}

#[test]
fn cli_non_interactive_build_writes_index_html() {
    let ws = workspace_root();
    let template_src = ws.join("Template");
    if !template_src.join("layout.html").is_file() {
        eprintln!("skip cli_non_interactive_build: Template not present");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let project = root.path();
    copy_dir_recursive(&template_src, &project.join("Template")).unwrap();

    let site_dir = project.join("content/mini");
    fs::create_dir_all(site_dir.join("pages")).unwrap();
    fs::write(
        site_dir.join("site.json"),
        br#"{"site_id":"mini","output_folder":"Results/Mini"}"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("pages/home.json"),
        br#"{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}"#,
    )
    .unwrap();

    let output = Command::new(bin())
        .args(["--site", "content/mini"])
        .current_dir(project)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Website generated successfully."),
        "expected build success message, got stdout={stdout:?}"
    );
    assert!(
        project.join("Results/Mini/index.html").is_file(),
        "expected generated index.html"
    );
}

#[test]
fn cli_build_demo_exits_zero() {
    let ws = workspace_root();
    if !ws.join("content/demo/site.json").is_file() {
        eprintln!("skip cli_build_demo: bundle not present");
        return;
    }

    let output = Command::new(bin())
        .args(["--site", "content/demo"])
        .current_dir(&ws)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        ws.join("Results/DemoWebsite/index.html").is_file(),
        "expected demo build output"
    );
}

#[test]
fn cli_build_my_studio_exits_zero() {
    let ws = workspace_root();
    if !ws.join("content/my-studio/site.json").is_file() {
        eprintln!("skip cli_build_my_studio: bundle not present");
        return;
    }

    let output = Command::new(bin())
        .args(["--site", "content/my-studio"])
        .current_dir(&ws)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        ws.join("Results/My-StudioWebsite/index.html").is_file(),
        "expected my-studio build output"
    );
}
