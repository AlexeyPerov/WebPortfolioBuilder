# Golden HTML fixtures (Go baseline)

These files are **frozen snapshots** of HTML produced by the Go generator (`go run . --site content/<bundle>`). They are the parity baseline for [Phase 1.4](../../Specs/tauri/execution-plan-phase-1.md) and later Rust `render_site_bundle` output.

## Bundles and paths

| Bundle ID | Source | Golden paths |
|-----------|--------|--------------|
| `kometa` | `content/kometa` | `golden/kometa/index.html` (home) |
| `demo` | `content/demo` | `golden/demo/index.html` (home), `golden/demo/about/index.html` (nested slug), `golden/demo/apps/index.html` (heavy widgets: `apps_showcase`, carousels) |
| `my-studio` | `content/my-studio` | `golden/my-studio/index.html` (home) |

Regenerate from repo root after intentional Go output changes:

```bash
go run . --site content/kometa
go run . --site content/demo
go run . --site content/my-studio
# then copy representative index.html files into golden/<bundle-id>/ (see table above)
```

## Phase 1.4 comparison

Integration tests will:

1. Run Rust render for the same bundle (dry-run or in-memory HTML string).
2. Normalize Rust and golden bytes (see below).
3. `assert_eq!` normalized strings, or use a unified diff helper with a clear failure message listing the golden path.

Do **not** require raw byte identity without normalization — template engines and whitespace may differ slightly while remaining semantically equivalent.

## Normalization rules

Apply **in order** to both golden and candidate HTML before comparison:

1. **Line endings** — normalize `\r\n` to `\n`.
2. **Trailing whitespace** — strip trailing spaces and tabs on each line.
3. **Trailing blank lines** — remove empty lines at end of file; ensure file ends with a single `\n`.
4. **Repeated blank lines** — collapse three or more consecutive newlines to two (preserves intentional paragraph breaks in `<pre>` only if needed; for layout HTML, double newline is enough).
5. **Leading/trailing file whitespace** — trim only if the document still starts with `<!DOCTYPE` or `<html` after trim; otherwise fail the test (corrupt output).

**Not normalized** (differences fail the test unless golden is updated):

- Attribute order within a tag
- Quote style (`"` vs `'`)
- Insignificant whitespace between tags (e.g. `><` vs `> <`) — if Rust/Minijinja diverges here, extend normalization in Phase 1.4 with an explicit rule and document it here
- Substantive text, `href`, `src`, `id`, or `class` values

**Forbidden in output** (hard fail, no normalization): the Go `html/template` unsafe marker `ZgotmplZ` must not appear in rendered HTML (see `render.go`).

## Go parity notes (Rust renderer)

The Rust renderer matches these Go `html/template` behaviors for golden comparison:

- Stylesheet `href`: `|` → `%7c`, `&` → `&amp;` (see `escape_stylesheet_href` in `html.rs`).
- Text nodes in templates: `+` → `&#43;`, apostrophe → `&#39;` (see `go_template_text_escape` / `configure_minijinja_html_escape`).
- Widget join spacing mirrors Go `renderWidgetTree` (extra blank line when a multiline widget precedes another multiline or a single-line widget).

## Attribute ordering (policy)

Golden files were captured from Go `html/template` as-is. Phase 1 should match Go output; if Minijinja emits the same attributes in a different order, either:

- fix templates/helpers to match Go order, or
- add an optional **stable attribute sort** step for comparison only (document the scope here before enabling).

Default for Phase 1.4: **no attribute reordering** — fix the renderer to match Go.
