# Architecture

How Portfolio Web Builder turns JSON content into static websites, and where each piece of the repo fits.

## Overview

Portfolio Web Builder is a **static site generator** with a **desktop studio** editor. Authors maintain one folder per site under `content/`. Each site has `site.json` (global settings), `pages/*.json` (page definitions with widget trees), and `assets/` (images). A shared `Template/` directory supplies HTML layouts, widget partials, CSS, and JavaScript.

The **core engine** (`crates/core`) loads and validates bundles, renders HTML with [Minijinja](https://github.com/mitsuhiko/minijinja), copies assets, and writes output to the path in `site.json` → `output_folder` (typically under `Results/`).

The **studio** (`studio/` + `src-tauri/`) is a Tauri desktop app: Svelte UI, Rust invoke commands, in-process calls to the core crate. Authors edit JSON (or form panels), validate, build, and preview over HTTP on `127.0.0.1`.

The **CLI** (`crates/cli`) exposes the same validate/build pipeline for scripts and CI.

## Repository layout

```
content/                  # Author bundles (one subfolder per site)
  <site-id>/
    site.json             # Theme, header, footer, social, output path
    pages/*.json          # One file per page (filename ≠ URL slug)
    assets/               # Site-local images and icons
  _template/              # Copy source for “New site” in the studio

Template/                 # Shared render templates (all sites)
  layout.html             # Page shell: head, header, main, footer, scripts
  widgets/*.html          # One Minijinja partial per widget type
  styles.css              # Shared site styles
  *.js                    # Page scripts (carousel, tabs, scroll reveal, lightbox, bg effects)

AGENTS.md                 # Agent maintenance rules (repo root)

docs/
  architecture.md         # This file
  widgets.md              # Widget reference for authors and contributors
  schema/
    site.schema.json      # JSON Schema for site.json (studio AJV lint)
    page.schema.json      # JSON Schema for pages/*.json

crates/
  core/                   # Load, validate, render, serve
  cli/                    # Command-line entry point

studio/                   # Svelte 5 + Vite UI
src-tauri/                # Tauri 2 shell and invoke commands

Results/                  # Default build output root (per site output_folder)
Specs/                    # Implementation changelog and active plans
```

**Project root** is the directory that contains `content/`, `Template/`, and `docs/schema/`. The studio and CLI resolve it from the current working directory or the packaged app location.

## End-to-end data flow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ content/<site>/ │     │  crates/core     │     │ output_folder/  │
│  site.json      │────▶│  load → validate │────▶│  index.html     │
│  pages/*.json   │     │  render → copy   │     │  <slug>/index…  │
│  assets/        │     │                  │     │  assets/ …      │
└─────────────────┘     └────────┬─────────┘     └─────────────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Template/        │
                        │  layout.html     │
                        │  widgets/*.html  │
                        │  styles.css, JS  │
                        └──────────────────┘
```

### Build pipeline (core)

1. **Discover / resolve** — `discover_content_bundles`, `resolve_site_dir` find `content/<site-id>/`.
2. **Load** — `load_site_bundle` reads `site.json` and all `pages/*.json` into a `SiteBundle`.
3. **Route index** — `build_route_index` maps each page’s `slug` to output paths (`""` → `index.html`, `"about"` → `about/index.html`).
4. **Validate** — asset references, nav slugs, widget props, strict-mode unknown keys; warnings vs errors per policy.
5. **Render** — for each page:
   - Walk the widget tree (`render_widget_tree` in `widgets.rs`).
   - Load Minijinja templates from `Template/widgets/`.
   - Assemble page data (theme CSS vars, header/footer, SEO, script needs).
   - Render `Template/layout.html`.
   - Fail if rendered HTML contains the unsafe-substitution marker (`ZgotmplZ` parity check).
6. **Write** — HTML files, copy referenced assets, copy static Template files (CSS, JS, fonts).

**Validate-only** runs steps 2–4 (and render checks without writing). **Build** runs the full pipeline.

## Application layers

| Layer | Location | Role |
|-------|----------|------|
| Studio UI | `studio/src/` | File tree, JSON/form editors, preview iframe, build log |
| Tauri shell | `src-tauri/src/` | Invoke commands, preview HTTP server, settings persistence |
| Core engine | `crates/core/src/` | Config loading, routing, widgets, HTML generation |
| CLI | `crates/cli/src/main.rs` | `--validate`, build, `--serve`, `--list-sites` |
| Templates | `Template/` | Presentation; edited when changing widget HTML/CSS/JS |
| Schemas | `docs/schema/` | Editor autocomplete and AJV validation in the studio |

### Studio → core path

The studio saves editor buffers via `write_bundle_file_cmd`, then calls `validate_site` or `build_site` in `src-tauri/src/site_ops.rs`, which delegates to `portfoliowebsitebuilder_core::validate_site` / `generate_site`.

Preview uses `start_preview_server` to serve the last build output over HTTP (not `file://` URLs).

Key invoke commands are documented in [`studio/README.md`](../studio/README.md).

### Widget rendering

- Widget types are a **closed registry** (see [`widgets.md`](./widgets.md)).
- Only layout widgets (`row`, `column`, `grid`) may contain `children`.
- Rust builds HTML for each widget in `crates/core/src/widgets.rs` (data prep + Minijinja or inline HTML helpers in `html.rs`).
- Page-level JS is included based on `PageScriptNeeds` (carousel, split-widget tabs, reference-panel tabs/dropdown, scroll reveal, lightbox). `reference_panel` loads `Template/reference-panel.js`.
- Optional `layout.background_effect` (`magic_dust`) injects a fixed canvas back-layer in `layout.html` and loads `background-effects.js`. When active, `body.has-bg-effect` keeps section panels transparent so the animation stays visible behind widgets (cards and media keep their own surfaces). Critical positioning CSS is inlined in the layout `<head>` so the canvas never participates in document flow before `styles.css` loads.

Template engine choice and escaping policy: [`crates/core/README.md`](../crates/core/README.md).

## Content model

### `site.json`

Site-wide configuration: identity, `output_folder`, `base_url`, `favicon`, `theme` (CSS custom properties), `typography`, `header` (brand + nav), `footer`, `social`, `store_icons`, optional `subscribe_block` and widget-level defaults.

Schema: [`docs/schema/site.schema.json`](./schema/site.schema.json).

### `pages/*.json`

Each file defines one page:

| Field | Purpose |
|-------|---------|
| `slug` | URL segment; `""` is the site root |
| `title` | Document title |
| `seo` | `description`, `og_image`, `canonical_url` |
| `layout` | `hide_header`, `hide_footer`, optional `background_effect` (`magic_dust`), optional `compact_sections` (halves section vertical padding) |
| `widgets` | Ordered top-level widget list |

Schema: [`docs/schema/page.schema.json`](./schema/page.schema.json).

The **filename** under `pages/` (e.g. `home.json`) is an authoring id only; routing uses `slug`.

### Assets

Paths in JSON are relative to the site’s `assets/` folder (or site root as documented per field). The build copies referenced assets into the output tree.

## Example sites

| Bundle | Purpose |
|--------|---------|
| `content/demo/` | Every widget type across multiple pages (always in repo) |
| `content/alexeyperov-io/` | Production author site in this repo |
| `content/my-studio/` | Studio dev / golden-test bundle |
| `content/_template/` | Starter bundle for new sites in the studio |
| `content/kometa/` | Full sample portfolio (local only — gitignored; used in CI when present) |

## Development entry points

| Task | Command (from repo root) |
|------|--------------------------|
| Desktop app (dev) | `cargo tauri dev` |
| Frontend only | `npm run dev --prefix studio` |
| CLI validate | `cargo run -p portfoliowebsitebuilder-cli -- --validate --site content/demo` |
| CLI build | `cargo run -p portfoliowebsitebuilder-cli -- --site content/demo` |
| Core tests | `cargo test -p portfoliowebsitebuilder-core` |
| Release build | `cargo tauri build` |

Full studio setup and platform notes: [`studio/README.md`](../studio/README.md).

## What to edit for common changes

| Change | Touch |
|--------|--------|
| New widget type | `crates/core/src/widgets.rs`, `Template/widgets/`, `docs/schema/page.schema.json`, studio form (if exposed), [`widgets.md`](./widgets.md) |
| Widget HTML/CSS | `Template/widgets/*.html`, `Template/styles.css` |
| Site theme tokens | Author `site.json` → `theme`; defaults/labels in schema + studio theme form |
| Studio UX | `studio/src/` |
| Build/validate behavior | `crates/core/` |
| Author docs | [`README.md`](../README.md), [`widgets.md`](./widgets.md) |

## Related docs

- [`widgets.md`](./widgets.md) — widget types, props, examples
- [`AGENTS.md`](../AGENTS.md) — rules for AI agents and doc maintenance
- [`README.md`](../README.md) — install, run, author quick start
- [`Specs/Changelog.md`](../Specs/Changelog.md) — implementation history
