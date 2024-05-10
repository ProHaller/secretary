use crate::config::config::Config;
use crate::models::dropbox_file_metadata::DropboxFileMetadata;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use std::error::Error;

#[derive(Clone)]
pub struct DropboxClient {
    pub client: Client,
    pub access_token: String,
    pub dropbox_client_id: String,
    pub dropbox_client_secret: String,
    pub audio_path: String,
}

pub type Audio = Vec<u8>;

#[derive(Debug, Deserialize)]
pub struct DropboxListFolderResponse {
    entries: Vec<DropboxFileMetadata>,
}

impl DropboxClient {
    pub fn new(config: &Config) -> Self {
        let access_token = config.dropbox_access_token.clone();
        let dropbox_client_id = config.dropbox_client_id.clone();
        let dropbox_client_secret = config.dropbox_client_secret.clone();
        let client = Client::new();
        let audio_path = config.dropbox_audio_path.clone();
        DropboxClient {
            client,
            access_token,
            dropbox_client_id,
            dropbox_client_secret,
            audio_path,
        }
    }

    pub fn audio_file_extensions() -> HashSet<&'static str> {
        let mut exts = HashSet::new();
        exts.insert(".mp3");
        exts.insert(".mp4");
        exts.insert(".mpeg");
        exts.insert(".mpga");
        exts.insert(".wav");
        exts.insert(".m4a");
        exts.insert(".flac");
        exts.insert(".ogg");
        exts.insert(".oga");
        exts.insert(".webm");
        exts
    }

    pub async fn list_files(&self) -> Result<Vec<DropboxFileMetadata>, Box<dyn std::error::Error>> {
        let url = "https://api.dropboxapi.com/2/files/list_folder";
        let body = json!({
            "path": self.audio_path,
            "recursive": false,
            "include_media_info": false,
            "include_deleted": false,
            "include_has_explicit_shared_members": false
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: DropboxListFolderResponse = response.json().await?;
            println!("Listed files: {:?}", response_body.entries);
            Ok(response_body.entries)
        } else {
            Err(format!("Failed to list files: {}", response.text().await?).into())
        }
    }

    pub async fn download_file(&self, file_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let url = "https://content.dropboxapi.com/2/files/download";
        let arg = json!({ "path": file_path });

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.access_token)
            .header("Dropbox-API-Arg", serde_json::to_string(&arg)?)
            .send()
            .await?;

        if response.status().is_success() {
            let file_content = response.bytes().await?;
            println!("Downloaded file: {}", file_path);
            Ok(file_content.to_vec())
        } else {
            let error_body = response.text().await?;
            Err(Box::new(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                format!(
                    "Failed to download file: '{}'. Server responded with: '{}'",
                    file_path, error_body
                ),
            )))
        }
    }

    pub async fn update_file(
        &self,
        updated_file_metadata: &DropboxFileMetadata,
    ) -> Result<DropboxFileMetadata, Box<dyn std::error::Error>> {
        let url = "https://api.dropboxapi.com/2/files/update";

        let body = json!({
            "path": updated_file_metadata.path_lower,
            "mode": "overwrite",
            "autorename": false,
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Dropbox-API-Arg", body.to_string())
            .send()
            .await?;

        match response.status().is_success() {
            true => {
                let confirmation: DropboxFileMetadata = response.json().await?;
                Ok(confirmation)
            }
            false => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to update file: {}", response.text().await?),
            ))),
        }
    }
}
