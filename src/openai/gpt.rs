use crate::config::config::Config;
use reqwest::Client;
use serde_json::json;

pub struct GptClient {
    client: Client,
    api_key: String,
}

impl GptClient {
    pub fn new(config: &Config) -> Self {
        let api_key = config.openai_api_key.clone();
        let client = Client::new();
        GptClient { client, api_key }
    }

    pub async fn fetch_completion(
        &mut self,
        prompt: &str,
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";

        let body = json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7
        });

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        println!("{:#?}", response_json);
        response_json["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Content not found in the response".into())
    }
}
