//! Port of Go `render_test.go` write / smoke tests and disk golden parity (Task 13).

mod common;

use common::{
    copy_dir_recursive, golden_root, normalize_html_for_test, template_dir, workspace_root,
};
use portfoliowebsitebuilder_core::{
    copy_referenced_site_assets, generate_site, load_site_bundle, render_site_bundle,
    HTML_TEMPLATE_FAILURE_MARKER,
};
use portfoliowebsitebuilder_core::fs_util::copy_template_static_assets;
use portfoliowebsitebuilder_core::types::{
    PageConfig, SiteBundle, SiteConfig, SitePageFile, WidgetNode,
};
use serde_json::json;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct WrittenGoldenCase {
    bundle_rel: &'static str,
    output_rel: &'static str,
    golden_rel: &'static str,
}

const WRITTEN_GOLDEN_CASES: &[WrittenGoldenCase] = &[
    WrittenGoldenCase {
        bundle_rel: "content/kometa",
        output_rel: "index.html",
        golden_rel: "kometa/index.html",
    },
    WrittenGoldenCase {
        bundle_rel: "content/demo",
        output_rel: "index.html",
        golden_rel: "demo/index.html",
    },
    WrittenGoldenCase {
        bundle_rel: "content/demo",
        output_rel: "about/index.html",
        golden_rel: "demo/about/index.html",
    },
    WrittenGoldenCase {
        bundle_rel: "content/demo",
        output_rel: "apps/index.html",
        golden_rel: "demo/apps/index.html",
    },
    WrittenGoldenCase {
        bundle_rel: "content/my-studio",
        output_rel: "index.html",
        golden_rel: "my-studio/index.html",
    },
];

fn assert_written_golden_parity(golden_path: &Path, golden_raw: &str, actual_raw: &str) {
    assert!(
        !actual_raw.contains(HTML_TEMPLATE_FAILURE_MARKER),
        "written HTML contains forbidden marker {:?} (golden: {})",
        HTML_TEMPLATE_FAILURE_MARKER,
        golden_path.display()
    );

    let golden = normalize_html_for_test(golden_raw);
    let actual = normalize_html_for_test(actual_raw);

    if golden == actual {
        return;
    }

    let diff = TextDiff::from_lines(&golden, &actual);
    let mut message = format!(
        "written HTML mismatch vs golden: {}\n\n--- unified diff (expected golden → actual) ---\n",
        golden_path.display()
    );
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => '-',
            ChangeTag::Insert => '+',
            ChangeTag::Equal => ' ',
        };
        message.push_str(&format!("{sign}{change}"));
    }
    panic!("{message}");
}

#[test]
fn render_site_bundle_writes_one_html_per_route() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip render_site_bundle_writes: Template not present");
        return;
    };

    let mut intro_props = HashMap::new();
    intro_props.insert("title".to_string(), json!("Home"));
    intro_props.insert("paragraphs".to_string(), json!([]));

    let bundle = SiteBundle {
        site_path: "content/demo/site.json".into(),
        site_dir: "content/demo".into(),
        site: SiteConfig {
            site_id: "demo-site".into(),
            ..Default::default()
        },
        pages: vec![
            SitePageFile {
                path: "content/demo/pages/home.json".into(),
                page: PageConfig {
                    slug: String::new(),
                    widgets: vec![WidgetNode {
                        widget_type: "intro".into(),
                        id: String::new(),
                        enabled: None,
                        props: intro_props,
                    }],
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
            SitePageFile {
                path: "content/demo/pages/about.json".into(),
                page: PageConfig {
                    slug: "about".into(),
                    widgets: vec![],
                    ..Default::default()
                },
                has_slug: true,
                has_widgets: true,
            },
        ],
    };

    let root = tempfile::tempdir().unwrap();
    let out_dir = root.path();
    render_site_bundle(&bundle, out_dir, &template_dir).unwrap();

    assert!(out_dir.join("index.html").is_file());
    assert!(out_dir.join("about/index.html").is_file());

    let html = fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(html.contains(r#"class="intro section"#));
    assert!(!html.contains("catalog-carousel.js"));
    assert!(!html.contains("split-widget.js"));
    assert!(!html.contains("image-lightbox.js"));
    assert!(html.contains("scroll-reveal.js"));
}

#[test]
fn render_demo_site_bundle_smoke() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip render_demo_smoke: Template not present");
        return;
    };

    let site_dir = workspace_root().join("content/demo");
    if !site_dir.join("site.json").is_file() {
        eprintln!("skip render_demo_smoke: bundle not present");
        return;
    }

    let (bundle, _) = load_site_bundle(&site_dir).unwrap();
    let root = tempfile::tempdir().unwrap();
    let out_dir = root.path();

    copy_template_static_assets(&template_dir, out_dir).unwrap();
    copy_referenced_site_assets(&bundle, out_dir).unwrap();
    render_site_bundle(&bundle, out_dir, &template_dir).unwrap();

    assert!(out_dir.join("index.html").is_file());
    assert!(out_dir.join("about/index.html").is_file());
    for page in ["layouts", "gallery", "apps", "careers"] {
        assert!(
            out_dir.join(page).join("index.html").is_file(),
            "missing {page}/index.html"
        );
    }

    let html = fs::read_to_string(out_dir.join("index.html")).unwrap();
    for needle in [
        r#"data-widget-type="project_grid""#,
        r#"href="about/""#,
        r#"href="layouts/""#,
        "project-card__cta",
        r#"id="offers""#,
        r#"class="photos section section-gradient scroll-reveal""#,
    ] {
        assert!(html.contains(needle), "expected {needle:?} in demo index.html");
    }

    let apps = fs::read_to_string(out_dir.join("apps/index.html")).unwrap();
    for needle in [
        r#"data-widget-type="apps_showcase""#,
        r#"data-widget-type="project_grid""#,
    ] {
        assert!(apps.contains(needle), "expected {needle:?} in demo apps/index.html");
    }

    let gallery = fs::read_to_string(out_dir.join("gallery/index.html")).unwrap();
    for needle in [
        r#"data-widget-type="media_swiper""#,
        r#"class="photos section section-gradient scroll-reveal""#,
    ] {
        assert!(
            gallery.contains(needle),
            "expected {needle:?} in demo gallery/index.html"
        );
    }

    let careers = fs::read_to_string(out_dir.join("careers/index.html")).unwrap();
    assert!(
        careers.contains(r#"id="vacancies""#),
        "expected id=\"vacancies\" in demo careers/index.html"
    );
}

#[test]
fn render_kometa_site_bundle_smoke() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip render_kometa_smoke: Template not present");
        return;
    };

    let site_dir = workspace_root().join("content/kometa");
    if !site_dir.join("site.json").is_file() {
        eprintln!("skip render_kometa_smoke: bundle not present");
        return;
    }

    let (bundle, _) = load_site_bundle(&site_dir).unwrap();
    let root = tempfile::tempdir().unwrap();
    let out_dir = root.path();

    copy_template_static_assets(&template_dir, out_dir).unwrap();
    copy_referenced_site_assets(&bundle, out_dir).unwrap();
    render_site_bundle(&bundle, out_dir, &template_dir).unwrap();

    let html = fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(!html.contains(HTML_TEMPLATE_FAILURE_MARKER));
    assert!(html.contains("--font-heading:"));
    assert!(html.contains("Quicksand"));

    for needle in [
        r#"id="site-widgets-config""#,
        "scroll-reveal.js",
        "catalog-carousel.js",
        "split-widget.js",
        "split-widget--single",
        "data-catalog-carousel",
        r#"id="vacancies""#,
        r#"id="apps""#,
        r#"<meta name="description" content="Kometa.Games is a mobile game studio"#,
        r#"<meta property="og:title" content="Kometa.Games">"#,
        r#"<meta property="og:url" content="https://YOUR-GITHUB-USER.github.io/YOUR-REPO-NAME/">"#,
        r##"<meta name="theme-color" content="#3296ed">"##,
        "data-image-lightbox",
        "image-lightbox.js",
        r#"alt="Alec Monopoly pop art print in the Kometa office""#,
    ] {
        assert!(html.contains(needle), "expected {needle:?} in kometa index.html");
    }
}

#[test]
fn generate_site_written_html_matches_golden() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip written golden parity: Template not present");
        return;
    };

    let ws = workspace_root();
    let golden_root = golden_root();

    for case in WRITTEN_GOLDEN_CASES {
        let site_dir = ws.join(case.bundle_rel);
        if !site_dir.join("site.json").is_file() {
            eprintln!("skip {}: bundle not present", case.bundle_rel);
            continue;
        }

        let root = tempfile::tempdir().unwrap();
        let project = root.path();
        copy_dir_recursive(&template_dir, &project.join("Template")).unwrap();
        copy_dir_recursive(&site_dir, &project.join(case.bundle_rel)).unwrap();

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let out_dir = generate_site(project, case.bundle_rel, false, &mut stdout, &mut stderr)
            .unwrap_or_else(|e| panic!("generate_site {}: {e}", case.bundle_rel));

        let written_path = Path::new(&out_dir).join(case.output_rel);
        let actual_raw = fs::read_to_string(&written_path).unwrap_or_else(|e| {
            panic!(
                "read written {}: {e}",
                written_path.display()
            )
        });

        let golden_path = golden_root.join(case.golden_rel);
        let golden_raw = fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("read golden {}: {e}", golden_path.display()));

        assert_written_golden_parity(&golden_path, &golden_raw, &actual_raw);
    }
}
