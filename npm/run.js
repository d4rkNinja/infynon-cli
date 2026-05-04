#!/usr/bin/env node
"use strict";

const path = require("path");
const { spawnSync } = require("child_process");
const fs = require("fs");

const BIN_PATH = path.join(
  __dirname,
  "bin",
  process.platform === "win32" ? "infynon.exe" : "infynon"
);

if (!fs.existsSync(BIN_PATH)) {
  console.error(
    "[infynon] Binary not found at: " + BIN_PATH + "\n" +
    "         Try reinstalling: npm install -g infynon\n" +
    "         Or download a release manually: https://github.com/d4rkNinja/infynon-cli/releases"
  );
  process.exit(1);
}

function runBinary(args) {
  if (process.platform === "win32") {
    const estimatedLength = BIN_PATH.length + args.reduce(function (total, arg) {
      return total + String(arg).length + 3;
    }, 0);
    if (estimatedLength > 30000) {
      return {
        error: new Error("command line is too long for Windows process creation; reduce arguments or use file-based package-manager input"),
      };
    }
  }
  return spawnSync(BIN_PATH, args, {
    stdio: "inherit",
    windowsHide: false,
  });
}

const result = runBinary(process.argv.slice(2));

if (result.error) {
  console.error("[infynon] Failed to run binary:", result.error.message);
  if (process.platform === "win32" && result.error.code === "UNKNOWN") {
    console.error("[infynon] The installed Windows binary could not be executed.");
    console.error("[infynon] Reinstall to download and verify a fresh release asset: npm install -g infynon");
  }
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status !== null ? result.status : 1);
