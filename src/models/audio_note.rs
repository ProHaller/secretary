use crate::models::dropbox_file_metadata::DropboxFileMetadata;
use std::fmt;
use std::fs;
use tokio::io::AsyncWriteExt;
use tempfile::NamedTempFile;
use std::path::PathBuf;
use std::error::Error;

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
    
    pub fn check_if_new_file(&mut self, path: &PathBuf) -> bool {
        // Attempt to read the directory entries
        let dir_entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
                return false;  // Decide on default behavior in case of error
            }
        };

        // Iterate over each entry in the directory
        for entry in dir_entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    continue;  // Continue with next entry on error
                }
            };
            let entry_path = entry.path();

            // Check if the file has a '.md' extension
            if entry_path.extension().and_then(std::ffi::OsStr::to_str) == Some("md") {
                // Read the content of the file
                let content = match fs::read_to_string(&entry_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", entry_path.display(), e);
                        continue;  // Continue with next file on error
                    }
                };

                // Check for the specific marker in the content
                let marker = format!("audio_file_name: {}", self.audio_file_metadata.name);
                if content.contains(&marker) {
                    return false;
                }
            }
        }

        // If no matching file is found or all files were processed with errors, return true
        true
    }






    pub async fn save_audio_file(&mut self, audio_file: Vec<u8>) -> Result<&AudioNote, Box<dyn Error>> {
        // Create a temporary file in the default temporary directory.
        let temp_file = NamedTempFile::new()?;

        // Write the audio file content to the temporary file asynchronously using tokio::fs::File.
        let mut file = tokio::fs::File::from_std(temp_file.reopen()?);
        file.write_all(&audio_file).await?;
        file.flush().await?;

        // Persist the temporary file and update the path.
        let persisted_path = temp_file.into_temp_path();
        let persisted_path_buf = persisted_path.to_path_buf();
        persisted_path.persist(&persisted_path_buf)?;
        self.local_audio_file_path = persisted_path_buf;

        println!("Audio file saved to: {}", self.local_audio_file_path.display());
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
        self.note_path.push(format!("{}.md", &self.note_name));
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
