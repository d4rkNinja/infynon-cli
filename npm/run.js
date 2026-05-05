#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawn } = require("child_process");

const PACKAGE_DIR = __dirname;
const VERSION = require("./package.json").version;
const RELEASES_URL = "https://github.com/d4rkNinja/infynon-cli/releases";
const LOCAL_BIN_PATH = path.join(
  PACKAGE_DIR,
  "bin",
  process.platform === "win32" ? "infynon.exe" : "infynon"
);
const POSTINSTALL_PATH = path.join(PACKAGE_DIR, "postinstall.js");

function getPlatformPackage() {
  if (process.platform === "win32" && process.arch === "x64") {
    return { packageName: "infynon-windows-x64", binaryName: "infynon.exe" };
  }
  if (process.platform === "linux" && process.arch === "x64") {
    return { packageName: "infynon-linux-x64", binaryName: "infynon" };
  }
  if (process.platform === "linux" && process.arch === "arm64") {
    return { packageName: "infynon-linux-arm64", binaryName: "infynon" };
  }
  if (process.platform === "darwin" && process.arch === "x64") {
    return { packageName: "infynon-darwin-x64", binaryName: "infynon" };
  }
  if (process.platform === "darwin" && process.arch === "arm64") {
    return { packageName: "infynon-darwin-arm64", binaryName: "infynon" };
  }
  return null;
}

function isFile(filePath) {
  try {
    return fs.statSync(filePath).isFile();
  } catch (_) {
    return false;
  }
}

function resolvePlatformBinary(notes) {
  const info = getPlatformPackage();
  if (!info) {
    notes.push("No optional native package is published for " + process.platform + " " + process.arch + ".");
    return null;
  }

  let packageJsonPath;
  try {
    packageJsonPath = require.resolve(info.packageName + "/package.json", { paths: [PACKAGE_DIR] });
  } catch (err) {
    if (err && err.code !== "MODULE_NOT_FOUND") {
      notes.push("Could not inspect " + info.packageName + ": " + err.message);
    }
    return null;
  }

  const packageDir = path.dirname(packageJsonPath);
  let packageVersion = null;
  try {
    packageVersion = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")).version;
  } catch (err) {
    notes.push("Could not read " + info.packageName + " metadata: " + err.message);
    return null;
  }

  if (packageVersion !== VERSION) {
    notes.push(
      info.packageName + " is version " + packageVersion + ", but the wrapper is version " + VERSION + "."
    );
    return null;
  }

  const binaryPath = path.join(packageDir, "bin", info.binaryName);
  if (!isFile(binaryPath)) {
    notes.push("Found " + info.packageName + ", but its binary is missing at " + binaryPath + ".");
    return null;
  }

  return { path: binaryPath, source: info.packageName };
}

function spawnInherited(command, args, options) {
  return new Promise(function (resolve, reject) {
    let child;
    try {
      child = spawn(
        command,
        args,
        Object.assign(
          {
            stdio: "inherit",
            windowsHide: false,
          },
          options || {}
        )
      );
    } catch (err) {
      reject(err);
      return;
    }

    child.on("error", reject);
    child.on("exit", function (code, signal) {
      resolve({ code: code, signal: signal });
    });
  });
}

async function runPostinstallFallback() {
  if (!isFile(POSTINSTALL_PATH)) {
    throw new Error("postinstall.js is missing at " + POSTINSTALL_PATH);
  }

  console.error("[infynon] Native binary is missing; running npm postinstall fallback once.");
  const result = await spawnInherited(process.execPath, [POSTINSTALL_PATH], {
    env: Object.assign({}, process.env, {
      INFYNON_NPM_POSTINSTALL_FALLBACK: "1",
    }),
  });

  if (result.signal) {
    throw new Error("postinstall.js was terminated by signal " + result.signal);
  }
  if (result.code !== 0) {
    throw new Error("postinstall.js failed with exit code " + result.code);
  }
}

async function resolveNativeBinary() {
  const notes = [];
  const platformBinary = resolvePlatformBinary(notes);
  if (platformBinary) {
    return platformBinary;
  }

  if (isFile(LOCAL_BIN_PATH)) {
    return { path: LOCAL_BIN_PATH, source: "npm/bin" };
  }

  await runPostinstallFallback();
  if (isFile(LOCAL_BIN_PATH)) {
    return { path: LOCAL_BIN_PATH, source: "npm/bin:postinstall" };
  }

  const message = [
    "Binary not found for " + process.platform + " " + process.arch + ".",
    "Expected local fallback at: " + LOCAL_BIN_PATH,
  ];
  if (notes.length > 0) {
    message.push("Resolution notes:");
    for (const note of notes) {
      message.push("  - " + note);
    }
  }
  message.push("Try reinstalling: npm install -g infynon");
  message.push("Or download a release manually: " + RELEASES_URL);
  const err = new Error(message.join("\n"));
  err.code = "BINARY_NOT_FOUND";
  throw err;
}

function assertWindowsCommandLineFits(binaryPath, args) {
  if (process.platform !== "win32") {
    return;
  }
  const estimatedLength = binaryPath.length + args.reduce(function (total, arg) {
    return total + String(arg).length + 3;
  }, 0);
  if (estimatedLength <= 30000) {
    return;
  }
  const err = new Error(
    "command line is too long for Windows process creation; reduce arguments or use file-based package-manager input"
  );
  err.code = "COMMAND_TOO_LONG";
  throw err;
}

async function runBinary(resolution, args) {
  assertWindowsCommandLineFits(resolution.path, args);
  return spawnInherited(resolution.path, args, {
    env: Object.assign({}, process.env, {
      INFYNON_NPM_WRAPPER: __filename,
      INFYNON_NPM_PACKAGE_DIR: PACKAGE_DIR,
      INFYNON_NPM_BINARY: resolution.path,
      INFYNON_NPM_BINARY_SOURCE: resolution.source,
    }),
  });
}

function exitFromChild(result) {
  if (result.signal) {
    try {
      process.kill(process.pid, result.signal);
    } catch (_) {
      const signalExitCodes = { SIGHUP: 129, SIGINT: 130, SIGTERM: 143 };
      process.exit(signalExitCodes[result.signal] || 1);
    }
    setTimeout(function () {
      process.exit(1);
    }, 1000);
    return;
  }

  process.exit(result.code === null ? 1 : result.code);
}

function printLaunchError(err, resolution) {
  const detected = [
    "platform: " + process.platform + " " + process.arch,
    "wrapper: " + __filename,
    "package dir: " + PACKAGE_DIR,
    "binary: " + (resolution ? resolution.path : "unresolved"),
    "binary source: " + (resolution ? resolution.source : "unresolved"),
    "node: " + process.version,
  ];

  if (err && err.code === "BINARY_NOT_FOUND") {
    console.error("[infynon] " + err.message);
    console.error("[infynon] Detected:\n  - " + detected.join("\n  - "));
    return;
  }

  if (resolution) {
    console.error("[infynon] Failed to launch native binary: " + resolution.path);
  } else {
    console.error("[infynon] Failed to resolve native binary.");
  }
  console.error("[infynon] " + (err && err.message ? err.message : String(err)));

  if (err && err.code === "COMMAND_TOO_LONG") {
    console.error("[infynon] On Windows, pass large package-manager input through files instead of argv.");
  } else if (err && err.code === "ENOENT") {
    console.error("[infynon] The resolved binary path no longer exists. Try reinstalling: npm install -g infynon");
  } else if (err && (err.code === "EACCES" || err.code === "EPERM")) {
    console.error("[infynon] The resolved binary is not executable. Try reinstalling: npm install -g infynon");
  } else if (process.platform === "win32") {
    console.error("[infynon] If Windows blocked or quarantined the executable, reinstall to restore a verified copy.");
    console.error("[infynon] Reinstall command: npm install -g infynon");
  } else {
    console.error("[infynon] Try reinstalling: npm install -g infynon");
  }
  console.error("[infynon] Detected:\n  - " + detected.join("\n  - "));
  console.error("[infynon] Diagnostics: infynon doctor npm");
  console.error("[infynon] Manual release downloads: " + RELEASES_URL);
}

async function main() {
  let resolution = null;
  try {
    resolution = await resolveNativeBinary();
    const result = await runBinary(resolution, process.argv.slice(2));
    exitFromChild(result);
  } catch (err) {
    printLaunchError(err, resolution);
    process.exit(1);
  }
}

main();
