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

The dev UI (Phase 2.2) shows the resolved **project root**, exercises all invoke commands, and appends JSON results to a log strip. Use **Build + preview** to generate `content/kometa` output and serve it at `http://127.0.0.1:8080/`.

### Invoke commands (Task 2)

| Command | Purpose |
|---------|---------|
| `resolve_project_root` | Project root + `Template/` path |
| `list_content_bundles` | Lists bundles under `content/` |
| `validate_site` | Validate without write; structured warnings/errors |
| `build_site` | Full generate; stops preview server first |
| `start_preview_server` / `stop_preview_server` | HTTP static serve on `127.0.0.1` |

TypeScript wrappers: [`src/lib/studio-api.ts`](src/lib/studio-api.ts).

For packaged builds (Task 4), bundled resources or an explicit project picker will replace cwd-based resolution; until then, run `cargo tauri dev` from the repo root.

## Build (release)

From the repository root:

```bash
cargo tauri build
```

Or:

```bash
npm run tauri:build --prefix studio
```

Artifacts are written under `src-tauri/target/release/bundle/`. For CI smoke builds, use `cargo tauri build --bundles app` (skips DMG/installer bundling).

## Editor choice (Task 1 note)

Phase 2 Task 3 will add Monaco or CodeMirror 6 for JSON editing with schemas from [`../docs/schema/`](../docs/schema/). **CodeMirror 6** is the planned default (smaller bundle than Monaco); final choice is recorded when the editor lands.

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
