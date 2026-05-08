# Specs / implementation changelog

## 2026-05-08 — Tasks 15–16 completed (cleanup + neutral catalog schema)

- Marked Tasks **15** and **16** as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- **Schema / types:** `games` → `apps` (`CatalogApp`), `game_store_icons` → `store_icons`, `game_subscribe` → `subscribe_block`, `widgets.game_swiper` → `widgets.carousel`; theme key `game_gradient` → `catalog_gradient`. Updated [sites/kometa/site.json](../sites/kometa/site.json) and legacy sample [config.json](../config.json).
- **Front end:** Renamed carousel script to [`Template/catalog-carousel.js`](../Template/catalog-carousel.js) (`[data-catalog-carousel]`, `.catalog-carousel__*`); refreshed [`Template/styles.css`](../Template/styles.css) (`catalog-app-card`, `#apps_title`, `--catalog-gradient`); synced widget templates and regenerated [KometaWebsite/](../KometaWebsite/).
- **Go:** Trimmed unused legacy placeholder pipeline (`loadConfig`, `applyConfigToDir`, root `collectAssetPaths` / `copyConfigAssets`), kept shared helpers in [`html.go`](../html.go) (footer, widgets JSON export, catalog store resolution, social SVG presets). Theme CSS variables now emit hyphenated custom property names (e.g. `catalog_gradient` → `--catalog-gradient`).
- **Docs:** README now describes the `sites/<site-id>/` workflow; [WidgetRegistryV1.md](./WidgetRegistryV1.md) and [ImplementationSpec.md](./ImplementationSpec.md) reference the new script and JSON keys.

---

## 2026-05-08 18:20 UTC+3 — Codex 5.3 — Tasks 12–14 completed

- Marked Tasks 12–14 as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- Implemented `project_grid` leaf widget: path-aware props validation, meta string/object with stable key order, CTA URL resolution via `Routes` + `resolveInternalSlugReference`, duplicate `props.section_id` enforcement across layout nesting ([`config.go`](./../config.go)), [`Template/widgets/project_grid.html`](../Template/widgets/project_grid.html), and styles in [`Template/styles.css`](../Template/styles.css).
- Implemented `media_swiper` using the same `data-game-swiper` / `.game-swiper__*` markup as catalog swipers so [`Template/game-swiper.js`](../Template/game-swiper.js) stays unchanged.
- Added Kometa acceptance bundle [`sites/kometa/`](../sites/kometa/) (`site.json`, `pages/home.json`, `assets/` placeholders mirroring [`config.json`](../config.json)); optional `widgets` tuning on `SiteConfig`; injected `site-widgets-config` via [`Template/layout.html`](../Template/layout.html) and [`render.go`](./../render.go).
- Tests: project/media widgets, duplicate `section_id` validation, `loadSiteBundle("sites/kometa")`, and full render smoke with static assets + referenced copies.

---

## 2026-05-08 15:50 UTC+3 — Codex 5.3 — Task 11 completed

- Marked Task 11 as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- Replaced `apps_showcase` widget stub with full template-driven rendering in [`Template/widgets/apps_showcase.html`](../Template/widgets/apps_showcase.html), preserving `game-swiper` JS hooks and card/store/subscribe structure.
- Added typed `apps_showcase` parsing + path-aware validation in [`widget_render.go`](../widget_render.go), including `props.apps` non-empty checks, per-entry image validation, asset href resolution, and store badge mapping.
- Extended site schema and asset collection for legacy storefront presets (`game_store_icons`, `game_subscribe`) and added focused tests for apps rendering/validation and nested asset copying.

---

## 2026-05-08 17:45 UTC+3 — Composer — Tasks 8–10 completed

- Marked Tasks 8–10 as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- Wired `html/template` widget partials (`Template/widgets/*.html`) with `widgetRenderContext` threaded from `renderSiteBundle`; layout widgets (`row`, `column`, `grid`) recurse children then execute templates with gap / `min_column_width` defaults and CSS (`Template/styles.css`).
- Implemented leaf widgets `intro`, `cover_banner`, `follow_us`, `info_grid`, `images_grid`, and `careers_tabs` with path-aware validation, asset URL resolution via `resolveAssetHrefForPage`, split-widget-compatible careers markup, plus stub templates for deferred types.
- Updated `render_test` / `widget_render_test` expectations for real widget output.

---

## 2026-05-08 13:00 UTC+3 — Codex 5.3 — Task 7 completed

- Marked Task 7 as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- Added a closed widget dispatcher/registry with fatal path-aware unknown-type errors and layout-only recursion rules (`row`, `column`, `grid`) for page widget rendering.
- Enforced Task-7 policy decisions: reject `columns` alias, recognize `media_swiper`, skip `enabled:false`, and reject `children` on non-layout widgets.
- Added Task 7 tests covering unknown types, recursion constraints, `media_swiper` recognition, disabled widget skipping, and render pipeline wiring.

---

## 2026-05-07 22:34 UTC+3 — Codex 5.3 — Tasks 1-6 completed

- Marked Tasks 1 through 6 as `[DONE]` in [ExecutionPlan.md](./ExecutionPlan.md).
- Implemented Task 6 asset pipeline behavior: strict `assets/...` local path policy, recursive site/page/widget asset reference traversal, deduplicated referenced-asset copying, and config-path-aware validation errors.
- Added Task 6 tests for strict prefix enforcement, escape rejection, recursive nested widget discovery, and deduplicated copy behavior.

---

## PortfolioWebsiteBuilder naming — specs + Task 16

- Added **Task 16** to [ExecutionPlan.md](./ExecutionPlan.md): rename product to **PortfolioWebsiteBuilder**, purge deprecated branding (`GameDevStudio*`, `GameDev`, `sitecreator`), and phased removal of `game*` tokens in first-party code/CSS/JSON.
- [ImplementationSpec.md](./ImplementationSpec.md), [UpgradeQuestions.md](./UpgradeQuestions.md), [WidgetRegistryV1.md](./WidgetRegistryV1.md), [Effort.md](./Effort.md) updated for product naming and neutral wording.
- Go module set to **`portfoliowebsitebuilder`** in root [`go.mod`](../go.mod).

---

## 2026-02-10 — ExecutionPlan filled — specs-only

- Added concrete **15-task** breakdown to [ExecutionPlan.md](./ExecutionPlan.md) (scores, deps, required context).
- Added [Changelog.md](./Changelog.md) stub and pointed ExecutionPlan changelog steps here.
- Updated [Effort.md](./Effort.md): repo-relative links, task effort table aligned with ExecutionPlan, critical path notes.

---

Append **new entries at the top**. Suggested heading format:

`YYYY-MM-DD HH:MM TZ — <model or agent id> — <short summary>`

Body: bullet list of what changed (tasks marked `[DONE]`, behavior, docs).

---
