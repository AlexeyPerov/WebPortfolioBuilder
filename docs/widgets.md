# Widget reference

Pages are built from an ordered list of **widgets**. Each widget node has a `type`, optional `id` and `enabled`, and a `props` object.

**Authoritative prop shapes:** [`docs/schema/page.schema.json`](./schema/page.schema.json) (used by the studio JSON editor and AJV). This document is the human-readable guide; when they differ, trust the schema and update this file.

**Live examples:** `content/demo/pages/` — see [`content/demo/site.json`](../content/demo/site.json) nav for page → widget mapping.

**Templates:** one Minijinja partial per type under `Template/widgets/`.

## Global rules

- Unknown widget `type` → **build error** with a path to the node.
- `enabled: false` skips rendering (node stays in JSON).
- `id` (optional) sets a stable DOM id on the widget root. For `images_grid`, this becomes the `<section>` id (default `photos` when omitted). Other widgets use it on their root element when supported.
- User-facing strings are HTML-escaped; authors do not inject raw HTML in v1.
- Only **layout widgets** may contain `children` (nested widget arrays).
- Canonical layout type is **`column`** (not `columns` — rejected by the generator).

### Widget node shape

```json
{
  "type": "intro",
  "id": "page-intro",
  "enabled": true,
  "props": { }
}
```

## Summary

| `type` | Category | Role |
|--------|----------|------|
| `intro` | Content | Page heading and paragraphs |
| `cover_banner` | Content | Full-width hero image band |
| `apps_showcase` | Content | Rich app/product cards with store links |
| `info_grid` | Content | Grid of small info cards |
| `images_grid` | Content | Photo / image gallery |
| `careers_tabs` | Content | Job listings in tabbed panels |
| `follow_us` | Content | Social link button row |
| `project_grid` | Content | Portfolio / tools card grid |
| `media_swiper` | Content | Image carousel |
| `reference_panel` | Content | API/MCP reference with sidebar tabs and detail panels |
| `row` | Layout | Horizontal group of child widgets |
| `column` | Layout | Vertical stack of child widgets |
| `grid` | Layout | Responsive CSS grid of child widgets |

### Demo page map

| Page file | Widgets showcased |
|-----------|-------------------|
| `content/demo/pages/home.json` | `intro`, `cover_banner`, `follow_us` |
| `content/demo/pages/layouts.json` | `row`, `column`, `grid` |
| `content/demo/pages/gallery.json` | `images_grid`, `media_swiper` |
| `content/demo/pages/apps.json` | `apps_showcase`, `project_grid` |
| `content/demo/pages/careers.json` | `careers_tabs`, `info_grid` |
| `content/demo/pages/about.json` | Mixed composition, `reference_panel` |

---

## Layout widgets

Only these types accept `props.children`: a non-empty array of widget nodes. Layouts may nest.

### `row`

| Prop | Required | Notes |
|------|----------|-------|
| `children` | yes | Non-empty widget array |
| `gap` | no | CSS length or token between children |

Template: `Template/widgets/row.html`

### `column`

| Prop | Required | Notes |
|------|----------|-------|
| `children` | yes | Non-empty widget array |

Template: `Template/widgets/column.html`

### `grid`

| Prop | Required | Default | Notes |
|------|----------|---------|-------|
| `children` | yes | — | Non-empty widget array |
| `min_column_width` | no | `260px` | Passed to `repeat(auto-fit, minmax(...))` |
| `gap` | no | site default | Grid gap |

Template: `Template/widgets/grid.html`

---

## Content widgets

### `intro`

Centered heading and one or more paragraphs.

| Prop | Required | Notes |
|------|----------|-------|
| `title` | no | Main heading |
| `paragraphs` | no | Array of strings; empty entries omitted |
| `cta` | no | `{ label?, url }` — pill button below paragraphs; default label `"Learn more"` when `url` is set |
| `link_buttons` | no | `{ title?, items[] }` — section heading (default `"Projects"`) plus a row of equal square link buttons; each item is `{ label, url }` |
| `pre` | no | Optional monospace block below paragraphs (plain text, rendered in `<pre>`) |

If `title`, `paragraphs`, and `pre` are all empty, the section is skipped (warning). `cta` is omitted when `url` is empty. `link_buttons` is omitted when `items` is empty or every item lacks `label`/`url`.

Template: `Template/widgets/intro.html` · Studio form: yes (`cta`, `link_buttons` via JSON / Props tab only)

### `cover_banner`

Full-width image strip, often at the top of a page.

| Prop | Required | Notes |
|------|----------|-------|
| `src` | yes | Asset path under the site bundle |
| `alt` | no | Defaults to `"Cover"` |

Template: `Template/widgets/cover_banner.html` · Studio form: yes

### `follow_us`

Section heading plus a row of social icons. Links come from `site.json` → `social` (GitHub, LinkedIn, Facebook URLs and optional custom entries).

| Prop | Required | Default | Notes |
|------|----------|---------|-------|
| `title` | no | `"Follow us"` | Section heading |

Template: `Template/widgets/follow_us.html` · Studio form: yes

### `info_grid`

Responsive grid of small cards (optional image, title, text).

| Prop | Required | Notes |
|------|----------|-------|
| `title` | no | Section `<h2>` |
| `items` | yes | Array of `{ image?, title, text }`, min length 1 |

Template: `Template/widgets/info_grid.html` · Studio form: yes

### `images_grid`

Photo wall / gallery grid.

| Prop | Required | Notes |
|------|----------|-------|
| `title` | no | Section heading |
| `images` | yes | Array of asset path strings or `{ src, alt? }`, min length 1 |

Set widget-level `id` (not a prop) to override the section anchor; defaults to `photos`.

Template: `Template/widgets/images_grid.html` · Studio form: yes

### `careers_tabs`

Roles as tabs; each panel shows requirements, responsibilities, advantages, and an apply link.

| Prop | Required | Notes |
|------|----------|-------|
| `title` | no | Section heading |
| `vacancies` | yes | Array of vacancy objects, min length 1 |
| `labels` | no | Override column titles: `requirements_title`, `responsibilities_title`, `advantages_title` |

**Vacancy object:**

| Field | Required | Notes |
|-------|----------|-------|
| `role` | yes | Tab label and heading |
| `requirements` | no | String array |
| `responsibilities` | no | String array |
| `advantages` | no | String array |
| `apply_url` | no | External apply link |
| `apply_label` | no | CTA button text |

Uses `Template/split-widget.js` for tab behavior.

Template: `Template/widgets/careers_tabs.html` · Studio form: yes

### `apps_showcase`

Vertical stack of large product cards: header image or title band, stats, body copy, screenshot swiper, store badges. Optional subscribe block from site-level `subscribe_block`.

| Prop | Required | Notes |
|------|----------|-------|
| `section_title` | no | Wrapped in section heading when set |
| `apps` | yes | Non-empty array of catalog app objects |

**Catalog app object** (common fields):

| Field | Notes |
|-------|-------|
| `image` | Required. App icon or hero |
| `header_image`, `swiper_images`, `card_background` | Optional visuals |
| `title`, `text_1`, `text_2` | Copy blocks |
| `stat_left_line_1`, `stat_left_line_2`, `stat_right_line_1`, `stat_right_line_2` | Stat lines |
| `google_play_url`, `app_store_url`, `amazon_store_url`, `galaxy_store_url` | Legacy store URLs |
| `store_links` | Preferred: `[{ url, aria_label, icon?, icon_image? }]` with `icon` in `google_play`, `app_store`, `amazon`, `galaxy` |

Store icon images resolve via `site.json` → `store_icons`.

Uses `Template/catalog-carousel.js` for swipers.

Template: `Template/widgets/apps_showcase.html` · Studio form: yes

### `project_grid`

Responsive card grid for projects, tools, or portfolio items. Static HTML only (no client-side filtering in v1).

| Prop | Required | Notes |
|------|----------|-------|
| `heading` | no | Section `<h2>` |
| `subheading` | no | Line under heading |
| `section_id` | no | HTML `id` on `<section>` for deep links |
| `min_card_column_width` | no | Default `260px` for `auto-fit` grid |
| `cards` | yes | Non-empty array of card objects |

**Card object** (required: `title`, `description`, `tags`, `meta`, `cta`):

| Field | Notes |
|-------|-------|
| `title` | Card title (plain text) |
| `description` | Short paragraph |
| `tags` | String array; rendered as non-interactive pills (may be empty) |
| `image` | Optional asset path; omit for text-only cards |
| `meta` | String **or** object of key/value pairs (secondary line) |
| `cta` | `{ label?, url }` — primary link; default label `"Learn more"` |
| `secondary_cta` | `{ label?, url }` — optional second link (e.g. GitHub next to docs) |

Template: `Template/widgets/project_grid.html` · Studio form: yes

### `media_swiper`

Generic image carousel (touch and keyboard). Not tied to app cards.

| Prop | Required | Default | Notes |
|------|----------|---------|-------|
| `images` | yes | — | Array of `{ src, alt? }`, min length 1 |
| `aria_label` | no | `"Image carousel"` | Accessible name for the carousel |

Reuses the catalog carousel script and markup contract (`data-catalog-carousel`).

Template: `Template/widgets/media_swiper.html` · Studio form: yes

### `reference_panel`

Interactive API or MCP reference: left sidebar list (desktop) or dropdown (mobile ≤900px), right detail panel with method name, signature, arguments, description, and optional code example.

| Prop | Required | Notes |
|------|----------|-------|
| `title` | no | Section `<h2>` |
| `intro` | no | Paragraph above the panel |
| `entries` | yes | Non-empty array of entry objects |

**Entry object** (required: `label`, `description`):

| Field | Notes |
|-------|-------|
| `label` | Sidebar / dropdown label |
| `method` | Display name (e.g. `UnityScannerBatch.RunAll` or MCP tool name) |
| `signature` | One-line signature |
| `description` | Body copy |
| `arguments` | `{ name, type?, description? }[]` |
| `example` | Plain text rendered in `<pre>` (not syntax-highlighted) |

Uses `Template/reference-panel.js`. Reuses split-widget tab styling on desktop.

Template: `Template/widgets/reference_panel.html` · Studio form: no (JSON only)

---

## Studio form coverage

The studio **Form** tab on `pages/*.json` includes typed editors for the nine original content widgets above. Layout widgets (`row`, `column`, `grid`) and extra props on any widget can be edited via the **Props (JSON)** block or the **JSON** tab.

Site-wide widget defaults (e.g. `widgets.split_widget` in `site.json`) remain JSON-only.

---

## Adding or changing widgets

When introducing a new widget or breaking props:

1. Update `docs/schema/page.schema.json`
2. Add renderer logic in `crates/core/src/widgets.rs`
3. Add `Template/widgets/<type>.html` (and CSS/JS if needed)
4. Update this file and [`README.md`](../README.md) widget summary
5. Add or extend examples in `content/demo/`
6. Add studio form support in `studio/src/` when appropriate
7. Record the change in [`Specs/Changelog.md`](../Specs/Changelog.md)

See [`AGENTS.md`](../AGENTS.md) for the full maintenance checklist.
