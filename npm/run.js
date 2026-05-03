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

const result = spawnSync(BIN_PATH, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
});

if (result.error) {
  console.error("[infynon] Failed to run binary:", result.error.message);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status !== null ? result.status : 1);

