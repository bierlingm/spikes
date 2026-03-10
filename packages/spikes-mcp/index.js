#!/usr/bin/env node

/**
 * spikes-mcp — npx wrapper for the Spikes MCP server.
 *
 * Downloads the platform-native binary, caches it, and spawns
 * `spikes mcp serve` on stdio so MCP clients can connect.
 */

"use strict";

var lib = require("./lib");
var spawn = require("child_process").spawn;

function main() {
  lib
    .ensureBinary()
    .then(function (binPath) {
      // Spawn the MCP server on stdio.
      // Forward all additional CLI args after `spikes-mcp`.
      var args = ["mcp", "serve"].concat(process.argv.slice(2));
      var child = spawn(binPath, args, {
        stdio: "inherit",
      });

      child.on("error", function (err) {
        process.stderr.write("spikes-mcp: failed to start: " + err.message + "\n");
        process.exit(1);
      });

      child.on("exit", function (code, signal) {
        if (signal) {
          process.kill(process.pid, signal);
        } else {
          process.exit(code || 0);
        }
      });

      // Forward signals to child
      function forwardSignal(sig) {
        process.on(sig, function () {
          child.kill(sig);
        });
      }
      forwardSignal("SIGINT");
      forwardSignal("SIGTERM");
    })
    .catch(function (err) {
      process.stderr.write("spikes-mcp: " + err.message + "\n");
      process.exit(1);
    });
}

main();
