# PromptPilot Voice

Windows-first voice typing for AI prompts, powered by local Whisper or Groq.

PromptPilot Voice is a desktop dictation tool for people who spend a lot of time prompting AI coding tools. Press a global hotkey, speak naturally, and the transcription is typed into the currently focused app.

> Status: public source, license pending. This repository is visible for learning, review, and transparency, but it is not an open-source release yet. Public licensing is pending upstream clarification because this project is derived from `albertshiney/typr`, which does not currently show a visible license.

## Features

- Global hotkey that works across Windows applications
- Toggle and push-to-talk recording modes
- Local transcription through a `whisper.cpp` sidecar
- Optional Groq cloud transcription for faster responses
- Dedicated Urdu hotkey
- Vocabulary hints for project names, technical terms, and teammate names
- Direct text injection with clipboard paste fallback
- Small always-on-top overlay for recording and transcription state

## Requirements

- Windows 10/11 x64
- Node.js 18 or newer
- Current Rust stable toolchain. Rust 1.69 is too old for the checked-in lockfile.
- MSVC Build Tools with the C++ workload
- `whisper.cpp` sidecar files for local transcription
- Optional: a Groq API key for cloud transcription

## Development Setup

```powershell
cd promptpilot-voice
npm ci
npm run tauri dev
```

The Tauri CLI starts the Vite dev server and the Rust backend. The first Rust build can take a few minutes.

## Sidecar Setup

Local transcription uses a pre-built `whisper.cpp` CLI binary as a Tauri sidecar. Download `whisper-bin-x64.zip` from the latest `whisper.cpp` release and place the files in `src-tauri/binaries/`.

Expected layout:

```text
src-tauri/binaries/
├── whisper-cpp-x86_64-pc-windows-msvc.exe
├── ggml.dll
├── ggml-base.dll
├── ggml-cpu.dll
└── whisper.dll
```

Rename `whisper-cli.exe` to `whisper-cpp-x86_64-pc-windows-msvc.exe`. The `src-tauri/binaries/` directory is ignored by Git so large binaries and DLLs are not committed.

## Configuration

Settings are saved locally at:

```text
%APPDATA%\io.github.farjad-hasan.promptpilot-voice\config.json
```

Example:

```json
{
  "microphone": "default",
  "engine": "local",
  "whisperModel": "small",
  "groqApiKey": "",
  "recordingMode": "toggle",
  "hotkey": "CmdOrCtrl+Shift+Space",
  "vocabularyPrompt": "Claude Code, ChatGPT, Cursor, GitHub Copilot, pull request",
  "urduHotkey": "Ctrl+Alt+Space",
  "pasteMethod": "direct"
}
```

Privacy note: Groq keys are currently stored in this local JSON config file. Do not commit files from `%APPDATA%`, screenshots showing keys, or copied config files containing secrets.

## Hotkeys

| Hotkey | Action |
| --- | --- |
| `Ctrl+Shift+Space` | Start/stop English recording |
| `Ctrl+Alt+Space` | Start/stop Urdu recording |

To change hotkeys, edit `config.json` and restart the app. Hotkey strings follow Tauri accelerator format such as `Ctrl`, `Shift`, `Alt`, `CmdOrCtrl`, and `Space`.

## Transcription Engines

| Engine | Notes |
| --- | --- |
| Local Whisper | Runs on-device through `whisper.cpp`. Requires a downloaded model and sidecar binary. |
| Groq Cloud | Sends audio to Groq's transcription API. Requires a Groq API key. |

Models download to the app config directory:

```text
%APPDATA%\io.github.farjad-hasan.promptpilot-voice\ggml-small.bin
%APPDATA%\io.github.farjad-hasan.promptpilot-voice\ggml-medium.bin
```

## Troubleshooting

### `whisper.cpp failed (exit -1073741515)`

A sidecar DLL is missing. Confirm all required DLLs are in `src-tauri/binaries/` and install the Visual C++ Redistributable x64 if needed.

### Transcription returns empty

Check the selected microphone, record for at least two seconds, and confirm the selected model exists in the app config directory.

### Text appears garbled or incomplete

Switch the paste method from direct input to clipboard. Some applications handle simulated keystrokes differently.

### Cargo cannot parse `Cargo.lock`

Update Rust with `rustup update stable`. Rust/Cargo 1.69 cannot read lockfile version 4.

## Verification

```powershell
npm ci
npm run build
cargo test --manifest-path src-tauri\Cargo.toml
cargo build --manifest-path src-tauri\Cargo.toml
npm run tauri build
```

`npm run tauri build` requires the `whisper.cpp` sidecar files listed above.

## Manual Testing

The `0.1.0` Windows installer has been manually tested on Windows with Groq cloud transcription. The tested path includes installing the app, launching it, entering a Groq API key, recording speech, transcribing through Groq, and pasting text into another focused application.

## Attribution

PromptPilot Voice is derived from [albertshiney/typr](https://github.com/albertshiney/typr) and was inspired by the YouTube video [I cancelled Wispr Flow... and built my own (it's free)](https://youtu.be/_ghw5bwBMjQ?si=LYxuleF29qNLMEs0).

## License

License is pending upstream clarification. Until that is resolved, this repository should be treated as source-available for review only, not as an open-source package for reuse or redistribution.

The intended license for this project is MIT once the upstream license or permission status is resolved.
