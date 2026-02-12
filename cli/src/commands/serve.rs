use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    body::Body,
    extract::State,
    http::{header, Response, StatusCode},
    routing::{get, post},
    Json, Router,
};
use tokio::fs as async_fs;
use tower_http::cors::{Any, CorsLayer};

use crate::error::Result;
use crate::spike::Spike;

const DEFAULT_PORT: u16 = 3847;
const WIDGET_JS: &str = include_str!("../../assets/spikes.js");
const DASHBOARD_HTML: &str = include_str!("../../assets/dashboard.html");
const REVIEW_JS: &str = include_str!("../../assets/review.js");

pub struct ServeOptions {
    pub port: u16,
    pub directory: String,
    pub marked: bool,
}

#[derive(Clone)]
struct AppState {
    serve_dir: PathBuf,
    spikes_dir: PathBuf,
    marked: bool,
}

pub fn run(opts: ServeOptions) -> Result<()> {
    let port = if opts.port == 0 { DEFAULT_PORT } else { opts.port };
    let serve_dir = PathBuf::from(&opts.directory).canonicalize().map_err(|e| {
        crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory not found: {} ({})", opts.directory, e),
        ))
    })?;

    let spikes_dir = serve_dir.join(".spikes");
    if !spikes_dir.exists() {
        fs::create_dir_all(&spikes_dir)?;
    }

    let feedback_file = spikes_dir.join("feedback.jsonl");
    if !feedback_file.exists() {
        fs::File::create(&feedback_file)?;
    }

    let state = AppState {
        serve_dir: serve_dir.clone(),
        spikes_dir,
        marked: opts.marked,
    };

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/spikes.js", get(serve_widget))
            .route("/review.js", get(serve_review))
            .route("/dashboard", get(serve_dashboard))
            .route("/spikes", get(get_spikes))
            .route("/spikes", post(save_spike))
            .fallback(serve_static)
            .layer(cors)
            .with_state(state.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        println!();
        println!("  🗡️  Spikes server running");
        println!();
        println!("  Local:      http://localhost:{}", port);
        println!("  Directory:  {}", serve_dir.display());
        println!("  Dashboard:  http://localhost:{}/dashboard", port);
        if state.marked {
            println!("  Review:     Marked mode enabled - spike markers will appear on pages");
        }
        println!();
        println!("  Press Ctrl+C to stop");
        println!();

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    Ok(())
}

async fn serve_widget() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(WIDGET_JS))
        .unwrap()
}

async fn serve_dashboard() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(DASHBOARD_HTML))
        .unwrap()
}

async fn serve_review() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(REVIEW_JS))
        .unwrap()
}

async fn get_spikes(State(state): State<AppState>) -> Response<Body> {
    let feedback_file = state.spikes_dir.join("feedback.jsonl");

    let content = match async_fs::read_to_string(&feedback_file).await {
        Ok(c) => c,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("[]"))
                .unwrap();
        }
    };

    let spikes: Vec<Spike> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let json = serde_json::to_string(&spikes).unwrap_or_else(|_| "[]".to_string());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))
        .unwrap()
}

async fn save_spike(
    State(state): State<AppState>,
    Json(spike): Json<Spike>,
) -> Response<Body> {
    let feedback_file = state.spikes_dir.join("feedback.jsonl");

    let mut json = match serde_json::to_string(&spike) {
        Ok(j) => j,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(r#"{{"error":"{}"}}"#, e)))
                .unwrap();
        }
    };
    json.push('\n');

    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&feedback_file)
        .and_then(|mut file| file.write_all(json.as_bytes()));

    match result {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"status":"saved"}"#))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!(r#"{{"error":"{}"}}"#, e)))
            .unwrap(),
    }
}

async fn serve_static(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Response<Body> {
    let path = request.uri().path();
    let path = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let file_path = state.serve_dir.join(path);

    if !file_path.starts_with(&state.serve_dir) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }

    let file_path = if file_path.is_dir() {
        file_path.join("index.html")
    } else {
        file_path
    };

    match async_fs::read(&file_path).await {
        Ok(content) => {
            let content_type = guess_content_type(&file_path);
            
            // If marked mode is enabled and this is an HTML file, inject review.js
            let final_content = if state.marked && content_type.starts_with("text/html") {
                inject_review_script(content)
            } else {
                content
            };
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from(final_content))
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

fn inject_review_script(content: Vec<u8>) -> Vec<u8> {
    let html = match String::from_utf8(content.clone()) {
        Ok(s) => s,
        Err(_) => return content,
    };
    
    let script_tag = r#"<script src="/review.js"></script>"#;
    
    // Try to inject before </body>
    if let Some(pos) = html.to_lowercase().rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script_tag.len() + 1);
        result.push_str(&html[..pos]);
        result.push_str(script_tag);
        result.push('\n');
        result.push_str(&html[pos..]);
        return result.into_bytes();
    }
    
    // Try to inject before </html>
    if let Some(pos) = html.to_lowercase().rfind("</html>") {
        let mut result = String::with_capacity(html.len() + script_tag.len() + 1);
        result.push_str(&html[..pos]);
        result.push_str(script_tag);
        result.push('\n');
        result.push_str(&html[pos..]);
        return result.into_bytes();
    }
    
    // Just append at the end
    let mut result = html;
    result.push_str(script_tag);
    result.into_bytes()
}

fn guess_content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}
