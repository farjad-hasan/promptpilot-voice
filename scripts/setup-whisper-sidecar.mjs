import { spawnSync } from "node:child_process";
import fs from "node:fs";
import https from "node:https";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const RELEASE_API_URL = "https://api.github.com/repos/ggml-org/whisper.cpp/releases/latest";
const WINDOWS_X64_ARCHIVE = "whisper-bin-x64.zip";
const SIDECAR_EXE = "whisper-cpp-x86_64-pc-windows-msvc.exe";

export const REQUIRED_SIDECAR_FILES = [
  SIDECAR_EXE,
  "ggml.dll",
  "ggml-base.dll",
  "ggml-cpu.dll",
  "whisper.dll",
];

export function selectWindowsX64Asset(assets) {
  const asset = assets.find((item) => item.name === WINDOWS_X64_ARCHIVE);
  if (!asset?.browser_download_url) {
    throw new Error(`Could not find ${WINDOWS_X64_ARCHIVE} in the latest whisper.cpp release.`);
  }
  return asset;
}

export function planSidecarFiles(sourceDir, destinationDir) {
  const files = [
    ["whisper-cli.exe", SIDECAR_EXE],
    ["ggml.dll", "ggml.dll"],
    ["ggml-base.dll", "ggml-base.dll"],
    ["ggml-cpu.dll", "ggml-cpu.dll"],
    ["whisper.dll", "whisper.dll"],
  ];

  return files.map(([sourceName, destinationName]) => ({
    sourceName,
    destinationName,
    sourcePath: path.join(sourceDir, sourceName),
    destinationPath: path.join(destinationDir, destinationName),
  }));
}

async function fetchJson(url) {
  const body = await fetchText(url);
  return JSON.parse(body);
}

function fetchText(url) {
  return new Promise((resolve, reject) => {
    request(url, (response) => {
      let body = "";
      response.setEncoding("utf8");
      response.on("data", (chunk) => {
        body += chunk;
      });
      response.on("end", () => resolve(body));
    }).on("error", reject);
  });
}

function downloadFile(url, destinationPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destinationPath);
    request(url, (response) => {
      response.pipe(file);
      file.on("finish", () => {
        file.close(resolve);
      });
    })
      .on("error", (error) => {
        file.close(() => {
          fs.rmSync(destinationPath, { force: true });
          reject(error);
        });
      });
  });
}

function request(url, callback, redirectsRemaining = 5) {
  const req = https.get(
    url,
    {
      headers: {
        "User-Agent": "promptpilot-voice-sidecar-setup",
        Accept: "application/vnd.github+json",
      },
    },
    (response) => {
      if (
        response.statusCode >= 300 &&
        response.statusCode < 400 &&
        response.headers.location &&
        redirectsRemaining > 0
      ) {
        response.resume();
        request(response.headers.location, callback, redirectsRemaining - 1);
        return;
      }

      if (response.statusCode < 200 || response.statusCode >= 300) {
        response.resume();
        req.emit("error", new Error(`Request failed with status ${response.statusCode}: ${url}`));
        return;
      }

      callback(response);
    },
  );
  return req;
}

export function buildExpandArchiveArgs(zipPath, destinationDir) {
  const quote = (value) => `'${String(value).replaceAll("'", "''")}'`;
  return [
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-Command",
    `Expand-Archive -LiteralPath ${quote(zipPath)} -DestinationPath ${quote(destinationDir)} -Force`,
  ];
}

function expandZip(zipPath, destinationDir) {
  const result = spawnSync("powershell.exe", buildExpandArchiveArgs(zipPath, destinationDir), {
    stdio: "inherit",
  });

  if (result.status !== 0) {
    throw new Error("Failed to extract whisper.cpp archive with Expand-Archive.");
  }
}

function findFile(startDir, fileName) {
  const entries = fs.readdirSync(startDir, { withFileTypes: true });
  for (const entry of entries) {
    const entryPath = path.join(startDir, entry.name);
    if (entry.isFile() && entry.name === fileName) {
      return entryPath;
    }
    if (entry.isDirectory()) {
      const found = findFile(entryPath, fileName);
      if (found) return found;
    }
  }
  return undefined;
}

async function main() {
  const repoRoot = process.cwd();
  const binariesDir = path.join(repoRoot, "src-tauri", "binaries");
  const workDir = path.join(os.tmpdir(), "promptpilot-whispercpp");
  const zipPath = path.join(workDir, WINDOWS_X64_ARCHIVE);
  const extractDir = path.join(workDir, "extracted");

  fs.mkdirSync(workDir, { recursive: true });
  fs.rmSync(extractDir, { recursive: true, force: true });
  fs.mkdirSync(extractDir, { recursive: true });

  console.log("Fetching latest whisper.cpp release metadata...");
  const release = await fetchJson(RELEASE_API_URL);
  const asset = selectWindowsX64Asset(release.assets ?? []);

  console.log(`Downloading ${asset.name}...`);
  await downloadFile(asset.browser_download_url, zipPath);

  console.log("Extracting archive...");
  expandZip(zipPath, extractDir);

  const whisperCli = findFile(extractDir, "whisper-cli.exe");
  if (!whisperCli) {
    throw new Error("Could not find whisper-cli.exe in the extracted archive.");
  }

  const sourceDir = path.dirname(whisperCli);
  const plan = planSidecarFiles(sourceDir, binariesDir);

  for (const item of plan) {
    if (!fs.existsSync(item.sourcePath)) {
      throw new Error(`Required file missing from archive: ${item.sourceName}`);
    }
  }

  fs.mkdirSync(binariesDir, { recursive: true });
  for (const item of plan) {
    fs.copyFileSync(item.sourcePath, item.destinationPath);
    console.log(`Copied ${item.sourceName} -> ${path.relative(repoRoot, item.destinationPath)}`);
  }

  console.log("whisper.cpp sidecar setup complete.");
}

const isCli = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;

if (isCli) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
