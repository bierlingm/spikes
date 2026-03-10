/**
 * spikes-mcp — MCP server wrapper for Spikes
 *
 * Detects platform, downloads the correct binary from GitHub releases,
 * caches it locally, and spawns `spikes mcp serve` on stdio.
 */

"use strict";

const { execFileSync } = require("child_process");
const { createWriteStream, existsSync, mkdirSync, chmodSync, unlinkSync, renameSync } = require("fs");
const { join } = require("path");
const https = require("https");
const http = require("http");

/**
 * GitHub release URL template.
 * Matches the pattern used by action/action.yml.
 */
const GITHUB_RELEASE_URL =
  "https://github.com/moritzbierling/spikes/releases/latest/download";

/**
 * Supported platform targets — same as action/action.yml.
 */
const PLATFORM_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
};

/**
 * Resolve the Rust target triple for the current platform.
 * @returns {{ target: string, key: string }}
 */
function detectPlatform() {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;
  const target = PLATFORM_MAP[key];

  if (!target) {
    const supported = Object.entries(PLATFORM_MAP)
      .map(([k, v]) => `  ${k} -> ${v}`)
      .join("\n");
    throw new Error(
      `Unsupported platform: ${platform}/${arch}\n` +
        `Spikes MCP supports:\n${supported}\n` +
        `Open an issue if you need this platform: https://github.com/moritzbierling/spikes/issues`
    );
  }

  return { target, key };
}

/**
 * Get the cache directory for the binary.
 * Uses node_modules/.cache/spikes-mcp/ relative to this package.
 * @returns {string}
 */
function getCacheDir() {
  return join(__dirname, "node_modules", ".cache", "spikes-mcp");
}

/**
 * Get the path to the cached binary.
 * @returns {string}
 */
function getCachedBinaryPath() {
  return join(getCacheDir(), "spikes");
}

/**
 * Download a URL, following redirects (up to 10 hops).
 * Returns a promise that resolves when the file is written.
 *
 * @param {string} url
 * @param {string} destPath
 * @returns {Promise<void>}
 */
function download(url, destPath, redirectCount) {
  if (redirectCount === undefined) redirectCount = 0;
  if (redirectCount > 10) {
    return Promise.reject(new Error("Too many redirects"));
  }

  return new Promise(function (resolve, reject) {
    var mod = url.startsWith("https") ? https : http;
    mod.get(url, function (res) {
      // Follow redirects
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume(); // consume response to free socket
        download(res.headers.location, destPath, redirectCount + 1).then(resolve, reject);
        return;
      }

      if (res.statusCode === 404) {
        res.resume();
        reject(
          new Error(
            "Binary not found (HTTP 404). The release may not exist yet for this version.\n" +
              "Fallback: install the Spikes CLI manually:\n" +
              "  curl -fsSL https://spikes.sh/install.sh | sh\n" +
              "Then run: spikes mcp serve"
          )
        );
        return;
      }

      if (res.statusCode !== 200) {
        res.resume();
        reject(
          new Error(
            "Download failed with HTTP " +
              res.statusCode +
              ".\n" +
              "Fallback: install the Spikes CLI manually:\n" +
              "  curl -fsSL https://spikes.sh/install.sh | sh\n" +
              "Then run: spikes mcp serve"
          )
        );
        return;
      }

      var tmpPath = destPath + ".tmp";
      var file = createWriteStream(tmpPath);
      res.pipe(file);
      file.on("finish", function () {
        file.close(function () {
          // Atomic-ish rename
          try {
            renameSync(tmpPath, destPath);
          } catch (e) {
            reject(e);
            return;
          }
          resolve();
        });
      });
      file.on("error", function (err) {
        // Clean up partial file
        try { unlinkSync(tmpPath); } catch (_) {}
        reject(err);
      });
    }).on("error", function (err) {
      reject(
        new Error(
          "Network error downloading binary: " +
            err.message +
            "\n" +
            "Fallback: install the Spikes CLI manually:\n" +
            "  curl -fsSL https://spikes.sh/install.sh | sh\n" +
            "Then run: spikes mcp serve"
        )
      );
    });
  });
}

/**
 * Extract a .tar.gz archive to a destination directory.
 * Uses the system `tar` command (available on all supported platforms).
 *
 * @param {string} archivePath
 * @param {string} destDir
 */
function extractTarGz(archivePath, destDir) {
  execFileSync("tar", ["-xzf", archivePath, "-C", destDir], {
    stdio: "pipe",
  });
}

/**
 * Ensure the spikes binary is available (download if needed).
 * Returns the path to the executable.
 *
 * @returns {Promise<string>}
 */
async function ensureBinary() {
  var binPath = getCachedBinaryPath();

  // Reuse cached binary
  if (existsSync(binPath)) {
    return binPath;
  }

  var info = detectPlatform();
  var cacheDir = getCacheDir();
  mkdirSync(cacheDir, { recursive: true });

  var assetName = "spikes-" + info.target + ".tar.gz";
  var url = GITHUB_RELEASE_URL + "/" + assetName;
  var archivePath = join(cacheDir, assetName);

  process.stderr.write(
    "spikes-mcp: downloading binary for " + info.target + "...\n"
  );

  await download(url, archivePath);

  // Extract
  extractTarGz(archivePath, cacheDir);

  // Clean up archive
  try {
    unlinkSync(archivePath);
  } catch (_) {}

  // Make executable
  if (existsSync(binPath)) {
    chmodSync(binPath, 0o755);
  } else {
    throw new Error(
      "Binary not found after extraction. Expected: " +
        binPath +
        "\nThe release archive may have a different structure."
    );
  }

  process.stderr.write("spikes-mcp: binary ready at " + binPath + "\n");
  return binPath;
}

module.exports = {
  PLATFORM_MAP,
  GITHUB_RELEASE_URL,
  detectPlatform,
  getCacheDir,
  getCachedBinaryPath,
  download,
  extractTarGz,
  ensureBinary,
};
