# Agent guide

Instructions for AI agents and contributors working in this repository.

## Read first

| Doc | When to use it |
|-----|----------------|
| [`docs/architecture.md`](./architecture.md) | System map, data flow, which crate/folder to edit |
| [`docs/widgets.md`](./widgets.md) | Widget types, props, nesting, demo examples |
| [`README.md`](../README.md) | User-facing install, run, and author quick start |
| [`studio/README.md`](../studio/README.md) | Tauri dev setup, invoke commands, studio features |
| [`crates/core/README.md`](../crates/core/README.md) | Minijinja choice, escaping policy, core public API |
| [`docs/schema/`](./schema/) | Machine-readable JSON Schema (source of truth for props) |
| [`Specs/Changelog.md`](../Specs/Changelog.md) | Implementation history — append on meaningful changes |

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
| New widget or breaking prop change | `docs/schema/page.schema.json`, `crates/core/src/widgets.rs`, `Template/widgets/`, `docs/widgets.md`, `README.md` widget table, `content/demo/`, `Specs/Changelog.md` |
| Widget HTML/CSS/JS only (same props) | Usually `Specs/Changelog.md` only; mention Template paths |
| New site.json field or theme token | `docs/schema/site.schema.json`, studio form if exposed, `docs/architecture.md` if structural, `Specs/Changelog.md` |
| Studio feature or invoke command | `studio/README.md`, `Specs/Changelog.md` |
| Core API or pipeline behavior | `crates/core/README.md` and/or `docs/architecture.md`, `Specs/Changelog.md` |
| Author workflow (install, run, publish) | `README.md`, `Specs/Changelog.md` if notable |
| Bug fix with no author-visible change | `Specs/Changelog.md` (brief entry) |

If unsure whether a doc needs updating, add a short Changelog entry and update the most specific doc affected.

## Changelog format

Append a new section at the **top** of [`Specs/Changelog.md`](../Specs/Changelog.md):

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
content/           Author JSON and assets
Template/          Shared HTML/CSS/JS templates
crates/core/       Generator engine
crates/cli/        CLI binary
studio/            Svelte UI
src-tauri/         Desktop shell + commands
docs/              Architecture, widgets, agent rules, JSON Schema
Specs/Changelog.md Implementation log
```
