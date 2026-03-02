//! Security tests for the serve command
//!
//! Tests path traversal protection and CORS defaults

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::sync::atomic::{AtomicU16, Ordering};

use reqwest::blocking::Client;
use reqwest::StatusCode;
use tempfile::TempDir;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(3900u16);

/// Get a unique port for each test to avoid conflicts
fn get_unique_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Helper to create a test directory with files
struct ServeTest {
    dir: TempDir,
    port: u16,
    server: Option<Child>,
    client: Client,
}

impl ServeTest {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let port = get_unique_port();
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create client");
        
        Self {
            dir,
            port,
            server: None,
            client,
        }
    }

    fn add_file(&self, name: &str, content: &str) -> std::path::PathBuf {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, content).expect("Failed to write file");
        path
    }

    fn add_secret_file(&self, content: &str) -> std::path::PathBuf {
        // Add a file outside the serve directory (in temp dir parent)
        let secret_path = self.dir.path().parent().unwrap().join("secret.txt");
        std::fs::write(&secret_path, content).expect("Failed to write secret file");
        secret_path
    }

    fn start_server(&mut self, cors_origin: Option<&str>) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_spikes"));
        cmd.arg("serve")
            .arg("--port").arg(self.port.to_string())
            .arg("--dir").arg(self.dir.path().to_string_lossy().to_string());
        
        if let Some(origin) = cors_origin {
            cmd.arg("--cors-allow-origin").arg(origin);
        }
        
        // Capture stderr for debugging, but stdout can be null
        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped());
        
        self.server = Some(cmd.spawn().expect("Failed to start server"));
        
        // Wait for server to be ready
        let url = format!("http://localhost:{}", self.port);
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            if self.client.get(&format!("{}/index.html", url)).send().is_ok() {
                return;
            }
        }
        panic!("Server failed to start within timeout");
    }

    fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    fn get(&self, path: &str) -> reqwest::blocking::Response {
        self.client
            .get(&format!("{}{}", self.url(), path))
            .send()
            .expect("Request failed")
    }

    fn get_with_origin(&self, path: &str, origin: &str) -> reqwest::blocking::Response {
        self.client
            .get(&format!("{}{}", self.url(), path))
            .header("Origin", origin)
            .send()
            .expect("Request failed")
    }

    /// Send a raw HTTP request without path normalization
    fn raw_get(&self, path: &str) -> (StatusCode, String, std::collections::HashMap<String, String>) {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))
            .expect("Failed to connect");
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
        
        let request = format!("GET {} HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n", path, self.port);
        stream.write_all(request.as_bytes()).expect("Failed to write request");
        
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("Failed to read response");
        
        // Parse response
        let mut headers = std::collections::HashMap::new();
        let mut body = String::new();
        let mut status = StatusCode::OK;
        
        let mut lines = response.lines();
        if let Some(status_line) = lines.next() {
            if let Some(code) = status_line.split_whitespace().nth(1) {
                if let Ok(code) = code.parse::<u16>() {
                    status = StatusCode::from_u16(code).unwrap_or(StatusCode::OK);
                }
            }
        }
        
        let mut in_body = false;
        for line in lines {
            if in_body {
                body.push_str(line);
            } else if line.is_empty() {
                in_body = true;
            } else if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }
        
        (status, body, headers)
    }
}

impl Drop for ServeTest {
    fn drop(&mut self) {
        if let Some(mut child) = self.server.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn test_serve_legitimate_file_succeeds() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(None);

    let resp = test.get("/index.html");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().unwrap();
    assert!(body.contains("Hello"));
}

#[test]
fn test_serve_index_html_as_default() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Index</body></html>");
    test.start_server(None);

    let resp = test.get("/");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().unwrap();
    assert!(body.contains("Index"));
}

#[test]
fn test_path_traversal_blocked_dotdot_slash() {
    let mut test = ServeTest::new();
    test.add_file("public/index.html", "<html><body>Public</body></html>");
    
    // Create a secret file outside the public directory
    let secret_path = test.dir.path().join("secret.txt");
    std::fs::write(&secret_path, "SECRET CONTENT").unwrap();
    
    // Also create a config file in .spikes that shouldn't be accessible via traversal
    let spikes_dir = test.dir.path().join("public/.spikes");
    std::fs::create_dir_all(&spikes_dir).unwrap();
    std::fs::write(spikes_dir.join("config.toml"), "secret=config").unwrap();
    
    // Start server serving the public directory
    let public_dir = test.dir.path().join("public");
    let port = test.port;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_spikes"));
    cmd.arg("serve")
        .arg("--port").arg(port.to_string())
        .arg("--dir").arg(public_dir.to_string_lossy().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    test.server = Some(cmd.spawn().expect("Failed to start server"));
    
    // Wait for server
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if test.client.get(&format!("http://localhost:{}/index.html", port)).send().is_ok() {
            break;
        }
    }
    
    // Use raw TCP to send path without normalization
    let (status, body, _headers) = test.raw_get("/../secret.txt");
    
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "Path traversal should be blocked, got status: {}",
        status
    );
    
    // Verify the secret content was NOT returned
    assert!(!body.contains("SECRET"), "Secret content should not be exposed");
}

#[test]
fn test_path_traversal_blocked_multiple_dotdot() {
    let mut test = ServeTest::new();
    test.add_file("public/index.html", "<html><body>Public</body></html>");
    
    let public_dir = test.dir.path().join("public");
    let port = test.port;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_spikes"));
    cmd.arg("serve")
        .arg("--port").arg(port.to_string())
        .arg("--dir").arg(public_dir.to_string_lossy().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    test.server = Some(cmd.spawn().expect("Failed to start server"));
    
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if test.client.get(&format!("http://localhost:{}/index.html", port)).send().is_ok() {
            break;
        }
    }
    
    // Use raw TCP to send deep path traversal
    let (status, body, _headers) = test.raw_get("/../../Cargo.toml");
    
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "Deep path traversal should be blocked, got status: {}",
        status
    );
}

#[test]
fn test_path_traversal_blocked_backslash() {
    let mut test = ServeTest::new();
    test.add_file("public/index.html", "<html><body>Public</body></html>");
    
    let public_dir = test.dir.path().join("public");
    let port = test.port;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_spikes"));
    cmd.arg("serve")
        .arg("--port").arg(port.to_string())
        .arg("--dir").arg(public_dir.to_string_lossy().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    test.server = Some(cmd.spawn().expect("Failed to start server"));
    
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if test.client.get(&format!("http://localhost:{}/index.html", port)).send().is_ok() {
            break;
        }
    }
    
    // Use raw TCP to send backslash path traversal
    let (status, body, _headers) = test.raw_get("/..\\..\\Cargo.toml");
    
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "Backslash path traversal should be blocked, got status: {}",
        status
    );
}

#[test]
fn test_path_traversal_blocked_spikes_config() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Public</body></html>");
    
    // Create a .spikes directory with config
    let spikes_dir = test.dir.path().join(".spikes");
    std::fs::create_dir_all(&spikes_dir).unwrap();
    std::fs::write(spikes_dir.join("config.toml"), "[project]\nkey = \"secret-project\"").unwrap();
    
    test.start_server(None);
    
    // Use raw TCP to send path without normalization
    // HTTP clients often normalize paths before sending
    let (status, body, _headers) = test.raw_get("/../.spikes/config.toml");
    
    eprintln!("Raw request to /../.spikes/config.toml");
    eprintln!("Status: {}", status);
    eprintln!("Body: {}", body);
    
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "Access to .spikes via traversal should be blocked, got status: {}",
        status
    );
    
    // Verify the secret content was NOT returned
    assert!(!body.contains("secret-project"), "Secret content should not be exposed");
}

#[test]
fn test_cors_no_headers_without_flag() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(None); // No CORS flag

    let resp = test.get_with_origin("/index.html", "http://evil.com");
    
    // Should NOT have Access-Control-Allow-Origin header
    assert!(
        resp.headers().get("Access-Control-Allow-Origin").is_none(),
        "CORS header should not be present without flag"
    );
}

#[test]
fn test_cors_no_wildcard_without_flag() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(None);

    let resp = test.get("/index.html");
    
    // Should NOT have wildcard CORS
    let cors_header = resp.headers().get("Access-Control-Allow-Origin");
    assert!(
        cors_header.is_none() || cors_header.unwrap() != "*",
        "Wildcard CORS should not be present without flag"
    );
}

#[test]
fn test_cors_allowed_with_flag() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(Some("https://spikes.sh"));

    let resp = test.get_with_origin("/index.html", "https://spikes.sh");
    
    // Should have the matching CORS header
    let cors_header = resp.headers().get("Access-Control-Allow-Origin");
    assert!(
        cors_header.is_some(),
        "CORS header should be present with flag"
    );
    assert_eq!(
        cors_header.unwrap().to_str().unwrap(),
        "https://spikes.sh",
        "CORS header should match allowed origin"
    );
}

#[test]
fn test_cors_rejects_other_origin() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(Some("https://spikes.sh"));

    // Request with different origin
    let resp = test.get_with_origin("/index.html", "http://evil.com");
    
    // Should NOT echo the evil.com origin
    let cors_header = resp.headers().get("Access-Control-Allow-Origin");
    if cors_header.is_some() {
        assert_ne!(
            cors_header.unwrap().to_str().unwrap(),
            "http://evil.com",
            "CORS header should not echo untrusted origin"
        );
    }
    // The response may still succeed (file is returned), but CORS headers shouldn't expose
}

#[test]
fn test_nested_file_served_correctly() {
    let mut test = ServeTest::new();
    test.add_file("assets/css/style.css", "body { color: red; }");
    test.start_server(None);

    let resp = test.get("/assets/css/style.css");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().unwrap();
    assert!(body.contains("color: red"));
}

#[test]
fn test_nonexistent_file_returns_404() {
    let mut test = ServeTest::new();
    test.add_file("index.html", "<html><body>Hello</body></html>");
    test.start_server(None);

    let resp = test.get("/nonexistent.html");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
