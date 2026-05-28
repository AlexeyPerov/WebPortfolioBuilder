# Portfolio Website Builder

Static site generator in Go (**PortfolioWebsiteBuilder**). The supported workflow is a **content bundle** under `content/<site-id>/`: metadata in `site.json`, one or more page JSON files under `pages/`, and static files under `assets/`. Running the CLI renders HTML into the path set by `output_folder` in `site.json` (relative to the project root) using [`Template/layout.html`](Template/layout.html), widget partials in [`Template/widgets/`](Template/widgets/), and shared CSS/JS copied from [`Template/`](Template/).

**Normative behavior** is documented in [Specs/ImplementationSpec.md](Specs/ImplementationSpec.md). Widget props and parity notes live in [Specs/WidgetRegistryV1.md](Specs/WidgetRegistryV1.md).

## Prerequisites

- [Go](https://go.dev/dl/) 1.20+ installed.
- Clone this repo and work from its root (module path [`portfoliowebsitebuilder`](go.mod)).

## Content bundle layout

Each site directory contains:

| Path | Purpose |
|------|---------|
| `site.json` | `site_id`, `output_folder`, `theme`, `typography`, `store_icons`, `subscribe_block`, `social`, `header`, `footer`, optional `widgets` tuning (`scroll_reveal` / **`carousel`** / `split_widget`). |
| `pages/*.json` | Page configs: `slug`, `widgets` tree, optional `title`, `seo`, `hero`, `layout`. |
| `assets/` | All local images/icons referenced from JSON (paths must start with `assets/`). |

The sample Kometa bundle is [`content/kometa/`](content/kometa/). Generated output goes to [`Results/`](Results/) (gitignored).

## Run the generator

From the repo root:

```bash
go run .
```

### CLI flags

| Flag | Description |
|------|-------------|
| `--site <path>` | Content bundle path (relative to project root or absolute). Skips the interactive bundle prompt. |
| `--validate` | Load and validate the bundle (JSON schema, asset references, render dry-run). Prints warnings; exits non-zero on error. Does **not** write or wipe output. |
| `--list-sites` | Print one content bundle path per line (directories under `content/` with a valid `site.json`) and exit. |

Examples:

```bash
# Non-interactive build (CI-friendly)
go run . --site content/kometa

# Validate without generating output
go run . --validate --site content/kometa

# List available bundles
go run . --list-sites
```

### Interactive mode

When no flags are passed, the generator prompts for a bundle path:

**Content bundle path** — press Enter for the default `./content/kometa` when only one bundle exists; when multiple bundles exist under `content/`, an empty line shows a numbered list to pick from. Type `?` at any time to list bundles. Paths are resolved under the **project root** (see below). You may also type another path relative to that project root, or an absolute path.

The tool picks the project root by: using the **current working directory** if `content/kometa/site.json` exists there; otherwise **the folder containing the running executable** when that folder contains `content/kometa/site.json` (so a binary placed in the repo root still works when you invoke it from `~`).

Example (default bundle → `Results/KometaWebsite/`):

```bash
go run . --site content/kometa
```

Or with the interactive prompt:

```bash
printf '\n' | go run .
```

The program writes to `<project-root>/<output_folder>/` using `output_folder` from `site.json`. Each run **clears** the target output directory, copies non-HTML assets from [`Template/`](Template/) (CSS/JS/fonts, etc.), copies every referenced bundle asset, then renders all routes.

Open the generated `index.html` in a browser.

## Root `config.json` (legacy sample only)

[`config.json`](config.json) is a **single-file legacy sample** reflecting the older monolithic schema. The supported pipeline is **`content/`** bundles. Field names mirror the renamed catalog vocabulary (`apps`, `store_icons`, `subscribe_block`, theme `catalog_gradient`, content keys such as `apps_title` / `nav_apps`). It is handy for parity checks but **not** read by `go run .` today.

## Widget tuning (`widgets` in `site.json`)

Optional; omitted branches keep built-in defaults. Values are injected as JSON in the page (`site-widgets-config`):

- **`scroll_reveal`** — `respect_reduced_motion` (default `true`; when enabled and the user prefers reduced motion, sections stay visible with no animation), `root_margin`, `threshold` (IntersectionObserver). Add **`scroll-reveal--immediate`** on a section class list (e.g. intro hero) to keep above-the-fold content visible on first paint without waiting for the observer. With JavaScript disabled, [`Template/layout.html`](Template/layout.html) noscript styles force all `.scroll-reveal` sections visible.
- **`carousel`** — **`swipe_threshold_px`** minimum horizontal swipe to advance screenshot carousels; **`keyboard_navigation`** (default `true`) enables Left/Right arrow keys when the carousel is focused, a visible slide counter (e.g. `2 / 5`), and an `aria-live` announcement on slide change (used by [`Template/catalog-carousel.js`](Template/catalog-carousel.js)).
- **`split_widget`** — `keyboard_navigation` for tabbed vacancy UI.

See [Specs/ImplementationSpec.md](Specs/ImplementationSpec.md) for full semantics.

## Store icons and app cards (`apps_showcase`)

- **`store_icons`** in `site.json`: map of preset keys (e.g. `google_play`, `app_store`) to image paths under `assets/`.
- **`subscribe_block`**: optional title + outbound links surfaced on catalog cards when links have URLs.

Per-app payloads use `CatalogApp`-shaped JSON (`apps` prop): icon and header imagery, swiper slides, legacy storefront URLs and/or **`store_links`** with `icon` / `icon_image` / `aria_label`.

## Social links (`social`)

Either legacy flat URLs (`github_url`, `linkedin_url`, `facebook_url`) or **`social.links`** as an explicit ordered array. Entries need a resolving `url` plus a built-in `icon` preset or `icon_image` under `assets/`.

## Development

```bash
gofmt -w .
go test ./...
go build ./...
```

## Forbidden-string check (contributors)

Task **16** in [Specs/ExecutionPlan.md](Specs/ExecutionPlan.md) defines the forbidden branding substrings and a sample `rg` invocation (excluding `Results/` preview output). Run that check after edits to docs or generator output paths.
