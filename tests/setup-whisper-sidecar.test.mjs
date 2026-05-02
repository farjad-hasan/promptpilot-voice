import assert from "node:assert/strict";
import test from "node:test";

import {
  REQUIRED_SIDECAR_FILES,
  buildExpandArchiveArgs,
  planSidecarFiles,
  selectWindowsX64Asset,
} from "../scripts/setup-whisper-sidecar.mjs";

test("selectWindowsX64Asset chooses the CPU Windows x64 archive", () => {
  const asset = selectWindowsX64Asset([
    { name: "whisper-bin-Win32.zip", browser_download_url: "win32" },
    { name: "whisper-cublas-12.4.0-bin-x64.zip", browser_download_url: "cuda" },
    { name: "whisper-bin-x64.zip", browser_download_url: "cpu-x64" },
  ]);

  assert.equal(asset.name, "whisper-bin-x64.zip");
  assert.equal(asset.browser_download_url, "cpu-x64");
});

test("selectWindowsX64Asset throws when the expected archive is missing", () => {
  assert.throws(
    () => selectWindowsX64Asset([{ name: "whisper-bin-Win32.zip" }]),
    /whisper-bin-x64\.zip/,
  );
});

test("planSidecarFiles maps whisper-cli and required DLLs to Tauri sidecar names", () => {
  const plan = planSidecarFiles("C:/extract/Release", "D:/repo/src-tauri/binaries");

  assert.deepEqual(
    plan.map((item) => item.destinationName),
    REQUIRED_SIDECAR_FILES,
  );

  assert.equal(plan[0].sourceName, "whisper-cli.exe");
  assert.equal(plan[0].destinationName, "whisper-cpp-x86_64-pc-windows-msvc.exe");
  assert.ok(plan.some((item) => item.sourceName === "ggml.dll"));
  assert.ok(plan.some((item) => item.sourceName === "ggml-base.dll"));
  assert.ok(plan.some((item) => item.sourceName === "ggml-cpu.dll"));
  assert.ok(plan.some((item) => item.sourceName === "whisper.dll"));
});

test("buildExpandArchiveArgs embeds literal paths for PowerShell", () => {
  const args = buildExpandArchiveArgs("C:/tmp/archive's.zip", "C:/tmp/extracted");

  assert.deepEqual(args.slice(0, 4), ["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]);
  assert.match(args[4], /Expand-Archive/);
  assert.match(args[4], /archive''s\.zip/);
  assert.match(args[4], /DestinationPath/);
});
