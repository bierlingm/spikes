/**
 * Tests for spikes-mcp platform detection and utility logic.
 *
 * Run with: node --test test.js
 */

"use strict";

var test = require("node:test");
var assert = require("node:assert/strict");
var path = require("path");
var lib = require("./lib");

// --- Platform detection tests ---

test("detectPlatform returns correct target for macOS ARM64", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "darwin", configurable: true });
  Object.defineProperty(process, "arch", { value: "arm64", configurable: true });

  try {
    var result = lib.detectPlatform();
    assert.equal(result.target, "aarch64-apple-darwin");
    assert.equal(result.key, "darwin-arm64");
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

test("detectPlatform returns correct target for macOS x64", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "darwin", configurable: true });
  Object.defineProperty(process, "arch", { value: "x64", configurable: true });

  try {
    var result = lib.detectPlatform();
    assert.equal(result.target, "x86_64-apple-darwin");
    assert.equal(result.key, "darwin-x64");
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

test("detectPlatform returns correct target for Linux x64", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "linux", configurable: true });
  Object.defineProperty(process, "arch", { value: "x64", configurable: true });

  try {
    var result = lib.detectPlatform();
    assert.equal(result.target, "x86_64-unknown-linux-gnu");
    assert.equal(result.key, "linux-x64");
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

test("detectPlatform returns correct target for Linux ARM64", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "linux", configurable: true });
  Object.defineProperty(process, "arch", { value: "arm64", configurable: true });

  try {
    var result = lib.detectPlatform();
    assert.equal(result.target, "aarch64-unknown-linux-gnu");
    assert.equal(result.key, "linux-arm64");
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

test("detectPlatform throws for unsupported platform", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "win32", configurable: true });
  Object.defineProperty(process, "arch", { value: "x64", configurable: true });

  try {
    assert.throws(
      function () { lib.detectPlatform(); },
      function (err) {
        assert.ok(err.message.includes("Unsupported platform: win32/x64"));
        assert.ok(err.message.includes("Spikes MCP supports:"));
        assert.ok(err.message.includes("darwin-arm64"));
        assert.ok(err.message.includes("linux-x64"));
        return true;
      }
    );
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

test("detectPlatform throws for unsupported architecture", function () {
  var origPlatform = Object.getOwnPropertyDescriptor(process, "platform");
  var origArch = Object.getOwnPropertyDescriptor(process, "arch");

  Object.defineProperty(process, "platform", { value: "linux", configurable: true });
  Object.defineProperty(process, "arch", { value: "s390x", configurable: true });

  try {
    assert.throws(
      function () { lib.detectPlatform(); },
      function (err) {
        assert.ok(err.message.includes("Unsupported platform: linux/s390x"));
        return true;
      }
    );
  } finally {
    if (origPlatform) Object.defineProperty(process, "platform", origPlatform);
    if (origArch) Object.defineProperty(process, "arch", origArch);
  }
});

// --- PLATFORM_MAP completeness ---

test("PLATFORM_MAP has exactly 4 entries", function () {
  assert.equal(Object.keys(lib.PLATFORM_MAP).length, 4);
});

test("PLATFORM_MAP matches action/action.yml targets", function () {
  var expectedTargets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
  ];
  var actualTargets = Object.values(lib.PLATFORM_MAP).sort();
  assert.deepEqual(actualTargets, expectedTargets.sort());
});

// --- Cache path tests ---

test("getCacheDir returns path under node_modules/.cache/spikes-mcp", function () {
  var dir = lib.getCacheDir();
  assert.ok(dir.includes("node_modules"));
  assert.ok(dir.includes(".cache"));
  assert.ok(dir.includes("spikes-mcp"));
});

test("getCachedBinaryPath ends with /spikes", function () {
  var binPath = lib.getCachedBinaryPath();
  assert.ok(binPath.endsWith(path.sep + "spikes"), "Expected path ending with /spikes, got: " + binPath);
});

// --- Download URL construction ---

test("GITHUB_RELEASE_URL points to correct repo", function () {
  assert.ok(lib.GITHUB_RELEASE_URL.includes("bierlingm/spikes"));
  assert.ok(lib.GITHUB_RELEASE_URL.includes("releases/latest/download"));
});
