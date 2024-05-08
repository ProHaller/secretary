use secretary2::config::config::Config;
use secretary2::secretary::Secretary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let mut secretary = Secretary::new(config);
    secretary.update_audio_notes().await?;
    secretary.download_audio_files().await?;
    secretary.transcribe_audio_files().await?;
    secretary.process_transcriptions().await?;
    secretary.save_note().await?;
    Ok(())
}
