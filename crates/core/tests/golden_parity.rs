//! Golden HTML parity: Rust render vs Go baseline (Phase 1.4 / Task 12).

mod common;

use common::{golden_root, normalize_html_for_test, template_dir, workspace_root};
use portfoliowebsitebuilder_core::{
    load_site_bundle, render_site_bundle_html, HTML_TEMPLATE_FAILURE_MARKER,
};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

struct GoldenCase {
    bundle_rel: &'static str,
    output_rel: &'static str,
    golden_rel: &'static str,
}

const CASES: &[GoldenCase] = &[
    GoldenCase {
        bundle_rel: "content/kometa",
        output_rel: "index.html",
        golden_rel: "kometa/index.html",
    },
    GoldenCase {
        bundle_rel: "content/demo",
        output_rel: "index.html",
        golden_rel: "demo/index.html",
    },
    GoldenCase {
        bundle_rel: "content/demo",
        output_rel: "about/index.html",
        golden_rel: "demo/about/index.html",
    },
    GoldenCase {
        bundle_rel: "content/demo",
        output_rel: "apps/index.html",
        golden_rel: "demo/apps/index.html",
    },
    GoldenCase {
        bundle_rel: "content/my-studio",
        output_rel: "index.html",
        golden_rel: "my-studio/index.html",
    },
];

fn assert_golden_parity(golden_path: &Path, golden_raw: &str, actual_raw: &str) {
    assert!(
        !actual_raw.contains(HTML_TEMPLATE_FAILURE_MARKER),
        "rendered HTML contains forbidden marker {:?} (golden: {})",
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
        "golden HTML mismatch: {}\n\n--- unified diff (expected golden → actual) ---\n",
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
fn golden_html_parity_regression_bundles() {
    let Some(template_dir) = template_dir() else {
        eprintln!("skip golden parity: Template not present");
        return;
    };

    let root = workspace_root();
    let golden_root = golden_root();

    for case in CASES {
        let site_dir = root.join(case.bundle_rel);
        if !site_dir.join("site.json").is_file() {
            eprintln!("skip {}: bundle not present", case.bundle_rel);
            continue;
        }

        let golden_path = golden_root.join(case.golden_rel);
        let golden_raw = fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("read golden {}: {e}", golden_path.display()));

        let (bundle, _warnings) = load_site_bundle(&site_dir).unwrap_or_else(|e| {
            panic!("load bundle {}: {e}", site_dir.display());
        });
        let (pages, _render_warnings) = render_site_bundle_html(&bundle, &template_dir)
            .unwrap_or_else(|e| panic!("render {}: {e}", case.bundle_rel));

        let actual_raw = pages.get(case.output_rel).unwrap_or_else(|| {
            panic!(
                "render {}: missing output page {:?}; available: {:?}",
                case.bundle_rel,
                case.output_rel,
                pages.keys().collect::<Vec<_>>()
            );
        });

        assert_golden_parity(&golden_path, &golden_raw, actual_raw);
    }
}
