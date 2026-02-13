use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use walkdir::WalkDir;

pub struct ShareOptions {
    pub directory: String,
    pub name: Option<String>,
    pub json: bool,
}

struct AuthConfig {
    token: String,
}

const INCLUDE_EXTENSIONS: &[&str] = &[
    "html", "css", "js", "json", "png", "jpg", "jpeg", "gif", "svg", "woff", "woff2", "ico",
];

const EXCLUDE_DIRS: &[&str] = &[".spikes", "node_modules", ".git"];
const EXCLUDE_FILES: &[&str] = &[".DS_Store"];

pub fn run(options: ShareOptions) -> Result<()> {
    let auth = load_auth_config()?;
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

    let result = upload_share(&auth, dir_path, &files, &slug)?;

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

fn load_auth_config() -> Result<AuthConfig> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))
        })?
        .join("spikes")
        .join("auth.json");

    if !config_path.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Not logged in. Run 'spikes login' or create {}\n\
                 with content: {{\"token\": \"your-api-token\"}}",
                config_path.display()
            ),
        )));
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    let token = parsed
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "auth.json missing 'token' field",
            ))
        })?
        .to_string();

    Ok(AuthConfig { token })
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
    auth: &AuthConfig,
    base_dir: &Path,
    files: &[PathBuf],
    slug: &str,
) -> Result<ShareResult> {
    use ureq::Agent;

    let agent = Agent::new();
    let url = "https://spikes.sh/shares";

    // Build multipart form
    let boundary = format!("----SpikesUpload{}", chrono::Utc::now().timestamp_millis());
    let mut body = Vec::new();

    // Add slug field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"slug\"\r\n\r\n");
    body.extend_from_slice(slug.as_bytes());
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

    let response = agent
        .post(url)
        .set("Authorization", &format!("Bearer {}", auth.token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Check your token or run 'spikes login'.",
        )));
    }

    if response.status() != 200 && response.status() != 201 {
        let status = response.status();
        let body_text = response.into_string().unwrap_or_default();
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Server returned status {}: {}", status, body_text),
        )));
    }

    let body_text = response
        .into_string()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

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
