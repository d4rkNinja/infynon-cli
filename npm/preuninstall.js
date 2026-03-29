#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const os = require("os");

// Mirror the same path logic as src/firewall/config.rs → config_dir()
const INFYNON_DIR = path.join(os.homedir(), ".infynon");

// Files/dirs managed by infynon that live outside the npm package
const MANAGED_PATHS = [
  { p: path.join(INFYNON_DIR, "infynon.toml"),    label: "firewall config" },
  { p: path.join(INFYNON_DIR, "eagle-eye.toml"),  label: "eagle eye config" },
  { p: path.join(INFYNON_DIR, "access.jsonl"),    label: "access log" },
  { p: path.join(INFYNON_DIR, "blocked.jsonl"),   label: "blocked log" },
  { p: path.join(INFYNON_DIR, "sbom.json"),       label: "SBOM" },
];

console.log("\n[infynon] Cleaning up...\n");

let removed = 0;

for (const { p, label } of MANAGED_PATHS) {
  if (fs.existsSync(p)) {
    try {
      fs.rmSync(p, { force: true });
      console.log("  removed  " + label + " (" + p + ")");
      removed++;
    } catch (err) {
      console.warn("  skipped  " + label + " — " + err.message);
    }
  }
}

// Remove ~/.infynon/ itself if it is now empty
if (fs.existsSync(INFYNON_DIR)) {
  try {
    const remaining = fs.readdirSync(INFYNON_DIR);
    if (remaining.length === 0) {
      fs.rmdirSync(INFYNON_DIR);
      console.log("  removed  ~/.infynon/ directory");
    } else {
      console.log(
        "  kept     ~/.infynon/ — " + remaining.length +
        " user file(s) remain: " + remaining.join(", ")
      );
    }
  } catch (_) {}
}

if (removed === 0) {
  console.log("  nothing to clean up.\n");
} else {
  console.log("\n[infynon] Clean uninstall complete.\n");
}
