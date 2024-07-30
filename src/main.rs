use clap::Parser;
use reqwest::Client;
use serde_json::json;
use std::fs::{File, create_dir_all};
use std::io::{Write, stdin};
use std::path::PathBuf;
use chrono::Utc;
use dirs::home_dir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
}

fn get_config_dir() -> PathBuf {
    home_dir().unwrap().join(".transcribe_cli")
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let key_file = config_dir.join("api_key");

    if key_file.exists() {
        Ok(std::fs::read_to_string(key_file)?.trim().to_string())
    } else {
        println!("Deepgram API key not found. Please enter it:");
        let mut key = String::new();
        stdin().read_line(&mut key)?;
        let key = key.trim().to_string();
        create_dir_all(&config_dir)?;
        std::fs::write(key_file, &key)?;
        println!("API key saved.");
        Ok(key)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let api_key = get_api_key()?;

    println!("Starting transcription...");

    let client = Client::new();
    let response = client
        .post("https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&paragraphs=true&diarize=true")
        .header("Authorization", format!("Token {}", api_key))
        .json(&json!({
            "url": args.url
        }))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let transcript = result["results"]["channels"][0]["alternatives"][0]["transcript"]
        .as_str()
        .unwrap_or("Transcription failed");

    let now = Utc::now();
    let filename = format!("transcription-{}:{}.md", 
        now.format("%Y-%m-%d"),
        now.format("%H-%M")
    );

    let desktop = dirs::desktop_dir().expect("Could not find desktop directory");
    let output_path = desktop.join(filename);
    let mut file = File::create(output_path)?;
    file.write_all(transcript.as_bytes())?;

    println!("Transcription successful. File saved on Desktop.");

    Ok(())
}