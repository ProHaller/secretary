use crate::models::dropbox_file_metadata::DropboxFileMetadata;
use std::fmt;
use std::fs;
use tokio::io::AsyncWriteExt;
use tempfile::NamedTempFile;
use std::time::SystemTime;
use std::path::PathBuf;
use std::error::Error;
use std::{io, io::BufRead};
use crate::utils::file_utils::parse_timestamp_to_system_time;

pub struct AudioNote {
    pub audio_file_metadata: DropboxFileMetadata,
    pub transcription: String,
    pub note: String,
    pub note_name: String,
    pub local_audio_file_path: PathBuf,
    pub note_path: PathBuf,
}

impl AudioNote{
    pub fn new_from_metadata(file_metadata: &DropboxFileMetadata) -> AudioNote {
        let audio_file_metadata = file_metadata.clone();
        let transcription = "".to_string();
        let note = "".to_string();
        let note_name = file_metadata.name.clone();
        let audio_file_path = PathBuf::new();
        let note_path = PathBuf::new();
        AudioNote{
            audio_file_metadata,
            transcription,
            note,
            note_name,
            local_audio_file_path: audio_file_path,
            note_path,
        }
    }

    pub fn check_if_new_file(&self, path: &PathBuf) -> bool {
        let creation_date_system_time = parse_timestamp_to_system_time(&self.audio_file_metadata.server_modified)
        .unwrap_or_else(|_| SystemTime::now()); 

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return true,
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                let metadata = match entry.metadata() {
                    Ok(metadata) => metadata,
                    Err(_) => continue,
                };

                if let Ok(modified) = metadata.modified() {
                    if modified > creation_date_system_time {
                        if let Ok(file) = fs::File::open(&path) {
                            let reader = io::BufReader::new(file);
                            let mut in_front_matter = false;

                            for line in reader.lines() {
                                match line {
                                    Ok(line) => {
                                        if line.trim() == "---" {
                                            in_front_matter = !in_front_matter;
                                        } else if in_front_matter && line.contains(&format!("note_name: {}", self.note_name)) {
                                            return false;
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error reading lines from file {}: {}", path.display(), e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        true 
    }
    pub async fn save_audio_file(&mut self, audio_file: Vec<u8>) -> Result<&AudioNote, Box<dyn Error>> {
        // Create a temporary file in the default temporary directory.
        let (temp_file, temp_path) = NamedTempFile::new()?.into_parts();

        // Write the audio file content to the temporary file asynchronously using tokio::fs::File.
        let mut file = tokio::fs::File::from_std(temp_file);
        file.write_all(&audio_file).await?;
        file.flush().await?;

        // Prepare to persist the file.
        let persist_path = temp_path.to_path_buf();
        let persisted_file = NamedTempFile::new_in(&persist_path)?;
        let persisted_path = persisted_file.into_temp_path();

        // Persist the file and update the path
        persisted_path.persist(&temp_path)?;
        self.local_audio_file_path = persist_path;

        Ok(self)
    }

    pub async fn make_note_name_from_title (&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let title_regex = regex::Regex::new(r"title:\s*(.+)").unwrap();
        self.note_name = title_regex
            .captures(&self.note)
            .and_then(|captures| captures.get(1))
            .map(|title| title.as_str().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        Ok(self)
    }

    pub async fn make_note_path (&mut self, path: &String) -> Result<&mut Self, Box<dyn Error>> {
        self.note_path = PathBuf::from(path);
        self.note_path.push(&self.note_name);
        Ok(self)
    }

}

impl fmt::Display for AudioNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Note Name: {}\nNote Path: {}\nAudio file Path: {}\nNote: {}\nTranscription: {} bytes\n",
            self.note_name, 
            self.note_path.to_string_lossy(), 
            self.local_audio_file_path.to_string_lossy(), 
            self.note, 
            self.transcription
        )
    }
}
