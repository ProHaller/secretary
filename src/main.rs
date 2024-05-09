use secretary2::config::config::Config;
use secretary2::secretary::Secretary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let mut secretary = Secretary::new(config);
    secretary.update_audio_notes().await?;
    println!("Audio notes updated");
    secretary.download_audio_files().await?;
    println!("Audio files downloaded");
    secretary.transcribe_audio_files().await?;
    println!("Audio files transcribed");
    secretary.process_transcriptions().await?;
    println!("Audio files processed");
    secretary.clean_notes().await?;
    println!("Notes cleaned");
    secretary.save_notes().await?;
    println!("Note saved");
    Ok(())
}

// TODO: Parallelize the API calls
// TODO: Prepare the multi completion requirements:
