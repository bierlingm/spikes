//! Integration tests for MCP server

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::process::Child;
use std::thread;
use std::time::Duration;

#[test]
fn test_mcp_initialize() {
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

    Command::cargo_bin("spikes")
        .unwrap()
        .arg("mcp")
        .arg("serve")
        .write_stdin(input)
        .assert()
        .stdout(predicate::str::contains("spikes-mcp"))
        .stdout(predicate::str::contains("2024-11-05"));
}

#[test]
fn test_mcp_tools_list_request() {
    // Test that tools/list is recognized (it will fail without proper init, but shows method exists)
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;

    Command::cargo_bin("spikes")
        .unwrap()
        .arg("mcp")
        .arg("serve")
        .write_stdin(input)
        .assert()
        // The server will error because we didn't initialize first, but it should recognize tools/list
        .stderr(predicate::str::contains("spikes-mcp").or(predicate::str::contains("MCP server")));
}

#[test]
fn test_mcp_sequential_requests() {
    // Initialize and then call tools/list
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#, "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#, "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#, "\n"
    );

    Command::cargo_bin("spikes")
        .unwrap()
        .arg("mcp")
        .arg("serve")
        .write_stdin(input)
        .assert()
        .stdout(predicate::str::contains("spikes-mcp"));
}

// HTTP Transport Tests

/// Helper to start HTTP MCP server in background with logging
fn start_http_server(port: u16) -> Child {
    use std::process::Stdio;

    let binary = assert_cmd::cargo::cargo_bin("spikes");

    std::process::Command::new(binary)
        .args(["mcp", "serve", "--transport", "http", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP HTTP server")
}

/// Helper to stop HTTP MCP server
fn stop_http_server(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Helper to extract JSON-RPC response from SSE format
/// The MCP HTTP transport returns responses in SSE format like:
/// data: {"jsonrpc":"2.0","id":1,"result":{...}}
fn extract_json_from_sse(sse_text: &str) -> Option<serde_json::Value> {
    for line in sse_text.lines() {
        if line.starts_with("data: ") {
            let json_str = line.strip_prefix("data: ").unwrap();
            if !json_str.is_empty() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    return Some(json);
                }
            }
        }
    }
    None
}

#[test]
#[serial(mcp_http)]
fn test_mcp_http_transport_starts() {
    // Use a unique port to avoid conflicts
    let port = 3849;
    let server = start_http_server(port);

    // Wait for server to start
    thread::sleep(Duration::from_millis(1000));

    // Try to connect - any response means server is running
    let client = reqwest::blocking::Client::new();
    let result = client
        .post(format!("http://127.0.0.1:{}/", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }))
        .send();

    stop_http_server(server);

    // Server should respond (even if with an error, it means it's running)
    assert!(result.is_ok(), "HTTP MCP server should respond to requests");
}

#[test]
#[serial(mcp_http)]
fn test_mcp_http_initialize() {
    let port = 3850;
    let server = start_http_server(port);

    // Wait for server to start
    thread::sleep(Duration::from_millis(1000));

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .http1_only()
        .build()
        .expect("Failed to create HTTP client");

    let response = client
        .post(format!("http://127.0.0.1:{}/", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }))
        .send();

    assert!(response.is_ok(), "Should get response from HTTP MCP server");
    let response = response.unwrap();

    // Verify the Mcp-Session-Id header is present (required by MCP Streamable HTTP spec)
    let session_id = response
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok());
    assert!(
        session_id.is_some(),
        "Initialize response must include Mcp-Session-Id header"
    );

    // Read response body using bytes() to avoid blocking on SSE streams
    let body_bytes = response.bytes().expect("Failed to read response");
    let body = String::from_utf8_lossy(&body_bytes);
    let json = extract_json_from_sse(&body)
        .expect("Response should contain valid JSON in SSE format");

    // Response should contain server info
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);
    assert!(json["result"].is_object());
    assert_eq!(json["result"]["serverInfo"]["name"], "spikes-mcp");

    stop_http_server(server);
}

#[test]
#[serial(mcp_http)]
fn test_mcp_http_tools_list() {
    let port = 3851;
    let server = start_http_server(port);

    // Wait for server to start
    thread::sleep(Duration::from_millis(1000));

    // Create client with explicit timeout configuration for SSE streams
    // Use HTTP/1.1 only to avoid connection reuse issues
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .http1_only()
        .build()
        .expect("Failed to create HTTP client");

    // Initialize first
    let init_response = client
        .post(format!("http://127.0.0.1:{}/", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }))
        .send();

    assert!(init_response.is_ok(), "Initialize should succeed");
    let init_response = init_response.unwrap();

    // CRITICAL: Capture the Mcp-Session-Id header from the initialize response
    // The MCP Streamable HTTP spec requires session persistence via this header
    let session_id = init_response
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .expect("Initialize response must include Mcp-Session-Id header");

    // Read the SSE response body using bytes() to avoid blocking on SSE streams
    // SSE streams from rmcp return priming events followed by the actual response
    let _init_bytes = init_response.bytes().expect("Failed to read init response");
    thread::sleep(Duration::from_millis(200));

    // Send initialized notification (required by MCP protocol)
    // Include the session ID header to maintain session state
    let notif_response = client
        .post(format!("http://127.0.0.1:{}/", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("Mcp-Session-Id", &session_id)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send();

    // Notification returns 202 Accepted - read and discard response
    if let Ok(resp) = notif_response {
        let _ = resp.bytes();
    }
    thread::sleep(Duration::from_millis(200));

    // Then call tools/list - include session ID header
    let tools_response = client
        .post(format!("http://127.0.0.1:{}/", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("Mcp-Session-Id", &session_id)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .send();

    stop_http_server(server);

    assert!(tools_response.is_ok(), "tools/list should succeed");
    let response = tools_response.unwrap();
    let body_bytes = response.bytes().expect("Failed to read tools response");
    let body = String::from_utf8_lossy(&body_bytes);

    // Parse response - must contain valid JSON
    let json = extract_json_from_sse(&body)
        .or_else(|| serde_json::from_str(&body).ok())
        .expect("Response should contain valid JSON");

    // Should list 9 tools - UNCONDITIONAL assertion (test must fail if tools not present)
    assert!(
        json["result"]["tools"].is_array(),
        "Response must contain tools array"
    );
    let tools = json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9, "Should have 9 MCP tools");

    // Verify tool names
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"get_spikes"), "Must have get_spikes tool");
    assert!(tool_names.contains(&"submit_spike"), "Must have submit_spike tool");
    assert!(tool_names.contains(&"get_usage"), "Must have get_usage tool");
}

#[test]
fn test_mcp_help_shows_transport_options() {
    Command::cargo_bin("spikes")
        .unwrap()
        .args(["mcp", "serve", "--help"])
        .assert()
        .stdout(predicate::str::contains("--transport"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--bind"));
}
