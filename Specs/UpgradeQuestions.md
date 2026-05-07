# Upgrade Questions

Use this document to capture decisions before writing implementation docs.

---

## 1) Site / Project Structure

1. Should one command build:
   - one specific site folder, or
   - all site folders in a parent directory?

ANSWER: one specific site folder

2. Is this folder structure acceptable?
   - `sites/<site-id>/site.json`
   - `sites/<site-id>/pages/*.json`
   - `sites/<site-id>/assets/...`

ANSWER: yes

3. Should `site_id` and output folder be separate (internal id vs deploy folder name)?

ANSWER: yes

## 2) URL and Page Routing Model

1. Should page config define route as:
   - `slug` (example: `about` -> `about/index.html`), or
   - output file path directly (example: `about.html`)?

ANSWER: as a slug
at most one page may use slug "" (and exactly one for normal sites). Avoid two “home” pages.

2. Do you want pretty URLs by default (`/projects/`)?

ANSWER: yes

3. Should there be a required home page flag, or just route `/`?

ANSWER: just route '/' (empty slug "" is a home)

## 3) Global vs Per-Page Config Boundaries

1. Which fields are global-only (header, footer, theme, social)?

ANSWER: header, footer, theme, social

2. Which fields can pages override (title, SEO, hero, nav visibility)?

ANSWER: title, SEO, hero, nav visibility

3. Can some pages hide global header/footer (landing-page style)?

ANSWER: yes

---

## 4) Widget System Contract

1. Do you want a closed widget registry (known types only), or allow unknown/custom types for future plugins?

ANSWER: known types only

2. For each widget type, should docs define:
   - required fields,
   - optional fields,
   - defaults,
   - validation rules?

ANSWER: yes, all of them

3. Should page widgets be only top-level sequence, or support nested layout widgets (for example: `grid` with `cards`)?

ANSWER: support for nested layout is only needed for things like grids, rows, columns 

---

## 5) Grid / Cards Use Case (GitVerse-like)

1. Do you need one generic `project_grid` widget, or multiple (`project_grid`, `tool_grid`, `news_grid`)?

ANSWER: one generic `project_grid`

2. Card fields: `title`, `description`, `tags`, `image`, `cta`, `meta` - which are required?

ANSWER: all you mentioned

3. Grid behavior: fixed columns or responsive auto-fit?

ANSWER: responsive auto-fit

4. Need filtering/sorting by tag/category on client side?

ANSWER: no

5. Need pagination, or all cards on one static page?

ANSWER: no classic pagination but some swiper widget might be needed (v1 optional media_swiper widget)

---

## 6) Navigation Behavior

1. Is header nav global and manually defined, or auto-generated from pages?

ANSWER: nav is manually defined (auto page order is N/A)

2. If auto-generated, should page order come from `site.json`?
3. For external links in nav: open in new tab by default, or explicit per item?

ANSWER: explicit per item

---

## 7) Templating / Render Architecture

1. Keep current placeholder replacement approach, or move to Go `html/template` partials?

ANSWER: full html/template for the page shell (layout, <head>, header/footer slots) plus partials per widget type; no global {{placeholder}} walk except maybe tiny injected bits

2. Do you want one base page template and one partial template per widget type?

ANSWER: per widget is preferred

3. Should each widget render function return HTML string (current style), or render via templates?

ANSWER: via templates is preferred

PS: there is one shared layout (layout.html or similar) that includes widget regions

---

## 8) Assets and Path Rules

1. Should assets be site-local only (`sites/<id>/assets`) or allow shared global assets too?

ANSWER: site local only

2. If two pages use the same image, should build deduplicate copies?

ANSWER: yes

3. Keep strict "no path escape" safety rules as now?

ANSWER: yes

---

## 9) Backward Compatibility

1. Must existing `config.json` still build KometaWebsite unchanged?

ANSWER: no

2. If yes, do you prefer:
   - legacy mode support indefinitely, or
   - migration tool + deprecation timeline?

ANSWER: skipped since §9 Q1 is no

3. Should docs include a "legacy config -> new site/page config" mapping table?
ANSWER: no

---

## 10) Validation and Authoring UX

1. Strict validation (fail on unknown keys), or permissive with warnings?

ANSWER: permissive

2. Should you introduce JSON Schema files for `site.json` and page widget configs?

ANSWER: no

3. Should CLI show precise error paths like:
   - `pages/home.json -> widgets[3].cards[2].title missing`

ANSWER: yes
---

## 11) Build CLI Behavior

1. Keep interactive prompts, or add non-interactive flags as first-class?

ANSWER: keep interactive

2. Should commands look like:
   - `go run . build --site sites/kometa`
   - `go run . build --all`

ANSWER: more like  `go run . ` and user chooses what to do in interactive mode
only one site could be built at one time

3. Should output be cleaned (`RemoveAll`) every build, or incremental?

ANSWER: cleaned

---

## 12) Non-Functional Defaults

1. Need per-page SEO (`title`, `description`, OpenGraph image)?

ANSWER: optional

2. Need i18n/localization support now, or explicitly "out of scope v1"?

ANSWER: out of scope v1

3. Accessibility baseline: required alt text and heading hierarchy checks?

ANSWER: optional

## 13) Additional info

1. One of created website is planned to be hosted on github.io (user/org (username.github.io)).
No server-side routing, no .htaccess tricks — only static paths.
All internal links and asset references are relative to the current HTML file (or relative to site root using paths that work from both / and /repo/).

2. Migration / parity: new configs must be able to reproduce KometaWebsite-equivalent pages (catalog showcase, careers, etc.).
