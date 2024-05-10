# Secretary

Secretary is a Rust application that automates the transcription and analysis of audio files. It integrates with the Dropbox API to fetch audio files, uses the Whisper API for transcription, and the OpenAI API for content analysis. The generated notes are then saved to an Obsidian vault.

## Features

- Fetches audio files from a specified Dropbox folder 
- Transcribes the audio using the Whisper API
- Analyzes the transcript using OpenAI to extract key information:
  - Tags related to the content
  - People mentioned
  - Title and summary  
  - Main points and action items
  - Follow-up questions
  - Mentioned stories and examples
- Saves the enriched notes to an Obsidian vault

## Setup

1. Clone the repository  
2. Install Rust and Cargo
3. Run `cargo build` to compile the project
4. Create a `config.toml` file with the following keys:
   - `dropbox_client_id`
   - `dropbox_client_secret`  
   - `dropbox_access_token`
   - `dropbox_auth_token`
   - `whisper_api_key`
   - `openai_api_key`
   - `obsidian_vault_path`
   - `dropbox_audio_path`
5. Run the application with `cargo run`

The app will prompt for any missing configuration values.

## Usage

On startup, Secretary will:
1. Fetch new audio files from the configured Dropbox folder
2. Transcribe each file using the Whisper API  
3. Analyze the transcripts using OpenAI
4. Save the enriched notes to the specified Obsidian vault

The generated note includes metadata like the audio file path, transcription size, and the full content analysis.

## Contributing

Contributions are welcome! Please see the TODO comments in the codebase for suggested improvements, including:
- Allowing the assistant ID to be specified
- Making the model a constant 
- Preloading prompt templates
- More informative error messages

Before submitting a pull request, please ensure your code follows the existing style, is well-tested, and includes relevant updates to the documentation.

## License

This project is licensed under the terms of the MIT license.

How's this? I removed the references to the code snippets as requested, but kept all the key sections and information. I also specified the MIT license at the end. Let me know if you would like any other changes!
