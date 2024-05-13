// TODO: Add error handling throughout main() to gracefully handle issues. [[4]](https://poe.com/citation?message_id=175047583702&citation=4)
// TODO: Use more descriptive variable names than just "model". [[4]](https://poe.com/citation?message_id=175047583702&citation=4)

// TODO: Add unit tests for key components like config parsing, Dropbox operations, etc.
// TODO: Integrate a logging framework for better troubleshooting.
// TODO: Implement proper secrets management rather than hardcoding in the config. [[3]](https://poe.com/citation?message_id=175047583702&citation=3)
// TODO: Document the main types and functions with doc comments.
// TODO: Set up CI to automatically run tests, linting, and formatting checks.

use secretary::auth::auth::initialize;
use secretary::config::config::Config;
use secretary::secretary::Secretary;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_file("config.toml")?;
    let mut secretary = Secretary::new(config);
    initialize(&mut secretary).await?;

    // Continuous loop to process new files
    loop {
        println!("Checking for new files...");
        secretary.update_audio_notes().await?;
        secretary.download_audio_files().await?;
        secretary.transcribe_audio_files().await?;
        secretary.process_transcriptions().await?;
        secretary.clean_notes().await?;
        secretary.save_notes().await?;

        // Wait for a specified duration before checking again
        sleep(Duration::from_secs(60)).await; // Check every 60 seconds
    }
}
