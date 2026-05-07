# Implementation specification

**Product:** PortfolioWebsiteBuilder (Portfolio Website Builder)

This document defines **what to build** for the multi-page, widget-based static site generator. Product decisions are captured in [UpgradeQuestions.md](./UpgradeQuestions.md); this spec turns them into implementable requirements.

---

## 1. Goals

- Generate **static HTML/CSS/JS** sites from **declarative JSON** configs.
- Support **multiple sites** in one repo (`sites/<site-id>/`), building **one site per invocation** (interactive CLI).
- Support **multiple pages per site** with **pretty URLs** (`/<slug>/` → `<slug>/index.html`).
- Compose each page from an **ordered list of widgets**, with **nested layouts** limited to rows, columns, and grids.
- Preserve **KometaWebsite-equivalent capability** via new configs (`apps_showcase`, careers, grids, footer, etc.) — see §11.

Non-goals for v1: CMS, backend, i18n, JSON Schema files, legacy `config.json` compatibility.

---

## 2. Repository layout (target)

```
sites/
  <site-id>/
    site.json              # global site config
    pages/
      <page-id>.json       # one file per page (filename is authoring id, not URL)
    assets/                # images, fonts, etc. (site-local only)
Template/                  # shared html/template roots (layout + widget partials + static JS/CSS)
  layout.html
  widgets/
    ...
  styles.css
  *.js
Specs/                     # author-facing specification docs (this folder)
```

Output: written under **`destination/<output_folder>/`** where `output_folder` comes from `site.json` (same conceptual rule as today: single segment, no `..`, no path separators).

---

## 3. `site.json` (global config)

### 3.1 Required fields

| Field | Type | Description |
|-------|------|-------------|
| `site_id` | string | Internal identifier (directory name under `sites/`). |
| `output_folder` | string | Single path segment; output directory name under user-chosen destination. |

### 3.2 Global-only sections (not overridable per page unless spec says otherwise)

Per [UpgradeQuestions.md](./UpgradeQuestions.md): **header** (brand + nav definition), **footer**, **theme**, **social** live at site level.

Suggested shape (implementers may refine names; behavior must match):

- **`theme`**: map of CSS custom properties or equivalent tokens (current project uses `theme_*` keys).
- **`typography`**: optional; Google Fonts URL + heading/body stacks (see existing `TypographyConfig`).
- **`social`**: optional links block (existing `SocialSection` semantics).
- **`header`**: brand (logo path, text), **`nav`**: ordered list of `{ label, href, open_in_new_tab }` — **manually defined**; `open_in_new_tab` applies only to `http(s)` hrefs (mirror current `navLinkAttrs` behavior).
- **`footer`**: existing footer semantics (`FooterConfig`), including optional disable globally — **page may hide footer** via override (see §5).

### 3.3 Optional hosting / SEO root

- **`base_url`**: optional absolute prefix for canonical / Open Graph when authors opt in; **internal links remain relative** for GitHub Pages compatibility (§12).

---

## 4. Page configs (`pages/<page-id>.json`)

### 4.1 Required fields

| Field | Type | Description |
|-------|------|-------------|
| `slug` | string | URL segment; **`""` allowed for exactly one page per site** (home → root `index.html`). Max one page with empty slug. |
| `widgets` | array | Ordered list of widget nodes (§6). |

### 4.2 Optional fields

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | `<title>` and primary heading context where applicable. |
| `seo` | object | Optional: `description`, `og_image`, `canonical_url` (all optional subfields). |
| `hero` | object | Optional page-level hero overrides if distinct from widgets (or omit if hero is only widgets — **pick one approach in implementation** and document). |
| `layout` | object | Optional: `hide_header`, `hide_footer` (booleans) for landing-style pages. |

Validation must enforce **unique slugs** across pages and **at most one** `slug === ""`.

### 4.3 Output paths

- Home (`slug ""`): `index.html` at output root.
- Non-empty slug `about`: `about/index.html` (pretty URL `/about/`).
- Emit internal links consistently (trailing slash on directory URLs or relative paths that resolve correctly — **pick one convention** and apply everywhere).

---

## 5. Merge model (global + page)

When rendering a page:

1. Load `site.json`.
2. Load page JSON.
3. Resolve **effective header/footer**: apply page `layout.hide_*` when present.
4. Resolve **effective `<title>`**: page `title` if set, else fallback to `site_id` or site-level default string (define default).
5. Resolve **SEO meta**: page `seo` overrides; missing fields omitted or fall back to site-level optional defaults if added later.

Theme, typography, social, and nav definitions come from **site** unless extended later by explicit design.

---

## 6. Widget system

### 6.1 Widget node shape

Every widget is a JSON object:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | yes | Must be in the **closed registry** — see [WidgetRegistryV1.md](./WidgetRegistryV1.md). |
| `id` | string | no | Stable id for anchors/testing. |
| `enabled` | bool | no | Default `true`; if `false`, skip rendering. |
| `props` | object | depends | Type-specific payload defined in the registry. |

**Nested widgets**: only **`row`**, **`column`** (and/or **`columns`** per registry), **`grid`** layout types may contain `children: []` widget arrays. Leaf widgets do not embed arbitrary nesting outside these containers.

### 6.2 Rendering

- Use **`html/template`** for **one shared layout** (`layout.html`) covering `<html>`, `<head>`, header slot, main content slot, footer slot.
- Each widget `type` maps to a **named partial** or dedicated template file under `Template/widgets/`.
- Widget rendering executes templates with a **typed Go struct** (or map with discipline) per widget; avoid manual HTML concatenation for user-facing text where templates provide escaping.

Small injections (e.g. JSON config blob for JS widgets) may remain string-built but must be **safely embedded** (existing pattern: escape `<` in JSON script bodies).

### 6.3 Closed registry (v1)

All widget types, **`props`** fields, defaults, validation, and behavioral notes (including **`project_grid`** semantics) are defined in **[WidgetRegistryV1.md](./WidgetRegistryV1.md)**.

---

## 7. Styling and scripts

- Keep **one primary stylesheet** (`styles.css`) or split by feature if maintainability improves; theme tokens injected via `:root` in layout from `site.theme`.
- Reuse existing JS modules where possible: `scroll-reveal.js`, screenshot carousel script (`Template/game-swiper.js`, renamed in Task 16), `split-widget.js`, with **`site-widgets-config`** JSON block for tunables (existing `WidgetsConfig` semantics).

New widgets that need JS must document **data attributes** and **config keys** (see registry + code).

---

## 8. Assets

- **Site-local only**: paths refer to files under `sites/<site-id>/assets/` (or paths relative to site bundle — **define one canonical prefix** e.g. `assets/...` in JSON).
- **Dedupe**: same source file referenced multiple times → **one copy** in output (same relative path in build output).
- **Safety**: preserve current rules — reject `..`, reject paths escaping site root after clean resolution; fail build with clear error including **config path** (§9).
- Copy assets referenced from configs during build (extend current `collectAssetPaths` pattern to scan all widget props).

---

## 9. Validation and errors

- **Permissive**: unknown JSON keys **do not fail** the build; emit **warnings** (stderr or log) listing path + key.
- **Structural errors fail**: unknown widget `type`, missing required fields, duplicate slugs, two home slugs, invalid `output_folder`, missing referenced asset file.
- Error messages **must** include a precise path, e.g.  
  `sites/kometa/pages/home.json → widgets[3].props.cards[2].title: required field missing`

No JSON Schema files required; validation lives in Go.

Optional SEO/a11y checks: **warn-only** unless future `--strict` flag is introduced.

---

## 10. CLI

- Default entry: `go run .` → **interactive** menu (or prompts): choose site folder path (`sites/<site-id>`), choose destination directory (empty = project root).
- **One site per run**.
- Before write: **delete output subdirectory entirely** (`RemoveAll` on target output folder), then write fresh tree.
- Non-interactive flags (`--site`, `--dest`) are **optional enhancements**; questionnaire prioritized interactive only.

---

## 11. Kometa parity (acceptance)

New configs **must** be sufficient to reproduce **KometaWebsite-equivalent** pages:

- Home (or chosen structure) with: intro copy, multiple catalog entries with swiper + store links + stats, `info_grid`, `images_grid`, careers with tabbed vacancies + apply links, follow-us social block, footer with contact + legal links.
- Styling must remain coherent with existing design system (tokens + layout).

Authors migrate manually from old single `config.json`; **no** automated migration tool or published legacy mapping table is required by questionnaire, but implementers may use internal notes.

---

## 12. GitHub Pages

Target: **user/org sites** (`https://<user>.github.io/`); keep compatibility with **project sites** (`/<repo>/`) where feasible.

Requirements:

- **Static files only** — no server rewrite rules.
- **Relative** internal links and asset URLs so the site works under `/` and under `/repo/`.
- Optional `base_url` only for absolute meta tags when authors supply it.

---

## 13. Implementation phases (suggested)

1. **Bootstrap**: site + page loading, slug routing, layout template, empty `widgets` page.
2. **Asset pipeline** + path security + dedupe.
3. **Layout widgets** (`row`, `column`, `grid`) — see [WidgetRegistryV1.md](./WidgetRegistryV1.md).
4. **Port existing sections** to widgets (`intro`, `apps_showcase`, …) using Kometa as golden sample.
5. **`project_grid`** + CSS auto-fit — registry § `project_grid`.
6. **Optional `media_swiper`**.
7. **Interactive CLI polish** + warning/error UX.

---

## 14. Open implementation choices (resolve during coding)

Document the chosen option in code comments or a short “Decisions” subsection once resolved:

- Cover banner: global-only vs per-page widget placement.
- Whether `hero` is a dedicated page field or only widgets.
- Exact relative-link convention (always `./`, `../`, vs root-relative paths).
- Whether `go run . validate` exists as a separate interactive action or is always part of build.
