use crate::error::{CoreError, CoreResult};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tiny_http::{Header, Method, Response, Server, StatusCode};

const DEFAULT_SERVE_PORT: u16 = 8080;

pub fn default_serve_port() -> u16 {
    DEFAULT_SERVE_PORT
}

pub fn serve_static_dir(dir: &Path, port: u16, stdout: &mut dyn Write) -> CoreResult<()> {
    let addr = format!("127.0.0.1:{port}");
    let server =
        Server::http(&addr).map_err(|e| CoreError::msg(format!("cannot bind {addr}: {e}")))?;
    let running = Arc::new(AtomicBool::new(true));
    let running_ctrl = running.clone();

    ctrlc_handler(running_ctrl);

    writeln!(stdout, "Serving {} at http://{addr}/", dir.display())?;
    writeln!(stdout, "Press Ctrl+C to stop.")?;

    let dir = dir.to_path_buf();
    for request in server.incoming_requests() {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        let dir = dir.clone();
        thread::spawn(move || handle_request(request, &dir));
    }
    Ok(())
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    });
}

/// Maps an HTTP URL path to a file under `dir`, rejecting path traversal.
pub fn resolve_static_file_path(dir: &Path, url_path: &str) -> Option<std::path::PathBuf> {
    let url_path = url_path.split('?').next().unwrap_or("/");
    let rel = url_path.trim_start_matches('/');
    let file_path = if rel.is_empty() {
        dir.join("index.html")
    } else {
        dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    let file_path = if file_path.is_dir() {
        file_path.join("index.html")
    } else {
        file_path
    };
    if file_path.starts_with(dir) && file_path.is_file() {
        Some(file_path)
    } else {
        None
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolve_static_file_path_serves_index_html() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), b"<html>ok</html>").unwrap();

        let root = dir.path().canonicalize().unwrap();
        let got = resolve_static_file_path(&root, "/index.html").unwrap();
        assert_eq!(fs::read_to_string(got).unwrap(), "<html>ok</html>");
    }

    #[test]
    fn resolve_static_file_path_root_url() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), b"home").unwrap();

        let root = dir.path().canonicalize().unwrap();
        assert!(resolve_static_file_path(&root, "/").is_some());
    }

    #[test]
    fn resolve_static_file_path_rejects_traversal() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), b"home").unwrap();
        let root = dir.path().canonicalize().unwrap();
        assert!(resolve_static_file_path(&root, "/../etc/passwd").is_none());
    }

    #[test]
    fn mime_guess_html_and_css() {
        assert_eq!(mime_guess(Path::new("a.html")), "text/html; charset=utf-8");
        assert_eq!(mime_guess(Path::new("a.css")), "text/css; charset=utf-8");
    }
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
