use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::audio::AudioRecorder;
use crate::cleanup::cleanup_text;
use crate::paste::paste_text;
use crate::settings::Settings;
use crate::transcribe_groq;
use crate::transcribe_local;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum RecordingState {
    Ready,
    Recording,
    Transcribing,
}

fn update_overlay(app: &AppHandle, state: &RecordingState) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let class = match state {
            RecordingState::Ready => "mic",
            RecordingState::Recording => "mic recording",
            RecordingState::Transcribing => "mic transcribing",
        };
        let js = format!("document.getElementById('mic').className = '{}';", class);
        let _ = overlay.eval(&js);
    }
}

pub struct Recorder {
    state: Arc<Mutex<RecordingState>>,
    audio_recorder: Arc<Mutex<AudioRecorder>>,
    pub current_language: Arc<Mutex<String>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Ready)),
            audio_recorder: Arc::new(Mutex::new(AudioRecorder::new())),
            current_language: Arc::new(Mutex::new(String::from("en"))),
        }
    }

    pub fn get_state(&self) -> RecordingState {
        self.state.lock().unwrap().clone()
    }

    pub fn start_recording(
        &self,
        app: &AppHandle,
        mic_name: &str,
        language: &str,
    ) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordingState::Ready {
            return Err("Already recording or transcribing".to_string());
        }

        *self.current_language.lock().unwrap() = language.to_string();

        let mut recorder = self.audio_recorder.lock().unwrap();
        recorder.start(mic_name)?;

        *state = RecordingState::Recording;
        let _ = app.emit("recording-state", RecordingState::Recording);
        update_overlay(app, &RecordingState::Recording);
        Ok(())
    }

    pub async fn stop_and_transcribe(
        &self,
        app: &AppHandle,
        settings: &Settings,
        app_dir: &PathBuf,
    ) -> Result<String, String> {
        // Stop recording
        {
            let mut state = self.state.lock().unwrap();
            if *state != RecordingState::Recording {
                return Err("Not currently recording".to_string());
            }
            *state = RecordingState::Transcribing;
            let _ = app.emit("recording-state", RecordingState::Transcribing);
            update_overlay(app, &RecordingState::Transcribing);
        }

        let temp_path = app_dir.join("temp_recording.wav");
        let language = self.current_language.lock().unwrap().clone();

        // Run transcription in a block so state always resets, even on error.
        let result: Result<String, String> = (async {
            {
                let mut recorder = self.audio_recorder.lock().unwrap();
                recorder.stop_and_save(&temp_path)?;
            }

            let raw_text = match settings.engine.as_str() {
                "local" => {
                    let model_path =
                        app_dir.join(transcribe_local::model_filename(&settings.whisper_model));
                    transcribe_local::transcribe_local(
                        app,
                        &model_path,
                        &temp_path,
                        &language,
                        &settings.vocabulary_prompt,
                    )
                    .await?
                }
                "cloud" => {
                    transcribe_groq::transcribe_groq(
                        &settings.groq_api_key,
                        &temp_path,
                        &language,
                        &settings.vocabulary_prompt,
                    )
                    .await?
                }
                _ => return Err(format!("Unknown engine: {}", settings.engine)),
            };

            let cleaned = cleanup_text(&raw_text);
            if !cleaned.is_empty() {
                paste_text(&cleaned, &settings.paste_method)?;
            }
            Ok(cleaned)
        })
        .await;

        let _ = std::fs::remove_file(&temp_path);

        // Always reset to Ready regardless of transcription success or failure
        {
            let mut state = self.state.lock().unwrap();
            *state = RecordingState::Ready;
            let _ = app.emit("recording-state", RecordingState::Ready);
            update_overlay(app, &RecordingState::Ready);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_ready() {
        let recorder = Recorder::new();
        assert_eq!(recorder.get_state(), RecordingState::Ready);
    }

    #[test]
    fn test_recorder_stores_language_default() {
        let recorder = Recorder::new();
        let lang = recorder.current_language.lock().unwrap().clone();
        assert_eq!(lang, "en");
    }
}
