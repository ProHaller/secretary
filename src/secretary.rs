use crate::config::config::Config;
use crate::dropbox::dropbox::DropboxClient;
use crate::models::audio_note::{self, AudioNote};
use crate::models::dropbox_file_metadata::DropboxFileMetadata;
use crate::openai::gpt::GptClient;
use crate::whisper::whisper::WhisperClient;
use std::path::PathBuf;

pub struct Secretary {
    pub config: Config,
    pub dropbox_client: DropboxClient,
    pub whisper_client: WhisperClient,
    pub gpt_client: GptClient,
    pub audio_notes: Vec<AudioNote>,
}

impl Secretary {
    pub fn new(config: Config) -> Secretary {
        let dropbox_client = DropboxClient::new(&config);
        let whisper_client = WhisperClient::new(&config);
        let gpt_client = GptClient::new(&config);
        Secretary {
            config,
            dropbox_client,
            whisper_client,
            gpt_client,
            audio_notes: Vec::new(),
        }
    }

    pub async fn update_audio_notes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let files_metadata = self.dropbox_client.list_files().await?;
        let extensions = DropboxClient::audio_file_extensions();
        let audio_files_metadata: Vec<DropboxFileMetadata> = files_metadata
            .into_iter()
            .filter(|metadata| {
                extensions
                    .iter()
                    .any(|ext| metadata.name.to_lowercase().ends_with(*ext))
            })
            .collect();

        for metadata in audio_files_metadata {
            let audio_note = AudioNote::new_from_metadata(&metadata);
            if audio_note.check_if_new_file(&PathBuf::from(&self.config.obsidian_vault_path)) {
                self.audio_notes.push(audio_note);
            }
        }
        Ok(())
    }

    pub async fn download_audio_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for audio_note in self.audio_notes.iter_mut() {
            let audio_path = audio_note
                .local_audio_file_path
                .to_str()
                .unwrap()
                .to_string();
            let audio_note_bytes = self.dropbox_client.download_file(&audio_path).await?;
            audio_note::AudioNote::save_audio_file(audio_note, audio_note_bytes).await?;
        }
        Ok(())
    }

    pub async fn transcribe_audio_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for audio_note in self.audio_notes.iter_mut() {
            self.whisper_client.transcribe_file(audio_note).await?;
        }
        Ok(())
    }

    pub async fn process_transcriptions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let model: [&str; 2] = ["gpt-3.5-turbo", "gpt-4"];
        let prompt_template = std::fs::read_to_string("prompt.md")?;
        for audio_note in self.audio_notes.iter_mut() {
            let prompt = prompt_template.replace("{transcription}", &audio_note.transcription);
            audio_note.note = self.gpt_client.fetch_completion(&prompt, model[0]).await?;
        }

        Ok(())
    }

    pub async fn save_note(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for note in &mut self.audio_notes {
            let _ = note.make_note_name_from_title().await;
            note.make_note_path(&self.config.obsidian_vault_path)
                .await?;

            // Construct a unique file name for each note to prevent overwrites
            let file_path = format!("{}{}", &self.config.obsidian_vault_path, "_note");

            // Asynchronously write the note content to the file
            tokio::fs::write(file_path, &note.note).await?;
        }
        Ok(())
    }
}
