#!/usr/bin/env node

const crypto = require("crypto");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const packageJson = require("../package.json");

const repo = "seguelabs/promemo";
const version = packageJson.version;
const tag = `v${version}`;
const vendorDir = path.join(__dirname, "..", "vendor");
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "promemo-npm-"));
const githubToken = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
let releaseMetadataPromise;

const targets = {
  "darwin-arm64": {
    asset: "promemo-aarch64-apple-darwin.tar.gz",
    binary: "promemo",
    archive: "tar.gz",
  },
  "linux-x64": {
    asset: "promemo-x86_64-unknown-linux-gnu.tar.gz",
    binary: "promemo",
    archive: "tar.gz",
  },
  "win32-x64": {
    asset: "promemo-x86_64-pc-windows-msvc.zip",
    binary: "promemo.exe",
    archive: "zip",
  },
};

function requestHeaders(url, extraHeaders = {}) {
  const headers = {
    "User-Agent": `promemo-npm/${version}`,
    ...extraHeaders,
  };

  if (
    githubToken &&
    (url.startsWith("https://github.com/") || url.startsWith("https://api.github.com/"))
  ) {
    headers.Authorization = `Bearer ${githubToken}`;
  }

  return headers;
}

function download(url, destination, extraHeaders = {}) {
  return new Promise((resolve, reject) => {
    const request = https.get(
      url,
      {
        headers: requestHeaders(url, extraHeaders),
      },
      (response) => {
        if (
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          download(response.headers.location, destination, extraHeaders).then(resolve, reject);
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`download failed with HTTP ${response.statusCode}: ${url}`));
          return;
        }

        const file = fs.createWriteStream(destination);
        response.pipe(file);
        file.on("finish", () => file.close(resolve));
        file.on("error", reject);
      },
    );

    request.on("error", reject);
  });
}

function fetchText(url, extraHeaders = {}) {
  return new Promise((resolve, reject) => {
    const request = https.get(
      url,
      {
        headers: requestHeaders(url, extraHeaders),
      },
      (response) => {
        if (
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          fetchText(response.headers.location, extraHeaders).then(resolve, reject);
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`request failed with HTTP ${response.statusCode}: ${url}`));
          return;
        }

        let body = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => resolve(body));
      },
    );

    request.on("error", reject);
  });
}

async function releaseMetadata() {
  if (!releaseMetadataPromise) {
    const url = `https://api.github.com/repos/${repo}/releases/tags/${tag}`;
    releaseMetadataPromise = fetchText(url, {
      Accept: "application/vnd.github+json",
    }).then((body) => JSON.parse(body));
  }

  return releaseMetadataPromise;
}

async function assetApiUrl(assetName) {
  const release = await releaseMetadata();
  const asset = release.assets.find((candidate) => candidate.name === assetName);
  if (!asset) {
    throw new Error(`release ${tag} does not include ${assetName}`);
  }

  return asset.url;
}

function sha256(filePath) {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

function expectedSha(sumsText, assetName) {
  const line = sumsText
    .split(/\r?\n/)
    .find((entry) => {
      return entry.trim().endsWith(`  ${assetName}`) || entry.trim().endsWith(` *${assetName}`);
    });

  if (!line) {
    throw new Error(`SHA256SUMS does not include ${assetName}`);
  }

  return line.trim().split(/\s+/)[0];
}

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: tmpDir,
    stdio: "inherit",
    windowsHide: false,
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with exit code ${result.status}`);
  }
}

function extract(target, archivePath, extractDir) {
  fs.mkdirSync(extractDir, { recursive: true });

  if (target.archive === "tar.gz") {
    run("tar", ["-xzf", archivePath, "-C", extractDir]);
    return;
  }

  if (target.archive === "zip") {
    const unzip = spawnSync("unzip", ["-q", archivePath, "-d", extractDir], {
      cwd: tmpDir,
      stdio: "inherit",
      windowsHide: false,
    });
    if (!unzip.error && unzip.status === 0) {
      return;
    }

    const script = [
      "Expand-Archive",
      "-LiteralPath",
      JSON.stringify(archivePath),
      "-DestinationPath",
      JSON.stringify(extractDir),
      "-Force",
    ].join(" ");

    run("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script]);
    return;
  }

  throw new Error(`unsupported archive type: ${target.archive}`);
}

function findExtractedBinary(directory, binaryName) {
  const entries = fs.readdirSync(directory, { withFileTypes: true });

  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isFile() && entry.name === binaryName) {
      return entryPath;
    }
    if (entry.isDirectory()) {
      const found = findExtractedBinary(entryPath, binaryName);
      if (found) {
        return found;
      }
    }
  }

  return undefined;
}

async function prepareTarget(platform, target, sumsText) {
  const archiveUrl = githubToken
    ? await assetApiUrl(target.asset)
    : `https://github.com/${repo}/releases/download/${tag}/${target.asset}`;
  const downloadHeaders = githubToken ? { Accept: "application/octet-stream" } : {};
  const archivePath = path.join(tmpDir, target.asset);
  const extractDir = path.join(tmpDir, platform);

  console.log(`Downloading Promemo ${tag} for ${platform}...`);
  await download(archiveUrl, archivePath, downloadHeaders);

  const expected = expectedSha(sumsText, target.asset);
  const actual = sha256(archivePath);
  if (actual !== expected) {
    throw new Error(`checksum mismatch for ${target.asset}: expected ${expected}, got ${actual}`);
  }

  extract(target, archivePath, extractDir);

  const extractedBinary = findExtractedBinary(extractDir, target.binary);
  if (!extractedBinary || !fs.existsSync(extractedBinary)) {
    throw new Error(`archive did not contain ${target.binary}`);
  }

  const platformDir = path.join(vendorDir, platform);
  fs.mkdirSync(platformDir, { recursive: true });
  const destination = path.join(platformDir, target.binary);
  fs.copyFileSync(extractedBinary, destination);
  if (!target.binary.endsWith(".exe")) {
    fs.chmodSync(destination, 0o755);
  }

  console.log(`Prepared ${destination}`);
}

async function main() {
  fs.mkdirSync(vendorDir, { recursive: true });
  fs.rmSync(path.join(vendorDir, "promemo"), { force: true });
  fs.rmSync(path.join(vendorDir, "promemo.exe"), { force: true });

  const sumsUrl = githubToken
    ? await assetApiUrl("SHA256SUMS")
    : `https://github.com/${repo}/releases/download/${tag}/SHA256SUMS`;
  const downloadHeaders = githubToken ? { Accept: "application/octet-stream" } : {};
  const sumsPath = path.join(tmpDir, "SHA256SUMS");
  await download(sumsUrl, sumsPath, downloadHeaders);
  const sumsText = fs.readFileSync(sumsPath, "utf8");

  for (const [platform, target] of Object.entries(targets)) {
    await prepareTarget(platform, target, sumsText);
  }
}

main()
  .catch((error) => {
    console.error(`Promemo binary preparation failed: ${error.message}`);
    process.exit(1);
  })
  .finally(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });
