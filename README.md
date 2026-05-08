# Portfolio Website Builder

Static site generator in Go (**PortfolioWebsiteBuilder**). The supported workflow is a **site bundle** under `sites/<site-id>/`: metadata in `site.json`, one or more page JSON files under `pages/`, and static files under `assets/`. Running the CLI renders HTML into the configured output folder (relative to the destination you choose) using [`Template/layout.html`](Template/layout.html), widget partials in [`Template/widgets/`](Template/widgets/), and shared CSS/JS copied from [`Template/`](Template/).

**Normative behavior** is documented in [Specs/ImplementationSpec.md](Specs/ImplementationSpec.md). Widget props and parity notes live in [Specs/WidgetRegistryV1.md](Specs/WidgetRegistryV1.md).

## Prerequisites

- [Go](https://go.dev/dl/) 1.20+ installed.
- Clone this repo and work from its root (module path [`portfoliowebsitebuilder`](go.mod)).

## Site bundle layout

Each site directory contains:

| Path | Purpose |
|------|---------|
| `site.json` | `site_id`, `output_folder`, `theme`, `typography`, `store_icons`, `subscribe_block`, `social`, `header`, `footer`, optional `widgets` tuning (`scroll_reveal` / **`carousel`** / `split_widget`). |
| `pages/*.json` | Page configs: `slug`, `widgets` tree, optional `title`, `seo`, `hero`, `layout`. |
| `assets/` | All local images/icons referenced from JSON (paths must start with `assets/`). |

The sample Kometa bundle is [`sites/kometa/`](sites/kometa/). Regenerated demo output matching that bundle is committed under [`KometaWebsite/`](KometaWebsite/).

## Run the generator

From the repo root:

```bash
go run .
```

Interactive prompts ([ImplementationSpec §10](Specs/ImplementationSpec.md)):

1. **Site bundle path** — press Enter for the default `./sites/kometa` (or enter another path relative to the project root, or an absolute path).
2. **Destination directory** — press Enter to use the **project root** as the parent of the output folder; the program writes into `<destination>/<output_folder>/` using `output_folder` from `site.json`.

Each run **clears** the target output directory, copies non-HTML assets from [`Template/`](Template/) (CSS/JS/fonts, etc.), copies every referenced bundle asset, then renders all routes.

Example (defaults → `./KometaWebsite/`):

```bash
printf '\n\n' | go run .
```

Open the generated `index.html` in a browser.

## Root `config.json` (legacy sample only)

[`config.json`](config.json) is a **single-file legacy sample** reflecting the older monolithic schema. The supported pipeline is **`sites/`** bundles. Field names mirror the renamed catalog vocabulary (`apps`, `store_icons`, `subscribe_block`, theme `catalog_gradient`, content keys such as `apps_title` / `nav_apps`). It is handy for parity checks but **not** read by `go run .` today.

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

Task **16** in [Specs/ExecutionPlan.md](Specs/ExecutionPlan.md) defines the forbidden branding substrings and a sample `rg` invocation (excluding `KometaWebsite/` preview output). Run that check after edits to docs or generator output paths.
