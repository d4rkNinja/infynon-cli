#!/usr/bin/env node
"use strict";

const https = require("https");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const REPO = "d4rkNinja/infynon-cli";
const VERSION = require("./package.json").version;
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, process.platform === "win32" ? "infynon.exe" : "infynon");

function getTarget() {
  if (process.platform === "win32" && process.arch === "x64") return { target: "x86_64-pc-windows-msvc", ext: ".exe" };
  if (process.platform === "linux" && process.arch === "x64") return { target: "x86_64-unknown-linux-musl", ext: "" };
  if (process.platform === "linux" && process.arch === "arm64") return { target: "aarch64-unknown-linux-musl", ext: "" };
  if (process.platform === "darwin" && process.arch === "x64") return { target: "x86_64-apple-darwin", ext: "" };
  if (process.platform === "darwin" && process.arch === "arm64") return { target: "aarch64-apple-darwin", ext: "" };
  return null;
}

function downloadFile(url, dest, redirects) {
  redirects = redirects === undefined ? 0 : redirects;
  if (redirects > 5) {
    throw new Error("Too many redirects while downloading binary");
  }

  return new Promise(function (resolve, reject) {
    https
      .get(url, { headers: { "User-Agent": "infynon-npm-installer" } }, function (res) {
        if (res.statusCode === 301 || res.statusCode === 302) {
          return downloadFile(res.headers.location, dest, redirects + 1)
            .then(resolve)
            .catch(reject);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error("Download failed with status " + res.statusCode + " from " + url));
        }
        const file = fs.createWriteStream(dest);
        file.on("error", function (err) {
          file.close(function () {
            fs.unlink(dest, function () {});
            reject(err);
          });
        });
        file.on("finish", function () {
          file.close(resolve);
        });
        res.pipe(file);
      })
      .on("error", function (err) {
        fs.unlink(dest, function () {});
        reject(err);
      });
  });
}

function verifyBinary() {
  const result = spawnSync(BIN_PATH, ["--version"], {
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.error) {
    throw new Error("Downloaded binary is not executable: " + result.error.message);
  }
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "").trim();
    throw new Error(
      "Downloaded binary failed verification" +
      (detail ? ": " + detail : " with exit code " + result.status)
    );
  }
}

async function main() {
  const info = getTarget();

  if (!info) {
    console.warn(
      "[infynon] Unsupported platform: " + process.platform + " " + process.arch + ".\n" +
      "         Download a release manually: https://github.com/" + REPO + "/releases"
    );
    process.exit(0);
  }

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
    console.error("[infynon] Manual install: https://github.com/" + REPO + "/releases/tag/" + tag);
    process.exit(1);
  }

  if (process.platform !== "win32") {
    fs.chmodSync(BIN_PATH, 0o755);
  }

  try {
    verifyBinary();
  } catch (err) {
    fs.unlink(BIN_PATH, function () {});
    console.error("[infynon] Binary verification failed: " + err.message);
    console.error("[infynon] Reinstall after the release asset is corrected: npm install -g infynon");
    process.exit(1);
  }

  console.log("[infynon] Installed successfully. Run: infynon --help");
}

main().catch(function (err) {
  console.error("[infynon] Unexpected error during install:", err.message);
  process.exit(1);
});
