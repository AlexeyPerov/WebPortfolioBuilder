use crate::site_ops::run_build;
use crate::diagnostics::{BuildSiteResult, PreviewServerInfo};
use crate::preview_server::PreviewServerState;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use portfoliowebsitebuilder_core::resolve_site_dir;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Debounce interval for content file changes (ms). Documented in studio/README.md.
pub const WATCH_DEBOUNCE_MS: u64 = 500;

pub const EVENT_WATCH_REBUILD_COMPLETE: &str = "watch-rebuild-complete";

#[derive(Clone, Serialize)]
pub struct WatchRebuildComplete {
    pub build: BuildSiteResult,
    pub preview: Option<PreviewServerInfo>,
}

struct ActiveWatch {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub struct ContentWatcherState {
    active: Mutex<Option<ActiveWatch>>,
    rebuilding: Mutex<()>,
}

impl Default for ContentWatcherState {
    fn default() -> Self {
        Self {
            active: Mutex::new(None),
            rebuilding: Mutex::new(()),
        }
    }
}

impl ContentWatcherState {
    pub fn stop(&self) {
        if let Some(watch) = self.active.lock().expect("content watcher lock").take() {
            watch.stop.store(true, Ordering::Relaxed);
            let _ = watch.handle.join();
        }
    }

    pub fn start(
        &self,
        app: AppHandle,
        project_root: PathBuf,
        site_path: String,
        strict: bool,
        preview_port: u16,
    ) -> Result<(), String> {
        self.stop();

        let site_dir = resolve_site_dir(&project_root, &site_path);
        if !site_dir.is_dir() {
            return Err(format!("bundle not found: {}", site_dir.display()));
        }

        let (trigger_tx, trigger_rx) = mpsc::channel();
        let debounce_ms = Duration::from_millis(WATCH_DEBOUNCE_MS);

        let mut debouncer = new_debouncer(debounce_ms, move |result: DebounceEventResult| {
            if result.is_ok() {
                let _ = trigger_tx.send(());
            }
        })
        .map_err(|e| format!("file watcher: {e}"))?;

        debouncer
            .watcher()
            .watch(&site_dir, notify::RecursiveMode::Recursive)
            .map_err(|e| format!("watch {}: {e}", site_dir.display()))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let app_thread = app.clone();
        let project_root_s = project_root.clone();
        let site_path_s = site_path.clone();

        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                match trigger_rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(()) => {
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }
                        run_watch_rebuild(
                            &app_thread,
                            &project_root_s,
                            &site_path_s,
                            strict,
                            preview_port,
                        );
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            drop(debouncer);
        });

        *self.active.lock().expect("content watcher lock") = Some(ActiveWatch { stop, handle });
        Ok(())
    }
}

fn run_watch_rebuild(
    app: &AppHandle,
    project_root: &PathBuf,
    site_path: &str,
    strict: bool,
    preview_port: u16,
) {
    let watcher = app.state::<ContentWatcherState>();
    let _guard = match watcher.rebuilding.try_lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let build = run_build(project_root, site_path, strict);

    let preview = app.state::<PreviewServerState>();

    let preview_info = if build.ok {
        build.output_dir.as_ref().and_then(|output_dir| {
            let path = PathBuf::from(output_dir);
            preview
                .start(&path, preview_port)
                .ok()
                .map(|(port, url)| PreviewServerInfo {
                    url,
                    port,
                    output_dir: output_dir.clone(),
                })
        })
    } else {
        None
    };

    let payload = WatchRebuildComplete {
        build,
        preview: preview_info,
    };

    if let Err(e) = app.emit(EVENT_WATCH_REBUILD_COMPLETE, payload) {
        log::warn!("failed to emit watch-rebuild-complete: {e}");
    }
}
