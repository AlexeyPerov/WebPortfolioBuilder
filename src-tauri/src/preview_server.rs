use portfoliowebsitebuilder_core::serve::resolve_static_file_path;
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub struct PreviewServerState {
    inner: Mutex<Option<RunningPreview>>,
}

impl Default for PreviewServerState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

struct RunningPreview {
    stop: Arc<AtomicBool>,
    port: u16,
    output_dir: Arc<Mutex<PathBuf>>,
    thread: JoinHandle<()>,
}

impl PreviewServerState {
    pub fn stop(&self) {
        let mut guard = self.inner.lock().expect("preview server lock");
        if let Some(running) = guard.take() {
            running.stop.store(false, Ordering::SeqCst);
            wake_listener(running.port);
            let _ = running.thread.join();
        }
    }

    pub fn start(
        &self,
        output_dir: &Path,
        port: u16,
    ) -> Result<(u16, String), String> {
        let output_dir = output_dir
            .canonicalize()
            .unwrap_or_else(|_| output_dir.to_path_buf());
        if !output_dir.is_dir() {
            return Err(format!(
                "output directory does not exist: {}",
                output_dir.display()
            ));
        }

        {
            let guard = self.inner.lock().expect("preview server lock");
            if let Some(running) = guard.as_ref() {
                if running.port == port {
                    *running
                        .output_dir
                        .lock()
                        .expect("preview output dir lock") = output_dir;
                    let url = format!("http://127.0.0.1:{port}/");
                    return Ok((port, url));
                }
            }
        }

        self.stop();

        let addr = format!("127.0.0.1:{port}");
        let server = bind_server(&addr)?;
        let stop = Arc::new(AtomicBool::new(true));
        let stop_listener = stop.clone();
        let output_dir_shared = Arc::new(Mutex::new(output_dir));

        let dir_for_thread = output_dir_shared.clone();
        let thread = thread::spawn(move || {
            for request in server.incoming_requests() {
                if !stop_listener.load(Ordering::SeqCst) {
                    break;
                }
                handle_request(request, &dir_for_thread);
            }
        });

        *self.inner.lock().expect("preview server lock") = Some(RunningPreview {
            stop,
            port,
            output_dir: output_dir_shared,
            thread,
        });

        let url = format!("http://{addr}/");
        Ok((port, url))
    }
}

fn wake_listener(port: u16) {
    let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{port}")) else {
        return;
    };
    let _ = write!(
        stream,
        "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    );
}

fn bind_server(addr: &str) -> Result<Server, String> {
    const ATTEMPTS: u32 = 10;
    const DELAY: Duration = Duration::from_millis(25);
    let mut last_err = String::new();
    for attempt in 0..ATTEMPTS {
        match Server::http(addr) {
            Ok(server) => return Ok(server),
            Err(e) => {
                last_err = format!("cannot bind {addr}: {e}");
                if attempt + 1 < ATTEMPTS {
                    thread::sleep(DELAY);
                }
            }
        }
    }
    Err(last_err)
}

fn handle_request(request: tiny_http::Request, output_dir: &Arc<Mutex<PathBuf>>) {
    let dir = output_dir
        .lock()
        .expect("preview output dir lock")
        .clone();
    if request.method() == &Method::Get || request.method() == &Method::Head {
        if let Some(file_path) = resolve_static_file_path(&dir, request.url()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    fn pick_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind ephemeral port")
            .local_addr()
            .expect("local addr")
            .port()
    }

    fn http_get(port: u16, path: &str) -> String {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect to preview server");
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .expect("write request");
        let mut body = String::new();
        stream.read_to_string(&mut body).expect("read response");
        body
    }

    #[test]
    fn preview_server_stop_and_restart_does_not_hang() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("index.html"), b"<html>preview ok</html>").unwrap();

        let state = PreviewServerState::default();
        let port = pick_port();

        let (_, url) = state
            .start(dir.path(), port)
            .expect("first preview start");
        assert_eq!(url, format!("http://127.0.0.1:{port}/"));

        let body = http_get(port, "/");
        assert!(body.contains("preview ok"), "body: {body}");

        let stop_started = Instant::now();
        state.stop();
        assert!(
            stop_started.elapsed() < Duration::from_secs(2),
            "preview stop hung for {:?}",
            stop_started.elapsed()
        );

        let restart_started = Instant::now();
        state.start(dir.path(), port).expect("second preview start");
        assert!(
            restart_started.elapsed() < Duration::from_secs(2),
            "preview restart hung for {:?}",
            restart_started.elapsed()
        );

        let body = http_get(port, "/");
        assert!(body.contains("preview ok"), "body after restart: {body}");

        state.stop();
    }

    #[test]
    fn preview_server_start_updates_directory_on_same_port() {
        let dir_a = tempdir().expect("tempdir a");
        let dir_b = tempdir().expect("tempdir b");
        std::fs::write(dir_a.path().join("index.html"), b"<html>site a</html>").unwrap();
        std::fs::write(dir_b.path().join("index.html"), b"<html>site b</html>").unwrap();

        let state = PreviewServerState::default();
        let port = pick_port();

        state.start(dir_a.path(), port).expect("first start");
        let body = http_get(port, "/");
        assert!(body.contains("site a"), "body: {body}");

        state
            .start(dir_b.path(), port)
            .expect("switch output directory");
        let body = http_get(port, "/");
        assert!(body.contains("site b"), "body after switch: {body}");

        state.stop();
    }

    #[test]
    fn preview_server_serves_index_with_cache_buster_query() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("index.html"), b"<html>cached ok</html>").unwrap();

        let state = PreviewServerState::default();
        let port = pick_port();

        state.start(dir.path(), port).expect("start");
        let body = http_get(port, "/?_r=42&out=%2Ftmp%2Fignored");
        assert!(body.contains("cached ok"), "body: {body}");

        state.stop();
    }

    #[test]
    fn preview_server_start_on_same_port_does_not_rebind() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("index.html"), b"ok").unwrap();

        let state = PreviewServerState::default();
        let port = pick_port();

        state.start(dir.path(), port).expect("first start");
        let started = Instant::now();
        state.start(dir.path(), port).expect("second start on same port");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "second start hung for {:?}",
            started.elapsed()
        );

        state.stop();
    }
}
