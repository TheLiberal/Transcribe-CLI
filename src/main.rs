use clap::Parser;
use reqwest::Client;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;
use dirs::home_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use url::Url;
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};
use futures_util::TryStreamExt;
use bytes::BytesMut;
use std::sync::{Arc, Mutex};
use tokio::time::{Instant, Duration};
use std::thread;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short = 'f', long)]
    is_file: bool,
}

fn get_config_dir() -> PathBuf {
    std::env::var("TRANSCRIBE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().unwrap().join(".transcribe_cli"))
}

// Add this function to get the output directory
fn get_output_dir() -> PathBuf {
    std::env::var("TRANSCRIBE_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::desktop_dir().expect("Could not find desktop directory"))
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let key_file = config_dir.join("api_key");

    if key_file.exists() {
        let key = std::fs::read_to_string(&key_file)?.trim().to_string();
        if key.is_empty() {
            return Err("API key file is empty".into());
        }
        Ok(key)
    } else {
        println!("Deepgram API key not found. Please enter it:");
        let mut key = String::new();
        std::io::stdin().read_line(&mut key)?;
        let key = key.trim().to_string();
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(&key_file, &key)?;
        println!("API key saved.");
        Ok(key)
    }
}

async fn send_request(
    client: &Client,
    request_url: &str,
    api_key: &str,
    args: &Args,
    upload_pb: &Arc<Mutex<ProgressBar>>,
    transcribe_pb: &Arc<Mutex<ProgressBar>>,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    if args.is_file {
        let file = TokioFile::open(&args.input).await?;
        let file_size = file.metadata().await?.len();
        upload_pb.lock().unwrap().set_length(file_size);
        upload_pb.lock().unwrap().set_message("Uploading...");
        transcribe_pb.lock().unwrap().set_message("Waiting for upload...");

        let upload_pb_clone = Arc::clone(upload_pb);
        let stream = FramedRead::new(file, BytesCodec::new())
            .map_ok(move |chunk: BytesMut| {
                let len = chunk.len() as u64;
                upload_pb_clone.lock().unwrap().inc(len);
                chunk.freeze()
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

        let body = reqwest::Body::wrap_stream(stream);

        upload_pb.lock().unwrap().set_message("Sending request...");
        transcribe_pb.lock().unwrap().set_message("Transcribing...");

        client
            .post(request_url)
            .header("Authorization", format!("Token {}", api_key))
            .header("Content-Type", "application/octet-stream")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Error sending file to Deepgram API: {}", e).into())
    } else {
        let url = Url::parse(&args.input).map_err(|_| "Invalid URL provided")?;
        
        transcribe_pb.lock().unwrap().set_message("Transcribing...");
        
        client
            .post(request_url)
            .header("Authorization", format!("Token {}", api_key))
            .json(&json!({ "url": url.as_str() }))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    "Failed to connect to Deepgram API. Please check your internet connection.".into()
                } else if e.is_timeout() {
                    "Request to Deepgram API timed out. Please try again later.".into()
                } else {
                    format!("Error sending request to Deepgram API: {}", e).into()
                }
            })
    }
}


async fn send_request_with_retry(
    client: &Client,
    request_url: &str,
    api_key: &str,
    args: &Args,
    upload_pb: &Arc<Mutex<ProgressBar>>,
    transcribe_pb: &Arc<Mutex<ProgressBar>>,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let max_retries = 3;
    for attempt in 1..=max_retries {
        match send_request(client, request_url, api_key, args, upload_pb, transcribe_pb).await {
            Ok(response) => {
                if response.status() == 401 {
                    return Err("Invalid API key".into());
                }
                if response.status().is_client_error() || response.status().is_server_error() {
                    let error_message = format!("API request failed with status: {}", response.status());
                    eprintln!("{}", error_message);
                    eprintln!("Response body: {}", response.text().await?);
                    return Err(error_message.into());
                }
                return Ok(response);
            },
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}. Retrying in 5 seconds...", attempt, e);
                upload_pb.lock().unwrap().reset();
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err("Max retries reached. Unable to connect to Deepgram API.".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let api_key = get_api_key()?;

    println!("Starting transcription process...");

    let m = MultiProgress::new();
    let upload_pb = Arc::new(Mutex::new(m.add(ProgressBar::new(0))));
    let transcribe_pb = Arc::new(Mutex::new(m.add(ProgressBar::new_spinner())));

    upload_pb.lock().unwrap().set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("##-"));

    transcribe_pb.lock().unwrap().set_style(ProgressStyle::default_spinner()
        .tick_chars("-\\|/")
        .template("{spinner} {msg} {elapsed_precise}")
        .unwrap());

    if args.is_file {
        upload_pb.lock().unwrap().set_message("Preparing...");
        transcribe_pb.lock().unwrap().set_message("Waiting...");
    } else {
        transcribe_pb.lock().unwrap().set_message("Preparing...");
    }

    let transcribe_pb_clone = Arc::clone(&transcribe_pb);
    let progress_handle = thread::spawn(move || {
        loop {
            transcribe_pb_clone.lock().unwrap().tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    let client = Client::new();
    let request_url = "https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&paragraphs=true&diarize=true";

    let start_time = Instant::now();

    let response = match send_request_with_retry(&client, request_url, &api_key, &args, &upload_pb, &transcribe_pb).await {
        Ok(response) => response,
        Err(e) => {
            upload_pb.lock().unwrap().finish_and_clear();
            transcribe_pb.lock().unwrap().finish_and_clear();
            eprintln!("Error: {}", e);
        if e.to_string() == "Invalid API key" {
            eprintln!("Please check your API key and try again.");
        }
        return Err(e);
        }
    };

    if !response.status().is_success() {
        upload_pb.lock().unwrap().finish_and_clear();
        transcribe_pb.lock().unwrap().finish_and_clear();
        eprintln!("API request failed with status: {}", response.status());
        eprintln!("Response body: {}", response.text().await?);
        return Err("API request failed".into());
    }

    let result: serde_json::Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            transcribe_pb.lock().unwrap().finish_and_clear();
            eprintln!("Failed to parse API response: {}", e);
            return Err("Failed to parse API response".into());
        }
    };

    let transcript = match result["results"]["channels"][0]["alternatives"][0]["transcript"].as_str() {
        Some(text) => text,
        None => {
            transcribe_pb.lock().unwrap().finish_and_clear();
            eprintln!("Failed to extract transcript from API response");
            return Err("Failed to extract transcript".into());
        }
    };

    let elapsed = start_time.elapsed();
    transcribe_pb.lock().unwrap().finish_and_clear();

    // Stop the progress bar thread
    progress_handle.thread().unpark();

    let now = Utc::now();
    let filename = format!("transcription-{}-{}.md", 
        now.format("%Y-%m-%d"),
        now.format("%H-%M-%S")
    );

    // let desktop = dirs::desktop_dir().expect("Could not find desktop directory");
    // let output_path = desktop.join(filename);
    // let mut file = File::create(output_path)?;
    // file.write_all(transcript.as_bytes())?;

    let output_dir = get_output_dir();
let output_path = output_dir.join(filename);
let mut file = File::create(output_path)?;
file.write_all(transcript.as_bytes())?;

    println!("Transcription successful. File saved on Desktop.");
    println!("Total time: {:.2?}", elapsed);

    Ok(())
}