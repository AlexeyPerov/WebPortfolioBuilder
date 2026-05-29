use portfoliowebsitebuilder_core::serve::resolve_static_file_path;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub struct PreviewServerState {
    inner: std::sync::Mutex<Option<RunningPreview>>,
}

impl Default for PreviewServerState {
    fn default() -> Self {
        Self {
            inner: std::sync::Mutex::new(None),
        }
    }
}

struct RunningPreview {
    server: Arc<Server>,
    thread: JoinHandle<()>,
}

impl PreviewServerState {
    pub fn stop(&self) {
        let mut guard = self.inner.lock().expect("preview server lock");
        if let Some(running) = guard.take() {
            drop(running.server);
            let _ = running.thread.join();
        }
    }

    pub fn start(
        &self,
        output_dir: &Path,
        port: u16,
    ) -> Result<(u16, String), String> {
        self.stop();

        let output_dir = output_dir
            .canonicalize()
            .unwrap_or_else(|_| output_dir.to_path_buf());
        if !output_dir.is_dir() {
            return Err(format!(
                "output directory does not exist: {}",
                output_dir.display()
            ));
        }

        let addr = format!("127.0.0.1:{port}");
        let server = Arc::new(
            Server::http(&addr).map_err(|e| format!("cannot bind {addr}: {e}"))?,
        );
        let server_thread = server.clone();
        let dir = output_dir.clone();

        let thread = thread::spawn(move || {
            for request in server_thread.incoming_requests() {
                let dir = dir.clone();
                thread::spawn(move || handle_request(request, &dir));
            }
        });

        *self.inner.lock().expect("preview server lock") = Some(RunningPreview { server, thread });

        let url = format!("http://{addr}/");
        Ok((port, url))
    }

}

fn handle_request(request: tiny_http::Request, dir: &Path) {
    if request.method() == &Method::Get || request.method() == &Method::Head {
        if let Some(file_path) = resolve_static_file_path(dir, request.url()) {
            if let Ok(data) = std::fs::read(&file_path) {
                let content_type = mime_guess(&file_path);
                let mut response = Response::from_data(data).with_status_code(StatusCode(200));
                if let Ok(h) = Header::from_bytes("Content-Type", content_type.as_bytes()) {
                    response = response.with_header(h);
                }
                let _ = request.respond(response);
                return;
            }
        }
    }
    let _ = request.respond(Response::from_string("Not Found").with_status_code(StatusCode(404)));
}

fn mime_guess(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}
