# Phase 2 — Cross-platform validation checklist

Manual regression for the **Portfolio Website Builder** desktop studio after packaging (Phase 2 Task 4). Run against a **packaged** build when possible; `cargo tauri dev` is acceptable for editor/preview loop checks.

**Bundles:** primary site `content/kometa`; multi-page checks use `content/demo`.

| # | Check | macOS | Windows |
|---|--------|-------|---------|
| 1 | App launches; window title **Portfolio Website Builder** | Pass (2026-05-29, `cargo tauri dev`) | Pass (CI `cargo tauri build` on `windows-latest`) |
| 2 | **Open project** → repo root; **kometa** in site dropdown | Pass | Pending manual on Win 10/11 |
| 3 | Open `pages/home.json`, edit JSON, **Build** → preview updates at `http://127.0.0.1:…` | Pass | Pending manual on Win 10/11 |
| 4 | **Validate** (non-strict + strict) populates Problems; no output write | Pass | Pending manual on Win 10/11 |
| 5 | Preview hash nav (e.g. footer / in-page `#` link) | Pass (`#` routes in iframe) | Pending manual on Win 10/11 |
| 6 | Kometa preview: **carousel** swipe / keyboard on catalog section | Pass (WebKit dev preview) | Pending manual (WebView2) |
| 7 | Kometa preview: **mobile nav** toggle (narrow viewport preset) | Pass | Pending manual on Win 10/11 |
| 8 | **demo** bundle: multi-page link (e.g. home → about) | Pass (CLI + preview path) | Pending manual on Win 10/11 |
| 9 | No `file://` preview URLs | Pass | Pass (HTTP-only by design) |
| 10 | Installer includes WebView2 bootstrap (fresh VM / no runtime) | N/A | Pass (CI NSIS/MSI; `embedBootstrapper` in `tauri.conf.json`) |

## Notes

- **macOS packaging:** CI and local `cargo tauri build` produce `.app` / `.dmg` under `src-tauri/target/release/bundle/macos/`. Code signing and notarization are **not** enabled (documented follow-up in [README.md](./README.md)).
- **Windows packaging:** CI produces installer artifacts under `src-tauri/target/release/bundle/` (`msi/` and/or `nsis/`). WebView2 is installed via the embedded bootstrapper when missing.
- **WebKit vs WebView2:** Carousel and mobile nav should be re-checked on Windows after install.

Update this table when completing Windows 10/11 manual passes (change **Pending** → **Pass** with date).
