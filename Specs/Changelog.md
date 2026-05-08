# Specs / implementation changelog

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
