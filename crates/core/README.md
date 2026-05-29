<<<<<<< Updated upstream
# `portfoliowebsitebuilder-core`

Rust library for loading, validating, and rendering portfolio content bundles. Replaces the Go engine in [Specs/tauri/requirements.md](../../Specs/tauri/requirements.md) Phase 1.

## Template engine: **Minijinja**
=======
# `core` — site bundle load, validate, and render

Rust library for the Portfolio Website Builder engine (Tauri migration). It mirrors the Go packages in the repository root: config loading, routing, asset references, template render, and widget tree.

## Template engine: Minijinja
>>>>>>> Stashed changes

**Decision:** use [Minijinja](https://github.com/mitsuhiko/minijinja) (not Askama).

**Rationale:**

<<<<<<< Updated upstream
| Criterion | Minijinja | Askama |
|-----------|-----------|--------|
| Existing `Template/*.html` | Go templates use `{{.Field}}` / `{{range}}` — close to Jinja; widgets can be ported with mechanical syntax tweaks (`define` → `{% macro %}` or includes). | Requires rewriting every partial as compile-time Rust structs and `.html` Askama syntax — high churn for ~15 widget files. |
| Runtime vs compile-time | Loads `Template/layout.html` and `Template/widgets/*.html` at runtime like Go — matches current deployment and author workflow. | Compile-time only; changing a widget requires rebuild; harder for agents to iterate on HTML. |
| Parity testing | Same files on disk as Go; golden HTML in `tests/golden/` stays the source of truth. | Generated Rust types must stay in sync with JSON props manually. |

Askama remains a valid choice for **new** typed fragments later; the v1 port keeps disk templates under `Template/` unchanged unless a syntax fix is required.

## Escaping policy (parity with Go `html/template`)

Go uses contextual auto-escaping (`template.URL`, `template.CSS`, `template.HTML`, etc.). Rust must reproduce the same safety boundaries:

| Context | Go type / behavior | Rust (Minijinja) |
|---------|-------------------|------------------|
| Plain text / attributes | auto-escaped string | default `{{ var }}` escaping |
| `<style>` CSS variables, font stacks | `template.CSS` | pass via explicit **safe CSS** helper or pre-validated string type; never inject raw author JSON into `<style>` |
| Stylesheet / font URLs in `<link href>` | `template.URL` | URL-escape or validate scheme (`https:`, relative path) before `| safe` |
| Widget body HTML | `template.HTML` (trusted pipeline output) | only markup produced by our widget renderer — `| safe` after generation |
| `site-widgets-config` JSON in `<script>` | `template.HTML` from `json.Marshal` | serialize JSON in Rust, embed in `<script type="application/json">` with HTML-escaped delimiters or use a dedicated JSON-for-script helper (no `</script>` injection) |

**Post-render check:** scan full page HTML for the substring `ZgotmplZ` (Go’s marker when a string is unsafe in CSS/script context). If present, fail the build with page path and route — same as [`render.go`](../../render.go).

Author-facing URLs in JSON (nav, store links, social) are validated at load time where Go already warns; rendering must not bypass escaping for convenience.

## Public API surface (Phase 1 target)

Planned crate root exports (names may be `pub use` from submodules):

```rust
/// Load `site.json`, `pages/*.json`, and bundle metadata from `content/<site-id>/`.
pub fn load_site_bundle(project_root: &Path, site_dir: &Path) -> Result<SiteBundle, Error>;

/// Validate bundle without writing output (equivalent to `go run . --validate`).
pub fn validate_site_bundle(bundle: &SiteBundle, template_dir: &Path, strict: bool) -> Result<Vec<ConfigWarning>, Error>;

/// Render all routes into `target_dir` (wipe + copy assets + write HTML).
pub fn render_site_bundle(
    bundle: &SiteBundle,
    target_dir: &Path,
    template_dir: &Path,
) -> Result<Vec<ConfigWarning>, Error>;
```

```rust
/// Non-fatal issues (unknown keys, empty store URLs, legacy fields).
pub struct ConfigWarning {
    pub path: String,
    pub message: String,
}
```

Submodules (see requirements §4 mapping): `config`, `bundles`, `strict`, `assets`, `routing`, `render`, `widgets`, `fs_util`, `serve`.

## Toolchain

Rust **1.77+**, edition **2021** (workspace `rust-version` in root `Cargo.toml`).
=======
- Existing templates live as `Template/layout.html` and `Template/widgets/*.html` with Jinja-like syntax (`{% %}`, `{{ }}`, includes/partials). Minijinja targets that model directly.
- Widget partials and nested includes match the Go + `html/template` layout structure with less rewrite than Askama’s compile-time templates.
- Minijinja supports auto-escaping modes and custom filters, which maps to Go’s context-aware escaping for HTML, URLs, and CSS.

Askama remains a poor fit for dozens of dynamic widget partials loaded from disk without a heavy code-generation step.

## Escaping policy (parity with Go `html/template`)

Go uses `html/template` with typed contexts (`template.URL`, `template.CSS`, etc.) and rejects unsafe substitutions (marker `ZgotmplZ` in output). Rust must preserve the same safety boundaries.

| Context | Policy |
|---------|--------|
| **Text / HTML body** | Auto-escape `&`, `<`, `>`, `"`, `'` in user and config strings (titles, labels, paragraphs, alt text). |
| **URLs** (`href`, `src`, store links, social links) | Emit only validated URLs; escape for HTML attribute context. Do not pass raw strings into `javascript:` or event-handler attributes. Use a dedicated URL type or filter (Minijinja `| e` in attribute context plus URL validation at render-data build time). |
| **CSS theme variables** (`<style>` custom properties, gradients) | Treat as CSS: allow only values produced by the theme builder (same allowlist as Go `template.CSS`). Never interpolate arbitrary JSON strings into `<style>`. |
| **JSON in `<script>`** (e.g. widget config export) | Serialize with a JSON encoder and embed via a **safe script** pattern (JSON bytes in script block, no `</script>` injection). Do not HTML-escape JSON inside script; do escape when the same data appears in HTML attributes. |
| **Raw HTML blocks** | Not exposed to site JSON; widgets render structured fields only. |

After render, run the same **unsafe marker check** as Go (`render.go`): if output contains `ZgotmplZ`, fail with page path and route context.

## Public API (planned)

Surface for `crates/cli` and Tauri commands (names align with Go):

| Function / type | Role |
|-----------------|------|
| `load_site_bundle(site_dir)` | Load `site.json`, `pages/*.json`, resolve paths; return `SiteBundle` + config warnings. |
| `validate_site_bundle(bundle, template_dir)` | Dry-run: routing, assets, template/widget resolution, render without writing files; accumulate warnings. |
| `render_site_bundle(bundle, target_dir, template_dir)` | Full generate: assets + HTML per route (empty `target_dir` = dry-run only). |
| `ConfigWarning` | Path + message (+ optional JSON pointer); serializable for `--strict` and UI Problems panel. |
| `discover_content_bundles(project_root)` | List `content/*` bundles (CLI `--list-sites`). |
| `resolve_site_dir` / `resolve_project_root` | Same resolution rules as Go CLI. |

Strict mode: warnings promoted to errors via a shared helper (`enforce_strict_warnings` equivalent).

## Tests

Golden HTML under `tests/golden/`; see [tests/README.md](./tests/README.md) for normalization rules used in Phase 1.4 parity tests.
>>>>>>> Stashed changes
