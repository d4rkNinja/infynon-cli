#!/usr/bin/env node
"use strict";

process.argv.splice(2, 0, "pkg");
require("./run.js");
