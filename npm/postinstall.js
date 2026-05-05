#!/usr/bin/env node
"use strict";

const crypto = require("crypto");
const https = require("https");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const REPO = "d4rkNinja/infynon-cli";
const VERSION = require("./package.json").version;
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, process.platform === "win32" ? "infynon.exe" : "infynon");
const TEMP_BIN_PATH = BIN_PATH + ".download-" + process.pid;
const TEMP_CHECKSUMS_PATH = BIN_PATH + ".checksums-" + process.pid + ".txt";
const TEMP_MANIFEST_PATH = BIN_PATH + ".manifest-" + process.pid + ".json";

function getTarget() {
  if (process.platform === "win32" && process.arch === "x64") {
    return {
      target: "x86_64-pc-windows-msvc",
      ext: ".exe",
      packageName: "@infynon/cli-win32-x64",
      binaryName: "infynon.exe",
    };
  }
  if (process.platform === "linux" && process.arch === "x64") {
    return {
      target: "x86_64-unknown-linux-musl",
      ext: "",
      packageName: "@infynon/cli-linux-x64",
      binaryName: "infynon",
    };
  }
  if (process.platform === "linux" && process.arch === "arm64") {
    return {
      target: "aarch64-unknown-linux-musl",
      ext: "",
      packageName: "@infynon/cli-linux-arm64",
      binaryName: "infynon",
    };
  }
  if (process.platform === "darwin" && process.arch === "x64") {
    return {
      target: "x86_64-apple-darwin",
      ext: "",
      packageName: "@infynon/cli-darwin-x64",
      binaryName: "infynon",
    };
  }
  if (process.platform === "darwin" && process.arch === "arm64") {
    return {
      target: "aarch64-apple-darwin",
      ext: "",
      packageName: "@infynon/cli-darwin-arm64",
      binaryName: "infynon",
    };
  }
  return null;
}

function downloadFile(url, dest, redirects) {
  redirects = redirects === undefined ? 0 : redirects;
  if (redirects > 5) {
    return Promise.reject(new Error("Too many redirects while downloading binary"));
  }

  return new Promise(function (resolve, reject) {
    https
      .get(url, { headers: { "User-Agent": "infynon-npm-installer" } }, function (res) {
        if (res.statusCode >= 300 && res.statusCode < 400) {
          res.resume();
          if (!res.headers.location) {
            return reject(new Error("Redirect response did not include a Location header"));
          }
          let nextUrl;
          try {
            nextUrl = new URL(res.headers.location, url).toString();
          } catch (err) {
            return reject(err);
          }
          return downloadFile(nextUrl, dest, redirects + 1)
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
        res.on("error", function (err) {
          file.close(function () {
            fs.unlink(dest, function () {});
            reject(err);
          });
        });
        res.pipe(file);
      })
      .on("error", function (err) {
        fs.unlink(dest, function () {});
        reject(err);
      });
  });
}

function removeQuietly(filePath) {
  try {
    fs.rmSync(filePath, { force: true });
  } catch (_) {}
}

function isFile(filePath) {
  try {
    return fs.statSync(filePath).isFile();
  } catch (_) {
    return false;
  }
}

function checksumForAsset(checksumsText, assetName) {
  const lines = checksumsText.split(/\r?\n/);
  for (const line of lines) {
    const match = line.match(/^([a-fA-F0-9]{64})\s+[* ]?(.+)$/);
    if (!match) continue;
    if (path.basename(match[2].trim()) === assetName) {
      return match[1].toLowerCase();
    }
  }
  throw new Error("checksums.txt does not include " + assetName);
}

function sha256File(filePath) {
  return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function verifyChecksum(checksumsPath, filePath, assetName) {
  const expected = checksumForAsset(fs.readFileSync(checksumsPath, "utf8"), assetName);
  const actual = sha256File(filePath);
  if (actual !== expected) {
    throw new Error("SHA-256 mismatch for " + assetName);
  }
}

function unavailable(message) {
  const err = new Error(message);
  err.fallbackAllowed = true;
  return err;
}

function integrityFailure(message) {
  const err = new Error(message);
  err.integrityFailure = true;
  return err;
}

function basenameMatches(value, assetName) {
  return typeof value === "string" && path.basename(value) === assetName;
}

function entryName(entry) {
  if (!entry || typeof entry !== "object") {
    return null;
  }
  return entry.name || entry.filename || entry.file || entry.path || entry.asset || entry.asset_name || null;
}

function objectEntryForAsset(objectValue, assetName) {
  if (!objectValue || typeof objectValue !== "object" || Array.isArray(objectValue)) {
    return null;
  }

  if (Object.prototype.hasOwnProperty.call(objectValue, assetName)) {
    const value = objectValue[assetName];
    if (value && typeof value === "object") {
      return Object.assign({ name: assetName }, value);
    }
    return { name: assetName, sha256: value };
  }

  for (const key of Object.keys(objectValue)) {
    const value = objectValue[key];
    if (path.basename(key) === assetName) {
      if (value && typeof value === "object") {
        return Object.assign({ name: key }, value);
      }
      return { name: key, sha256: value };
    }
    if (value && typeof value === "object" && basenameMatches(entryName(value), assetName)) {
      return value;
    }
  }

  return null;
}

function findManifestEntry(manifest, assetName) {
  if (basenameMatches(entryName(manifest), assetName)) {
    return manifest;
  }

  const collections = [
    manifest && manifest.assets,
    manifest && manifest.files,
    manifest && manifest.binaries,
    manifest && manifest.artifacts,
    manifest && manifest.release_assets,
  ];

  for (const collection of collections) {
    if (Array.isArray(collection)) {
      for (const entry of collection) {
        if (basenameMatches(entryName(entry), assetName)) {
          return entry;
        }
      }
    } else {
      const entry = objectEntryForAsset(collection, assetName);
      if (entry) {
        return entry;
      }
    }
  }

  return objectEntryForAsset(manifest, assetName);
}

function fieldValue(entry, names) {
  for (const name of names) {
    if (Object.prototype.hasOwnProperty.call(entry, name)) {
      return entry[name];
    }
  }
  return null;
}

function normalizeSha256(value) {
  if (typeof value !== "string") {
    return null;
  }
  const normalized = value.trim().toLowerCase().replace(/^sha256[:=\s]+/, "");
  if (/^[a-f0-9]{64}$/.test(normalized)) {
    return normalized;
  }
  return null;
}

function normalizeSize(value) {
  if (typeof value === "number" && Number.isSafeInteger(value) && value >= 0) {
    return value;
  }
  if (typeof value === "string" && /^\d+$/.test(value.trim())) {
    const parsed = Number(value.trim());
    if (Number.isSafeInteger(parsed)) {
      return parsed;
    }
  }
  return null;
}

function manifestVerificationForAsset(manifestPath, assetName) {
  let manifest;
  try {
    manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  } catch (err) {
    throw unavailable("release-manifest.json is not valid JSON: " + err.message);
  }

  const entry = findManifestEntry(manifest, assetName);
  if (!entry) {
    throw unavailable("release-manifest.json does not include " + assetName);
  }

  const sha256 = normalizeSha256(
    fieldValue(entry, ["sha256", "sha_256", "sha256sum", "checksum", "digest"])
  );
  if (!sha256) {
    throw unavailable("release-manifest.json does not include a SHA-256 for " + assetName);
  }

  const size = normalizeSize(fieldValue(entry, ["size", "size_bytes", "bytes", "length"]));
  if (size === null) {
    throw unavailable("release-manifest.json does not include a size for " + assetName);
  }

  return { sha256: sha256, size: size };
}

async function fetchManifestVerification(manifestUrl, assetName) {
  try {
    removeQuietly(TEMP_MANIFEST_PATH);
    await downloadFile(manifestUrl, TEMP_MANIFEST_PATH);
    return manifestVerificationForAsset(TEMP_MANIFEST_PATH, assetName);
  } catch (err) {
    if (err && err.integrityFailure) {
      throw err;
    }
    console.warn("[infynon] release-manifest.json verification unavailable: " + err.message);
    console.warn("[infynon] Falling back to checksums.txt.");
    return null;
  } finally {
    removeQuietly(TEMP_MANIFEST_PATH);
  }
}

function verifyManifestAsset(expected, filePath, assetName) {
  const actualSize = fs.statSync(filePath).size;
  if (actualSize !== expected.size) {
    throw integrityFailure("Size mismatch for " + assetName);
  }
  const actualSha256 = sha256File(filePath);
  if (actualSha256 !== expected.sha256) {
    throw integrityFailure("SHA-256 mismatch for " + assetName);
  }
}

function verifyBinary(binaryPath, label) {
  const result = spawnSync(binaryPath, ["--version"], {
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.error) {
    throw new Error(label + " is not executable: " + result.error.message);
  }
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "").trim();
    throw new Error(
      label + " failed verification" + (detail ? ": " + detail : " with exit code " + result.status)
    );
  }
  const versionFields = String((result.stdout || "") + " " + (result.stderr || ""))
    .trim()
    .split(/\s+/)
    .map(function (field) {
      return field.replace(/^v/, "");
    });
  if (versionFields.indexOf(VERSION) === -1) {
    throw new Error(label + " did not report version " + VERSION);
  }
}

function resolvePlatformPackageBinary(info) {
  let packageJsonPath;
  try {
    packageJsonPath = require.resolve(info.packageName + "/package.json", { paths: [__dirname] });
  } catch (err) {
    if (err && err.code !== "MODULE_NOT_FOUND") {
      console.warn("[infynon] Could not inspect " + info.packageName + ": " + err.message);
    }
    return null;
  }

  let packageVersion = null;
  try {
    packageVersion = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")).version;
  } catch (err) {
    console.warn("[infynon] Could not read " + info.packageName + " metadata: " + err.message);
    return null;
  }

  if (packageVersion !== VERSION) {
    console.warn(
      "[infynon] Ignoring " +
        info.packageName +
        " " +
        packageVersion +
        "; expected wrapper version " +
        VERSION +
        "."
    );
    return null;
  }

  const binaryPath = path.join(path.dirname(packageJsonPath), "bin", info.binaryName);
  if (!isFile(binaryPath)) {
    console.warn("[infynon] Ignoring " + info.packageName + "; missing binary at " + binaryPath + ".");
    return null;
  }

  return { path: binaryPath, packageName: info.packageName };
}

async function main() {
  const info = getTarget();

  if (!info) {
    console.warn(
      "[infynon] Unsupported platform: " +
        process.platform +
        " " +
        process.arch +
        ".\n" +
        "         Download a release manually: https://github.com/" +
        REPO +
        "/releases"
    );
    process.exit(0);
  }

  const platformBinary = resolvePlatformPackageBinary(info);
  if (platformBinary) {
    try {
      verifyBinary(platformBinary.path, "Installed platform package " + platformBinary.packageName);
    } catch (err) {
      console.error("[infynon] Platform package verification failed: " + err.message);
      console.error("[infynon] Reinstall after the package is corrected: npm install -g infynon");
      process.exit(1);
    }
    console.log("[infynon] Using installed native package " + platformBinary.packageName + ".");
    return;
  }

  const tag = "v" + VERSION;
  const assetName = "infynon-" + info.target + info.ext;
  const url = "https://github.com/" + REPO + "/releases/download/" + tag + "/" + assetName;
  const manifestUrl = "https://github.com/" + REPO + "/releases/download/" + tag + "/release-manifest.json";
  const checksumsUrl = "https://github.com/" + REPO + "/releases/download/" + tag + "/checksums.txt";

  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  console.log("[infynon] Downloading " + assetName + " from " + tag + " release...");

  try {
    removeQuietly(TEMP_BIN_PATH);
    removeQuietly(TEMP_CHECKSUMS_PATH);
    removeQuietly(TEMP_MANIFEST_PATH);

    const manifestVerification = await fetchManifestVerification(manifestUrl, assetName);
    await downloadFile(url, TEMP_BIN_PATH);
    if (manifestVerification) {
      verifyManifestAsset(manifestVerification, TEMP_BIN_PATH, assetName);
    } else {
      await downloadFile(checksumsUrl, TEMP_CHECKSUMS_PATH);
      verifyChecksum(TEMP_CHECKSUMS_PATH, TEMP_BIN_PATH, assetName);
    }

    removeQuietly(BIN_PATH);
    fs.renameSync(TEMP_BIN_PATH, BIN_PATH);
  } catch (err) {
    removeQuietly(TEMP_BIN_PATH);
    removeQuietly(TEMP_CHECKSUMS_PATH);
    removeQuietly(TEMP_MANIFEST_PATH);
    console.error("[infynon] Download failed: " + err.message);
    console.error("[infynon] Manual install: https://github.com/" + REPO + "/releases/tag/" + tag);
    process.exit(1);
  } finally {
    removeQuietly(TEMP_CHECKSUMS_PATH);
    removeQuietly(TEMP_MANIFEST_PATH);
  }

  if (process.platform !== "win32") {
    fs.chmodSync(BIN_PATH, 0o755);
  }

  try {
    verifyBinary(BIN_PATH, "Downloaded binary");
  } catch (err) {
    removeQuietly(BIN_PATH);
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
