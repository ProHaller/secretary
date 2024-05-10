use crate::config::config::Config;
use crate::models::audio_note::AudioNote;
use reqwest::{multipart, Client};
use serde_json::Value;
use std::error::Error;
use std::fs;

#[derive(Clone)]
pub struct WhisperClient {
    client: Client,
    api_key: String,
}

impl WhisperClient {
    pub fn new(config: &Config) -> Self {
        let api_key = config.whisper_api_key.clone();
        let client = Client::new();
        WhisperClient { client, api_key }
    }

    pub async fn transcribe_file(&self, audio_note: &mut AudioNote) -> Result<(), Box<dyn Error>> {
        let file_name = &audio_note.note_name;

        let audio_bytes = fs::read(&audio_note.local_audio_file_path)?;

        let form = multipart::Form::new().text("model", "whisper-1").part(
            "file",
            multipart::Part::bytes(audio_bytes).file_name(file_name.clone()),
        );

        let url = "https://api.openai.com/v1/audio/transcriptions";
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        let response_json = response.json::<Value>().await?;

        if let Some(text) = response_json["text"].as_str() {
            audio_note.transcription = text.to_string();
            Ok(())
        } else if let Some(error) = response_json["error"]["message"].as_str() {
            Err(format!("Error during transcription: {}", error).into())
        } else {
            Err("Unexpected response format".into())
        }
    }
}
