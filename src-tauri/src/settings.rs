use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_vocabulary_prompt() -> String {
    String::from(
        "Claude Code, ChatGPT, Cursor, GitHub Copilot, pull request, TypeScript, Rust, Tauri",
    )
}

fn default_urdu_hotkey() -> String {
    String::from("Ctrl+Alt+Space")
}

fn default_paste_method() -> String {
    String::from("direct")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub microphone: String,
    pub engine: String,
    #[serde(rename = "whisperModel")]
    pub whisper_model: String,
    #[serde(rename = "groqApiKey")]
    pub groq_api_key: String,
    #[serde(rename = "recordingMode")]
    pub recording_mode: String,
    pub hotkey: String,
    #[serde(rename = "vocabularyPrompt", default = "default_vocabulary_prompt")]
    pub vocabulary_prompt: String,
    #[serde(rename = "urduHotkey", default = "default_urdu_hotkey")]
    pub urdu_hotkey: String,
    #[serde(rename = "pasteMethod", default = "default_paste_method")]
    pub paste_method: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            microphone: "default".to_string(),
            engine: "local".to_string(),
            whisper_model: "small".to_string(),
            groq_api_key: String::new(),
            recording_mode: "toggle".to_string(),
            hotkey: "CmdOrCtrl+Shift+Space".to_string(),
            vocabulary_prompt: default_vocabulary_prompt(),
            urdu_hotkey: default_urdu_hotkey(),
            paste_method: default_paste_method(),
        }
    }
}

impl Settings {
    pub fn config_path(app_dir: &PathBuf) -> PathBuf {
        app_dir.join("config.json")
    }

    pub fn load(app_dir: &PathBuf) -> Self {
        let path = Self::config_path(app_dir);
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, app_dir: &PathBuf) -> Result<(), String> {
        let path = Self::config_path(app_dir);
        fs::create_dir_all(app_dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.microphone, "default");
        assert_eq!(settings.engine, "local");
        assert_eq!(settings.whisper_model, "small");
        assert_eq!(settings.groq_api_key, "");
        assert_eq!(settings.recording_mode, "toggle");
        assert_eq!(settings.hotkey, "CmdOrCtrl+Shift+Space");
    }

    #[test]
    fn test_new_fields_have_correct_defaults() {
        let s = Settings::default();
        assert_eq!(s.paste_method, "direct");
        assert_eq!(s.urdu_hotkey, "Ctrl+Alt+Space");
        assert!(!s.vocabulary_prompt.is_empty());
        assert!(s.vocabulary_prompt.contains("Claude Code"));
    }

    #[test]
    fn test_deserialize_old_config_uses_new_field_defaults() {
        let json = r#"{
            "microphone": "default",
            "engine": "local",
            "whisperModel": "small",
            "groqApiKey": "",
            "recordingMode": "toggle",
            "hotkey": "CmdOrCtrl+Shift+Space"
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.paste_method, "direct");
        assert_eq!(s.urdu_hotkey, "Ctrl+Alt+Space");
        assert!(s.vocabulary_prompt.contains("Claude Code"));
    }

    #[test]
    fn test_save_and_load() {
        let dir = temp_dir().join("promptpilot_voice_test_settings");
        let _ = fs::remove_dir_all(&dir);

        let mut settings = Settings::default();
        settings.engine = "cloud".to_string();
        settings.groq_api_key = "test-key-123".to_string();

        settings.save(&dir).unwrap();
        let loaded = Settings::load(&dir);

        assert_eq!(loaded.engine, "cloud");
        assert_eq!(loaded.groq_api_key, "test-key-123");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = temp_dir().join("promptpilot_voice_test_missing");
        let _ = fs::remove_dir_all(&dir);
        let settings = Settings::load(&dir);
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn test_load_corrupt_json_returns_default() {
        let dir = temp_dir().join("promptpilot_voice_test_corrupt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.json"), "not json").unwrap();

        let settings = Settings::load(&dir);
        assert_eq!(settings, Settings::default());

        let _ = fs::remove_dir_all(&dir);
    }
}
