//! Integration tests for MCP server

use assert_cmd::Command;
use predicates::prelude::*;

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
