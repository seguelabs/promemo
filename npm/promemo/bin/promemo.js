#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const targets = {
  "darwin-arm64": "darwin-arm64/promemo",
  "linux-x64": "linux-x64/promemo",
  "win32-x64": "win32-x64/promemo.exe",
};

const target = targets[`${process.platform}-${process.arch}`];

if (!target) {
  console.error(`Promemo does not publish an npm binary for ${process.platform}-${process.arch} yet.`);
  process.exit(1);
}

const binaryPath = path.join(__dirname, "..", "vendor", target);

if (!fs.existsSync(binaryPath)) {
  console.error(
    [
      "Promemo native binary is missing.",
      "",
      "This package may have been published without bundled native binaries.",
      "Please reinstall or report this at:",
      "  https://github.com/seguelabs/promemo/issues",
    ].join("\n"),
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
});

if (result.error) {
  console.error(`Failed to run Promemo: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
