# `portfoliowebsitebuilder-core`

Rust library for loading, validating, and rendering portfolio content bundles. Replaces the Go engine in [Specs/tauri/requirements.md](../../Specs/tauri/requirements.md) Phase 1.

## Template engine: **Minijinja**

**Decision:** use [Minijinja](https://github.com/mitsuhiko/minijinja) (not Askama).

**Rationale:**

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
