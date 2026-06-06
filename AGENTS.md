# Agent guide

Instructions for AI agents and contributors working in this repository.

## Read first

| Doc | When to use it |
|-----|----------------|
| [`docs/architecture.md`](docs/architecture.md) | System map, data flow, which crate/folder to edit |
| [`docs/widgets.md`](docs/widgets.md) | Widget types, props, nesting, demo examples |
| [`README.md`](README.md) | User-facing install, run, and author quick start |
| [`studio/README.md`](studio/README.md) | Tauri dev setup, invoke commands, studio features |
| [`crates/core/README.md`](crates/core/README.md) | Minijinja choice, escaping policy, core public API |
| [`docs/schema/`](docs/schema/) | Machine-readable JSON Schema (source of truth for props) |
| [`Specs/Changelog.md`](Specs/Changelog.md) | Implementation history — append on meaningful changes |

**Do not treat `Specs/archive/` as current documentation.** It is being removed; use the docs listed above.

## Repository conventions

- Content bundles live under **`content/<site-id>/`**, not `sites/`.
- **Project root** must contain `content/`, `Template/`, and `docs/schema/`.
- Run **`cargo tauri dev`** from the repo root for the desktop app.
- Prefer **minimal, focused diffs**; match existing naming and patterns in the touched area.
- **Rust validation** is authoritative for build errors; JSON Schema supports the studio editor.
- Widget layout type is **`column`** only (`columns` is rejected).
- Generated output goes to the path in `site.json` → `output_folder` (often `Results/...`).

## When to update documentation

Use this matrix after code or content changes:

| Change | Update |
|--------|--------|
| New widget or breaking prop change | `docs/schema/page.schema.json`, `crates/core/src/widgets.rs`, `Template/widgets/`, **`docs/widgets.md`**, `README.md` widget table, `content/demo/`, `Specs/Changelog.md` |
| Widget HTML/CSS/JS only (same props) | Usually `Specs/Changelog.md` only; mention Template paths |
| New or changed widget prop (non-breaking) | **`docs/widgets.md`**, `docs/schema/page.schema.json` if schema changed, `Specs/Changelog.md` |
| New site.json field or theme token | `docs/schema/site.schema.json`, studio form if exposed, **`docs/architecture.md`** if structural, `Specs/Changelog.md` |
| Page layout field (`layout.*` on pages) | `docs/schema/page.schema.json`, **`docs/architecture.md`**, **`docs/widgets.md`** if author-facing, `Specs/Changelog.md` |
| Studio feature or invoke command | `studio/README.md`, **`docs/architecture.md`** (studio → core path), `Specs/Changelog.md` |
| Core API or pipeline behavior | `crates/core/README.md` and/or **`docs/architecture.md`**, `Specs/Changelog.md` |
| New content bundle or example-site role | **`docs/architecture.md`** example-sites table, `README.md` if author-facing |
| Author workflow (install, run, publish) | `README.md`, `Specs/Changelog.md` if notable |
| Bug fix with no author-visible change | `Specs/Changelog.md` (brief entry) |

If unsure whether a doc needs updating, add a short Changelog entry and update the most specific doc affected.

### Keeping `docs/architecture.md` accurate

Update when repo structure, data flow, or contributor touch points change. After edits, verify:

| Section | Must match |
|---------|------------|
| Repository layout | Actual top-level folders (`content/`, `Template/`, `crates/`, `studio/`, `src-tauri/`, `docs/`) |
| Build pipeline steps | `crates/core/src/{bundles,config,routing,render,widgets}.rs` |
| Application layers table | `studio/src/`, `src-tauri/src/`, `crates/core/src/`, `crates/cli/` |
| Studio → core path | `src-tauri/src/site_ops.rs` invoke → `validate_site` / `generate_site` |
| Widget rendering notes | `PageScriptNeeds`, `layout.background_effect`, `Template/*.js` on disk |
| Content model (`site.json`, `pages/*.json`) | `docs/schema/*.schema.json` and `crates/core/src/types.rs` |
| Example sites table | Bundles under `content/` (note gitignored bundles such as `kometa/`) |
| Development commands | `cargo tauri dev`, CLI package name `portfoliowebsitebuilder-cli` |
| “What to edit” table | Cross-links to `docs/widgets.md` for widget work |

`AGENTS.md` lives at the **repo root**, not under `docs/`.

### Keeping `docs/widgets.md` accurate

This file is the **human-readable widget library**. JSON Schema (`docs/schema/page.schema.json`) is the machine source of truth for prop shapes; when code or schema changes, update both and keep them aligned.

After widget-related changes, verify:

| Check | Source of truth |
|-------|-----------------|
| Widget type list in Summary table | `page.schema.json` → `$defs.widgetType.enum` and `Template/widgets/*.html` |
| Per-widget prop tables | Schema `$defs/*Props`, parsers in `crates/core/src/widgets.rs` |
| Layout nesting rules | Only `row`, `column`, `grid` have `props.children` |
| Demo page map | `content/demo/pages/*.json` and `content/demo/site.json` nav |
| Studio form coverage | `studio/src/lib/widget-types.ts` → `SUPPORTED_FORM_WIDGET_TYPES` |
| Global rules (`column` vs `columns`, `enabled`, `id`) | `widgets.rs` validation and render paths |
| Script dependencies per widget | `collect_page_script_needs` in `widgets.rs` |

When adding a widget section, include: role, prop table, template path, studio form yes/no, and any client JS (`Template/*.js`).

### Doc sync before finishing

When your change touches widgets, the build pipeline, or site/page config:

1. Update the most specific doc (`docs/widgets.md` or `docs/architecture.md`).
2. Update `docs/schema/*.schema.json` if props or fields changed.
3. Update `README.md` widget table or author steps when behavior is user-visible.
4. Append `Specs/Changelog.md`.
5. Run validation: `cargo run -p portfoliowebsitebuilder-cli -- --validate --site content/demo`

## Changelog format

Append a new section at the **top** of [`Specs/Changelog.md`](Specs/Changelog.md):

```markdown
## YYYY-MM-DD — <Agent or author name> — <Short title>

- Bullet describing what changed and why.
- File paths or areas touched when helpful.
```

Separate sections with `---`.

## Code change guidelines

### Widget work

Typical touch points:

- `crates/core/src/widgets.rs` — render logic, script needs, validation
- `crates/core/src/html.rs` — shared HTML helpers
- `Template/widgets/<type>.html` — markup
- `Template/styles.css` — shared styles
- `Template/*.js` — client behavior when needed
- `docs/schema/page.schema.json` — prop definitions for editors
- `studio/src/` — form editors for supported widgets
- `crates/core/tests/widget_render.rs` — golden/render tests when behavior changes

### Validate before finishing

From repo root (adjust site path):

```bash
cargo run -p portfoliowebsitebuilder-cli -- --validate --site content/demo
cargo test -p portfoliowebsitebuilder-core
```

For studio-related changes, smoke-test with `cargo tauri dev` when feasible.

### Strict mode

`--validate --strict` treats unknown top-level keys and unknown widget prop keys as errors. Use when hardening configs.

## What not to do

- Do not remove or rename `Template/` without a migration plan.
- Do not commit secrets (`.env`, credentials).
- Do not expand scope into unrelated refactors.
- Do not reference removed `Specs/archive/` docs in new work.
- Do not create commits or PRs unless the user asks.

## Quick file map

```
AGENTS.md          This file — agent rules and doc maintenance
content/           Author JSON and assets
Template/          Shared HTML/CSS/JS templates
crates/core/       Generator engine
crates/cli/        CLI binary
studio/            Svelte UI
src-tauri/         Desktop shell + commands
docs/              Architecture, widgets reference, JSON Schema
Specs/Changelog.md Implementation log
```
