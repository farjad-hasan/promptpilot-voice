use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

pub async fn transcribe_local(
    app: &AppHandle,
    model_path: &PathBuf,
    audio_path: &PathBuf,
    language: &str,
    prompt: &str,
) -> Result<String, String> {
    if !model_path.exists() {
        return Err("Whisper model not found. Please download a model first.".to_string());
    }

    println!(
        "[PromptPilot Voice] Running whisper.cpp sidecar with model {:?}",
        model_path
    );

    let mut cmd = app
        .shell()
        .sidecar("whisper-cpp")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?;

    // In dev builds, whisper DLLs live in src-tauri/binaries/ which cargo clean never touches.
    // Add that dir to PATH for the subprocess so Windows finds ggml*.dll / whisper.dll there.
    #[cfg(all(target_os = "windows", debug_assertions))]
    if let Some(binaries_dir) = std::env::current_exe().ok().and_then(|exe| {
        // exe = src-tauri/target/debug/promptpilot-voice.exe → go up 3 to reach src-tauri/
        let dir = exe.parent()?.parent()?.parent()?.join("binaries");
        if dir.exists() {
            Some(dir)
        } else {
            None
        }
    }) {
        let path = format!(
            "{};{}",
            binaries_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        cmd = cmd.env("PATH", path);
    }

    let output = cmd
        .args([
            "-m",
            model_path.to_str().unwrap(),
            "-f",
            audio_path.to_str().unwrap(),
            "--no-timestamps",
            "-l",
            language,
            "--prompt",
            prompt,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run whisper.cpp: {}", e))?;

    if output.status.code() != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let code = output.status.code().unwrap_or(-1);
        eprintln!(
            "[PromptPilot Voice] whisper.cpp exit={} stderr={:?} stdout={:?}",
            code, stderr, stdout
        );
        return Err(format!("whisper.cpp failed (exit {}): {}", code, stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("[PromptPilot Voice] Whisper output: {}", text);
    Ok(text)
}

pub fn model_filename(model_size: &str) -> String {
    format!("ggml-{}.bin", model_size)
}

pub fn model_download_url(model_size: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_filename() {
        assert_eq!(model_filename("small"), "ggml-small.bin");
        assert_eq!(model_filename("medium"), "ggml-medium.bin");
    }

    #[test]
    fn test_model_download_url() {
        assert_eq!(
            model_download_url("small"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
        );
    }
}
