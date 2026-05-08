# PortfolioWebsiteBuilder — Execution Plan

How to use this plan: each task lists **Required context** — read those docs/files before coding. Cross-cutting **Confidence and Risks** applies to every task.

## Assumptions

- Implementation is **agent-led**; human role is approval/review.
- Product and repository naming follow **PortfolioWebsiteBuilder** / **Portfolio Website Builder** (see Task 16).
- **No** requirement to keep building legacy root [`config.json`](../config.json) unchanged; Kometa-level behavior is reproduced via new **`sites/<site-id>/`** bundles ([UpgradeQuestions.md §9](./UpgradeQuestions.md)).
- **Incremental refactor**: replace single-page placeholder pipeline with **`html/template`** layout + widget partials per [ImplementationSpec.md §6](./ImplementationSpec.md).
- **Interactive CLI** is primary ([ImplementationSpec.md §10](./ImplementationSpec.md)); optional `--site` / `--dest` flags are nice-to-have later.
- Each build **cleans** the target output subdirectory before writing ([ImplementationSpec.md §10](./ImplementationSpec.md)).
- **Limited duplication** during migration is acceptable (e.g. temporary coexistence of old and new code paths until Task 15).

## Confidence and Risks

**Confidence:** Medium–High.

Resolved constraints:

1. Folder layout (`sites/<id>/site.json`, `pages/*.json`, `assets/`) and slug routing (`""` → root `index.html`) are specified in [ImplementationSpec.md §2, §4](./ImplementationSpec.md).
2. Widget contracts and `project_grid` semantics are defined in [WidgetRegistryV1.md](./WidgetRegistryV1.md).
3. Assets: site-local only, dedupe on copy, path escape forbidden — [ImplementationSpec.md §8](./ImplementationSpec.md).

Residual uncertainties:

1. **Relative URL convention** for nav between nested pages (ImplementationSpec §14) — must be decided early in Task 2 and applied consistently.
2. **`apps_showcase`** parity with current [`html.go`](../html.go) markup/CSS/JS may require coordinated template + stylesheet updates.
3. **Recursive widget renderer** + validation error paths (`pages/x.json → widgets[i].…`) need a consistent helper pattern ([ImplementationSpec.md §9](./ImplementationSpec.md)).

## Agent Level Legend

- `easy`: straightforward implementation, clear requirements.
- `medium`: moderate complexity, some design decisions needed.
- `heavy`: complex logic, strong reasoning and long-context required.

## Changelog Instructions

- When a task is completed, append **`[DONE]`** to its title in this file.
- Append a new entry at the **top** of [Changelog.md](./Changelog.md) with date, time (timezone if known), and model/agent identifier in the heading.

## Task Breakdown

#### Task 1: Site bundle loader and structural validation [DONE] [Score:3] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §2–§5, §9](./ImplementationSpec.md)
2. [UpgradeQuestions.md §1–§2](./UpgradeQuestions.md)
3. Existing patterns: [`config.go`](../config.go) (`loadConfig`)

- Introduce types and loader for `sites/<site-id>/site.json` plus glob/read of `sites/<site-id>/pages/*.json`.
- Validate: unique slugs across pages; **at most one** page with `slug === ""`; required fields per ImplementationSpec.
- **Permissive unknown keys**: warn with precise file path + key name; do not fail build for unknown top-level keys ([ImplementationSpec.md §9](./ImplementationSpec.md)).
- Fail build on structural errors (missing required fields, duplicate slugs, invalid `output_folder`).

Dependencies: none.

---

#### Task 2: Output path routing for pages [DONE] [Score:5] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §4.3, §12](./ImplementationSpec.md)
2. [UpgradeQuestions.md §13](./UpgradeQuestions.md) (relative links / GitHub Pages)

- Map `slug ""` → output root `index.html`; non-empty slug → `<slug>/index.html` under `destination/<output_folder>/`.
- **Document and implement** one convention for internal navigation links (site header nav + cross-page links): relative paths that work for both `/` and future `/repo/` hosting.
- Ensure generated asset references remain valid from each output HTML depth.

Dependencies: Task 1.

---

#### Task 3: `html/template` page shell [DONE] [Score:5] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §6.2, §7](./ImplementationSpec.md)
2. Current shell: [`Template/index.html`](../Template/index.html)

- Add shared **`layout.html`** (or equivalent) with slots for `<head>`, header, main widget stream, footer, scripts.
- Inject `:root` CSS variables from `site.theme` and typography ([ImplementationSpec.md §3](./ImplementationSpec.md)).
- Reduce or eliminate directory-wide `{{placeholder}}` replacement for the main layout per spec; keep tiny injections only if justified ([`fs.go`](../fs.go) `applyConfigToDir`).

Dependencies: Task 1.

---

#### Task 4: Global + page merge model [DONE] [Score:3] [Agent:easy]

**Required context**

1. [ImplementationSpec.md §5, §3.3, §4.2](./ImplementationSpec.md)

- Compute effective **header/footer** using page `layout.hide_header` / `hide_footer` when present.
- Compute effective **document title** (`<title>`): page `title` → fallback (e.g. `site_id` or site-level default).
- Optional **SEO** meta and **`base_url`** for canonical/OG when fields present; internal anchors remain relative ([ImplementationSpec.md §12](./ImplementationSpec.md)).

Dependencies: Task 3.

---

#### Task 5: Interactive CLI — single site build [DONE] [Score:3] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §10](./ImplementationSpec.md)
2. [UpgradeQuestions.md §11](./UpgradeQuestions.md)
3. [`main.go`](../main.go)

- Interactive flow: choose **site bundle directory** (`sites/<id>`), choose **destination root** (empty = cwd).
- Resolve `output_folder` from loaded `site.json`; **`RemoveAll`** on `filepath.Join(dest, output_folder)` before generating ([ImplementationSpec.md §10](./ImplementationSpec.md)).
- One site per invocation.

Dependencies: Tasks 2, 3 (routing + layout wired enough to emit at least one empty or stub page).

---

#### Task 6: Asset pipeline — site-local paths, dedupe, safety [DONE] [Score:8] [Agent:heavy]

**Required context**

1. [ImplementationSpec.md §8](./ImplementationSpec.md)
2. [`assets.go`](../assets.go) (`resolveAssetUnderProject`, `copyProjectAsset`, `collectAssetPaths`)

- Canonical asset prefix under **`sites/<site-id>/assets/`** (document in code/README).
- Traverse site config + **all widget props** to collect referenced paths; copy into output preserving deduped relative paths ([ImplementationSpec.md §8](./ImplementationSpec.md)).
- Enforce no `..` escape and root confinement analogous to current [`resolveAssetUnderProject`](../assets.go).
- Errors must cite config path + offending field.

Dependencies: Task 1.

---

#### Task 7: Widget dispatcher and registry [DONE] [Score:5] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §6](./ImplementationSpec.md)
2. [WidgetRegistryV1.md](./WidgetRegistryV1.md)

- Map widget `type` string → renderer function or template execution.
- Unknown `type` → **fatal** with JSON pointer path ([ImplementationSpec.md §9](./ImplementationSpec.md)).
- Implement recursion **only** for layout widgets (`row`, `column`/`columns`, `grid`) per registry.

Dependencies: Task 3.

---

#### Task 8: Layout widgets — `row`, `column`/`columns`, `grid` [Score:5] [Agent:medium]

**Required context**

1. [WidgetRegistryV1.md — Layout widgets](./WidgetRegistryV1.md)
2. [`Template/styles.css`](../Template/styles.css)

- Templates under `Template/widgets/` for each layout type; render `children` in order.
- Add minimal CSS for gaps / grid `min_column_width` defaults ([WidgetRegistryV1.md](./WidgetRegistryV1.md)).

Dependencies: Task 7.

---

#### Task 9: Leaf widgets — intro, cover, follow_us, info_grid, images_grid [Score:8] [Agent:medium]

**Required context**

1. [WidgetRegistryV1.md](./WidgetRegistryV1.md) — sections for each type
2. Parity reference in [`html.go`](../html.go): `buildIntroSectionHTML`, cover banner, `buildFollowUsSection`, `buildOffersSectionHTML`, `buildPhotosSectionHTML`

- One template file per widget type under `Template/widgets/`.
- Match semantic HTML and class names where parity with Kometa matters ([ImplementationSpec.md §11](./ImplementationSpec.md)).

Dependencies: Tasks 7, 8.

---

#### Task 10: Leaf widget — `careers_tabs` (split-widget JS) [Score:5] [Agent:medium]

**Required context**

1. [WidgetRegistryV1.md — careers_tabs](./WidgetRegistryV1.md)
2. [`html.go`](../html.go) (`buildVacanciesWidget`, `buildSplitWidget`)
3. [`Template/split-widget.js`](../Template/split-widget.js)

- Preserve tab markup and `data-split-widget` behavior expected by existing JS.
- Wire vacancy props per registry; escape user text in templates.

Dependencies: Tasks 7, 9.

---

#### Task 11: Leaf widget — `apps_showcase` [Score:13] [Agent:heavy]

**Required context**

1. [WidgetRegistryV1.md — apps_showcase](./WidgetRegistryV1.md)
2. [`html.go`](../html.go) (`buildGamesColumns`, store rows, swiper HTML)
3. [`Template/game-swiper.js`](../Template/game-swiper.js), [`Template/styles.css`](../Template/styles.css)

- Port catalog card markup to templates; retain swiper/store/subscribe behavior ([ImplementationSpec.md §11](./ImplementationSpec.md)).
- Ensure [`collectAssetPaths`](../assets.go) extensions (Task 6) pick up all nested image/icon paths.

Dependencies: Tasks 7, 6.

---

#### Task 12: Leaf widget — `project_grid` [Score:5] [Agent:medium]

**Required context**

1. [WidgetRegistryV1.md — project_grid](./WidgetRegistryV1.md)
2. [`Template/styles.css`](../Template/styles.css)

- Implement cards with required fields; tags as non-interactive pills; responsive `auto-fit` grid; optional 16:9 media styling per registry.
- No client-side filter/sort/pagination ([UpgradeQuestions.md §5](./UpgradeQuestions.md)).

Dependencies: Tasks 7, 8.

---

#### Task 13 (optional): `media_swiper` widget [Score:5] [Agent:medium]

**Required context**

1. [WidgetRegistryV1.md — media_swiper](./WidgetRegistryV1.md)
2. [`Template/game-swiper.js`](../Template/game-swiper.js)

- Generic carousel: neutral markup/data attributes or reuse the screenshot carousel script with generalized selectors (document choice).
- Optional for “full” v1 release path ([ImplementationSpec.md §13](./ImplementationSpec.md)).

Dependencies: Task 7.

---

#### Task 14: Kometa parity acceptance site bundle [Score:8] [Agent:medium]

**Required context**

1. [ImplementationSpec.md §11](./ImplementationSpec.md)
2. [WidgetRegistryV1.md](./WidgetRegistryV1.md)
3. Reference content: [`config.json`](../config.json), [`KometaWebsite/`](../KometaWebsite/) (expected outcome shape)

- Add `sites/<id>/` with `site.json`, `pages/*.json`, and `assets/` mirroring Kometa content ([ImplementationSpec.md §11](./ImplementationSpec.md)).
- Run generator; manually compare output to existing Kometa site for intro, apps showcase, info/images grids, careers, follow-us, footer.

Dependencies: Tasks 11, 12, 10, 9.

---

#### Task 15: Cleanup, entrypoint, README [Score:3] [Agent:easy]

**Required context**

1. [ImplementationSpec.md](./ImplementationSpec.md)
2. [`main.go`](../main.go), [`README.md`](../README.md)

- Remove or isolate legacy single-config code path; ensure default UX matches interactive multi-site spec ([ImplementationSpec.md §10](./ImplementationSpec.md)).
- Update [README.md](../README.md): `sites/` workflow, link to [Specs/ImplementationSpec.md](./ImplementationSpec.md); ensure terminology matches **Task 16** after schema/CSS passes.

Dependencies: Task 14 (coordinate **Task 16** Phase B before freezing README details).

---

#### Task 16: Rename product to PortfolioWebsiteBuilder and purge deprecated naming [Score:8] [Agent:heavy]

**Required context**

1. [ImplementationSpec.md](./ImplementationSpec.md)
2. [WidgetRegistryV1.md](./WidgetRegistryV1.md)
3. [README.md](../README.md), [`go.mod`](../go.mod), [`types.go`](../types.go), [`html.go`](../html.go), [`assets.go`](../assets.go), [`Template/`](../Template/)

**Goals**

- **Canonical names:** Repository / product identifier **PortfolioWebsiteBuilder**; Go module **`portfoliowebsitebuilder`** ([`go.mod`](../go.mod)); human-readable title **Portfolio Website Builder**.
- **Forbidden branding substrings** (must not appear in authored docs, identifiers, JSON keys, CSS classes, filenames, or generator output after this task):  
  `GameDevStudio`, `GameDevStudio-SiteCreator`, `GameDev`, `sitecreator` (legacy module slug), and legacy marketing slug **Site Creator** tied to the old product name.
- **Neutral technical vocabulary:** Replace remaining `game*` / `Game*` / `games` tokens in **first-party** code and docs with portfolio/catalog terminology (examples: JSON `games` → `apps`, `game_store_icons` → neutral preset map key, `game_swiper` → `carousel`, CSS `game-*` → `catalog-*`, [`Template/game-swiper.js`](../Template/game-swiper.js) → renamed script + matching selectors). Third-party **client samples** (e.g. Kometa trade names in copy) may stay only inside clearly labeled customer content; regenerated HTML/CSS/JS under output folders must not reintroduce forbidden generator tokens.

**Steps**

- Run ripgrep baseline; repeat until clean for first-party paths (exclude vendor if added later):  
  `rg -n 'GameDevStudio|GameDevStudio-SiteCreator|GameDev\b|sitecreator|Site Creator' --glob '!**/KometaWebsite/**'` (tune `--glob` if sample sites stay vendored).
- Align [`README.md`](../README.md), all [`Specs/*.md`](./), and root docs with final JSON field names after schema renames.
- Regenerate [`KometaWebsite/`](../KometaWebsite/) (or other demo output) via `go run .` after Go/template/CSS/JS changes so committed artifacts match generators.

**Verification**

- `go test ./...` or `go build ./...` succeeds.
- Above ripgrep passes for excluded paths policy you document in the PR/commit body.

Dependencies: none (**recommended completed early**; Phase B schema/CSS may logically follow Task 11 if it touches the same templates).

---

### Ordering guidelines (reference)

- Skeleton (load/validate, routing, layout, CLI) before widgets.
- Asset pipeline in parallel dependency chain with templates once structs exist (Task 6 after Task 1).
- **`apps_showcase`** after simple leaf widgets and asset collection are proven.
- Integration sample (Kometa parity) before declaring README complete.
- Run **Task 16** early for branding/module cleanup; finish Phase B schema/CSS renames before freezing README (coordinate with Task 15).

### Raw story-point total

Sum of **[Score]** values above: **91** (drop Task 13 → **86**).
