# Contributing

Thanks for your interest in PromptPilot Voice.

This project is currently a Windows-first developer preview with public source and a pending license. Before public contributions or reuse open, upstream licensing needs to be clarified because this code is derived from `albertshiney/typr`.

## Local Setup

```powershell
npm ci
npm run tauri dev
```

Use a current Rust stable toolchain. Rust/Cargo 1.69 cannot parse the checked-in lockfile.

## Before Sending Changes

Run the relevant checks:

```powershell
npm run build
cargo test --manifest-path src-tauri\Cargo.toml
cargo build --manifest-path src-tauri\Cargo.toml
```

For changes touching local transcription or release packaging, also run:

```powershell
npm run tauri build
```

## Secrets

Do not commit Groq API keys, copied config files, model binaries, sidecar binaries, recordings, or files from `%APPDATA%`.
