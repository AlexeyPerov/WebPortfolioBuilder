//! Phase 0: ensure Go baseline golden HTML is present (Phase 1.4 adds content parity).

use std::path::PathBuf;

fn golden_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

const EXPECTED: &[&str] = &[
    "kometa/index.html",
    "demo/index.html",
    "demo/about/index.html",
    "demo/apps/index.html",
    "my-studio/index.html",
];

#[test]
fn golden_html_fixtures_exist_and_non_empty() {
    let root = golden_root();
    for rel in EXPECTED {
        let path = root.join(rel);
        assert!(path.is_file(), "missing golden fixture: {}", path.display());
        let meta = std::fs::metadata(&path).expect("metadata");
        assert!(meta.len() > 0, "empty golden fixture: {}", path.display());
    }
}
