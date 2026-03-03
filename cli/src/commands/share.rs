use std::fs;
use std::path::{Path, PathBuf};

use crate::auth::AuthConfig;
use crate::error::{map_http_error, map_network_error, Error, Result};
use walkdir::WalkDir;

pub struct ShareOptions {
    pub directory: String,
    pub name: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub json: bool,
}

const INCLUDE_EXTENSIONS: &[&str] = &[
    "html", "css", "js", "json", "png", "jpg", "jpeg", "gif", "svg", "woff", "woff2", "ico",
];

const EXCLUDE_DIRS: &[&str] = &[".spikes", "node_modules", ".git"];
const EXCLUDE_FILES: &[&str] = &[".DS_Store"];

pub fn run(options: ShareOptions) -> Result<()> {
    let token = AuthConfig::token()?
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' first.",
            ))
        })?;
    let dir_path = Path::new(&options.directory);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory not found: {}", options.directory),
        )));
    }

    let files = collect_files(dir_path)?;
    if files.is_empty() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No uploadable files found in directory",
        )));
    }

    let slug = options.name.unwrap_or_else(|| {
        dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let result = upload_share(&token, dir_path, &files, &slug, options.password.as_deref(), &options.host)?;

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "url": result.url,
                "slug": result.slug,
                "files": result.file_count
            })
        );
    } else {
        println!();
        println!("  ┌────────────────────────────────────────────┐");
        println!("  │  /  Your mockups are live                  │");
        println!("  │                                            │");
        println!("  │  {}  │", pad_center(&result.url, 40));
        println!("  │                                            │");
        println!("  │  Share this link with reviewers.           │");
        println!("  │  Feedback syncs automatically.             │");
        println!("  │                                            │");
        println!("  │  Pull feedback: spikes pull --from <url>   │");
        println!("  │  Delete share:  spikes unshare <url>       │");
        println!("  └────────────────────────────────────────────┘");
        println!();
    }

    Ok(())
}

fn pad_center(s: &str, width: usize) -> String {
    if s.len() >= width {
        return s.to_string();
    }
    let pad = width - s.len();
    let left = pad / 2;
    let right = pad - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip excluded directories
        if path.is_dir() {
            continue;
        }

        // Check if any parent is excluded
        let should_skip = path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| EXCLUDE_DIRS.contains(&n))
                .unwrap_or(false)
        });
        if should_skip {
            continue;
        }

        // Skip excluded files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if EXCLUDE_FILES.contains(&name) {
                continue;
            }
        }

        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if INCLUDE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

struct ShareResult {
    url: String,
    slug: String,
    file_count: usize,
}

fn upload_share(
    token: &str,
    base_dir: &Path,
    files: &[PathBuf],
    slug: &str,
    password: Option<&str>,
    host: &str,
) -> Result<ShareResult> {
    use ureq::Agent;

    let agent = Agent::new();
    let url = format!("{}/shares", host.trim_end_matches('/'));

    // Build multipart form
    let boundary = format!("----SpikesUpload{}", chrono::Utc::now().timestamp_millis());
    let mut body = Vec::new();

    // Add metadata field (includes slug and password)
    let mut metadata = serde_json::json!({ "name": slug });
    if let Some(pw) = password {
        metadata["password"] = serde_json::Value::String(pw.to_string());
    }
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"metadata\"\r\n\r\n");
    body.extend_from_slice(metadata.to_string().as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add each file
    for file_path in files {
        let relative = file_path
            .strip_prefix(base_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let content = fs::read(file_path)?;
        let mime = guess_mime(file_path);

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",
                relative
            )
            .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime).as_bytes());
        body.extend_from_slice(&content);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let response = match agent
        .post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().ok();
            return Err(map_http_error(status, body_text.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status != 200 && status != 201 {
        let body_text = response.into_string().ok();
        return Err(map_http_error(status, body_text.as_deref()));
    }

    let body_text = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&body_text)?;

    let result_url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let result_slug = parsed
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or(slug)
        .to_string();

    Ok(ShareResult {
        url: result_url,
        slug: result_slug,
        file_count: files.len(),
    })
}

fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
