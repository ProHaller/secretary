// TODO: Support specifying the assistant ID as a parameter to process_transcriptions(). [[5]](https://poe.com/citation?message_id=175047583702&citation=5)
// TODO: Make the model array a constant at the top of the file. [[5]](https://poe.com/citation?message_id=175047583702&citation=5)
// TODO: Load the prompt template at startup rather than on each iteration. [[5]](https://poe.com/citation?message_id=175047583702&citation=5)
// TODO: Provide more informative error messages on failure. [[5]](https://poe.com/citation?message_id=175047583702&citation=5)
use crate::config::config::Config;
use crate::dropbox::dropbox::DropboxClient;
use crate::models::audio_note::{self, AudioNote};
use crate::models::dropbox_file_metadata::DropboxFileMetadata;
use crate::openai::gpt::GptClient;
use crate::whisper::whisper::WhisperClient;
use std::path::PathBuf;

#[derive(Clone)]
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
            let mut audio_note = AudioNote::new_from_metadata(&metadata);
            if audio_note.check_if_new_file(&PathBuf::from(&self.config.obsidian_vault_path)) {
                self.audio_notes.push(audio_note);
            }
        }
        Ok(())
    }

    pub async fn download_audio_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for audio_note in self.audio_notes.iter_mut() {
            let audio_note_bytes = self
                .dropbox_client
                .download_file(&audio_note.audio_file_metadata.path_lower)
                .await?;
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
    // TODO: Implement the multi call system with function calling.
    // TODO: Add support for assistant id.
    pub async fn process_transcriptions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let model: [&str; 3] = ["gpt-3.5-turbo", "gpt-4", "gpt-4o"];
        let prompt_template = std::fs::read_to_string("prompt.md")?;
        for audio_note in self.audio_notes.iter_mut() {
            println!("{}", audio_note);
            let prompt = prompt_template.replace("{transcription}", &audio_note.transcription);
            audio_note.note = self.gpt_client.fetch_completion(&prompt, model[2]).await?;
        }

        Ok(())
    }

    pub async fn clean_notes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for audio_note in self.audio_notes.iter_mut() {
            let audio_file_name = &audio_note.audio_file_metadata.name;

            // Split the note into two parts and add audio file name
            if let Some((part1, part2)) = audio_note.note.split_once("---") {
                audio_note.note = format!(
                    "{}---\naudio_file_name: {}\n{}",
                    part1, audio_file_name, part2
                );
            } else {
                // If the split was unsuccessful, add audio file name at the start
                audio_note.note = format!(
                    "{}\naudio_file_name: {}\n",
                    audio_note.note, audio_file_name
                );
            }

            // Append the transcription at the end
            audio_note.note = format!(
                "{}\n## Transcription\n{}",
                audio_note.note, audio_note.transcription
            );
        }
        Ok(())
    }

    pub async fn save_notes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for note in &mut self.audio_notes {
            let _name_without_extension = note.make_note_name_from_title().await;
            note.make_note_path(&self.config.obsidian_vault_path)
                .await?;

            // Construct a unique file name for each note to prevent overwrites
            let file_path = &note.note_path;

            // Asynchronously write the note content to the file
            tokio::fs::write(file_path, &note.note).await?;
            println!("Note saved: {}", file_path.display());
        }
        Ok(())
    }
}
