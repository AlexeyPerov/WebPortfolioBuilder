# Portfolio Website Builder — Studio

Desktop studio shell for **Portfolio Website Builder** (Phase 2). The UI is **Svelte 5 + Vite** in this directory; the native shell and Rust commands live in [`../src-tauri/`](../src-tauri/) and call [`../crates/core`](../crates/core) in-process.

## Prerequisites

| Tool | Version | Notes |
|------|---------|--------|
| [Rust](https://www.rust-lang.org/tools/install) | **1.77+** | Same as workspace `rust-version` in root [`Cargo.toml`](../Cargo.toml). |
| [Node.js](https://nodejs.org/) | **20+** (LTS recommended) | For Vite dev server and frontend build. |
| [Tauri CLI](https://v2.tauri.app/reference/cli/) | **2.x** | Install once: `cargo install tauri-cli --locked` |

### Platform dependencies (Tauri 2)

Follow the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS:

- **macOS:** Xcode Command Line Tools (`xcode-select --install`).
- **Windows:** Microsoft C++ Build Tools, WebView2 (usually preinstalled on Windows 10/11).
- **Linux** (optional): `webkit2gtk`, `libayatana-appindicator`, etc. — not required for macOS/Windows CI targets.

## First-time setup

From the **repository root**:

```bash
npm install --prefix studio
cargo install tauri-cli --locked   # skip if already installed
```

## Development

Run the full desktop app (Vite + Tauri) from the **repository root**:

```bash
cargo tauri dev
```

This starts the Vite dev server on `http://localhost:5173`, compiles `src-tauri`, and opens a window with the Svelte UI.

Alternative (same behavior, uses the npm-wrapped CLI):

```bash
npm run tauri:dev --prefix studio
```

Frontend-only (browser, no Tauri APIs):

```bash
npm run dev --prefix studio
```

## Project root and `Template/` resolution

Build and validate commands resolve paths relative to the **project root** — the directory that contains `Template/`, `content/`, and `docs/schema/`.

During **`cargo tauri dev`**, run from the repo root so the current working directory is the project root. The studio invokes `portfoliowebsitebuilder_core::resolve_project_root()`, which:

1. Uses **cwd** when `content/kometa/site.json` exists there (or when cwd is the repo root in dev).
2. Falls back to the directory containing the running executable (packaged app).

The studio UI (Phase 2.3) provides the full author layout: toolbar, bundle file tree, tabbed JSON editor, Problems panel, HTTP preview iframe, and build log. **Open project** uses a native folder dialog and persists the last path in app config. **Build** saves dirty editor buffers, runs `build_site`, then `start_preview_server` at `http://127.0.0.1:8080/` (no `file://` URLs).

### Invoke commands

| Command | Purpose |
|---------|---------|
| `resolve_project_root` | Project root + `Template/` path (auto-detect) |
| `project_info_for_root` | Validate a chosen project folder |
| `get_studio_settings` / `save_studio_settings` | Persist last project path |
| `list_content_bundles` | Lists bundles under `content/` |
| `list_bundle_files_cmd` | File tree entries for active bundle |
| `read_bundle_file_cmd` / `write_bundle_file_cmd` | UTF-8 load/save (`site.json`, `pages/*.json`) |
| `validate_site` | Validate without write; structured warnings/errors |
| `build_site` | Full generate; stops preview server first |
| `set_auto_rebuild` | Start/stop debounced file watcher on active bundle |
| `start_preview_server` / `stop_preview_server` | HTTP static serve on `127.0.0.1` |

TypeScript wrappers: [`src/lib/studio-api.ts`](src/lib/studio-api.ts). UI components live under [`src/components/`](src/components/).

Use **Open project** to point at the repo root (or rely on auto-detect when `cargo tauri dev` runs from the repo root). Select `content/kometa`, open `pages/home.json`, edit, then **Build** to refresh the preview.

### Auto-rebuild (Phase 3)

Enable **Auto-rebuild** in the toolbar to watch `content/<active-site>/` recursively (JSON and assets under the bundle; `Results/` is not watched). After you save a file, the studio debounces changes for **500 ms** (`WATCH_DEBOUNCE_MS` in `src-tauri/src/content_watcher.rs`, mirrored as `AUTO_REBUILD_DEBOUNCE_MS` in the UI) and then runs `build_site`, restarts the preview server, and updates the Problems panel and build log. Rapid saves within that window coalesce to a single build.

Auto-rebuild is **off by default**. With it disabled, behavior matches Phase 2 manual **Build** only. Build failures are shown in Problems/log; the file watcher keeps running.

## Build (release)

From the repository root:

```bash
cargo tauri build
```

Or:

```bash
npm run tauri:build --prefix studio
```

Release artifacts are written under `src-tauri/target/release/bundle/`:

| OS | Typical artifacts | Install / run |
|----|-------------------|---------------|
| **macOS** | `macos/*.app`, `macos/*.dmg` | Open the `.app`, or mount the DMG and drag to Applications |
| **Windows** | `msi/*.msi`, `nsis/*-setup.exe` | Run the installer; WebView2 is installed via embedded bootstrapper if missing (Windows 10/11) |

GitHub Actions (**Rust CI** → `cargo tauri build`) uploads these folders as workflow artifacts for `macos-latest` and `windows-latest`.

Quick local smoke without DMG/installer:

```bash
cargo tauri build --bundles app
```

### macOS code signing (follow-up)

CI and local release builds are **unsigned** by default. For distribution outside your machine, configure Apple Developer signing and notarization in Tauri — see [macOS code signing](https://v2.tauri.app/distribute/sign/macos/). Not required for dev or internal CI artifacts.

### Cross-platform validation

Manual regression (kometa carousel, mobile nav, demo multi-page links, hash preview) is tracked in [VALIDATION-CHECKLIST.md](./VALIDATION-CHECKLIST.md).

## JSON editor

**CodeMirror 6** (`@codemirror/lang-json` + `@codemirror/lint`) with **Ajv** validation against [`../docs/schema/site.schema.json`](../docs/schema/site.schema.json) and [`page.schema.json`](../docs/schema/page.schema.json). Monaco was not used to keep the bundle smaller (~515 kB minified JS for the full studio build).

## Layout

```
studio/           ← Svelte 5 UI (this directory)
src-tauri/        ← Tauri 2 shell, invoke commands
crates/core/      ← site load, validate, render (Phase 1)
Template/         ← layout + widgets (read at build time)
content/          ← author bundles
docs/schema/      ← JSON Schema for editors
```

See [Specs/tauri/requirements.md](../Specs/tauri/requirements.md) and [execution-plan-phase-2.md](../Specs/tauri/execution-plan-phase-2.md) for the full Phase 2 plan.
