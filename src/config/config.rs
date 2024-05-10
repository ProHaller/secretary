use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub dropbox_client_id: String,
    pub dropbox_client_secret: String,
    pub dropbox_access_token: String,
    pub dropbox_auth_token: String,
    pub whisper_api_key: String,
    pub openai_api_key: String,
    pub obsidian_vault_path: String,
    pub dropbox_audio_path: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config: Config;

        if !path.as_ref().exists() {
            println!("Config file not found. Creating a new one.");
            config = Config {
                dropbox_access_token: "".to_string(),
                dropbox_auth_token: "".to_string(),
                whisper_api_key: "".to_string(),
                openai_api_key: "".to_string(),
                obsidian_vault_path: "".to_string(),
                dropbox_audio_path: "".to_string(),
                dropbox_client_id: "".to_string(),
                dropbox_client_secret: "".to_string(),
            };
            config.prompt_and_update_if_empty();
            let toml = toml::to_string(&config)?;
            fs::write(&path, toml)?;
            println!("Config file created at: {}", path.as_ref().display());
        } else {
            let file_contents = fs::read_to_string(&path)?;
            config = toml::from_str(&file_contents)?;
            if config.has_empty_values() {
                println!("Config file has empty values. Prompting for missing values.");
                config.prompt_and_update_if_empty();
                let toml = toml::to_string(&config)?;
                fs::write(&path, toml)?;
                println!("Config file updated at: {}", path.as_ref().display());
            } else {
                println!("Config file loaded from: {}", path.as_ref().display());
            }
        }

        Ok(config)
    }

    fn has_empty_values(&self) -> bool {
        self.dropbox_access_token.is_empty()
            || self.whisper_api_key.is_empty()
            || self.openai_api_key.is_empty()
            || self.obsidian_vault_path.is_empty()
            || self.dropbox_client_id.is_empty()
            || self.dropbox_client_secret.is_empty()
            || self.dropbox_audio_path.is_empty()
    }

    fn prompt_and_update_if_empty(&mut self) {
        self.dropbox_access_token =
            Config::prompt_if_empty(self.dropbox_access_token.clone(), "Dropbox Access Token");
        self.whisper_api_key =
            Config::prompt_if_empty(self.whisper_api_key.clone(), "Whisper API Key");
        self.openai_api_key =
            Config::prompt_if_empty(self.openai_api_key.clone(), "OpenAI API Key");
        self.obsidian_vault_path =
            Config::prompt_if_empty(self.obsidian_vault_path.clone(), "Obsidian Vault Path");
        self.dropbox_client_id =
            Config::prompt_if_empty(self.dropbox_client_id.clone(), "Dropbox Client ID");
        self.dropbox_client_secret =
            Config::prompt_if_empty(self.dropbox_client_secret.clone(), "Dropbox Client Secret");
        self.dropbox_audio_path =
            Config::prompt_if_empty(self.dropbox_audio_path.clone(), "Dropbox Audio Path");
    }

    fn prompt_if_empty(value: String, prompt: &str) -> String {
        if value.is_empty() {
            let mut input = String::new();
            println!("Please enter your {}: ", prompt);
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            input.trim().to_string()
        } else {
            value
        }
    }
}
