#!/usr/bin/env node
"use strict";

const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");
const { execSync } = require("child_process");

const REPO = "d4rkNinja/infynon-cli";
const VERSION = require("./package.json").version;
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, process.platform === "win32" ? "infynon.exe" : "infynon");

// Map Node.js platform/arch to the GitHub release target triple
function getTarget() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "win32" && arch === "x64") return { target: "x86_64-pc-windows-msvc", ext: ".exe" };
  if (platform === "linux" && arch === "x64")  return { target: "x86_64-unknown-linux-musl", ext: "" };
  if (platform === "linux" && arch === "arm64") return { target: "aarch64-unknown-linux-musl", ext: "" };
  if (platform === "darwin" && arch === "x64") return { target: "x86_64-apple-darwin", ext: "" };
  if (platform === "darwin" && arch === "arm64") return { target: "aarch64-apple-darwin", ext: "" };

  return null;
}

function downloadFile(url, dest, redirects) {
  redirects = redirects === undefined ? 0 : redirects;
  if (redirects > 5) {
    throw new Error("Too many redirects while downloading binary");
  }

  return new Promise(function (resolve, reject) {
    const file = fs.createWriteStream(dest);
    https
      .get(url, { headers: { "User-Agent": "infynon-npm-installer" } }, function (res) {
        if (res.statusCode === 301 || res.statusCode === 302) {
          file.close();
          fs.unlinkSync(dest);
          return downloadFile(res.headers.location, dest, redirects + 1)
            .then(resolve)
            .catch(reject);
        }
        if (res.statusCode !== 200) {
          file.close();
          fs.unlinkSync(dest);
          return reject(new Error("Download failed with status " + res.statusCode + " from " + url));
        }
        res.pipe(file);
        file.on("finish", function () {
          file.close(resolve);
        });
      })
      .on("error", function (err) {
        fs.unlink(dest, function () {});
        reject(err);
      });
  });
}

async function main() {
  const info = getTarget();

  if (!info) {
    console.warn(
      "[infynon] Unsupported platform: " + process.platform + " " + process.arch + ".\n" +
      "         Build from source: cargo install --git https://github.com/" + REPO
    );
    process.exit(0); // non-fatal — let the install succeed
  }

  // Strip npm prerelease suffix for the GitHub tag (beta.6 → beta.6 stays; just prefix with v)
  const tag = "v" + VERSION;
  const assetName = "infynon-" + info.target + info.ext;
  const url = "https://github.com/" + REPO + "/releases/download/" + tag + "/" + assetName;

  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  console.log("[infynon] Downloading " + assetName + " from " + tag + " release...");

  try {
    await downloadFile(url, BIN_PATH);
  } catch (err) {
    console.error("[infynon] Download failed: " + err.message);
    console.error("[infynon] You can install manually: https://github.com/" + REPO + "/releases/tag/" + tag);
    process.exit(0); // non-fatal
  }

  // Make executable on Unix
  if (process.platform !== "win32") {
    fs.chmodSync(BIN_PATH, 0o755);
  }

  console.log("[infynon] Installed successfully. Run: infynon --help");
}

main().catch(function (err) {
  console.error("[infynon] Unexpected error during install:", err.message);
  process.exit(0); // non-fatal — don't break npm install
});
