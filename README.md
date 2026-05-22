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

Interactive prompt ([ImplementationSpec §10](Specs/ImplementationSpec.md)):

**Content bundle path** — press Enter for the default `./content/kometa` (resolved under the **project root**). The tool picks the project root by: using the **current working directory** if `content/kometa/site.json` exists there; otherwise **the folder containing the running executable** when that folder contains `content/kometa/site.json` (so a binary placed in the repo root still works when you invoke it from `~`). You may also type another path relative to that project root, or an absolute path.

The program writes to `<project-root>/<output_folder>/` using `output_folder` from `site.json` (for example `Results/KometaWebsite_2`).

Each run **clears** the target output directory, copies non-HTML assets from [`Template/`](Template/) (CSS/JS/fonts, etc.), copies every referenced bundle asset, then renders all routes.

Example (default bundle → `Results/KometaWebsite_2/`):

```bash
printf '\n' | go run .
```

Open the generated `index.html` in a browser.

## Root `config.json` (legacy sample only)

[`config.json`](config.json) is a **single-file legacy sample** reflecting the older monolithic schema. The supported pipeline is **`content/`** bundles. Field names mirror the renamed catalog vocabulary (`apps`, `store_icons`, `subscribe_block`, theme `catalog_gradient`, content keys such as `apps_title` / `nav_apps`). It is handy for parity checks but **not** read by `go run .` today.

## Widget tuning (`widgets` in `site.json`)

Optional; omitted branches keep built-in defaults. Values are injected as JSON in the page (`site-widgets-config`):

- **`scroll_reveal`** — `respect_reduced_motion`, `root_margin`, `threshold` (IntersectionObserver).
- **`carousel`** — **`swipe_threshold_px`** minimum horizontal swipe to advance screenshot carousels (used by [`Template/catalog-carousel.js`](Template/catalog-carousel.js)).
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
