# Specs / implementation changelog

## 2026-05-29 — Composer — Tauri Phase 3 Task 4: schema-driven form panels

- Marked **Task 4** as `[DONE]` in [tauri/execution-plan-phase-3.md](./tauri/execution-plan-phase-3.md); Phase 3 exit criterion for theme/nav forms checked.
- Studio: **JSON** / **Form** tabs when editing `site.json`. Form panels driven by [`docs/schema/site.schema.json`](../docs/schema/site.schema.json) for `theme` tokens and `header.nav` table; round-trip with JSON tab; invalid JSON disables form. Other `site.json` sections stay JSON-only. Documented in [studio/README.md](../studio/README.md).

---

## 2026-05-29 — Composer — Tauri Phase 3 Task 3: CLI install without GUI

- Marked **Task 3** as `[DONE]` in [tauri/execution-plan-phase-3.md](./tauri/execution-plan-phase-3.md).
- Root [README.md](../README.md): **Headless / CI vs Studio app** comparison table; `cargo install --path crates/cli --locked` and release-binary install for terminal/CI workflows. CLI documented as fully independent of Tauri.

---

## 2026-05-29 — Composer — Tauri Phase 3 Task 2: reveal output and new site

- Marked **Task 2** as `[DONE]` in [tauri/execution-plan-phase-3.md](./tauri/execution-plan-phase-3.md).
- **Open output folder:** `revealItemInDir` on last successful `output_dir`; error dialog when missing or reveal fails.
- **New site:** `content/_template/` skeleton; `create_site_from_template` command validates id, copies bundle, patches `site.json`, verifies `site_id` matches folder; studio **New site** button + dropdown refresh. `_`-prefixed content dirs excluded from `list_content_bundles`.

---

## 2026-05-29 — Composer — Tauri Phase 3 Task 1: debounced watch rebuild

- Marked **Task 1** as `[DONE]` in [tauri/execution-plan-phase-3.md](./tauri/execution-plan-phase-3.md).
- Rust: `content_watcher` module (`notify` + `notify-debouncer-mini`, **500 ms** debounce) watches `content/<site>/`; on trigger runs `build_site`, restarts preview, emits `watch-rebuild-complete`. New command `set_auto_rebuild`; build logic extracted to `site_ops.rs`.
- Studio: **Auto-rebuild** toolbar toggle (default off); debounced auto-save when enabled; Problems/log/preview update from watcher events. Documented in [studio/README.md](../studio/README.md).

---

## 2026-05-29 — Composer — Tauri Phase 2 Task 4: packaging and cross-platform validation

- Marked **Task 4** as `[DONE]` in [tauri/execution-plan-phase-2.md](./tauri/execution-plan-phase-2.md); Phase 2 completion criteria checked in [tauri/requirements.md](./tauri/requirements.md).
- Windows installer: `embedBootstrapper` WebView2 mode in `src-tauri/tauri.conf.json`; app name/icons already **Portfolio Website Builder**.
- CI: full `cargo tauri build` on `macos-latest` and `windows-latest` with bundle artifacts uploaded.
- [studio/VALIDATION-CHECKLIST.md](../studio/VALIDATION-CHECKLIST.md) for manual regression; [studio/README.md](../studio/README.md) and root [README.md](../README.md) document packaged install and `cargo tauri dev`.

---

## 2026-05-29 — Composer — Tauri Phase 2 Task 3: Studio UI v1

- Marked **Task 3** as `[DONE]` in [tauri/execution-plan-phase-2.md](./tauri/execution-plan-phase-2.md).
- Studio layout: toolbar (open project, site dropdown, build/validate/strict, open output), file tree, tabbed CodeMirror JSON editor with schema lint, Problems panel, HTTP preview iframe with desktop/phone widths, build log.
- New Tauri commands: settings persistence, `project_info_for_root`, bundle file list/read/write; `tauri-plugin-dialog` and `tauri-plugin-opener`.
- Build flow saves dirty buffers → `build_site` → `start_preview_server` → preview reload on `127.0.0.1`.

---

## 2026-05-29 — Composer — Tauri Phase 2 Task 2: invoke API over core

- Marked **Task 2** as `[DONE]` in [tauri/execution-plan-phase-2.md](./tauri/execution-plan-phase-2.md).
- Tauri commands: `resolve_project_root`, `list_content_bundles`, `validate_site`, `build_site`, `start_preview_server`, `stop_preview_server` (structured diagnostics with path, message, severity).
- Managed preview server (`tiny_http` on `127.0.0.1`); stops on build, window close, and explicit stop.
- Studio dev panel (`studio/src/App.svelte`) exercises all commands with a log strip; typed wrappers in `studio/src/lib/studio-api.ts`.

---

## 2026-05-29 — Composer — Tauri Phase 2 Task 1: Tauri + Svelte scaffold

- Marked **Task 1** as `[DONE]` in [tauri/execution-plan-phase-2.md](./tauri/execution-plan-phase-2.md).
- Added `studio/` (Svelte 5 + Vite) and `src-tauri/` (Tauri 2) at repo root; wired `portfolio-website-builder-studio` to `crates/core`.
- Placeholder UI invokes `resolve_project_root_info` and shows project root + `Template/` paths; dev entry documented in [studio/README.md](../studio/README.md).
- CI: `cargo tauri build --bundles app` smoke on `macos-latest` and `windows-latest`.

---

## 2026-05-29 — Composer — Tauri Phase 1 Task 14: decommission Go and update docs

- Marked **Task 14** and scope **Task 6** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md); Phase 1 completion criteria checked in [tauri/requirements.md](./tauri/requirements.md).
- Removed all Go sources (`*.go`, `go.mod`), committed `portfoliowebsitebuilder` binary, and documented that **`go run .` is intentionally unavailable** after Minijinja migration.
- Updated [README.md](../README.md), [ImplementationSpec.md](./ImplementationSpec.md), golden/tests README, and `docs/schema/site.schema.json` for Rust CLI workflow.
- Added serve path tests in `serve.rs`, CLI tests for `--validate`+`--serve` conflict and interactive bundle prompt in `cli_integration.rs`.

---

## 2026-05-29 — Composer — Tauri Phase 1 Task 13: CLI build and asset pipeline tests

- Marked **Task 13** and scope **Task 5** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md).
- Added `fs_util.rs` (destination wipe, template static copy, rebuild clears stale files) and `build_pipeline.rs` (render write smoke tests, disk golden parity via `generate_site`).
- Extended `cli_integration.rs` with non-interactive build, demo/my-studio build exit checks.

---

## 2026-05-29 — Composer — Tauri Phase 1 Task 12: golden HTML parity tests

- Marked **Task 12** and scope **Task 4** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md).
- Added `golden_parity.rs` with `normalize_html_for_test()` and unified diff on failure; exported `render_site_bundle_html` for in-memory comparison.
- Fixed Go parity drift: Minijinja HTML escape policy, stylesheet href encoding, widgets-config JSON key order, inter-widget newline spacing.

---

## 2026-05-29 — Composer — Tauri Phase 1 Task 11: widget and script-needs tests

- Marked **Task 11** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md).
- Added integration tests: `widget_render.rs` (20 cases from Go `widget_render_test.go`) and `widget_scripts.rs` (4 cases from Go `widget_scripts_test.go`).
- Fixed optional widget props to match Go: `apps_showcase.section_title` / `apps`, `project_grid` card `cta`/`tags`/`image`, `media_swiper` slide `alt` via `#[serde(default)]`.

---

## 2026-05-29 — Composer — Tauri Phase 1 Tasks 9–10: routing, assets, render tests

- Marked **Tasks 9–10** and scope **Tasks 2–3** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md).
- Added integration tests: `routing.rs`, `assets.rs`, `render_page_data.rs`, `html_footer.rs`; CLI `cli_validate_demo`.
- Fixed `relative_dir_link` for home→nested slug paths (match Go `filepath.Rel`); `project_grid` optional props via `#[serde(default)]`.

---

## 2026-05-29 — Composer — Tauri Phase 1 Task 8: config/strict/bundle tests

- Marked **Task 8** and **Task 1** as `[DONE]` in [tauri/execution-plan-phase-1.md](./tauri/execution-plan-phase-1.md).
- Added Rust integration tests mirroring Go `config_loader_test.go`, `strict_test.go`, and `cli_test.go` discover/list/validate cases under `crates/core/tests/` (`config_loader.rs`, `strict_mode.rs`, `bundles.rs`, `common/mod.rs`) and `crates/cli/tests/cli_integration.rs`.
- Fixed `intro` widget props parsing: optional `title` / `paragraphs` with `#[serde(default)]` to match Go strict validate fixtures.

---

## 2026-05-29 — Composer — Tauri Phase 0: preparation complete

- Marked Tasks **1–4** as `[DONE]` in [tauri/execution-plan-phase-0.md](./tauri/execution-plan-phase-0.md); Phase 0 exit criteria checked off.
- **Task 1:** [README.md](../README.md) **Migration (Tauri + Rust)** subsection links to [tauri/requirements.md](./tauri/requirements.md) and [tauri/execution-plan.md](./tauri/execution-plan.md) (index already lists Phase 0–3 plans).
- **Task 2:** Golden HTML from Go builds for `kometa`, `demo` (home, `about`, `apps`), and `my-studio` under `crates/core/tests/golden/`; [crates/core/tests/README.md](../crates/core/tests/README.md) documents normalization rules for Phase 1.4.
- **Task 3:** [crates/core/README.md](../crates/core/README.md) locks **Minijinja**, escaping policy, and planned `load_site_bundle` / `validate_site_bundle` / `render_site_bundle` API.
- **Task 4:** [.github/workflows/rust-ci.yml](../.github/workflows/rust-ci.yml) matrix `ubuntu-latest` / `macos-latest` / `windows-latest`, Rust 1.77+, `cargo test --workspace`.

---

## 2026-05-28 — Composer — UX Iter 1: UX-23, UX-24 completed

- Marked Tasks **UX-23** and **UX-24** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-23:** Per-page widget tree walk collects script needs; [`Template/layout.html`](../Template/layout.html) omits unused `scroll-reveal.js`, `catalog-carousel.js`, `split-widget.js`, and `image-lightbox.js` tags; `site-widgets-config` is included only when at least one of those scripts loads ([`widget_render.go`](../widget_render.go), [`render.go`](../render.go)).
- **UX-24:** README **Deploy to GitHub Pages** section documents source vs output, user vs project site URLs, `base_url`, `output_folder` choices, and relative link rules aligned with live bundle paths.

---

## 2026-05-28 — Composer — UX Iter 1: UX-22 completed

- Marked Task **UX-22** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-22:** Added `--strict` CLI flag; unknown top-level JSON keys and unknown widget `props` keys fail the build or `--validate` run; default remains warn-only ([`strict.go`](../strict.go), [`cli.go`](../cli.go), [`config.go`](../config.go)); README documents strict mode and per-widget props key registry.

---

## 2026-05-28 — Composer — UX Iter 1: UX-21 completed

- Marked Task **UX-21** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-21:** Added [`Specs/schema/site.schema.json`](./schema/site.schema.json) and [`Specs/schema/page.schema.json`](./schema/page.schema.json) for author config autocomplete; widget `type` enum matches the closed v1 registry; [README.md](../README.md) documents VS Code / Cursor `json.schemas` mapping and `ajv-cli` validation; [`.vscode/settings.json`](../.vscode/settings.json) maps schemas to `content/` paths.

---

## 2026-05-28 — Composer — UX Iter 1: UX-19, UX-20 completed

- Marked Tasks **UX-19** and **UX-20** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-19:** Added [`content/demo/`](../content/demo/) bundle (option A): `site.json`, `pages/home.json` + `pages/about.json`, minimal `assets/`; header nav mixes hash links on home and cross-page `about/` route; output `Results/DemoWebsite` with `index.html` and `about/index.html`.
- **UX-20:** Home page includes `project_grid` with two sample cards (tags, meta, image, CTA including internal slug link to About).
- Home page showcases all v1 widget types plus `row` / `column` / `grid` layout containers.

---

## 2026-05-28 — UX-18: Remove dead `page.hero` field

- Marked Task **UX-18** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-18:** Removed unused `PageConfig.Hero` and hero asset scanning; legacy top-level `hero` in page JSON emits a descriptive build warning pointing authors to `intro` / `cover_banner` widgets ([`types.go`](../types.go), [`config.go`](../config.go), [`assets.go`](../assets.go)).
- Updated [ImplementationSpec.md](./ImplementationSpec.md) §4.2 and [README.md](../README.md) to document widget-only page heroes.

---

## 2026-05-28 — UX-17: Local preview static server

- Marked Task **UX-17** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-17:** Added `--serve` and `--port` CLI flags; after build, serves `output_folder` via Go stdlib `net/http` FileServer on `127.0.0.1` ([`serve.go`](../serve.go), [`cli.go`](../cli.go)).
- README documents built-in preview, plus `python3 -m http.server` and `npx serve` one-liners for manual HTTP preview.

---

## 2026-05-28 — UX-14, UX-15, UX-16: CLI validate, --site, bundle discovery

- Marked Tasks **UX-14**, **UX-15**, and **UX-16** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-14:** `--validate` loads a bundle, checks asset references and dry-run render, prints warnings, exits non-zero on error; no output directory write or wipe ([`cli.go`](../cli.go), [`render.go`](../render.go), [`assets.go`](../assets.go)).
- **UX-15:** `--site <path>` skips the interactive bundle prompt for CI-style builds ([`main.go`](../main.go), [`cli.go`](../cli.go)).
- **UX-16:** `--list-sites` prints valid bundles under `content/`; interactive prompt lists bundles on `?` or empty input when multiple exist ([`bundles.go`](../bundles.go)).
- README documents new flags and interactive bundle selection.

---

## 2026-05-28 — UX-12, UX-13: Footer polish and apps_showcase store guards

- Marked Tasks **UX-12** and **UX-13** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-12:** Removed `scroll-reveal` from footer in [`html.go`](../html.go); added `id="footer-contact"` on the contact row; legal links already omitted when URL is empty after trim (tests in [`html_test.go`](../html_test.go)).
- **UX-13:** Build warns when a store preset is present in app JSON with an empty URL (`catalogStoreURLWarnings`); empty URLs never render store buttons; subscribe block hidden when no links resolve (tests in [`widget_render_test.go`](../widget_render_test.go)).

---

## 2026-05-28 — UX-9, UX-10, UX-11: SEO meta tags, sample content, images lightbox

- Marked Tasks **UX-9**, **UX-10**, and **UX-11** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-9:** Extended [`Template/layout.html`](../Template/layout.html) and [`render.go`](../render.go) with Open Graph tags (`og:title`, `og:description`, `og:url`, `og:type`), Twitter card tags, and `theme-color` from site theme accent; empty fields omit tags.
- **UX-10:** Added `seo`, placeholder `base_url`, and `widgets.split_widget.keyboard_navigation` to [`content/kometa/`](../content/kometa/) and mirrored in [`content/my-studio/`](../content/my-studio/).
- **UX-11:** Kometa gallery images use descriptive `{ src, alt }` objects; bare paths or generic `photo N` alts emit build warnings; new [`Template/image-lightbox.js`](../Template/image-lightbox.js) + CSS for click-to-enlarge with Escape/backdrop dismiss and basic focus trap.

---

## 2026-05-28 — UX-6, UX-7, UX-8: Carousel a11y, single-vacancy careers, scroll-reveal safety

- Marked Tasks **UX-6**, **UX-7**, and **UX-8** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-6:** [`Template/catalog-carousel.js`](../Template/catalog-carousel.js) — Left/Right arrow keys when carousel is focused (configurable via `carousel.keyboard_navigation`, default `true`); visible slide counter (`2 / 5`); `aria-live="polite"` announcements with optional image alt; carousel roots in apps_showcase get `role="region"`.
- **UX-7:** Single-vacancy `careers_tabs` renders panel content directly (`split-widget--single`) without tab chrome; multi-vacancy configs unchanged; tests updated in [`widget_render_test.go`](../widget_render_test.go) and [`render_test.go`](../render_test.go).
- **UX-8:** Intro uses `scroll-reveal--immediate` for above-the-fold visibility; CSS and [`Template/scroll-reveal.js`](../Template/scroll-reveal.js) skip observer for immediate sections; reduced-motion path unchanged; README widgets section documents tuning.

---

## 2026-05-28 — UX-3, UX-4, UX-5: Contacts IA, scroll-spy, accessibility baseline

- Marked Tasks **UX-3**, **UX-4**, and **UX-5** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- **UX-3 (Approach A):** Retargeted **Contacts** nav to `#footer` in [`content/kometa/site.json`](../content/kometa/site.json) and [`content/my-studio/site.json`](../content/my-studio/site.json); renamed follow-us section id from `contact` to `follow_us` in [`Template/widgets/follow_us.html`](../Template/widgets/follow_us.html); added `#footer` scroll-margin in [`Template/styles.css`](../Template/styles.css).
- **UX-4:** Extended [`Template/nav.js`](../Template/nav.js) with IntersectionObserver scroll-spy for same-page hash links; toggles `.site-nav__link--active` and `aria-current="location"`; click updates active state; works alongside UX-2 mobile menu.
- **UX-5:** Skip link (“Skip to main content”) as first focusable element in [`Template/layout.html`](../Template/layout.html) targeting `#main-content`; visible `:focus-visible` rings on nav, carousel, tabs, store buttons, social CTAs, and footer links using `--accent`.

---

## 2026-05-27 — UX-2: Collapsible mobile site navigation

- Marked Task **UX-2** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- Added hamburger toggle to [`Template/layout.html`](../Template/layout.html) (`aria-expanded`, `aria-controls`, accessible Menu/Close labels).
- New [`Template/nav.js`](../Template/nav.js): open/close toggle, Escape closes and refocuses toggle, auto-close on link click and viewport resize above 720px (focus trap omitted — documented in script).
- Mobile CSS in [`Template/styles.css`](../Template/styles.css): grid header layout, nav hidden until `.site-nav--open`, 44px toggle target; desktop unchanged above 720px.
- Wired `NavScriptHref` in [`render.go`](../render.go).

---

## 2026-05-27 — UX-1: Mobile-responsive catalog carousel

- Marked Task **UX-1** as `[DONE]` in [UX-Improvements-Iter-1.md](./UX-Improvements-Iter-1.md).
- Added responsive `@media (max-width: 720px)` and `(max-width: 640px)` rules in [`Template/styles.css`](../Template/styles.css) for `.catalog-carousel*`: reduced viewport height, slide offsets, and image max-heights; clipped overflow on the viewport; enlarged arrow touch targets to 44px on phones. Desktop Kometa layout unchanged above 720px.

---

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
