# Widget registry — v1 (closed)

This document is **normative** for widget `type` strings, `props` shape, defaults, and validation. It accompanies [ImplementationSpec.md](./ImplementationSpec.md).

**Rules (global):**

- Unknown widget `type` → **build error** with path to the widget node.
- Each widget may define `id` (optional) on the node for anchors/DOM ids; if omitted, generator may synthesize stable ids.
- User-facing copy is rendered through **`html/template`** so text is escaped unless a field is explicitly documented as trusted HTML (none in v1).
- **Unknown JSON keys** elsewhere in configs → permissive warning (per main spec), but **within `props`** implementers may choose strict keys per widget for clarity; document the chosen behavior.

---

## Summary table

| `type` | Role |
|--------|------|
| `intro` | Company/page intro: heading + paragraphs. |
| `apps_showcase` | Vertical stack of rich portfolio cards (Kometa-style sample). |
| `info_grid` | Responsive grid of small info cards (image + title + text). |
| `images_grid` | Responsive grid of images. |
| `careers_tabs` | Vacancies as tabbed panels (Kometa-style careers). |
| `follow_us` | Social link buttons row. |
| `cover_banner` | Full-width cover image band. |
| `project_grid` | Responsive **card grid** for projects/tools/portfolio items (see § below). |
| `row` | Layout: horizontal group of child widgets. |
| `column` / `columns` | Layout: vertical stack of child widgets (pick **one** canonical name in code; accept alias if documented). |
| `grid` | Layout: CSS grid of child widgets. |

### Optional v1

| `type` | Role |
|--------|------|
| `media_swiper` | Generic image carousel (optional JS). |

---

## Layout widgets (`row`, `column` | `columns`, `grid`)

**Purpose:** Only these types may contain **`children`**: an ordered array of widget nodes.

### `row`

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `children` | yes | — | Non-empty array of widgets. |
| `gap` | no | token or CSS length | Spacing between children. |

**Validation:** `children` must have length ≥ 1. Nested layout allowed.

### `column` / `columns`

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `children` | yes | — | Non-empty array of widgets. |

Implement **one** canonical `type` string in JSON (recommend `column`); if both spellings are accepted, treat them identically.

### `grid`

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `children` | yes | — | Non-empty array of widgets. |
| `min_column_width` | no | `260px` | Passed to `repeat(auto-fit, minmax(...))` or equivalent. |
| `gap` | no | site default | Grid gap. |

**Validation:** `min_column_width` must parse as a positive CSS length if present.

---

## `intro`

**Purpose:** Replace legacy intro section: centered heading + one or two descriptive paragraphs.

**Parity:** Current `buildIntroSectionHTML` (`intro_title`, `company_description`, …).

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `title` | no | `""` | Main `<h1>` or `<h2>` per visual hierarchy rules. |
| `paragraphs` | no | `[]` | Array of strings; empty paragraphs omitted. |

**Validation:** At least one of `title` or non-empty `paragraphs` should be present; if both empty, **warn** and render nothing.

---

## `apps_showcase`

**Purpose:** Vertical stack of **large product cards**: optional header image or title band, icon row + stats, body copy, screenshot swiper, store badges, optional subscribe links.

**Parity:** Catalog entry struct in [`types.go`](../types.go) (`CatalogApp`), shared store resolution in [`html.go`](../html.go), screenshot carousel ([`Template/catalog-carousel.js`](../Template/catalog-carousel.js)), store badges, subscribe block from site-level `subscribe_block`.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `section_title` | no | `""` | Wrapped in section heading if non-empty. |
| `apps` | yes | — | Non-empty array of **entry** objects (mirror [`types.go`](../types.go) payloads + store links; sample JSON in [`config.json`](../config.json)). |

**Entry object:** Align fields with existing JSON (`image`, `header_image`, `swiper_images`, `card_background`, `title`, `text_1`, `text_2`, stats lines, legacy storefront URLs and/or `store_links`, etc.).

**Validation:**

- `apps` length ≥ 1.
- Each entry: enforce same asset and URL rules as today (store icons resolved via site-level **`store_icons`** preset map in [`site.json`](../sites/kometa/site.json)).

---

## `info_grid`

**Purpose:** “Why join us” / features grid — small cards with optional emoji/image, title, short text.

**Parity:** Current `offers` + `OfferItem`.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `title` | no | `""` | Section `<h2>`. |
| `items` | yes | — | Array of `{ image?, title, text }`. |

**Validation:** `items` non-empty; each item needs at least one of `title` or `text` after trim.

**Layout:** Responsive grid, `auto-fit` with sensible `minmax` (e.g. `240px`).

---

## `images_grid`

**Purpose:** Photo wall / gallery grid.

**Parity:** Current `photos` array.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `title` | no | `""` | Section heading. |
| `images` | yes | — | Array of `{ src, alt? }` or legacy list of path strings (implementer picks one schema and documents it). |

**Validation:** `images` non-empty; each image `src` must resolve to an asset.

**Layout:** Responsive columns (e.g. 3 → 2 → 1 breakpoints).

---

## `careers_tabs`

**Purpose:** List roles as tabs; panel shows requirements, responsibilities, advantages, apply CTA.

**Parity:** Current `vacancies` + `split-widget.js`.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `title` | no | `""` | Section heading. |
| `vacancies` | yes | — | Array matching current `Vacancy` shape. |
| `labels` | no | from site locale | Optional overrides for column headings (“Requirements”, …). |

**Validation:** `vacancies` non-empty for visible section; reuse apply URL/link rules.

---

## `follow_us`

**Purpose:** Heading + row of social icons/links.

**Parity:** Current `follow_us` / `SocialSection`.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `title` | no | `"Follow us"` | Section heading. |
| — | — | — | **Alternatively** inherit links only from `site.json` `social` and omit props except `title`; implementers choose minimal duplication. |

**Validation:** If no links resolved, **warn**; render empty section or skip per ImplementationSpec merge rules.

---

## `cover_banner`

**Purpose:** Full-width image strip (often above `<main>`).

**Parity:** Current `cover_banner_section` / `cover_image`.

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `src` | yes | — | Image path under site assets. |
| `alt` | no | `"Cover"` | Accessibility. |

**Open choice:** Global-only injection vs first widget on page — resolve in implementation and document.

---

## `project_grid` — portfolio / tools landing grid

**Purpose:** A single section that presents **many comparable items** (repos, tools, shipped products) as **uniform cards** in a **responsive CSS grid**. Static HTML only: **no** client-side filtering, sorting, or pagination in v1.

Do **not** rely on the phrase “GitVerse-like” in code or validation messages. The behavior below is **normative**.

### Non-normative visual reference

Authors may use [https://gitverse.ru/home/](https://gitverse.ru/home/) as **visual inspiration** (dense landing page with bands and card grids). Only this specification defines required behavior.

### Section chrome (optional)

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `heading` | no | `""` | Section title (`<h2>`). |
| `subheading` | no | `""` | Short line under heading. |
| `section_id` | no | auto | HTML `id` on `<section>` for deep links; validate uniqueness per page if set. |

Wrap content in the same **page container** width as other sections (`container`).

### Cards

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `cards` | yes | — | Non-empty array of card objects. |

**Each card object** (all fields **required** per [UpgradeQuestions.md](./UpgradeQuestions.md)):

| Field | Type | Notes |
|-------|------|--------|
| `title` | string | Card title; plain text, escaped. |
| `description` | string | Short paragraph; plain text, escaped. |
| `tags` | array of string | Render as **non-interactive pills/chips** (no filter UI in v1). |
| `image` | string | Asset path; hero/thumbnail for the card. |
| `cta` | object | `{ "label": string, "url": string }`. Primary action (project page, repo, store). Use `rel`/`target` rules consistent with site nav for external URLs. |
| `meta` | string **or** object | Secondary line under title or above tags. If **string**, render one subdued line. If **object**, render compact key/value pairs (e.g. `"year": "2024", "platform": "Web"`); keys sorted alphabetically for stable output. |

**Layout:**

- Grid: `display: grid`; `grid-template-columns: repeat(auto-fit, minmax(M, 1fr))` where **`M`** defaults to **`260px`** unless overridden by widget prop `min_card_column_width`.
- Card media: fixed **aspect ratio** recommended **`16 / 9`** for predictable grids (`aspect-ratio` + `object-fit: cover`).

**Validation:**

- `cards.length >= 1`.
- Each card: `title`, `description`, `image`, `meta`, and `cta` present; `title`, `description`, `image`, `cta.url` non-empty after trim. `meta`: if string, non-empty after trim; if object, non-empty after serializing (at least one key).
- `tags` must be an array of strings (may be empty; renders no pills).
- `cta.label` default `"Learn more"` if empty string after trim.

**Out of scope v1:** Tag click filters, search, pagination, carousel inside card (use separate **`media_swiper`** widget if needed).

---

## `media_swiper` (optional v1)

**Purpose:** Generic touch/keyboard carousel for images (not tied to app cards).

| `props` | Required | Default | Notes |
|---------|----------|---------|--------|
| `images` | yes | — | Array of `{ src, alt? }`. |
| `aria_label` | no | `"Image carousel"` | For wrapper. |

**Behavior:** Reuses [`Template/catalog-carousel.js`](../Template/catalog-carousel.js): root `[data-catalog-carousel]` and `.catalog-carousel__*` elements (same contract as `apps_showcase` swiper markup).

**Validation:** `images` length ≥ 1.

---

## Template files (suggestion)

Under `Template/widgets/`:

- One template per `type` above (e.g. `project_grid.html`, `apps_showcase.html`).
- Layout widgets recurse into `children` via shared snippet or Go-driven recursion.

---

## Versioning

Bump this document when adding widget types or breaking `props` shapes; keep [ImplementationSpec.md](./ImplementationSpec.md) linking to the latest registry filename or version section.
