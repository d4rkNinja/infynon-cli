#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const BIN_DIR = path.join(__dirname, "bin");

if (fs.existsSync(BIN_DIR)) {
  fs.rmSync(BIN_DIR, { recursive: true, force: true });
}
