#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const requiredBinaries = [
  ["darwin-arm64", "promemo"],
  ["linux-x64", "promemo"],
  ["win32-x64", "promemo.exe"],
];

const staleBinaries = ["promemo", "promemo.exe"].filter((binary) => {
  return fs.existsSync(path.join(__dirname, "..", "vendor", binary));
});

if (staleBinaries.length > 0) {
  console.error("Promemo npm package has stale root-level vendor binaries:");
  for (const binary of staleBinaries) {
    console.error(`  vendor/${binary}`);
  }
  console.error("");
  console.error("Run this before packing or publishing:");
  console.error("  npm run prepare-binaries");
  process.exit(1);
}

const missing = requiredBinaries.filter(([platform, binary]) => {
  return !fs.existsSync(path.join(__dirname, "..", "vendor", platform, binary));
});

if (missing.length > 0) {
  console.error("Promemo npm package is missing bundled native binaries:");
  for (const [platform, binary] of missing) {
    console.error(`  vendor/${platform}/${binary}`);
  }
  console.error("");
  console.error("Run this before packing or publishing:");
  console.error("  npm run prepare-binaries");
  process.exit(1);
}
